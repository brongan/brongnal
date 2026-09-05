#![feature(trivial_bounds)]
#![feature(iterator_try_collect)]
use anyhow::Context;
use async_stream::try_stream;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use client::MessageState;
pub use client::X3DHClient;
use ed25519_dalek::{SigningKey, VerifyingKey};
use prost::Message as _;
use proto::application::Message as ApplicationMessageProto;
use proto::application::RatchetMessage as RatchetProto;
use proto::service::brongnal_service_client::BrongnalServiceClient;
use proto::service::{
    IdentityKeyRequest, Message as MessageProto, PreKeyBundleRequest, RegisterPreKeyBundleRequest,
    RetrieveMessagesRequest, SendMessageRequest,
};
use proto::{parse_verifying_key, ApplicationMessage, RatchetMessage};
use protocol::x3dh::{initiate_recv, initiate_send, Message as X3DHMessage, X3DHError};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use tonic::{Request, Streaming};
use tracing::{error, info, warn};

use crate::client::MessageModel;

pub mod client;

type BrongnalClient = BrongnalServiceClient<Channel>;
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
    Send(#[from] SendError<ApplicationMessageProto>),
    #[error("x3dh error: {0}")]
    X3DH(#[from] X3DHError),
    #[error("decode error: {0}")]
    Decode(#[from] prost::DecodeError),
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

#[derive(Clone)]
pub struct User {
    brongnal: BrongnalClient,
    x3dh: Arc<X3DHClient>,
    username: String,
}

pub struct MessageSubscriber {
    stream: Streaming<MessageProto>,
    ik: SigningKey,
    x3dh: Arc<X3DHClient>,
    brongnal: BrongnalClient,
    username: String,
}

impl MessageSubscriber {
    pub fn into_stream(mut self) -> impl Stream<Item = ClientResult<MessageModel>> {
        try_stream! {
            let messages = into_message_stream(self.stream);
            tokio::pin!(messages);
            while let Some(message) = messages.next().await {
                if let Err(e) = message {
                    warn!("Message was not validly serialized: {e}");
                    continue;
                }
                let message = message.unwrap();
                let res = {
                    let opk = if let Some(opk) = message.opk {
                        Some(self.x3dh.fetch_wipe_opk(opk).await?)
                    } else {
                        None
                    };
                    // TODO: Caller must delete the session keys with the peer on an error.
                    let (_sk, decrypted) = initiate_recv(
                        &self.ik,
                        &self.x3dh.get_pre_key(message.pre_key).await?,
                        &message.ik,
                        message.ek,
                        opk,
                        &message.ciphertext,
                    )?;
                    // TODO: Handle the ratchet header.
                    let ratchet_message: RatchetMessage = RatchetProto::decode(&*decrypted)?.try_into()?;
                    let sender_key = match get_identity_key(&mut self.brongnal, &ratchet_message.message.sender).await {
                        Ok(key) => key,
                        Err(e) => {
                            warn!("Failed to look up claimed sender: {e}");
                            continue;
                        }
                    };
                    if sender_key != message.ik {
                        warn!("Message failed username validation. Claimed sender: {}", &ratchet_message.message.sender);
                        continue;
                    }
                    let application_message = ratchet_message.message;
                    let ApplicationMessage {
                        sender,
                        text
                    } = application_message;
                    let _id = self.x3dh.persist_message(sender.clone(), self.username.clone(), text.clone(), MessageState::Delivered).await?;
                    Ok::<MessageModel, ClientError>(MessageModel {
                        sender,
                        receiver: self.username.clone(),
                        db_recv_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
                        state: MessageState::Delivered,
                        text,
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
}

impl User {
    /// Create a new User with lazy gRPC connections.
    /// The underlying Channel connects on first RPC and auto-reconnects on failure.
    #[tracing::instrument(skip(x3dh))]
    pub fn new(addr: String, x3dh: Arc<X3DHClient>, username: String) -> ClientResult<Self> {
        let channel = tonic::transport::Endpoint::from_shared(addr)
            .map_err(|e| ClientError::Grpc(tonic::Status::unavailable(e.to_string())))?
            .connect_lazy();
        let brongnal = BrongnalClient::new(channel);
        Ok(User {
            brongnal,
            x3dh,
            username,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn register(&mut self, fcm_token: Option<String>) -> ClientResult<()> {
        let mut brongnal = self.brongnal.clone();
        register_device(&mut brongnal, &self.x3dh, &self.username, fcm_token).await?;
        Ok(())
    }

    pub async fn get_messages(&self) -> ClientResult<MessageSubscriber> {
        let mut brongnal = self.brongnal.clone();
        let ik = self.x3dh.get_ik();
        let stream = brongnal
            .retrieve_messages(RetrieveMessagesRequest {
                identity_key: Some(ik.verifying_key().as_bytes().to_vec()),
            })
            .await?
            .into_inner();
        Ok(MessageSubscriber {
            stream,
            ik,
            x3dh: self.x3dh.clone(),
            brongnal,
            username: self.username.clone(),
        })
    }

    pub async fn send_message(&self, peer_username: String, message: String) -> ClientResult<i64> {
        let mut brongnal = self.brongnal.clone();
        // TODO: Create Ratchet Header
        let ratchet_message = RatchetMessage {
            header: None,
            message: ApplicationMessage {
                sender: self.username.clone(),
                text: message.clone(),
            },
        };
        let ik = self.x3dh.get_ik();
        let recipient = get_identity_key(&mut brongnal, &peer_username).await?;
        let bundle = brongnal
            .request_pre_keys(PreKeyBundleRequest {
                identity_key: Some(recipient.to_bytes().to_vec()),
            })
            .await?
            .into_inner()
            .try_into()?;
        let ratchet_message: RatchetProto = ratchet_message.into();
        let (_, encrypted) = initiate_send(bundle, &ik, &ratchet_message.encode_to_vec())?;
        let request = SendMessageRequest {
            recipient_identity_key: Some(recipient.to_bytes().to_vec()),
            message: Some(encrypted.into()),
        };

        let row_id = self
            .x3dh
            .persist_message(
                self.username.clone(),
                peer_username,
                message,
                MessageState::Sending,
            )
            .await?;
        brongnal.send_message(tokio_stream::once(request)).await?;
        self.x3dh
            .persist_message_state(row_id, MessageState::Sent)
            .await?;
        Ok(row_id)
    }

    pub async fn get_message(&self, id: i64) -> ClientResult<MessageModel> {
        self.x3dh.get_message(id).await
    }

    pub async fn get_message_history(&self) -> ClientResult<Vec<MessageModel>> {
        self.x3dh.get_messages().await
    }
}

async fn register_device(
    stub: &mut BrongnalClient,
    x3dh_client: &X3DHClient,
    username: &str,
    fcm_token: Option<String>,
) -> ClientResult<()> {
    let ik = x3dh_client.get_ik().verifying_key().as_bytes().to_vec();
    #[allow(deprecated)]
    let ik_str = base64::encode(&ik);
    info!("Registering {ik_str}!",);

    let request = Request::new(RegisterPreKeyBundleRequest {
        identity_key: Some(ik.clone()),
        signed_pre_key: Some(x3dh_client.get_spk().await?.into()),
        one_time_key_bundle: Some(x3dh_client.create_opks(0).await?.into()),
        fcm_token,
        username: Some(username.to_owned()),
    });
    let res = stub.register_pre_key_bundle(request).await?.into_inner();
    info!("Registered. {} keys remaining!", res.num_keys());
    if res.num_keys() < 100 {
        info!("Adding 100 keys!");
        let request = Request::new(RegisterPreKeyBundleRequest {
            identity_key: Some(ik),
            signed_pre_key: Some(x3dh_client.get_spk().await?.into()),
            one_time_key_bundle: Some(x3dh_client.create_opks(100).await?.into()),
            fcm_token: None,
            username: Some(username.to_owned()),
        });
        stub.register_pre_key_bundle(request).await?.into_inner();
    }
    Ok(())
}

async fn get_identity_key(stub: &mut BrongnalClient, username: &str) -> ClientResult<VerifyingKey> {
    let response = stub
        .get_identity_key(IdentityKeyRequest {
            username: Some(username.to_owned()),
        })
        .await?
        .into_inner();
    parse_verifying_key(response.identity_key())
        .map_err(|_| ClientError::Grpc(tonic::Status::data_loss("invalid directory identity key")))
}
