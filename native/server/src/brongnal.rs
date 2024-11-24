use crate::persistence::SqliteStorage;
use ed25519_dalek::{Signature, VerifyingKey};
use proto::service::brongnal_service_server::BrongnalService;
use proto::service::Message as MessageProto;
use proto::service::PreKeyBundle as PreKeyBundleProto;
use proto::service::RequestPreKeyBundleResponse;
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

pub struct BrongnalController {
    storage: SqliteStorage,
    receivers: Arc<Mutex<HashMap<VerifyingKey, Sender<tonic::Result<MessageProto>>>>>,
}

impl BrongnalController {
    pub fn new(storage: SqliteStorage) -> BrongnalController {
        BrongnalController {
            storage,
            receivers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn handle_register_pre_key_bundle(
        &self,
        request: RegisterPreKeyBundleRequest,
    ) -> tonic::Result<RegisterPreKeyBundleResponse> {
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
        self.storage.add_opks(identity.clone(), pre_keys).await?;
        let num_keys = Some(self.storage.get_one_time_prekey_count(identity).await?);
        Ok(RegisterPreKeyBundleResponse { num_keys })
    }

    async fn handle_send_message(
        &self,
        request: SendMessageRequest,
    ) -> tonic::Result<SendMessageResponse> {
        let message_proto: MessageProto = request
            .message
            .ok_or(Status::invalid_argument("request missing message"))?;
        let claimed_sender_identity = request
            .claimed_sender_identity
            .ok_or(Status::invalid_argument("request missing sender identity"))?;
        if !claimed_sender_identity.len() == 32 {
            return Err(Status::invalid_argument("invalid sender identity"));
        }
        let recipient = parse_verifying_key(
            &request
                .recipient_identity_key
                .ok_or(Status::invalid_argument("missing recipient identity key"))?,
        )
        .map_err(|_| Status::invalid_argument("invalid recipient identity key"))?;

        // Do some basic validation on the message before persisting it or sending it to the
        // recipient.
        let message = protocol::x3dh::Message::try_from(message_proto.clone())?;

        info!("Received request to send message to: \"{:?}\".", recipient);

        let tx = self.receivers.lock().unwrap().get(&message.ik).cloned();
        if let Some(tx) = tx {
            if let Ok(()) = tx.send(Ok(message_proto.clone())).await {
                return Ok(SendMessageResponse {});
            }
        }

        self.storage.add_message(&recipient, message_proto).await?;

        Ok(SendMessageResponse {})
    }
}

#[tonic::async_trait]
impl BrongnalService for BrongnalController {
    async fn register_pre_key_bundle(
        &self,
        request: Request<RegisterPreKeyBundleRequest>,
    ) -> tonic::Result<Response<RegisterPreKeyBundleResponse>> {
        let request = request.into_inner();
        info!("Registering PreKeyBundle");
        let response = self
            .handle_register_pre_key_bundle(request)
            .await
            .inspect_err(|e| error!("Failed to register pre key bundle: {e}"))?;
        Ok(Response::new(response))
    }

    async fn request_pre_keys(
        &self,
        request: Request<RequestPreKeysRequest>,
    ) -> tonic::Result<Response<RequestPreKeyBundleResponse>> {
        let request = request.into_inner();

        let ik = parse_verifying_key(
            &request
                .identity_key
                .ok_or(Status::invalid_argument("missing recipient identity key"))?,
        )
        .map_err(|_| Status::invalid_argument("invalid recipient identity key"))?;

        info!("Retrieving PreKeyBundle for \"{:?}\".", ik);

        let (spk, opk) = tokio::join!(
            self.storage.get_current_spk(&ik),
            // TODO(https://github.com/brongan/brongnal/issues/26) - Prevent one time key pop abuse.
            self.storage.pop_opk(&ik)
        );
        let spk = spk?;
        let opk = opk?;

        let reply = RequestPreKeyBundleResponse {
            bundles: vec![PreKeyBundleProto {
                identity_key: Some(ik.as_bytes().into()),
                one_time_key: opk.map(|opk| opk.as_bytes().into()),
                signed_pre_key: Some(spk),
            }],
        };
        Ok(Response::new(reply))
    }

    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> tonic::Result<Response<SendMessageResponse>> {
        let request = request.into_inner();
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

        let ik = parse_verifying_key(
            &request
                .identity_key
                .ok_or(Status::invalid_argument("missing recipient identity key"))?,
        )
        .map_err(|_| Status::invalid_argument("invalid recipient identity key"))?;

        info!("Retrieving \"{ik:?}\"'s messages.");

        let (tx, rx) = mpsc::channel(100);

        // TODO(#14) - RetrieveMessages requires proof of possession
        for message in self
            .storage
            .get_messages(&ik)
            .await
            .inspect_err(|e| error!("Failed to retrieve messages from storage: {e}"))?
        {
            // TODO handle result.
            let _ = tx.send(Ok(message)).await;
        }
        self.receivers.lock().unwrap().insert(ik, tx);

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
