#![feature(map_try_insert)]
#![feature(trait_upcasting)]
#![allow(dead_code)]
use crate::aead::{decrypt_data, encrypt_data};
use crate::bundle::*;
use anyhow::{anyhow, Context, Result};
use blake2::{Blake2b512, Digest};
use chacha20poly1305::aead::OsRng;
use chacha20poly1305::{
    aead::{KeyInit, Payload},
    ChaCha20Poly1305,
};
use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use hkdf::Hkdf;
use sha2::Sha256;
use std::collections::HashMap;
use x25519_dalek::{
    PublicKey as X25519PublicKey, ReusableSecret as X25519ReusableSecret,
    StaticSecret as X25519StaticSecret,
};

mod aead;
mod bundle;

type Identity = String;

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

// KDF(KM) represents 32 bytes of output from the HKDF algorithm [3] with inputs:
//    HKDF input key material = F || KM, where KM is an input byte sequence containing secret key material, and F is a byte sequence containing 32 0xFF bytes if curve is X25519, and 57 0xFF bytes if curve is X448. F is used for cryptographic domain separation with XEdDSA [2].
//    HKDF salt = A zero-filled byte sequence with length equal to the hash output length.
//    HKDF info = An ASCII string identifying the application.
fn kdf(km: &[u8]) -> [u8; 32] {
    let salt = [0; 32];
    let f = [0xFF, 32];
    let ikm = [&f, km].concat();
    let hk = Hkdf::<Sha256>::new(Some(&salt), &ikm);
    let mut okm = [0u8; 32];
    hk.expand(b"Brongnal", &mut okm).unwrap();
    okm
}

struct X3DHSendKeyAgreement {
    ephemeral_key: X25519PublicKey,
    secret_key: [u8; 32],
}

// If the bundle does not contain a one-time prekey, she calculates:
//    DH1 = DH(IKA, SPKB)
//    DH2 = DH(EKA, IKB)
//    DH3 = DH(EKA, SPKB)
//    SK = KDF(DH1 || DH2 || DH3)
//If the bundle does contain a one-time prekey, the calculation is modified to include an additional DH:
//    DH4 = DH(EKA, OPKB)
//    SK = KDF(DH1 || DH2 || DH3 || DH4)
fn x3dh_initiate_send_get_sk(
    identity_key: VerifyingKey,
    signed_pre_key: SignedPreKey,
    one_time_key: Option<X25519PublicKey>,
    sender_key: &SigningKey,
) -> Result<X3DHSendKeyAgreement> {
    let _ = verify_bundle(
        &identity_key,
        &[signed_pre_key.pre_key],
        &signed_pre_key.signature,
    )
    .map_err(|e| anyhow!("Failed to verify bundle: {e}"));

    let ephemeral_secret = X25519ReusableSecret::random();
    let ephemeral_key = X25519PublicKey::from(&ephemeral_secret);
    let dh1 = X25519StaticSecret::from(sender_key.to_scalar_bytes())
        .diffie_hellman(&signed_pre_key.pre_key);
    let dh2 = ephemeral_secret.diffie_hellman(&X25519PublicKey::from(
        identity_key.to_montgomery().to_bytes(),
    ));
    let dh3 = ephemeral_secret.diffie_hellman(&signed_pre_key.pre_key);

    let secret_key = match one_time_key {
        Some(one_time_key) => {
            let dh4 = ephemeral_secret.diffie_hellman(&one_time_key);
            kdf(&[
                dh1.to_bytes(),
                dh2.to_bytes(),
                dh3.to_bytes(),
                dh4.to_bytes(),
            ]
            .concat())
        }
        None => kdf(&[dh1.to_bytes(), dh2.to_bytes(), dh3.to_bytes()].concat()),
    };

    Ok(X3DHSendKeyAgreement {
        ephemeral_key,
        secret_key,
    })
}

// Alice then sends Bob an initial message containing:
//    Alice's identity key IKA
//    Alice's ephemeral key EKA
//    Identifiers stating which of Bob's prekeys Alice used
//    An initial ciphertext encrypted with some AEAD encryption scheme [4] using AD as associated data and using an encryption key which is either SK or the output from some cryptographic PRF keyed by SK.
fn x3dh_initiate_send(
    server: &mut dyn X3DHServer,
    client: &mut dyn Client,
    recipient_identity: &Identity,
    sender_key: &SigningKey,
    message: &str,
) -> Result<Message> {
    let PreKeyBundle {
        identity_key,
        otk,
        spk,
    } = server.fetch_prekey_bundle(recipient_identity)?;
    let X3DHSendKeyAgreement {
        ephemeral_key,
        secret_key,
    } = x3dh_initiate_send_get_sk(identity_key, spk, otk, &sender_key)?;
    // Alice then calculates an "associated data" byte sequence AD that contains identity information for both parties:
    //   AD = Encode(IKA) || Encode(IKB)
    // Alice may optionally append additional information to AD, such as Alice and Bob's usernames, certificates, or other identifying information.
    let associated_data = [
        sender_key.verifying_key().to_bytes(),
        identity_key.to_bytes(),
    ]
    .concat();

    client.set_session_key(recipient_identity.clone(), &secret_key);

    // The initial ciphertext is typically the first message in some post-X3DH communication protocol. In other words, this ciphertext typically has two roles, serving as the first message within some post-X3DH protocol, and as part of Alice's X3DH initial message.
    // After sending this, Alice may continue using SK or keys derived from SK within the post-X3DH protocol for communication with Bob
    let ciphertext = encrypt_data(
        Payload {
            msg: message.as_bytes(),
            aad: &associated_data,
        },
        &client.get_encryption_key(recipient_identity)?,
    )?;

    Ok(Message {
        sender_identity_key: client.get_identity_key()?.verifying_key(),
        ephemeral_key,
        otk,
        ciphertext,
    })
}

fn x3dh_initiate_recv_get_sk(
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
        // Bob deletes any one-time prekey private key that was used, for forward secrecy.
        let dh4 = client
            .fetch_wipe_one_time_secret_key(&one_time_key)?
            .diffie_hellman(&ephemeral_key);
        Ok(kdf(&[
            dh1.to_bytes(),
            dh2.to_bytes(),
            dh3.to_bytes(),
            dh4.to_bytes(),
        ]
        .concat()))
    } else {
        Ok(kdf(
            &[dh1.to_bytes(), dh2.to_bytes(), dh3.to_bytes()].concat()
        ))
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
    // Upon receiving Alice's initial message, Bob retrieves Alice's identity key and ephemeral key from the message.
    let identity_key = client.get_identity_key()?;
    let pre_key = client.get_pre_key()?;
    // Bob also loads his identity private key, and the private key(s) corresponding to whichever signed prekey and one-time prekey (if any) Alice used.
    // Using these keys, Bob repeats the DH and KDF calculations from the previous section to derive SK, and then deletes the DH values.
    let secret_key = x3dh_initiate_recv_get_sk(
        client,
        sender_identity_key,
        ephemeral_key,
        one_time_key,
        &identity_key,
        pre_key,
    )?;

    // Bob then constructs the AD byte sequence using IKA and IKB, as described in the previous section.
    // AD = Encode(IKA) || Encode(IKB)
    let associated_data = [
        sender_identity_key.to_bytes(),
        identity_key.verifying_key().to_bytes(),
    ]
    .concat();

    // Bob may then continue using SK or keys derived from SK within the post-X3DH protocol for communication with Alice.
    client.set_session_key(sender.clone(), &secret_key);

    // Finally, Bob attempts to decrypt the initial ciphertext using SK and AD.
    let cipher = ChaCha20Poly1305::new_from_slice(&secret_key)?;
    match decrypt_data(ciphertext, &associated_data, &cipher) {
        Ok(msg) => Ok(msg),
        Err(e) => {
            //If the initial ciphertext fails to decrypt, then Bob aborts the protocol and deletes SK.
            client.destroy_session_key(&sender);
            Err(e)
        }
    }
}

struct Message {
    sender_identity_key: VerifyingKey,
    ephemeral_key: X25519PublicKey,
    otk: Option<X25519PublicKey>,
    ciphertext: String,
}

struct PreKeyBundle {
    identity_key: VerifyingKey,
    otk: Option<X25519PublicKey>,
    spk: SignedPreKey,
}

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

#[cfg(test)]
mod tests {
    use crate::*;

    struct TestOTKManager {
        private_key: X25519StaticSecret,
        public_key: X25519PublicKey,
    }
    impl OTKManager for TestOTKManager {
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
        let X3DHSendKeyAgreement {
            ephemeral_key,
            secret_key,
        } = x3dh_initiate_send_get_sk(bob_ik.verifying_key(), bob_spk, Some(otk_pub), &alice_ik)?;

        let recv_sk = x3dh_initiate_recv_get_sk(
            &mut TestOTKManager {
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

        // 1. Bob publishes his identity key and prekeys to a server.
        let mut bob = InMemoryClient::new();
        let bob_otks = bob.add_one_time_keys(100);
        server.set_spk(
            "Bob".to_string(),
            bob.identity_key.verifying_key(),
            bob.get_spk()?,
        )?;
        server.publish_otk_bundle("Bob".to_owned(), bob.identity_key.verifying_key(), bob_otks)?;

        // 2. Alice fetches a "prekey bundle" from the server, and uses it to send an initial message to Bob.
        let plaintext = "Hello";
        let mut alice = InMemoryClient::new();
        let alice_ik = alice.identity_key.clone();
        let message = x3dh_initiate_send(
            &mut server,
            &mut alice,
            &"Bob".to_owned(),
            &alice_ik,
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
            &x3dh_message.sender_identity_key,
            x3dh_message.ephemeral_key,
            x3dh_message.otk,
            &x3dh_message.ciphertext,
        )?;
        assert_eq!(plaintext, String::from_utf8(decrypted)?);

        Ok(())
    }
}
