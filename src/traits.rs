use crate::x3dh::*;
use anyhow::Result;
use chacha20poly1305::ChaCha20Poly1305;
use ed25519_dalek::{SigningKey, VerifyingKey};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

pub trait X3DHServer<Identity> {
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

pub trait OTKManager {
    fn fetch_wipe_one_time_secret_key(
        &mut self,
        one_time_key: &X25519PublicKey,
    ) -> Result<X25519StaticSecret>;
}

pub trait KeyManager {
    fn get_identity_key(&self) -> Result<SigningKey>;
    fn get_pre_key(&mut self) -> Result<X25519StaticSecret>;
    fn get_spk(&self) -> Result<SignedPreKey>;
}

pub trait SessionKeyManager<Identity> {
    fn set_session_key(&mut self, recipient_identity: Identity, secret_key: &[u8; 32]);
    fn get_encryption_key(&mut self, recipient_identity: &Identity) -> Result<ChaCha20Poly1305>;
    fn destroy_session_key(&mut self, peer: &Identity);
}

pub trait Client<Identity>: OTKManager + KeyManager + SessionKeyManager<Identity> {
    fn add_one_time_keys(&mut self, num_keys: u32) -> SignedPreKeys;
}
