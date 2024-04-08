#![feature(map_try_insert)]
#![feature(trait_upcasting)]
#![allow(dead_code)]
use crate::bundle::*;
use crate::traits::{X3DHClient, X3DHServer};
use crate::x3dh::*;
use anyhow::{Context, Result};
use blake2::{Blake2b512, Digest};
use chacha20poly1305::aead::OsRng;
use chacha20poly1305::{aead::KeyInit, ChaCha20Poly1305};
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tarpc::context;
use thiserror::Error;
use tokio::sync::Mutex;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

mod aead;
pub mod bundle;
pub mod traits;
pub mod x3dh;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum BrongnalServerError {
    #[error("Error Running X3DH.")]
    X3DHError(#[from] X3DHError),
    #[error("Signature failed to validate.")]
    SignatureValidation,
    #[error("User is not registered.")]
    PreconditionError,
}

#[derive(Clone)]
pub struct MemoryServer {
    identity_key: Arc<Mutex<HashMap<String, VerifyingKey>>>,
    current_pre_key: Arc<Mutex<HashMap<String, SignedPreKey>>>,
    one_time_pre_keys: Arc<Mutex<HashMap<String, Vec<X25519PublicKey>>>>,
    messages: Arc<Mutex<HashMap<String, Vec<Message>>>>,
}

impl Default for MemoryServer {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryServer {
    pub fn new() -> Self {
        MemoryServer {
            identity_key: Arc::new(Mutex::new(HashMap::new())),
            current_pre_key: Arc::new(Mutex::new(HashMap::new())),
            one_time_pre_keys: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn spawn(fut: impl futures::Future<Output = ()> + Send + 'static) {
        tokio::spawn(fut);
    }
}

impl X3DHServer for MemoryServer {
    async fn set_spk(
        self,
        _: context::Context,
        identity: String,
        ik: VerifyingKey,
        spk: SignedPreKey,
    ) -> Result<(), BrongnalServerError> {
        verify_bundle(&ik, &[spk.pre_key], &spk.signature)
            .map_err(|_| BrongnalServerError::SignatureValidation)?;
        self.identity_key.lock().await.insert(identity.clone(), ik);
        self.current_pre_key.lock().await.insert(identity, spk);
        Ok(())
    }

    async fn publish_otk_bundle(
        self,
        _: context::Context,
        identity: String,
        ik: VerifyingKey,
        otk_bundle: SignedPreKeys,
    ) -> Result<(), BrongnalServerError> {
        verify_bundle(&ik, &otk_bundle.pre_keys, &otk_bundle.signature)
            .map_err(|_| BrongnalServerError::SignatureValidation)?;
        let mut one_time_pre_keys = self.one_time_pre_keys.lock().await;
        let _ = one_time_pre_keys.try_insert(identity.clone(), Vec::new());
        one_time_pre_keys
            .get_mut(&identity)
            .unwrap()
            .extend(otk_bundle.pre_keys);
        Ok(())
    }

    async fn fetch_prekey_bundle(
        self,
        _: context::Context,
        recipient_identity: String,
    ) -> Result<PreKeyBundle, BrongnalServerError> {
        let identity_key = *self
            .identity_key
            .lock()
            .await
            .get(&recipient_identity)
            .ok_or(BrongnalServerError::PreconditionError)?;
        let spk = self
            .current_pre_key
            .lock()
            .await
            .get(&recipient_identity)
            .ok_or(BrongnalServerError::PreconditionError)?
            .clone();
        let otk = if let Some(otks) = self
            .one_time_pre_keys
            .lock()
            .await
            .get_mut(&recipient_identity)
        {
            otks.pop()
        } else {
            None
        };

        Ok(PreKeyBundle {
            identity_key,
            otk,
            spk,
        })
    }

    async fn send_message(
        self,
        _: context::Context,
        recipient_identity: String,
        message: Message,
    ) -> Result<(), BrongnalServerError> {
        let mut messages = self.messages.lock().await;
        let _ = messages.try_insert(recipient_identity.clone(), Vec::new());
        messages.get_mut(&recipient_identity).unwrap().push(message);
        Ok(())
    }

    async fn retrieve_messages(self, _: context::Context, identity: String) -> Vec<Message> {
        self.messages
            .lock()
            .await
            .remove(&identity)
            .unwrap_or(Vec::new())
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

fn ratchet(key: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let mut hasher = Blake2b512::new();
    hasher.update(key);
    let blake2b_mac = hasher.finalize();
    let mut l = [0; 32];
    let mut r = [0; 32];
    l.clone_from_slice(&blake2b_mac[0..32]);
    r.clone_from_slice(&blake2b_mac[32..]);
    (l, r)
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
