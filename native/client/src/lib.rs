use anyhow::Context;
use async_stream::try_stream;
use blake2::{Blake2b, Digest};
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
pub use client::X3DHClient;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use prost::Message as _;
use proto::gossamer::gossamer_service_client::GossamerServiceClient;
use proto::gossamer::{ActionRequest, GetLedgerRequest, Ledger, SignedMessage};
use proto::parse_verifying_key;
use proto::service::brongnal_service_client::BrongnalServiceClient;
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
use tonic::{Request, Streaming};
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
    mut stub: BrongnalServiceClient<Channel>,
    x3dh_client: Arc<X3DHClient>,
    key: VerifyingKey,
) -> impl Stream<Item = ClientResult<DecryptedMessage>> {
    try_stream! {
        let stream = stub
            .retrieve_messages(RetrieveMessagesRequest {
                identity_key: Some(key.to_bytes().to_vec()),
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
                    &x3dh_client.get_ik().await?,
                    &x3dh_client.get_pre_key(message.pre_key).await?,
                    &message.ik,
                    message.ek,
                    opk,
                    &message.ciphertext,
                )?;
                Ok::<DecryptedMessage, ClientError>(DecryptedMessage {
                    // TODO(https://github.com/brongan/brongnal/issues/15): Don't blindly trust the
                    // sender's claimed identity.
                    sender_identity: String::from(""),
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

pub async fn register_username(
    stub: &mut GossamerServiceClient<Channel>,
    ik: SigningKey,
    name: String,
) -> ClientResult<()> {
    let provider = Blake2b::<blake2::digest::typenum::U32>::digest(name.as_bytes()).to_vec();

    let message = protocol::gossamer::Message {
        provider,
        public_key: ik.verifying_key(),
        action: protocol::gossamer::Action::AppendKey,
    };
    let message: proto::gossamer::Message = message.into();
    let contents = message.encode_to_vec();
    let signature = ik.sign(&contents);

    let signed_message = SignedMessage {
        contents: Some(contents),
        identity_key: Some(ik.verifying_key().as_bytes().to_vec()),
        signature: Some(signature.to_vec()),
    };
    let request = Request::new(ActionRequest {
        message: Some(signed_message.into()),
    });
    stub.action(request).await?;
    Ok(())
}

pub async fn register_device(
    stub: &mut BrongnalServiceClient<Channel>,
    x3dh_client: &X3DHClient,
    name: String,
) -> ClientResult<()> {
    info!("Registering {name}!");
    let ik = x3dh_client
        .get_ik()
        .await?
        .verifying_key()
        .as_bytes()
        .to_vec();

    let request = Request::new(RegisterPreKeyBundleRequest {
        identity_key: Some(ik.clone()),
        signed_pre_key: Some(x3dh_client.get_spk().await?.into()),
        one_time_key_bundle: Some(x3dh_client.create_opks(0).await?.into()),
    });
    let res = stub.register_pre_key_bundle(request).await?.into_inner();
    info!("Registered: {}. {} keys remaining!", name, res.num_keys());
    if res.num_keys() < 100 {
        info!("Adding 100 keys!");
        let request = Request::new(RegisterPreKeyBundleRequest {
            identity_key: Some(ik),
            signed_pre_key: Some(x3dh_client.get_spk().await?.into()),
            one_time_key_bundle: Some(x3dh_client.create_opks(100).await?.into()),
        });
        stub.register_pre_key_bundle(request).await?.into_inner();
    }
    Ok(())
}

// Get the key from the mapping.
// Get the PrekeyBundle for the key.
// Send da message
pub async fn send_message(
    stub: &mut BrongnalServiceClient<Channel>,
    x3dh_client: &X3DHClient,
    sender_identity: String,
    recipient: &VerifyingKey,
    message: &str,
) -> ClientResult<()> {
    let message = message.as_bytes();
    let sender_identity: Vec<u8> =
        Blake2b::<blake2::digest::typenum::U32>::digest(sender_identity.as_bytes()).to_vec();
    let request = Request::new(RequestPreKeysRequest {
        identity_key: Some(recipient.to_bytes().to_vec()),
    });
    let response = stub.request_pre_keys(request).await?.into_inner();
    for bundle in response.bundles {
        let (_sk, message) =
            initiate_send(bundle.try_into()?, &x3dh_client.get_ik().await?, message)?;
        info!("Sending message: {message}");
        let request = Request::new(SendMessageRequest {
            claimed_sender_identity: Some(sender_identity.clone()),
            recipient_identity_key: Some(recipient.to_bytes().to_vec()),
            message: Some(message.into()),
        });
        stub.send_message(request).await?;
    }
    Ok(())
}

pub async fn get_ledger(stub: &mut GossamerServiceClient<Channel>) -> ClientResult<Ledger> {
    let request = Request::new(GetLedgerRequest {});
    let ledger = stub.get_ledger(request).await?.into_inner();
    Ok(ledger)
}

pub async fn get_keys(
    stub: &mut GossamerServiceClient<Channel>,
    peer_username: &str,
) -> ClientResult<Vec<VerifyingKey>> {
    let recipient_user_id =
        Blake2b::<blake2::digest::typenum::U32>::digest(peer_username.as_bytes()).to_vec();
    let ledger = get_ledger(stub).await?;
    Ok(ledger
        .users
        .into_iter()
        .filter(|user| user.provider() == recipient_user_id)
        .map(|user| {
            user.public_keys
                .into_iter()
                .map(|key| parse_verifying_key(&key).unwrap())
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>())
}
