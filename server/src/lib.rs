#![feature(map_try_insert)]
use ed25519_dalek::{Signature, VerifyingKey};
use protocol::bundle::verify_bundle;
use protocol::x3dh::{Message, PreKeyBundle, SignedPreKey, SignedPreKeys, X3DHError};
use serde::{Deserialize, Serialize};
use service::brongnal_server::{Brongnal, BrongnalServer};
use service::{
    PreKeyBundle as PreKeyBundleProto, RegisterPreKeyBundleRequest, RegisterPreKeyBundleResponse,
    RequestPreKeysRequest, SignedPreKey as SignedPreKeyProto,
};
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
    #[error("Incorrectly formatted request field: `{0}`")]
    InvalidArgument(String),
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

impl Into<SignedPreKeyProto> for SignedPreKey {
    fn into(self) -> SignedPreKeyProto {
        todo!()
    }
}

impl TryFrom<SignedPreKeyProto> for SignedPreKey {
    type Error = tonic::Status;

    fn try_from(value: SignedPreKeyProto) -> Result<Self, Self::Error> {
        if value.pre_key().len() != 32 {
            return Err(Status::invalid_argument(
                "Pre Key is not an X25519PublicKey",
            ));
        }

        if value.signature().len() != 64 {
            return Err(Status::invalid_argument(
                "Pre Key has an invalid X25519 Signature",
            ));
        }
        // TODO verify point is on curve.
        let pre_key = X25519PublicKey::from(value.pre_key().try_into()?);
        let signature = Signature::from_slice(value.signature()).map_err(|e| {
            Status::invalid_argument("Pre Key has an invalid X25519 Signature: {e}")
        })?;
        Ok(SignedPreKey { pre_key, signature })
    }
}

#[tonic::async_trait]
impl Brongnal for MemoryServer {
    async fn register_pre_key_bundle(
        &self,
        request: Request<RegisterPreKeyBundleRequest>,
    ) -> Result<Response<RegisterPreKeyBundleResponse>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let identity = request.identity().to_owned();
        // TODO Verify ik is a curve25519_dalek::curve::CompressedEdwardsY.
        // TODO Remove unwrap.
        let ik = VerifyingKey::from_bytes(request.ik().try_into().unwrap()).unwrap();
        // TODO Remove unwrap.
        let spk = SignedPreKey::try_from(
            request
                .spk
                .ok_or(BrongnalServerError::InvalidArgument(
                    "Request Missing SPK.".to_owned(),
                ))
                .unwrap(),
        )
        .unwrap();
        verify_bundle(&ik, &[spk.pre_key], &spk.signature)
            .map_err(|_| Status::unauthenticated("SPK signature invalid."))?;
        self.identity_key
            .lock()
            .unwrap()
            .insert(identity.clone(), ik);
        self.current_pre_key.lock().unwrap().insert(identity, spk);
        self.one_time_pre_keys.lock().unwrap().clear();
        Ok(Response::new(RegisterPreKeyBundleResponse {}))
    }

    async fn request_pre_keys(
        &self,
        request: Request<RequestPreKeysRequest>,
    ) -> Result<Response<PreKeyBundleProto>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let identity_key = *self
            .identity_key
            .lock()
            .unwrap()
            .get(request.identity())
            .ok_or(Status::not_found("User not found."))?;
        let spk = self
            .current_pre_key
            .lock()
            .unwrap()
            .get(request.identity())
            .ok_or(Status::not_found("User not found."))?
            .to_owned();
        let otk = if let Some(otks) = self
            .one_time_pre_keys
            .lock()
            .unwrap()
            .get_mut(request.identity())
        {
            otks.pop()
        } else {
            None
        };

        let reply = PreKeyBundleProto {
            identity_key: Some(identity_key.as_bytes().into()),
            otk: otk.map(|otk| otk.as_bytes().into()),
            spk: Some(spk.into()),
        };
        Ok(Response::new(reply))
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
