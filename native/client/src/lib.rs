use anyhow::{Context, Result};
use chacha20poly1305::aead::OsRng;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use ed25519_dalek::SigningKey;
use protocol::bundle::{create_prekey_bundle, sign_bundle};
use protocol::x3dh::{
    x3dh_initiate_recv, x3dh_initiate_send, Message, SignedPreKey, SignedPreKeys,
};
use server::proto::brongnal_client::BrongnalClient;
use server::proto::{
    RegisterPreKeyBundleRequest, RequestPreKeysRequest, RetrieveMessagesRequest, SendMessageRequest,
};
use std::collections::HashMap;
use tonic::transport::Channel;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

pub trait X3DHClient {
    fn fetch_wipe_one_time_secret_key(
        &mut self,
        one_time_key: &X25519PublicKey,
    ) -> Result<X25519StaticSecret, anyhow::Error>;
    fn get_identity_key(&self) -> Result<SigningKey, anyhow::Error>;
    fn get_pre_key(&mut self) -> Result<X25519StaticSecret, anyhow::Error>;
    fn get_spk(&self) -> Result<SignedPreKey, anyhow::Error>;
    fn add_one_time_keys(&mut self, num_keys: u32) -> SignedPreKeys;
}

struct SessionKeys<T> {
    session_keys: HashMap<T, [u8; 32]>,
}

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

pub struct MemoryClient {
    identity_key: SigningKey,
    pre_key: X25519StaticSecret,
    one_time_pre_keys: HashMap<X25519PublicKey, X25519StaticSecret>,
}

impl Default for MemoryClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryClient {
    pub fn new() -> Self {
        Self {
            identity_key: SigningKey::generate(&mut OsRng),
            pre_key: X25519StaticSecret::random_from_rng(OsRng),
            one_time_pre_keys: HashMap::new(),
        }
    }
}

impl X3DHClient for MemoryClient {
    fn fetch_wipe_one_time_secret_key(
        &mut self,
        one_time_key: &X25519PublicKey,
    ) -> Result<X25519StaticSecret> {
        self.one_time_pre_keys
            .remove(one_time_key)
            .context("Client failed to find pre key.")
    }

    fn get_identity_key(&self) -> Result<SigningKey> {
        Ok(self.identity_key.clone())
    }

    fn get_pre_key(&mut self) -> Result<X25519StaticSecret> {
        Ok(self.pre_key.clone())
    }

    fn get_spk(&self) -> Result<SignedPreKey> {
        Ok(SignedPreKey {
            pre_key: X25519PublicKey::from(&self.pre_key),
            signature: sign_bundle(
                &self.identity_key,
                &[(self.pre_key.clone(), X25519PublicKey::from(&self.pre_key))],
            ),
        })
    }

    fn add_one_time_keys(&mut self, num_keys: u32) -> SignedPreKeys {
        let otks = create_prekey_bundle(&self.identity_key, num_keys);
        let pre_keys = otks.bundle.iter().map(|(_, _pub)| *_pub).collect();
        for otk in otks.bundle {
            self.one_time_pre_keys.insert(otk.1, otk.0);
        }
        SignedPreKeys {
            pre_keys,
            signature: otks.signature,
        }
    }
}

pub struct BrongnalUser {
    stub: BrongnalClient<Channel>,
    x3dh_client: MemoryClient,
    name: Option<String>,
}

impl BrongnalUser {
    pub async fn memory_user() -> Result<Self> {
        Ok(BrongnalUser {
            stub: BrongnalClient::connect("https://signal.brongan.com:443").await?,
            x3dh_client: MemoryClient::new(),
            name: None,
        })
    }

    pub async fn register(&mut self, name: &str) -> Result<()> {
        self.name = Some(name.to_owned());
        println!("Registering {name}!");
        let ik = self
            .x3dh_client
            .get_identity_key()?
            .verifying_key()
            .as_bytes()
            .to_vec();
        let spk = self.x3dh_client.get_spk()?;
        let otk_bundle = self.x3dh_client.add_one_time_keys(100);
        let request = tonic::Request::new(RegisterPreKeyBundleRequest {
            ik: Some(ik),
            identity: self.name.clone(),
            spk: Some(spk.into()),
            otk_bundle: Some(otk_bundle.into()),
        });
        self.stub.register_pre_key_bundle(request).await?;
        println!("Registered: {name}!");
        Ok(())
    }

    pub async fn message(&mut self, name: &str, message: &str) -> Result<()> {
        let message = message.as_bytes();
        println!("Messaging {name}.");
        let request = tonic::Request::new(RequestPreKeysRequest {
            identity: Some(name.to_owned()),
        });
        let response = self.stub.request_pre_keys(request).await?;
        let (_sk, message) = x3dh_initiate_send(
            response.into_inner().try_into()?,
            &self.x3dh_client.get_identity_key()?,
            message,
        )?;
        let request = tonic::Request::new(SendMessageRequest {
            recipient_identity: Some(name.to_owned()),
            message: Some(message.into()),
        });
        self.stub.send_message(request).await?;
        println!("Message Sent!");
        Ok(())
    }

    pub async fn get_messages(&mut self) -> Result<()> {
        let response = self
            .stub
            .retrieve_messages(RetrieveMessagesRequest {
                identity: self.name.clone(),
            })
            .await?;
        let messages = response.into_inner().messages;
        println!("Retrieved {} messages.", messages.len());
        for message in messages {
            let Message {
                sender_identity_key,
                ephemeral_key,
                otk,
                ciphertext,
            } = message.try_into()?;
            let otk = if let Some(otk) = otk {
                Some(self.x3dh_client.fetch_wipe_one_time_secret_key(&otk)?)
            } else {
                None
            };
            let (_sk, message) = x3dh_initiate_recv(
                &self.x3dh_client.get_identity_key()?,
                &self.x3dh_client.get_pre_key()?,
                &sender_identity_key,
                ephemeral_key,
                otk,
                &ciphertext,
            )?;
            let message = String::from_utf8(message)?;
            println!("{message}");
        }
        Ok(())
    }
}
