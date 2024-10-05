use ed25519_dalek::{Signature, VerifyingKey};
use proto::service::brongnal_server::Brongnal;
use proto::service::Message as MessageProto;
use proto::service::SignedPreKey as SignedPreKeyProto;
use proto::service::{
    PreKeyBundle, RegisterPreKeyBundleRequest, RegisterPreKeyBundleResponse, RequestPreKeysRequest,
    RetrieveMessagesRequest, SendMessageRequest, SendMessageResponse,
};
use proto::{parse_verifying_key, parse_x25519_public_key};
use protocol::bundle::verify_bundle;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Result, Status};
use x25519_dalek::PublicKey as X25519PublicKey;

pub trait Storage: std::fmt::Debug {
    /// Add a new identity to the storage.
    /// For now, repeated calls should not return an error.
    // TODO(#25) - Return error when attempting to overwrite registration.
    fn add_user(
        &self,
        identity: String,
        identity_key: VerifyingKey,
        signed_pre_key: SignedPreKeyProto,
    ) -> Result<()>;

    /// Replaces the signed pre key for a given identity.
    // TODO(#27) -  Implement signed pre key rotation.
    #[allow(dead_code)]
    fn update_pre_key(&self, identity: &str, pre_key: SignedPreKeyProto) -> Result<()>;

    /// Appends new unburnt one time pre keys for others to message a given identity.
    fn add_one_time_keys(&self, identity: &str, pre_keys: Vec<X25519PublicKey>) -> Result<()>;

    /// Retrieves the identity key and signed pre key for a given identity.
    /// A client must first invoke this before messaging a peer.
    fn get_current_keys(&self, identity: &str) -> Result<(VerifyingKey, SignedPreKeyProto)>;

    /// Retrieve a one time pre key for an identity.
    fn pop_one_time_key(&self, identity: &str) -> Result<Option<X25519PublicKey>>;

    /// Enqueue a message for a given recipient.
    fn add_message(&self, recipient: &str, message: MessageProto) -> Result<()>;

    /// Retrieve enqueued messages for a given identity.
    fn get_messages(&self, identity: &str) -> Result<Vec<MessageProto>>;
}

#[derive(Debug)]
pub struct BrongnalController {
    storage: Box<dyn Storage + Send + Sync>,
    receivers: Arc<Mutex<HashMap<String, mpsc::Sender<Result<MessageProto>>>>>,
}

impl BrongnalController {
    pub fn new(storage: Box<dyn Storage + Send + Sync>) -> BrongnalController {
        BrongnalController {
            storage,
            receivers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tonic::async_trait]
impl Brongnal for BrongnalController {
    async fn register_pre_key_bundle(
        &self,
        request: Request<RegisterPreKeyBundleRequest>,
    ) -> Result<Response<RegisterPreKeyBundleResponse>> {
        let request = request.into_inner();
        println!("Registering PreKeyBundle for \"{}\".", request.identity());

        let identity: String = request
            .identity
            .clone()
            .ok_or(Status::invalid_argument("request missing identity"))?;
        let identity_key = parse_verifying_key(&request.identity_key())
            .map_err(|_| Status::invalid_argument("request has invalid identity_key"))?;
        let signed_pre_key_proto = request
            .signed_pre_key
            .ok_or(Status::invalid_argument("request is missing signed prekey"))?;
        let signed_pre_key = protocol::x3dh::SignedPreKey::try_from(signed_pre_key_proto.clone())?;
        verify_bundle(
            &identity_key,
            &[signed_pre_key.pre_key],
            &signed_pre_key.signature,
        )
        .map_err(|_| Status::unauthenticated("failed to validate signed prekey signature"))?;

        let one_time_keys = request.one_time_key_bundle.ok_or(Status::invalid_argument(
            "request missing one_time_key_bundle",
        ))?;
        let pre_keys: Vec<X25519PublicKey> = one_time_keys
            .pre_keys
            .iter()
            .map(|key| parse_x25519_public_key(&key))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("invalid prekey bundle"))?;
        let signature = Signature::from_slice(one_time_keys.signature()).map_err(|_e| {
            Status::invalid_argument("one time prekey bundle signature is invalid")
        })?;
        verify_bundle(&identity_key, &pre_keys, &signature).map_err(|_| {
            Status::unauthenticated("failed to validate one time prekey bundle signature")
        })?;

        self.storage
            .add_user(identity.clone(), identity_key, signed_pre_key_proto)?;
        self.storage.add_one_time_keys(&identity, pre_keys)?;

        Ok(Response::new(RegisterPreKeyBundleResponse {}))
    }

    async fn request_pre_keys(
        &self,
        request: Request<RequestPreKeysRequest>,
    ) -> Result<Response<PreKeyBundle>> {
        let request = request.into_inner();
        println!("Retrieving PreKeyBundle for \"{}\".", request.identity());

        let (identity_key, signed_pre_key) = self.storage.get_current_keys(request.identity())?;
        // TODO(#26) - Prevent one time key pop abuse.
        let one_time_key = self.storage.pop_one_time_key(request.identity())?;

        let reply = PreKeyBundle {
            identity_key: Some(identity_key.as_bytes().into()),
            one_time_key: one_time_key.map(|otk| otk.as_bytes().into()),
            signed_pre_key: Some(signed_pre_key.into()),
        };
        Ok(Response::new(reply))
    }

    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageResponse>> {
        let request = request.into_inner();
        println!(
            "Received request to send message to: \"{}\".",
            request.recipient_identity()
        );

        let recipient_identity = request.recipient_identity.ok_or(Status::invalid_argument(
            "request missing recipient_identity",
        ))?;
        let message_proto: MessageProto = request
            .message
            .ok_or(Status::invalid_argument("request missing message"))?
            .into();
        let _ = protocol::x3dh::Message::try_from(message_proto.clone())?;

        let tx = self
            .receivers
            .lock()
            .unwrap()
            .get(&recipient_identity)
            .map(|tx| tx.clone());
        if let Some(tx) = tx {
            if let Ok(()) = tx.send(Ok(message_proto.clone())).await {
                return Ok(Response::new(SendMessageResponse {}));
            }
        }

        self.storage
            .add_message(&recipient_identity, message_proto)?;
        Ok(Response::new(SendMessageResponse {}))
    }

    type RetrieveMessagesStream = ReceiverStream<Result<MessageProto>>;
    async fn retrieve_messages(
        &self,
        request: Request<RetrieveMessagesRequest>,
    ) -> Result<Response<Self::RetrieveMessagesStream>> {
        let request = request.into_inner();
        println!("Retrieving \"{}\"'s messages.", request.identity());

        let identity = request
            .identity
            .ok_or(Status::invalid_argument("request missing identity"))?;
        let (tx, rx) = mpsc::channel(100);

        // TODO(#14) - RetrieveMessages requires proof of possession
        for message in self.storage.get_messages(&identity)? {
            // TODO handle result.
            let _ = tx.send(Ok(message.into())).await;
        }
        self.receivers.lock().unwrap().insert(identity, tx);

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
