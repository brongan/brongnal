use crate::X3DHClient;
use anyhow::{Context, Result};
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

impl X3DHClient for MemoryClient {
    fn fetch_wipe_opk(
        &mut self,
        opk: &X25519PublicKey,
    ) -> Result<X25519StaticSecret> {
        self.opks
            .remove(opk)
            .context("Client failed to find pre key.")
    }

    fn get_ik(&self) -> Result<SigningKey> {
        Ok(self.ik.clone())
    }

    fn get_pre_key(&self) -> Result<X25519StaticSecret> {
        Ok(self.pre_key.clone())
    }

    fn get_spk(&self) -> Result<SignedPreKey> {
        Ok(SignedPreKey {
            pre_key: X25519PublicKey::from(&self.pre_key),
            signature: sign_bundle(
                &self.ik,
                &[(self.pre_key.clone(), X25519PublicKey::from(&self.pre_key))],
            ),
        })
    }

    fn create_opks(&mut self, num_keys: u32) -> Result<SignedPreKeys> {
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
