use anyhow::{Context, Result};
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use ed25519_dalek::SigningKey;
use proto::service::brongnal_client::BrongnalClient;
use proto::service::{
    Message as MessageProto, RegisterPreKeyBundleRequest, RequestPreKeysRequest,
    RetrieveMessagesRequest, SendMessageRequest,
};
use protocol::x3dh;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::Streaming;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};
use x3dh::{initiate_recv, initiate_send, SignedPreKey, SignedPreKeys};

pub mod memory_client;
pub mod sqlite_client;

pub trait X3DHClient {
    fn fetch_wipe_one_time_secret_key(
        &mut self,
        one_time_key: &X25519PublicKey,
    ) -> Result<X25519StaticSecret, anyhow::Error>;
    fn get_identity_key(&self) -> Result<SigningKey, anyhow::Error>;
    fn get_pre_key(&self) -> Result<X25519StaticSecret, anyhow::Error>;
    fn get_spk(&self) -> Result<SignedPreKey, anyhow::Error>;
    fn add_one_time_keys(&mut self, num_keys: u32) -> Result<SignedPreKeys>;
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

    fn get_encryption_key(&mut self, recipient_identity: &Identity) -> Result<ChaCha20Poly1305> {
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

pub async fn listen(
    mut stub: BrongnalClient<Channel>,
    x3dh_client: Arc<Mutex<dyn X3DHClient + Send>>,
    name: String,
    tx: Sender<DecryptedMessage>,
) -> Result<()> {
    let stream = stub
        .retrieve_messages(RetrieveMessagesRequest {
            identity: Some(name),
        })
        .await;
    if let Err(e) = &stream {
        eprintln!("Failed to retrieve messages: {e}");
    }
    if let Err(e) = get_messages(stream?.into_inner(), x3dh_client, tx).await {
        eprintln!("get_messages terminated with: {e}");
        return Err(e);
    }
    Ok(())
}

pub async fn register(
    stub: &mut BrongnalClient<Channel>,
    x3dh_client: Arc<Mutex<dyn X3DHClient + Send>>,
    name: String,
) -> Result<()> {
    eprintln!("Registering {name}!");
    let request = {
        let mut x3dh_client = x3dh_client.lock().await;
        let ik = x3dh_client
            .get_identity_key()?
            .verifying_key()
            .as_bytes()
            .to_vec();
        tonic::Request::new(RegisterPreKeyBundleRequest {
            identity_key: Some(ik),
            identity: Some(name.clone()),
            signed_pre_key: Some(x3dh_client.get_spk()?.into()),
            one_time_key_bundle: Some(x3dh_client.add_one_time_keys(100)?.into()),
        })
    };
    stub.register_pre_key_bundle(request).await?;
    eprintln!("Registered: {}!", name);
    Ok(())
}

pub async fn message(
    stub: &mut BrongnalClient<Channel>,
    x3dh_client: Arc<Mutex<dyn X3DHClient + Send>>,
    sender_identity: String,
    recipient_identity: &str,
    message: &str,
) -> Result<()> {
    let message = message.as_bytes();
    let request = tonic::Request::new(RequestPreKeysRequest {
        identity: Some(recipient_identity.to_owned()),
    });
    let response = stub.request_pre_keys(request).await?;
    let (_sk, message) = initiate_send(
        response.into_inner().try_into()?,
        sender_identity,
        &x3dh_client.lock().await.get_identity_key()?,
        message,
    )?;
    let request = tonic::Request::new(SendMessageRequest {
        recipient_identity: Some(recipient_identity.to_owned()),
        message: Some(message.into()),
    });
    stub.send_message(request).await?;
    Ok(())
}

// TODO(https://github.com/brongan/brongnal/issues/23) - Replace with stream of decrypted messages.
// TODO(https://github.com/brongan/brongnal/issues/24) - Avoid blocking sqlite calls from async.
pub async fn get_messages(
    mut stream: Streaming<MessageProto>,
    x3dh_client: Arc<Mutex<dyn X3DHClient + Send>>,
    tx: Sender<DecryptedMessage>,
) -> Result<()> {
    while let Some(message) = stream.message().await? {
        let x3dh::Message {
            sender_identity,
            sender_identity_key,
            ephemeral_key,
            one_time_key: otk,
            ciphertext,
        } = message.try_into()?;
        let mut x3dh_client = x3dh_client.lock().await;
        let otk = if let Some(otk) = otk {
            // TODO(#28) - Handle a missing one-time prekey.
            Some(x3dh_client.fetch_wipe_one_time_secret_key(&otk)?)
        } else {
            None
        };
        let (_sk, message) = initiate_recv(
            &x3dh_client.get_identity_key()?,
            &x3dh_client.get_pre_key()?,
            &sender_identity_key,
            ephemeral_key,
            otk,
            &ciphertext,
        )?;
        tx.send(DecryptedMessage {
            sender_identity,
            message,
        })
        .await?;
    }
    eprintln!("Server terminated message stream.");
    Ok(())
}
