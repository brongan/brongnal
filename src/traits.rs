use crate::x3dh::*;
use crate::BrongnalServerError;
use ed25519_dalek::{SigningKey, VerifyingKey};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

type Identity = String;

#[tarpc::service]
pub trait X3DHServer {
    // Bob publishes a set of elliptic curve public keys to the server, containing:
    //    Bob's identity key IKB
    //    Bob's signed prekey SPKB
    //    Bob's prekey signature Sig(IKB, Encode(SPKB))
    //    A set of Bob's one-time prekeys (OPKB1, OPKB2, OPKB3, ...)
    async fn set_spk(
        identity: Identity,
        ik: VerifyingKey,
        spk: SignedPreKey,
    ) -> Result<(), BrongnalServerError>;

    async fn publish_otk_bundle(
        identity: Identity,
        ik: VerifyingKey,
        otk_bundle: SignedPreKeys,
    ) -> Result<(), BrongnalServerError>;

    // To perform an X3DH key agreement with Bob, Alice contacts the server and fetches a "prekey bundle" containing the following values:
    //    Bob's identity key IKB
    //    Bob's signed prekey SPKB
    //    Bob's prekey signature Sig(IKB, Encode(SPKB))
    //    (Optionally) Bob's one-time prekey OPKB
    async fn fetch_prekey_bundle(
        recipient_identity: Identity,
    ) -> Result<PreKeyBundle, BrongnalServerError>;

    // The server can store messages from Alice to Bob which Bob can later retrieve.
    async fn send_message(
        recipient_identity: Identity,
        message: Message,
    ) -> Result<(), BrongnalServerError>;
    async fn retrieve_messages(identity: Identity) -> Vec<Message>;
}

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
