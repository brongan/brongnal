use anyhow::Context;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
pub use client::X3DHClient;
use proto::service::brongnal_client::BrongnalClient;
use proto::service::{
    Message as MessageProto, RegisterPreKeyBundleRequest, RequestPreKeysRequest,
    RetrieveMessagesRequest, SendMessageRequest,
};
use protocol::x3dh::{initiate_recv, initiate_send, Message, X3DHError};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;
use tonic::transport::Channel;
use tonic::Streaming;
use tracing::{debug, error, info, warn};

pub mod client;

type ClientResult<T> = Result<T, ClientError>;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("failed to load identity key")]
    GetIdentityKey,
    #[error("failed to save identity key")]
    InsertIdentityKey(rusqlite::Error),
    #[error("rusqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("tokio_rusqlite error: {0}")]
    TokioSqlite(#[from] tokio_rusqlite::Error),
    #[error("failed to insert pre keys: {0}")]
    InsertPreKey(rusqlite::Error),
    #[error("failed to retrieve OPK private key for pubkey: {0}")]
    WipeOpk(String),
    #[error("failed to retrieve pre key")]
    GetPreKey(rusqlite::Error),
    #[error("grpc error: {0}")]
    Grpc(#[from] tonic::Status),
    #[error("send decrypted message error: {0}")]
    Send(#[from] SendError<DecryptedMessage>),
    #[error("x3dh error: {0}")]
    X3DH(#[from] X3DHError),
}

#[allow(dead_code)]
struct SessionKeys<T> {
    session_keys: HashMap<T, [u8; 32]>,
}

#[allow(dead_code)]
impl<Identity: Eq + std::hash::Hash> SessionKeys<Identity> {
    fn set_session_key(&mut self, recipient_identity: Identity, secret_key: &[u8; 32]) {
        self.session_keys.insert(recipient_identity, *secret_key);
    }

    fn get_encryption_key(
        &mut self,
        recipient_identity: &Identity,
    ) -> anyhow::Result<ChaCha20Poly1305> {
        let key = self
            .session_keys
            .get(recipient_identity)
            .context("Session key not found.")?;
        Ok(ChaCha20Poly1305::new_from_slice(key).unwrap())
    }

    fn destroy_session_key(&mut self, peer: &Identity) {
        self.session_keys.remove(peer);
    }
}

pub struct DecryptedMessage {
    pub sender_identity: String,
    pub message: Vec<u8>,
}

/// Should be spawned as an async task that opens a stream of messages from the server, decrypts
/// them, and sends the decrypted messages to `tx`.
pub async fn listen(
    mut stub: BrongnalClient<Channel>,
    x3dh_client: Arc<X3DHClient>,
    name: String,
    tx: Sender<DecryptedMessage>,
) -> ClientResult<()> {
    let stream = stub
        .retrieve_messages(RetrieveMessagesRequest {
            identity: Some(name),
        })
        .await;
    if let Err(e) = &stream {
        error!("Failed to retrieve messages: {e}");
    }
    if let Err(e) = get_messages(stream?.into_inner(), &x3dh_client, tx).await {
        error!("get_messages terminated with: {e}");
        return Err(e);
    }
    Ok(())
}

pub async fn register(
    stub: &mut BrongnalClient<Channel>,
    x3dh_client: &X3DHClient,
    name: String,
) -> ClientResult<()> {
    debug!("Registering {name}!");
    let request = {
        let ik = x3dh_client
            .get_ik()
            .await?
            .verifying_key()
            .as_bytes()
            .to_vec();
        tonic::Request::new(RegisterPreKeyBundleRequest {
            identity_key: Some(ik),
            identity: Some(name.clone()),
            signed_pre_key: Some(x3dh_client.get_spk().await?.into()),
            one_time_key_bundle: Some(x3dh_client.create_opks(100).await?.into()),
        })
    };
    stub.register_pre_key_bundle(request).await?;
    info!("Registered: {}!", name);
    Ok(())
}

pub async fn message(
    stub: &mut BrongnalClient<Channel>,
    x3dh_client: &X3DHClient,
    sender_identity: String,
    recipient_identity: &str,
    message: &str,
) -> ClientResult<()> {
    let message = message.as_bytes();
    let request = tonic::Request::new(RequestPreKeysRequest {
        identity: Some(recipient_identity.to_owned()),
    });
    let response = stub.request_pre_keys(request).await?;
    let (_sk, message) = initiate_send(
        response.into_inner().try_into()?,
        sender_identity,
        &x3dh_client.get_ik().await?,
        message,
    )?;
    debug!("Sending message: {message}");
    let request = tonic::Request::new(SendMessageRequest {
        recipient_identity: Some(recipient_identity.to_owned()),
        message: Some(message.into()),
    });
    stub.send_message(request).await?;
    Ok(())
}

// TODO(https://github.com/brongan/brongnal/issues/23) - Replace with stream of decrypted messages.
pub async fn get_messages(
    mut stream: Streaming<MessageProto>,
    x3dh_client: &X3DHClient,
    tx: Sender<DecryptedMessage>,
) -> ClientResult<()> {
    while let Some(message) = stream.message().await? {
        let res = {
            let message: Message = message.try_into()?;
            debug!("Received Message Proto: {message}");
            let opk = if let Some(opk) = message.opk {
                Some(x3dh_client.fetch_wipe_opk(opk).await?)
            } else {
                None
            };
            let (_sk, decrypted) = initiate_recv(
                &x3dh_client.get_ik().await?,
                &x3dh_client.get_pre_key(message.pre_key).await?,
                &message.sender_ik,
                message.ek,
                opk,
                &message.ciphertext,
            )?;
            Ok::<DecryptedMessage, ClientError>(DecryptedMessage {
                // TODO(https://github.com/brongan/brongnal/issues/15): Don't blindly trust the
                // sender's claimed identity.
                sender_identity: message.sender_identity,
                message: decrypted,
            })
        };
        match res {
            Ok(decrypted) => tx.send(decrypted).await?,
            Err(e) => {
                error!("Failed to decrypt message: {e}");
            }
        }
    }
    warn!("Server terminated message stream.");
    Ok(())
}
