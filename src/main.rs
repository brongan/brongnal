#![feature(map_try_insert)]
use anyhow::{anyhow, Context, Result};
use blake2::{Blake2b512, Digest};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
    ChaCha20Poly1305, Nonce,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hex_literal::hex;
use hkdf::Hkdf;
use sha2::Sha256;
use std::thread;
use std::{
    collections::HashMap,
    sync::{mpsc, Arc},
};
use x25519_dalek::{
    PublicKey as X25519PublicKey, ReusableSecret as X25519ReusableSecret,
    StaticSecret as X25519StaticSecret,
};

const NTHREADS: i32 = 16;
const NONCE_LEN: usize = 12;

type Identity = String;

fn encrypt_data(payload: Payload, cipher: &ChaCha20Poly1305) -> Result<String> {
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, payload)
        .map_err(|e| anyhow!("encrypt failed: {e}"))?;

    Ok(format!(
        "{}{}{}",
        "v1",
        hex::encode(&nonce),
        hex::encode(&ciphertext)
    ))
}

fn decrypt_data(ciphertext: String, aad: &[u8], cipher: &ChaCha20Poly1305) -> Result<Vec<u8>> {
    let version = &ciphertext[0..2];
    if version != "v1" {
        return Err(anyhow!("Invalid version."));
    }

    let nonce_bytes = hex::decode(&ciphertext[2..(NONCE_LEN * 2 + 2)])
        .map_err(|e| anyhow!("Failed to decode nonce: {e}."))?;
    let msg = hex::decode(&ciphertext[(2 + NONCE_LEN * 2)..])
        .map_err(|e| anyhow!("Failed to decode ciphertext: {e}."))?;
    cipher
        .decrypt(&Nonce::from_slice(&nonce_bytes), Payload { msg: &msg, aad })
        .map_err(|e| anyhow!("decrypt failed: {e}"))
}

fn threads_test() -> Result<()> {
    let mut children = Vec::new();
    let key = ChaCha20Poly1305::generate_key(&mut OsRng);
    let cipher = Arc::new(ChaCha20Poly1305::new(&key));

    let (tx, rx) = mpsc::channel();
    for id in 0..NTHREADS {
        let thread_tx = tx.clone();
        let cipher = cipher.clone();
        let child = thread::spawn(move || {
            let ciphertext: String = encrypt_data(
                Payload {
                    msg: format!("Hello I am thread: {id}").as_bytes(),
                    aad: &[],
                },
                &cipher,
            )
            .unwrap();
            thread_tx.send(ciphertext).unwrap();
        });

        children.push(child);
    }

    let mut ciphertexts: Vec<String> = Vec::with_capacity(NTHREADS as usize);
    for _ in 0..NTHREADS {
        ciphertexts.push(rx.recv().unwrap());
    }

    for child in children {
        child.join().expect("oops! the child thread panicked");
    }

    let cipher = ChaCha20Poly1305::new(&key);
    for ciphertext in ciphertexts {
        let decrypted_data =
            decrypt_data(ciphertext, &[], &cipher).context("decryption failed.")?;
        println!(
            "Received: {}",
            String::from_utf8(decrypted_data.to_vec()).unwrap()
        );
    }
    Ok(())
}

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

struct X3DHPreKey {
    signature: Signature,
    bundle: Vec<(X25519StaticSecret, X25519PublicKey)>,
}

fn x3dh_pre_key(signing_key: &SigningKey, num_keys: u32) -> X3DHPreKey {
    let bundle: Vec<_> = (0..num_keys)
        .map(|_| {
            let pkey = X25519StaticSecret::random();
            let pubkey = X25519PublicKey::from(&pkey);
            (pkey, pubkey)
        })
        .collect();
    let signature = sign_bundle(signing_key, &bundle);
    X3DHPreKey { signature, bundle }
}

#[derive(Clone)]
struct SignedPreKey {
    pre_key: X25519PublicKey,
    signature: Signature,
}

struct X3DHInitialResponse {
    identity_key: VerifyingKey,
    signed_pre_key: SignedPreKey,
    one_time_key: Option<X25519PublicKey>,
}

struct X3DHInitiateSendGetSkResult {
    identity_key: VerifyingKey,
    ephemeral_key: X25519PublicKey,
    secret_key: [u8; 32],
    one_time_key: Option<X25519PublicKey>,
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
fn x3dh_initiate_send_get_sk(
    identity_key: VerifyingKey,
    signed_pre_key: SignedPreKey,
    one_time_key: Option<X25519PublicKey>,
    sender_key: &SigningKey,
) -> Result<X3DHInitiateSendGetSkResult> {
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

    Ok(X3DHInitiateSendGetSkResult {
        identity_key,
        ephemeral_key: X25519PublicKey::from(&reusable_secret),
        secret_key,
        one_time_key,
    })
}

struct X3DHInitiateResponse {
    identity_key: VerifyingKey,
    signed_pre_key: SignedPreKey,
    one_time_key: Option<X25519PublicKey>,
}

struct Message {
    sender_key: VerifyingKey,
    ephemeral_key: X25519PublicKey,
    pre_key: Option<X25519PublicKey>,
    ciphertext: String,
}

trait X3DHServer {
    fn publish_keys(
        &mut self,
        identity: Identity,
        identity_key: VerifyingKey,
        signed_pre_key: SignedPreKey,
        one_time_pre_keys: Vec<X25519PublicKey>,
    ) -> Result<()>;

    fn initiate_fetch_bundle(
        &mut self,
        recipient_identity: &Identity,
    ) -> Result<X3DHInitiateResponse>;

    fn send_message(&mut self, recipient_identity: &Identity, message: Message) -> Result<()>;

    fn retrieve_messages(&mut self, identity: &Identity) -> Vec<Message>;
}

struct ClientData {
    identity_key: VerifyingKey,
    signed_pre_key: SignedPreKey,
    one_time_pre_keys: Vec<X25519PublicKey>,
}

struct InMemoryServer {
    client_data: HashMap<Identity, ClientData>,
    messages: HashMap<Identity, Vec<Message>>,
}

impl X3DHServer for InMemoryServer {
    fn publish_keys(
        &mut self,
        identity: Identity,
        identity_key: VerifyingKey,
        signed_pre_key: SignedPreKey,
        one_time_pre_keys: Vec<X25519PublicKey>,
    ) -> Result<()> {
        self.client_data.insert(
            identity,
            ClientData {
                identity_key,
                signed_pre_key,
                one_time_pre_keys,
            },
        );
        Ok(())
    }

    fn initiate_fetch_bundle(
        &mut self,
        recipient_identity: &Identity,
    ) -> Result<X3DHInitiateResponse> {
        if let Some(data) = self.client_data.get_mut(recipient_identity) {
            let one_time_key = data.one_time_pre_keys.pop();
            Ok(X3DHInitiateResponse {
                identity_key: data.identity_key,
                signed_pre_key: data.signed_pre_key.clone(),
                one_time_key,
            })
        } else {
            Err(anyhow!("Missing client data for: {recipient_identity}"))
        }
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

trait X3DHSessionKeyManager {
    fn set_session_key(&mut self, recipient_identity: Identity, secret_key: &[u8; 32]);
    fn get_encryption_key(&mut self, recipient_identity: &Identity) -> Result<ChaCha20Poly1305>;
    fn destroy_session_key(&mut self, sender: &Identity);
}

struct SessionKeyManager(HashMap<Identity, [u8; 32]>);

impl X3DHSessionKeyManager for SessionKeyManager {
    fn set_session_key(&mut self, recipient_identity: Identity, secret_key: &[u8; 32]) {
        self.0.insert(recipient_identity, *secret_key);
    }

    fn get_encryption_key(&mut self, recipient_identity: &Identity) -> Result<ChaCha20Poly1305> {
        if let Some(key) = self.0.get_mut(recipient_identity) {
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

    fn destroy_session_key(&mut self, sender: &Identity) {
        self.0.remove(sender);
    }
}

struct X3DHInitiateSendResult {
    identity_key: VerifyingKey,
    ephemeral_key: X25519PublicKey,
    one_time_key: Option<X25519PublicKey>,
    ciphertext: String,
}

fn x3dh_initiate_send(
    server: &mut dyn X3DHServer,
    session_key_manger: &mut SessionKeyManager,
    recipient_identity: &Identity,
    sender_key: SigningKey,
    message: &str,
) -> Result<X3DHInitiateSendResult> {
    let response = server.initiate_fetch_bundle(recipient_identity)?;
    let X3DHInitiateSendGetSkResult {
        identity_key,
        ephemeral_key,
        secret_key,
        one_time_key,
    } = x3dh_initiate_send_get_sk(
        response.identity_key,
        response.signed_pre_key,
        response.one_time_key,
        &sender_key,
    )?;
    let associated_data = [
        sender_key.verifying_key().to_bytes(),
        identity_key.to_bytes(),
    ]
    .concat();

    session_key_manger.set_session_key(recipient_identity.clone(), &secret_key);

    let ciphertext = encrypt_data(
        Payload {
            msg: message.as_bytes(),
            aad: &associated_data,
        },
        &session_key_manger.get_encryption_key(recipient_identity)?,
    )?;

    Ok(X3DHInitiateSendResult {
        identity_key,
        ephemeral_key,
        one_time_key,
        ciphertext,
    })
}

trait X3DHClient {
    fn fetch_wipe_one_time_secret_key(
        &mut self,
        one_time_key: X25519PublicKey,
    ) -> Result<X25519StaticSecret>;
    fn get_identity_key(&self) -> Result<&SigningKey>;
    fn get_pre_key(&mut self) -> Result<X25519StaticSecret>;
}

struct InMemoryClient {
    identity_key: SigningKey,
    signed_pre_key: SignedPreKey,
    one_time_pre_keys: Vec<X25519StaticSecret>,
}

impl X3DHClient for InMemoryClient {
    fn fetch_wipe_one_time_secret_key(
        &mut self,
        one_time_key: X25519PublicKey,
    ) -> Result<X25519StaticSecret> {
    }

    fn get_identity_key(&self) -> Result<&SigningKey> {
        Ok(&self.identity_key)
    }

    fn get_pre_key(&mut self) -> Result<X25519StaticSecret> {
        self.one_time_pre_keys
            .pop()
            .ok_or_else(|| anyhow!("no more pre keys."))
    }
}

fn x3dh_initiate_receive_sk(
    client: &mut dyn X3DHClient,
    sender_identity_key: &VerifyingKey,
    ephemeral_key: X25519PublicKey,
    one_time_key: Option<X25519PublicKey>,
    identity_key: &SigningKey,
    pre_key: X25519StaticSecret,
) -> Result<[u8; 32]> {
    let dh1 = pre_key.diffie_hellman(&X25519PublicKey::from(
        sender_identity_key.to_montgomery().to_bytes(),
    ));
    let dh2 =
        X25519StaticSecret::from(identity_key.to_scalar_bytes()).diffie_hellman(&ephemeral_key);
    let dh3 = pre_key.diffie_hellman(&ephemeral_key);

    if let Some(one_time_key) = one_time_key {
        let dh4 = client
            .fetch_wipe_one_time_secret_key(one_time_key)?
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
    client: &mut dyn X3DHClient,
    session_key_manager: &mut SessionKeyManager,
    sender: &Identity,
    sender_identity_key: &VerifyingKey,
    ephemeral_key: X25519PublicKey,
    one_time_key: Option<X25519PublicKey>,
    ciphertext: String,
) -> Result<Vec<u8>> {
    let identity_key = client.get_identity_key()?;
    let pre_key = client.get_pre_key()?;
    let secret_key = x3dh_initiate_receive_sk(
        client,
        sender_identity_key,
        ephemeral_key,
        one_time_key,
        &identity_key,
        pre_key,
    )?;

    let associated_data = [sender_identity_key.to_bytes(), identity_key.to_bytes()].concat();
    session_key_manager.set_session_key(sender.clone(), &secret_key);
    let cipher = ChaCha20Poly1305::new_from_slice(&secret_key)?;
    match decrypt_data(ciphertext, &associated_data, &cipher) {
        Ok(msg) => Ok(msg),
        Err(e) => {
            session_key_manager.destroy_session_key(&sender);
            Err(e)
        }
    }
}

fn main() {
    threads_test().unwrap();
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn x3dh_key_agreement() -> Result<()> {
        let mut session_key_manager = SessionKeyManager(HashMap::new());
        let sender_identity = "Sender".to_string();
        let recipient_identity = "Recipient".to_string();
        let sender_key = SigningKey::generate(&mut OsRng);
        let recipient_key = SigningKey::generate(&mut OsRng);
        let message = "Hello".to_string();

        x3dh_initiate_send(
            &mut session_key_manager,
            &recipient_identity,
            sender_key,
            &message,
        )?;
    }
}
