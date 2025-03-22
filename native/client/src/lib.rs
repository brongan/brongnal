use anyhow::Context;
use async_stream::try_stream;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
pub use client::X3DHClient;
use proto::service::brongnal_service_client::BrongnalServiceClient as BrongnalClient;
use proto::service::{
    Message as MessageProto, RegisterPreKeyBundleRequest, RequestPreKeysRequest,
    RetrieveMessagesRequest, SendMessageRequest,
};
use protocol::x3dh::{initiate_recv, initiate_send, Message as X3DHMessage, X3DHError};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use tonic::Streaming;
use tracing::{error, info, warn};

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

fn into_message_stream(
    mut stream: Streaming<MessageProto>,
) -> impl Stream<Item = ClientResult<X3DHMessage>> {
    try_stream! {
        while let Some(message) = stream.message().await? {
            let message: X3DHMessage = message.try_into()?;
            yield message;
        }
    }
}

/// Takes a Brongnal RPC Client and an X3DH client and returns a stream of decrypted messages.
pub fn get_messages(
    mut stub: BrongnalClient<Channel>,
    x3dh_client: Arc<X3DHClient>,
    name: String,
) -> impl Stream<Item = ClientResult<DecryptedMessage>> {
    try_stream! {
        let stream = stub
            .retrieve_messages(RetrieveMessagesRequest {
                identity: Some(name),
            })
        .await;
        let messages = into_message_stream(stream?.into_inner());
        tokio::pin!(messages);
        while let Some(message) = messages.next().await {
            if let Err(e) = message {
                warn!("Message was not validly serialized: {e}");
                continue;
            }
            let message = message.unwrap();
            let res = {
                let opk = if let Some(opk) = message.opk {
                    Some(x3dh_client.fetch_wipe_opk(opk).await?)
                } else {
                    None
                };
                // TODO: Caller must delete the session keys with the peer on an error.
                let (_sk, decrypted) = initiate_recv(
                    &x3dh_client.get_ik(),
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
                Ok(decrypted) => yield decrypted,
                Err(e) => {
                    error!("Failed to decrypt message: {e}");
                }
            }
        }
        warn!("Server terminated message stream.");
    }
}

pub async fn register(
    stub: &mut BrongnalClient<Channel>,
    x3dh_client: &X3DHClient,
    name: String,
) -> ClientResult<()> {
    info!("Registering {name}!");
    let ik = x3dh_client.get_ik().verifying_key().as_bytes().to_vec();

    let request = tonic::Request::new(RegisterPreKeyBundleRequest {
        identity_key: Some(ik.clone()),
        identity: Some(name.clone()),
        signed_pre_key: Some(x3dh_client.get_spk().await?.into()),
        one_time_key_bundle: Some(x3dh_client.create_opks(0).await?.into()),
    });
    let res = stub.register_pre_key_bundle(request).await?.into_inner();
    info!("Registered: {}. {} keys remaining!", name, res.num_keys());
    if res.num_keys() < 100 {
        info!("Adding 100 keys!");
        let request = tonic::Request::new(RegisterPreKeyBundleRequest {
            identity_key: Some(ik),
            identity: Some(name.clone()),
            signed_pre_key: Some(x3dh_client.get_spk().await?.into()),
            one_time_key_bundle: Some(x3dh_client.create_opks(100).await?.into()),
        });
        stub.register_pre_key_bundle(request).await?.into_inner();
    }
    Ok(())
}

pub async fn send_message(
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
        &x3dh_client.get_ik(),
        message,
    )?;
    info!("Sending message: {message}");
    let request = tonic::Request::new(SendMessageRequest {
        recipient_identity: Some(recipient_identity.to_owned()),
        message: Some(message.into()),
    });
    stub.send_message(request).await?;
    Ok(())
}
