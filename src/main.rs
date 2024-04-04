#![feature(map_try_insert)]
#![feature(trait_upcasting)]
#![allow(dead_code)]
use crate::aead::{decrypt_data, encrypt_data};
use anyhow::{anyhow, Context, Result};
use blake2::{Blake2b512, Digest};
use chacha20poly1305::{
    aead::{KeyInit, Payload},
    ChaCha20Poly1305,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hex_literal::hex;
use hkdf::Hkdf;
use sha2::Sha256;
use std::collections::HashMap;
use x25519_dalek::{
    PublicKey as X25519PublicKey, ReusableSecret as X25519ReusableSecret,
    StaticSecret as X25519StaticSecret,
};

mod aead;

type Identity = String;

fn sign_bundle(
    signing_key: &SigningKey,
    key_pairs: &[(X25519StaticSecret, X25519PublicKey)],
) -> Signature {
    let mut hasher = Blake2b512::new();
    hasher.update(key_pairs.len().to_be_bytes());
    for key_pair in key_pairs {
        hasher.update(key_pair.1.as_bytes());
    }
    signing_key.sign(&hasher.finalize())
}

fn verify_bundle(
    verifying_key: &VerifyingKey,
    public_keys: &[X25519PublicKey],
    signature: &Signature,
) -> Result<(), ed25519_dalek::ed25519::Error> {
    let mut hasher = Blake2b512::new();
    hasher.update(public_keys.len().to_be_bytes());
    for public_key in public_keys {
        hasher.update(public_key.as_bytes());
    }
    verifying_key.verify(&hasher.finalize(), signature)
}

struct X3DHPreKeyBundle {
    bundle: Vec<(X25519StaticSecret, X25519PublicKey)>,
    signature: Signature,
}

fn create_prekey_bundle(signing_key: &SigningKey, num_keys: u32) -> X3DHPreKeyBundle {
    let bundle: Vec<_> = (0..num_keys)
        .map(|_| {
            let pkey = X25519StaticSecret::random();
            let pubkey = X25519PublicKey::from(&pkey);
            (pkey, pubkey)
        })
        .collect();
    let signature = sign_bundle(signing_key, &bundle);
    X3DHPreKeyBundle { signature, bundle }
}

#[derive(Clone)]
struct SignedPreKey {
    pre_key: X25519PublicKey,
    signature: Signature,
}

#[derive(Clone)]
struct SignedPreKeys {
    pre_keys: Vec<X25519PublicKey>,
    signature: Signature,
}

struct X3DHInitialResponse {
    identity_key: VerifyingKey,
    signed_pre_key: SignedPreKeys,
    one_time_key: Option<X25519PublicKey>,
}

struct X3DHInitiateSendSkResult {
    ephemeral_key: X25519PublicKey,
    secret_key: [u8; 32],
}

fn kdf(prk: &[u8]) -> Result<[u8; 32]> {
    let ikm = [
        &hex!("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),
        prk,
    ]
    .concat();
    let hk = Hkdf::<Sha256>::new(None, &ikm);
    let mut okm = [0u8; 32];
    hk.expand(b"Brongnal", &mut okm).unwrap();
    Ok(okm)
}

// Initiate Conversation Server Response
fn x3dh_initiate_send_sk(
    identity_key: VerifyingKey,
    signed_pre_key: SignedPreKey,
    one_time_key: Option<X25519PublicKey>,
    sender_key: &SigningKey,
) -> Result<X3DHInitiateSendSkResult> {
    let _ = verify_bundle(
        &identity_key,
        &[signed_pre_key.pre_key],
        &signed_pre_key.signature,
    )
    .map_err(|e| anyhow!("Failed to verify bundle: {e}"));

    let reusable_secret = X25519ReusableSecret::random();
    let dh1 = X25519StaticSecret::from(sender_key.to_scalar_bytes())
        .diffie_hellman(&signed_pre_key.pre_key);
    let dh2 = reusable_secret.diffie_hellman(&X25519PublicKey::from(
        identity_key.to_montgomery().to_bytes(),
    ));
    let dh3 = reusable_secret.diffie_hellman(&signed_pre_key.pre_key);

    let secret_key = if let Some(one_time_key) = one_time_key {
        let dh4 = reusable_secret.diffie_hellman(&one_time_key);
        kdf(&[
            dh1.to_bytes(),
            dh2.to_bytes(),
            dh3.to_bytes(),
            dh4.to_bytes(),
        ]
        .concat())
    } else {
        kdf(&[dh1.to_bytes(), dh2.to_bytes(), dh3.to_bytes()].concat())
    }?;

    Ok(X3DHInitiateSendSkResult {
        ephemeral_key: X25519PublicKey::from(&reusable_secret),
        secret_key,
    })
}

struct X3DHInitiateResponse {
    identity_key: VerifyingKey,
    otk: Option<X25519PublicKey>,
    spk: SignedPreKey,
}

struct Message {
    identity_key: VerifyingKey,
    ephemeral_key: X25519PublicKey,
    otk: Option<X25519PublicKey>,
    ciphertext: String,
}

trait X3DHServer {
    fn set_spk(&mut self, identity: Identity, ik: VerifyingKey, spk: SignedPreKey) -> Result<()>;

    fn publish_otk_bundle(
        &mut self,
        identity: Identity,
        ik: VerifyingKey,
        otk_bundle: SignedPreKeys,
    ) -> Result<()>;

    fn fetch_prekey_bundle(
        &mut self,
        recipient_identity: &Identity,
    ) -> Result<X3DHInitiateResponse>;

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

    fn fetch_prekey_bundle(
        &mut self,
        recipient_identity: &Identity,
    ) -> Result<X3DHInitiateResponse> {
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

        Ok(X3DHInitiateResponse {
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

fn x3dh_initiate_send(
    server: &mut dyn X3DHServer,
    client: &mut dyn Client,
    recipient_identity: &Identity,
    sender_key: SigningKey,
    message: &str,
) -> Result<Message> {
    let X3DHInitiateResponse {
        identity_key,
        otk,
        spk,
    } = server.fetch_prekey_bundle(recipient_identity)?;
    let X3DHInitiateSendSkResult {
        ephemeral_key,
        secret_key,
    } = x3dh_initiate_send_sk(identity_key, spk, otk, &sender_key)?;
    let associated_data = [
        sender_key.verifying_key().to_bytes(),
        identity_key.to_bytes(),
    ]
    .concat();

    client.set_session_key(recipient_identity.clone(), &secret_key);

    let ciphertext = encrypt_data(
        Payload {
            msg: message.as_bytes(),
            aad: &associated_data,
        },
        &client.get_encryption_key(recipient_identity)?,
    )?;

    Ok(Message {
        identity_key,
        ephemeral_key,
        otk,
        ciphertext,
    })
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
}

trait SessionKeyManager {
    fn set_session_key(&mut self, recipient_identity: Identity, secret_key: &[u8; 32]);
    fn get_encryption_key(&mut self, recipient_identity: &Identity) -> Result<ChaCha20Poly1305>;
    fn destroy_session_key(&mut self, peer: &Identity);
}

trait Client: OTKManager + KeyManager + SessionKeyManager {}

struct InMemoryClient {
    identity_key: SigningKey,
    pre_key: X25519StaticSecret,
    one_time_pre_keys: HashMap<X25519PublicKey, X25519StaticSecret>,
    session_keys: HashMap<Identity, [u8; 32]>,
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
}

impl Client for InMemoryClient {}

impl SessionKeyManager for InMemoryClient {
    fn set_session_key(&mut self, recipient_identity: Identity, secret_key: &[u8; 32]) {
        self.session_keys.insert(recipient_identity, *secret_key);
    }

    fn get_encryption_key(&mut self, recipient_identity: &Identity) -> Result<ChaCha20Poly1305> {
        if let Some(key) = self.session_keys.get_mut(recipient_identity) {
            let mut hasher = Blake2b512::new();
            hasher.update(&key);
            let blake2b_mac = hasher.finalize();
            key.clone_from_slice(&blake2b_mac[0..32]);
            ChaCha20Poly1305::new_from_slice(&blake2b_mac[32..]).map_err(|e| anyhow!("oop: {e}"))
        } else {
            Err(anyhow!(
                "SessionKeyManager does not contain {recipient_identity}"
            ))
        }
    }

    fn destroy_session_key(&mut self, peer: &Identity) {
        self.session_keys.remove(peer);
    }
}

fn x3dh_initiate_recv_sk(
    client: &mut dyn OTKManager,
    sender_identity_key: &VerifyingKey,
    ephemeral_key: X25519PublicKey,
    otk: Option<X25519PublicKey>,
    identity_key: &SigningKey,
    pre_key: X25519StaticSecret,
) -> Result<[u8; 32]> {
    let dh1 = pre_key.diffie_hellman(&X25519PublicKey::from(
        sender_identity_key.to_montgomery().to_bytes(),
    ));
    let dh2 =
        X25519StaticSecret::from(identity_key.to_scalar_bytes()).diffie_hellman(&ephemeral_key);
    let dh3 = pre_key.diffie_hellman(&ephemeral_key);

    if let Some(one_time_key) = otk {
        let dh4 = client
            .fetch_wipe_one_time_secret_key(&one_time_key)?
            .diffie_hellman(&ephemeral_key);
        kdf(&[
            dh1.to_bytes(),
            dh2.to_bytes(),
            dh3.to_bytes(),
            dh4.to_bytes(),
        ]
        .concat())
    } else {
        kdf(&[dh1.to_bytes(), dh2.to_bytes(), dh3.to_bytes()].concat())
    }
}

fn x3dh_initiate_recv(
    client: &mut dyn Client,
    sender: &Identity,
    sender_identity_key: &VerifyingKey,
    ephemeral_key: X25519PublicKey,
    one_time_key: Option<X25519PublicKey>,
    ciphertext: &str,
) -> Result<Vec<u8>> {
    let identity_key = client.get_identity_key()?;
    let pre_key = client.get_pre_key()?;
    let secret_key = x3dh_initiate_recv_sk(
        client,
        sender_identity_key,
        ephemeral_key,
        one_time_key,
        &identity_key,
        pre_key,
    )?;

    let associated_data = [sender_identity_key.to_bytes(), identity_key.to_bytes()].concat();
    client.set_session_key(sender.clone(), &secret_key);
    let cipher = ChaCha20Poly1305::new_from_slice(&secret_key)?;
    match decrypt_data(ciphertext, &associated_data, &cipher) {
        Ok(msg) => Ok(msg),
        Err(e) => {
            client.destroy_session_key(&sender);
            Err(e)
        }
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use crate::*;
    use chacha20poly1305::aead::OsRng;

    struct TestOtkManager {
        private_key: X25519StaticSecret,
        public_key: X25519PublicKey,
    }
    impl OTKManager for TestOtkManager {
        fn fetch_wipe_one_time_secret_key(
            &mut self,
            one_time_key: &X25519PublicKey,
        ) -> Result<X25519StaticSecret> {
            if &self.public_key == one_time_key {
                Ok(self.private_key.clone())
            } else {
                Err(anyhow!(
                    "Otk mismatch. Expected: {:?}, Found: {:?}",
                    self.public_key,
                    one_time_key
                ))
            }
        }
    }

    #[test]
    // 1. Bob publishes his identity key and prekeys to a server.
    // 2. Alice fetches a "prekey bundle" from the server, and uses it to send an initial message to Bob.
    // 3. Bob receives and processes Alice's initial message.
    fn x3dh_key_agreement() -> Result<()> {
        let bob_ik = SigningKey::generate(&mut OsRng);
        let bob_spk = create_prekey_bundle(&bob_ik, 1);
        let bob_spk_secret = bob_spk.bundle[0].clone().0;
        let bob_spk = SignedPreKey {
            pre_key: bob_spk.bundle[0].1,
            signature: bob_spk.signature,
        };
        let otk = X25519StaticSecret::random_from_rng(&mut OsRng);
        let otk_pub = X25519PublicKey::from(&otk);
        let alice_ik = SigningKey::generate(&mut OsRng);
        let X3DHInitiateSendSkResult {
            ephemeral_key,
            secret_key,
        } = x3dh_initiate_send_sk(bob_ik.verifying_key(), bob_spk, Some(otk_pub), &alice_ik)?;

        let recv_sk = x3dh_initiate_recv_sk(
            &mut TestOtkManager {
                private_key: otk,
                public_key: otk_pub,
            },
            &alice_ik.verifying_key(),
            ephemeral_key,
            Some(otk_pub),
            &bob_ik,
            bob_spk_secret,
        )?;
        assert_eq!(secret_key, recv_sk);
        Ok(())
    }

    #[test]
    fn x3dh_send_recv() -> Result<()> {
        let mut server = InMemoryServer::new();
        let bob_ik = SigningKey::generate(&mut OsRng);
        let plaintext = "Hello".to_string();
        let bob_spk = create_prekey_bundle(&bob_ik, 1);
        let bob_otks = create_prekey_bundle(&bob_ik, 100);
        let bob_signed_prekeys = SignedPreKeys {
            pre_keys: bob_otks
                .bundle
                .iter()
                .map(|(_, _pub)| _pub.clone())
                .collect(),
            signature: bob_otks.signature,
        };

        let alice = InMemoryClient {
            identity_key: SigningKey::generate(&mut OsRng),
            pre_key: X25519StaticSecret::random_from_rng(&mut OsRng),
            one_time_pre_keys: HashMap::new(),
            session_keys: HashMap::new(),
        };

        let mut bob = InMemoryClient {
            identity_key: bob_ik.clone(),
            pre_key: bob_spk.bundle.get(0).unwrap().0.clone(),
            one_time_pre_keys: bob_otks
                .bundle
                .into_iter()
                .map(|(_0, _1)| (_1, _0))
                .collect(),
            session_keys: HashMap::new(),
        };

        // 1. Bob publishes his identity key and prekeys to a server.
        server.set_spk(
            "Bob".to_string(),
            bob_ik.verifying_key(),
            SignedPreKey {
                pre_key: bob_spk.bundle[0].1,
                signature: bob_spk.signature,
            },
        )?;
        server.publish_otk_bundle("Bob".to_owned(), bob_ik.verifying_key(), bob_signed_prekeys)?;

        // 2. Alice fetches a "prekey bundle" from the server, and uses it to send an initial message to Bob.
        let message = x3dh_initiate_send(
            &mut server,
            &mut bob,
            &"Bob".to_owned(),
            alice.identity_key.clone(),
            &plaintext,
        )?;

        server.send_message(&"Bob".to_owned(), message)?;

        // 3. Bob receives and processes Alice's initial message.
        let x3dh_messages = server.retrieve_messages(&"Bob".to_owned());
        assert_eq!(x3dh_messages.len(), 1);
        let x3dh_message = &x3dh_messages[0];
        let decrypted = x3dh_initiate_recv(
            &mut bob,
            &"Bob".to_string(),
            &x3dh_message.identity_key,
            x3dh_message.ephemeral_key,
            x3dh_message.otk,
            &x3dh_message.ciphertext,
        )?;
        assert_eq!(plaintext, x3dh_message.ciphertext);
        assert_eq!(plaintext, String::from_utf8(decrypted)?);

        Ok(())
    }
}
