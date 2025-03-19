#![feature(result_flattening)]
use anyhow::Context;
use async_stream::{stream, try_stream};
use blake2::{Blake2b, Digest};
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
pub use client::X3DHClient;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use prost::Message as _;
use proto::application::Message as ApplicationMessageProto;
use proto::gossamer::gossamer_service_client::GossamerServiceClient;
use proto::gossamer::{ActionRequest, GetLedgerRequest, Ledger, SignedMessage};
use proto::service::brongnal_service_client::BrongnalServiceClient;
use proto::service::{
    Message as MessageProto, PreKeyBundleRequest, RegisterPreKeyBundleRequest,
    RetrieveMessagesRequest, SendMessageRequest,
};
use proto::{parse_verifying_key, ApplicationMessage};
use protocol::x3dh::{
    initiate_recv, initiate_send, Message as X3DHMessage, PreKeyBundle, X3DHError,
};
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

type BrongnalClient = BrongnalServiceClient<Channel>;
type GossamerClient = GossamerServiceClient<Channel>;
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

pub struct User {
    brongnal: BrongnalClient,
    gossamer: GossamerClient,
    x3dh: Arc<X3DHClient>,
    username: String,
}

pub struct MessageSubscriber {
    stream: Streaming<MessageProto>,
    ik: SigningKey,
    x3dh: Arc<X3DHClient>,
}

impl MessageSubscriber {
    pub fn into_stream(self) -> impl Stream<Item = ClientResult<ApplicationMessage>> {
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
                    let message: ApplicationMessage = ApplicationMessageProto::decode(&*decrypted)?.try_into()?;
                    Ok::<ApplicationMessage, ClientError>(message)
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
    pub async fn new(
        mut brongnal: BrongnalClient,
        mut gossamer: GossamerClient,
        x3dh: Arc<X3DHClient>,
        username: String,
    ) -> ClientResult<Self> {
        register_username(&mut gossamer, x3dh.get_ik(), username.clone()).await?;
        register_device(&mut brongnal, &x3dh).await?;
        Ok(User {
            brongnal,
            gossamer,
            x3dh,
            username,
        })
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
        })
    }

    pub async fn send_message(&self, peer_username: &str, message: String) -> ClientResult<()> {
        let mut brongnal = self.brongnal.clone();
        let mut gossamer = self.gossamer.clone();
        let message = ApplicationMessage {
            claimed_sender: self.username.clone(),
            text: message,
        };
        let ik = self.x3dh.get_ik();
        let keys = get_keys(&mut gossamer, peer_username).await?;
        let mut bundles = Vec::with_capacity(keys.len());
        for key in keys {
            let recipient = key.as_bytes().to_vec();
            let request = Request::new(PreKeyBundleRequest {
                identity_key: Some(recipient),
            });
            bundles.push(
                brongnal
                    .request_pre_keys(request)
                    .await?
                    .into_inner()
                    .try_into()?,
            );
        }
        let requests = send_message_requests(bundles, ik, message);

        brongnal.send_message(Request::new(requests)).await?;
        Ok(())
    }
}

async fn register_username(
    stub: &mut GossamerServiceClient<Channel>,
    ik: SigningKey,
    name: String,
) -> ClientResult<()> {
    info!("Registering {name}!");
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
        message: Some(signed_message),
    });
    stub.action(request).await?;
    Ok(())
}

async fn register_device(
    stub: &mut BrongnalServiceClient<Channel>,
    x3dh_client: &X3DHClient,
) -> ClientResult<()> {
    let ik = x3dh_client.get_ik().verifying_key().as_bytes().to_vec();
    #[allow(deprecated)]
    let ik_str = base64::encode(&ik);
    info!("Registering {ik_str}!",);

    let request = Request::new(RegisterPreKeyBundleRequest {
        identity_key: Some(ik.clone()),
        signed_pre_key: Some(x3dh_client.get_spk().await?.into()),
        one_time_key_bundle: Some(x3dh_client.create_opks(0).await?.into()),
    });
    let res = stub.register_pre_key_bundle(request).await?.into_inner();
    info!("Registered. {} keys remaining!", res.num_keys());
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

fn send_message_requests(
    bundles: Vec<PreKeyBundle>,
    ik: SigningKey,
    message: ApplicationMessage,
) -> impl Stream<Item = SendMessageRequest> {
    let message: ApplicationMessageProto = message.into();
    stream! {
        for bundle in bundles {
            let recipient_identity_key = Some(bundle.ik.as_bytes().to_vec());
            let (_sk, message) = match initiate_send(
                bundle,
                &ik,
                &message.encode_to_vec(),
            ) {
                Ok((sk, message)) => (sk, message),
                Err(e) => {
                    error!("Failed to x3dh::initiate_send: {e}");
                    continue
                },
            };

            info!("Sending message:\n{message}\n");

            yield SendMessageRequest {
                recipient_identity_key,
                message: Some(message.into()),
            };
        }
    }
}

async fn get_ledger(stub: &mut GossamerServiceClient<Channel>) -> ClientResult<Ledger> {
    let request = Request::new(GetLedgerRequest {});
    let ledger = stub.get_ledger(request).await?.into_inner();
    Ok(ledger)
}

async fn get_keys(
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
        .flat_map(|user| {
            user.public_keys
                .into_iter()
                .map(|key| parse_verifying_key(&key).unwrap())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>())
}
