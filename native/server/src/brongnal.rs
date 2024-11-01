use async_trait::async_trait;
use ed25519_dalek::{Signature, VerifyingKey};
use proto::service::brongnal_server::Brongnal;
use proto::service::Message as MessageProto;
use proto::service::PreKeyBundle as PreKeyBundleProto;
use proto::service::SignedPreKey as SignedPreKeyProto;
use proto::service::{
    RegisterPreKeyBundleRequest, RegisterPreKeyBundleResponse, RequestPreKeysRequest,
    RetrieveMessagesRequest, SendMessageRequest, SendMessageResponse,
};
use proto::{parse_verifying_key, parse_x25519_public_key};
use protocol::bundle::verify_bundle;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::error;
use tracing::info;
use x25519_dalek::PublicKey as X25519PublicKey;

pub type CurrentKeys = (VerifyingKey, SignedPreKeyProto);

#[async_trait]
pub trait Storage {
    /// Add a new identity to the storage.
    /// For now, repeated calls should not return an error.
    // TODO(https://github.com/brongan/brongnal/issues/25) - Return error when attempting to overwrite registration.
    async fn register_user(
        &self,
        identity: String,
        ik: VerifyingKey,
        spk: SignedPreKeyProto,
    ) -> tonic::Result<()>;

    /// Replaces the signed pre key for a given identity.
    // TODO(https://github.com/brongan/brongnal/issues/27) -  Implement signed pre key rotation.
    async fn update_spk(&self, identity: String, pre_key: SignedPreKeyProto) -> tonic::Result<()>;

    /// Appends new unburnt one time pre keys for others to message a given identity.
    async fn add_opks(&self, identity: String, pre_keys: Vec<X25519PublicKey>)
        -> tonic::Result<()>;

    /// Retrieves the identity key and signed pre key for a given identity.
    /// A client must first invoke this before messaging a peer.
    async fn get_current_keys(&self, identity: String) -> tonic::Result<CurrentKeys>;

    /// Retrieve a one time pre key for an identity.
    async fn pop_opk(&self, identity: String) -> tonic::Result<Option<X25519PublicKey>>;

    /// Enqueue a message for a given recipient.
    async fn add_message(&self, recipient: String, message: MessageProto) -> tonic::Result<()>;

    /// Retrieve enqueued messages for a given identity.
    async fn get_messages(&self, identity: String) -> tonic::Result<Vec<MessageProto>>;
}

pub struct BrongnalController {
    storage: Arc<dyn Storage + Send + Sync + 'static>,
    receivers: Arc<Mutex<HashMap<String, Sender<tonic::Result<MessageProto>>>>>,
}

impl BrongnalController {
    pub fn new(storage: Arc<dyn Storage + Send + Sync + 'static>) -> BrongnalController {
        BrongnalController {
            storage,
            receivers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn handle_register_pre_key_bundle(
        &self,
        request: RegisterPreKeyBundleRequest,
    ) -> tonic::Result<RegisterPreKeyBundleResponse> {
        let identity: String = request
            .identity
            .clone()
            .ok_or(Status::invalid_argument("request missing identity"))?;
        let ik = parse_verifying_key(request.identity_key())
            .map_err(|_| Status::invalid_argument("request has invalid identity_key"))?;
        let spk_proto = request
            .signed_pre_key
            .ok_or(Status::invalid_argument("request is missing signed prekey"))?;
        let spk = protocol::x3dh::SignedPreKey::try_from(spk_proto.clone())?;
        verify_bundle(&ik, &[spk.pre_key], &spk.signature)
            .map_err(|_| Status::unauthenticated("failed to validate signed prekey signature"))?;

        let opks = request.one_time_key_bundle.ok_or(Status::invalid_argument(
            "request missing one_time_prekey_bundle",
        ))?;
        let pre_keys: Vec<X25519PublicKey> = opks
            .pre_keys
            .iter()
            .map(|key| parse_x25519_public_key(key))
            .collect::<tonic::Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("invalid prekey bundle"))?;
        let signature = Signature::from_slice(opks.signature()).map_err(|_e| {
            Status::invalid_argument("one time prekey bundle signature is invalid")
        })?;
        verify_bundle(&ik, &pre_keys, &signature).map_err(|_| {
            Status::unauthenticated("failed to validate one time prekey bundle signature")
        })?;

        self.storage
            .register_user(identity.clone(), ik, spk_proto)
            .await?;
        self.storage.add_opks(identity, pre_keys).await?;
        Ok(RegisterPreKeyBundleResponse {})
    }

    async fn handle_send_message(
        &self,
        request: SendMessageRequest,
    ) -> tonic::Result<SendMessageResponse> {
        let recipient_identity = request.recipient_identity.ok_or(Status::invalid_argument(
            "request missing recipient_identity",
        ))?;
        let message_proto: MessageProto = request
            .message
            .ok_or(Status::invalid_argument("request missing message"))?;
        // Do some basic validation on the message before persisting it or sending it to the
        // recipient.
        let _ = protocol::x3dh::Message::try_from(message_proto.clone())?;

        let tx = self
            .receivers
            .lock()
            .unwrap()
            .get(&recipient_identity)
            .cloned();
        if let Some(tx) = tx {
            if let Ok(()) = tx.send(Ok(message_proto.clone())).await {
                return Ok(SendMessageResponse {});
            }
        }

        self.storage
            .add_message(recipient_identity, message_proto)
            .await?;

        Ok(SendMessageResponse {})
    }
}

#[tonic::async_trait]
impl Brongnal for BrongnalController {
    async fn register_pre_key_bundle(
        &self,
        request: Request<RegisterPreKeyBundleRequest>,
    ) -> tonic::Result<Response<RegisterPreKeyBundleResponse>> {
        let request = request.into_inner();
        info!("Registering PreKeyBundle for \"{}\".", request.identity());
        let response = self
            .handle_register_pre_key_bundle(request)
            .await
            .inspect_err(|e| error!("Failed to register pre key bundle: {e}"))?;
        Ok(Response::new(response))
    }

    async fn request_pre_keys(
        &self,
        request: Request<RequestPreKeysRequest>,
    ) -> tonic::Result<Response<PreKeyBundleProto>> {
        let request = request.into_inner();
        info!("Retrieving PreKeyBundle for \"{}\".", request.identity());

        let (keys, opk) = tokio::join!(
            self.storage.get_current_keys(request.identity().to_owned()),
            // TODO(https://github.com/brongan/brongnal/issues/26) - Prevent one time key pop abuse.
            self.storage.pop_opk(request.identity().to_owned())
        );
        let (ik, spk) = keys?;
        let opk = opk?;

        let reply = PreKeyBundleProto {
            identity_key: Some(ik.as_bytes().into()),
            one_time_key: opk.map(|opk| opk.as_bytes().into()),
            signed_pre_key: Some(spk),
        };
        Ok(Response::new(reply))
    }

    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> tonic::Result<Response<SendMessageResponse>> {
        let request = request.into_inner();
        info!(
            "Received request to send message to: \"{}\".",
            request.recipient_identity()
        );
        let response = self
            .handle_send_message(request)
            .await
            .inspect_err(|e| error!("Failed to send message: {e}"))?;

        Ok(Response::new(response))
    }

    type RetrieveMessagesStream = ReceiverStream<tonic::Result<MessageProto>>;
    async fn retrieve_messages(
        &self,
        request: Request<RetrieveMessagesRequest>,
    ) -> tonic::Result<Response<Self::RetrieveMessagesStream>> {
        let request = request.into_inner();
        info!("Retrieving \"{}\"'s messages.", request.identity());

        let identity = request
            .identity
            .ok_or(Status::invalid_argument("request missing identity"))
            .inspect_err(|e| error!("Failed to retrieve messages: {e}"))?;
        let (tx, rx) = mpsc::channel(100);

        // TODO(#14) - RetrieveMessages requires proof of possession
        for message in self
            .storage
            .get_messages(identity.clone())
            .await
            .inspect_err(|e| error!("Failed to retrieve messages from storage: {e}"))?
        {
            // TODO handle result.
            let _ = tx.send(Ok(message)).await;
        }
        self.receivers.lock().unwrap().insert(identity, tx);

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
