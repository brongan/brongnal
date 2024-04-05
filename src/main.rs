#![feature(map_try_insert)]
#![feature(trait_upcasting)]
#![allow(dead_code)]
use crate::bundle::*;
use crate::traits::{Client, KeyManager, OTKManager, SessionKeyManager, X3DHServer};
use crate::x3dh::*;
use anyhow::{Context, Result};
use blake2::{Blake2b512, Digest};
use chacha20poly1305::aead::OsRng;
use chacha20poly1305::{aead::KeyInit, ChaCha20Poly1305};
use ed25519_dalek::{SigningKey, VerifyingKey};
use std::collections::HashMap;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

mod aead;
mod bundle;
mod traits;
mod x3dh;

struct InMemoryServer {
    identity_key: HashMap<String, VerifyingKey>,
    current_pre_key: HashMap<String, SignedPreKey>,
    one_time_pre_keys: HashMap<String, Vec<X25519PublicKey>>,
    messages: HashMap<String, Vec<Message>>,
}

impl InMemoryServer {
    fn new() -> Self {
        InMemoryServer {
            identity_key: HashMap::new(),
            current_pre_key: HashMap::new(),
            one_time_pre_keys: HashMap::new(),
            messages: HashMap::new(),
        }
    }
}

impl X3DHServer<String> for InMemoryServer {
    fn set_spk(&mut self, identity: String, ik: VerifyingKey, spk: SignedPreKey) -> Result<()> {
        verify_bundle(&ik, &[spk.pre_key], &spk.signature)?;
        self.identity_key.insert(identity.clone(), ik);
        self.current_pre_key.insert(identity, spk);
        Ok(())
    }

    fn publish_otk_bundle(
        &mut self,
        identity: String,
        ik: VerifyingKey,
        otk_bundle: SignedPreKeys,
    ) -> Result<()> {
        verify_bundle(&ik, &otk_bundle.pre_keys, &otk_bundle.signature)?;
        let _ = self
            .one_time_pre_keys
            .try_insert(identity.clone(), Vec::new());
        self.one_time_pre_keys
            .get_mut(&identity)
            .unwrap()
            .extend(otk_bundle.pre_keys);
        Ok(())
    }

    fn fetch_prekey_bundle(&mut self, recipient_identity: &String) -> Result<PreKeyBundle> {
        let identity_key = self
            .identity_key
            .get(recipient_identity)
            .context("Server has IK.")?
            .clone();
        let spk = self
            .current_pre_key
            .get(recipient_identity)
            .context("Server has spk.")?
            .clone();
        let otk = if let Some(otks) = self.one_time_pre_keys.get_mut(recipient_identity) {
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

    fn send_message(&mut self, recipient_identity: &String, message: Message) -> Result<()> {
        let _ = self
            .messages
            .try_insert(recipient_identity.clone(), Vec::new());
        self.messages
            .get_mut(recipient_identity)
            .unwrap()
            .push(message);
        Ok(())
    }

    fn retrieve_messages(&mut self, identity: &String) -> Vec<Message> {
        self.messages.remove(identity).unwrap_or(Vec::new())
    }
}

struct InMemoryClient<String> {
    identity_key: SigningKey,
    pre_key: X25519StaticSecret,
    one_time_pre_keys: HashMap<X25519PublicKey, X25519StaticSecret>,
    session_keys: HashMap<String, [u8; 32]>,
}

impl InMemoryClient<String> {
    fn new() -> Self {
        Self {
            identity_key: SigningKey::generate(&mut OsRng),
            pre_key: X25519StaticSecret::random_from_rng(&mut OsRng),
            one_time_pre_keys: HashMap::new(),
            session_keys: HashMap::new(),
        }
    }
}

impl OTKManager for InMemoryClient<String> {
    fn fetch_wipe_one_time_secret_key(
        &mut self,
        one_time_key: &X25519PublicKey,
    ) -> Result<X25519StaticSecret> {
        self.one_time_pre_keys
            .remove(&one_time_key)
            .context("Client failed to find pre key.")
    }
}

impl KeyManager for InMemoryClient<String> {
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
}

impl Client<String> for InMemoryClient<String> {
    fn add_one_time_keys(&mut self, num_keys: u32) -> SignedPreKeys {
        let otks = create_prekey_bundle(&self.identity_key, num_keys);
        let pre_keys = otks.bundle.iter().map(|(_, _pub)| _pub.clone()).collect();
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
    hasher.update(&key);
    let blake2b_mac = hasher.finalize();
    let mut l = [0; 32];
    let mut r = [0; 32];
    l.clone_from_slice(&blake2b_mac[0..32]);
    r.clone_from_slice(&blake2b_mac[32..]);
    (l, r)
}

impl SessionKeyManager<String> for InMemoryClient<String> {
    fn set_session_key(&mut self, recipient_identity: String, secret_key: &[u8; 32]) {
        self.session_keys.insert(recipient_identity, *secret_key);
    }

    fn get_encryption_key(&mut self, recipient_identity: &String) -> Result<ChaCha20Poly1305> {
        let key = self
            .session_keys
            .get(recipient_identity)
            .context("Session key not found.")?;
        Ok(ChaCha20Poly1305::new_from_slice(key).unwrap())
    }

    fn destroy_session_key(&mut self, peer: &String) {
        self.session_keys.remove(peer);
    }
}

fn main() {}
