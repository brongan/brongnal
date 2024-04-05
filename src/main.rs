#![feature(map_try_insert)]
#![feature(trait_upcasting)]
#![allow(dead_code)]
use crate::bundle::*;
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
mod x3dh;

type Identity = String;

trait X3DHServer {
    // Bob publishes a set of elliptic curve public keys to the server, containing:
    //    Bob's identity key IKB
    //    Bob's signed prekey SPKB
    //    Bob's prekey signature Sig(IKB, Encode(SPKB))
    //    A set of Bob's one-time prekeys (OPKB1, OPKB2, OPKB3, ...)
    fn set_spk(&mut self, identity: Identity, ik: VerifyingKey, spk: SignedPreKey) -> Result<()>;
    fn publish_otk_bundle(
        &mut self,
        identity: Identity,
        ik: VerifyingKey,
        otk_bundle: SignedPreKeys,
    ) -> Result<()>;

    // To perform an X3DH key agreement with Bob, Alice contacts the server and fetches a "prekey bundle" containing the following values:
    //    Bob's identity key IKB
    //    Bob's signed prekey SPKB
    //    Bob's prekey signature Sig(IKB, Encode(SPKB))
    //    (Optionally) Bob's one-time prekey OPKB
    fn fetch_prekey_bundle(&mut self, recipient_identity: &Identity) -> Result<PreKeyBundle>;

    // The server can store messages from Alice to Bob which Bob can later retrieve.
    fn send_message(&mut self, recipient_identity: &Identity, message: Message) -> Result<()>;
    fn retrieve_messages(&mut self, identity: &Identity) -> Vec<Message>;
}

struct InMemoryServer {
    identity_key: HashMap<Identity, VerifyingKey>,
    current_pre_key: HashMap<Identity, SignedPreKey>,
    one_time_pre_keys: HashMap<Identity, Vec<X25519PublicKey>>,
    messages: HashMap<Identity, Vec<Message>>,
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

impl X3DHServer for InMemoryServer {
    fn set_spk(&mut self, identity: Identity, ik: VerifyingKey, spk: SignedPreKey) -> Result<()> {
        verify_bundle(&ik, &[spk.pre_key], &spk.signature)?;
        self.identity_key.insert(identity.clone(), ik);
        self.current_pre_key.insert(identity, spk);
        Ok(())
    }

    fn publish_otk_bundle(
        &mut self,
        identity: Identity,
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

    fn fetch_prekey_bundle(&mut self, recipient_identity: &Identity) -> Result<PreKeyBundle> {
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

    fn send_message(&mut self, recipient_identity: &Identity, message: Message) -> Result<()> {
        let _ = self
            .messages
            .try_insert(recipient_identity.clone(), Vec::new());
        self.messages
            .get_mut(recipient_identity)
            .unwrap()
            .push(message);
        Ok(())
    }

    fn retrieve_messages(&mut self, identity: &Identity) -> Vec<Message> {
        self.messages.remove(identity).unwrap_or(Vec::new())
    }
}

trait OTKManager {
    fn fetch_wipe_one_time_secret_key(
        &mut self,
        one_time_key: &X25519PublicKey,
    ) -> Result<X25519StaticSecret>;
}

trait KeyManager {
    fn get_identity_key(&self) -> Result<SigningKey>;
    fn get_pre_key(&mut self) -> Result<X25519StaticSecret>;
    fn get_spk(&self) -> Result<SignedPreKey>;
}

trait SessionKeyManager {
    fn set_session_key(&mut self, recipient_identity: Identity, secret_key: &[u8; 32]);
    fn get_encryption_key(&mut self, recipient_identity: &Identity) -> Result<ChaCha20Poly1305>;
    fn destroy_session_key(&mut self, peer: &Identity);
}

trait Client: OTKManager + KeyManager + SessionKeyManager {
    fn add_one_time_keys(&mut self, num_keys: u32) -> SignedPreKeys;
}

struct InMemoryClient {
    identity_key: SigningKey,
    pre_key: X25519StaticSecret,
    one_time_pre_keys: HashMap<X25519PublicKey, X25519StaticSecret>,
    session_keys: HashMap<Identity, [u8; 32]>,
}

impl InMemoryClient {
    fn new() -> Self {
        Self {
            identity_key: SigningKey::generate(&mut OsRng),
            pre_key: X25519StaticSecret::random_from_rng(&mut OsRng),
            one_time_pre_keys: HashMap::new(),
            session_keys: HashMap::new(),
        }
    }
}

impl OTKManager for InMemoryClient {
    fn fetch_wipe_one_time_secret_key(
        &mut self,
        one_time_key: &X25519PublicKey,
    ) -> Result<X25519StaticSecret> {
        self.one_time_pre_keys
            .remove(&one_time_key)
            .context("Client failed to find pre key.")
    }
}

impl KeyManager for InMemoryClient {
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

impl Client for InMemoryClient {
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

impl SessionKeyManager for InMemoryClient {
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

fn main() {}
