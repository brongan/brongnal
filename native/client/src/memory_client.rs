use crate::{ClientError, ClientResult, X3DHClient};
use async_trait::async_trait;
use chacha20poly1305::aead::OsRng;
use ed25519_dalek::SigningKey;
use protocol::bundle::{create_prekey_bundle, sign_bundle};
use protocol::x3dh;
use std::collections::HashMap;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};
use x3dh::{SignedPreKey, SignedPreKeys};

pub struct MemoryClient {
    ik: SigningKey,
    pre_key: X25519StaticSecret,
    opks: HashMap<X25519PublicKey, X25519StaticSecret>,
}

impl Default for MemoryClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryClient {
    pub fn new() -> Self {
        Self {
            ik: SigningKey::generate(&mut OsRng),
            pre_key: X25519StaticSecret::random_from_rng(OsRng),
            opks: HashMap::new(),
        }
    }
}

#[async_trait]
impl X3DHClient for MemoryClient {
    async fn fetch_wipe_opk(&mut self, opk: X25519PublicKey) -> ClientResult<X25519StaticSecret> {
        #[allow(deprecated)]
        self.opks
            .remove(&opk)
            .ok_or_else(|| ClientError::WipeOpk(base64::encode(opk.to_bytes())))
    }

    async fn get_ik(&self) -> ClientResult<SigningKey> {
        Ok(self.ik.clone())
    }

    async fn get_pre_key(&self, _pre_key: X25519PublicKey) -> ClientResult<X25519StaticSecret> {
        Ok(self.pre_key.clone())
    }

    async fn get_spk(&self) -> ClientResult<SignedPreKey> {
        Ok(SignedPreKey {
            pre_key: X25519PublicKey::from(&self.pre_key),
            signature: sign_bundle(
                &self.ik,
                &[(self.pre_key.clone(), X25519PublicKey::from(&self.pre_key))],
            ),
        })
    }

    async fn create_opks(&mut self, num_keys: u32) -> ClientResult<SignedPreKeys> {
        let opks = create_prekey_bundle(&self.ik, num_keys);
        let pre_keys = opks.bundle.iter().map(|(_, _pub)| *_pub).collect();
        for opk in opks.bundle {
            self.opks.insert(opk.1, opk.0);
        }
        Ok(SignedPreKeys {
            pre_keys,
            signature: opks.signature,
        })
    }
}
