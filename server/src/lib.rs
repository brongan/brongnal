#![feature(map_try_insert)]
use ed25519_dalek::VerifyingKey;
use futures::lock::Mutex;
use protocol::bundle::verify_bundle;
use protocol::x3dh::{Message, PreKeyBundle, SignedPreKey, SignedPreKeys, X3DHError};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tarpc::context;
use thiserror::Error;
use x25519_dalek::PublicKey as X25519PublicKey;

type Identity = String;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum BrongnalServerError {
    #[error("Error Running X3DH.")]
    X3DHError(#[from] X3DHError),
    #[error("Signature failed to validate.")]
    SignatureValidation,
    #[error("User is not registered.")]
    PreconditionError,
}

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
