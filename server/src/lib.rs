#![feature(map_try_insert)]
use ed25519_dalek::VerifyingKey;
use protocol::bundle::verify_bundle;
use protocol::x3dh::{Message, PreKeyBundle, SignedPreKey, SignedPreKeys, X3DHError};
use serde::{Deserialize, Serialize};
use service::brongnal::{Brongnal, BrongnalServer};
use service::{PreKeyBundle, RegisterPreKeyBundleRequest};
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use tonic::{transport::Server, Request, Response, Status};
use x25519_dalek::PublicKey as X25519PublicKey;

pub mod service {
    tonic::include_proto!("service"); // The string specified here must match the proto package name
}

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

#[derive(Clone, Debug)]
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

#[tonic::async_trait]
impl Brongnal for MemoryServer {
    async fn set_spk(
        self,
        identity: String,
        ik: VerifyingKey,
        spk: SignedPreKey,
    ) -> Result<(), BrongnalServerError> {
        eprintln!("Identity: {identity} set their IK and SPK");
        verify_bundle(&ik, &[spk.pre_key], &spk.signature)
            .map_err(|_| BrongnalServerError::SignatureValidation)?;
        self.identity_key
            .lock()
            .unwrap()
            .insert(identity.clone(), ik);
        self.current_pre_key.lock().unwrap().insert(identity, spk);
        self.one_time_pre_keys.lock().unwrap().clear();
        Ok(())
    }

    async fn publish_otk_bundle(
        self,
        identity: String,
        ik: VerifyingKey,
        otk_bundle: SignedPreKeys,
    ) -> Result<(), BrongnalServerError> {
        eprintln!("Identity: {identity} added otk bundle.");
        verify_bundle(&ik, &otk_bundle.pre_keys, &otk_bundle.signature)
            .map_err(|_| BrongnalServerError::SignatureValidation)?;
        let mut one_time_pre_keys = self.one_time_pre_keys.lock().unwrap();
        let _ = one_time_pre_keys.try_insert(identity.clone(), Vec::new());
        one_time_pre_keys
            .get_mut(&identity)
            .unwrap()
            .extend(otk_bundle.pre_keys);
        Ok(())
    }

    async fn fetch_prekey_bundle(
        self,
        recipient_identity: String,
    ) -> Result<PreKeyBundle, BrongnalServerError> {
        eprintln!("PreKeyBundle requested for: {recipient_identity}.");
        eprintln!("{:?}", self.identity_key);
        let identity_key = *self
            .identity_key
            .lock()
            .unwrap()
            .get(&recipient_identity)
            .ok_or(BrongnalServerError::PreconditionError)?;
        let spk = self
            .current_pre_key
            .lock()
            .unwrap()
            .get(&recipient_identity)
            .ok_or(BrongnalServerError::PreconditionError)?
            .clone();
        let otk = if let Some(otks) = self
            .one_time_pre_keys
            .lock()
            .unwrap()
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
        recipient_identity: String,
        message: Message,
    ) -> Result<(), BrongnalServerError> {
        eprintln!("Message sent to: {recipient_identity}");
        let mut messages = self.messages.lock().unwrap();
        let _ = messages.try_insert(recipient_identity.clone(), Vec::new());
        messages.get_mut(&recipient_identity).unwrap().push(message);
        Ok(())
    }

    async fn retrieve_messages(self, identity: String) -> Vec<Message> {
        eprintln!("Retrieving messages for: {identity}");
        self.messages
            .lock()
            .unwrap()
            .remove(&identity)
            .unwrap_or(Vec::new())
    }
}
