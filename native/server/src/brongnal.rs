use crate::persistence::SqliteStorage;
use ed25519_dalek::{Signature, VerifyingKey};
use proto::service::brongnal_service_server::BrongnalService;
use proto::service::{
    Message as MessageProto, PreKeyBundle as PreKeyBundleProto, PreKeyBundleRequest,
    RegisterPreKeyBundleRequest, RegisterPreKeyBundleResponse, RetrieveMessagesRequest,
    SendMessageRequest, SendMessageResponse,
};
use proto::{parse_verifying_key, parse_x25519_public_key};
use protocol::bundle::verify_bundle;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{error, info};
use x25519_dalek::PublicKey as X25519PublicKey;

pub struct BrongnalController {
    storage: SqliteStorage,
    receivers: Arc<Mutex<HashMap<VerifyingKey, Sender<Result<MessageProto>>>>>,
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
    ) -> Result<RegisterPreKeyBundleResponse> {
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
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("invalid prekey bundle"))?;
        let signature = Signature::from_slice(opks.signature()).map_err(|_e| {
            Status::invalid_argument("one time prekey bundle signature is invalid")
        })?;
        verify_bundle(&ik, &pre_keys, &signature).map_err(|_| {
            Status::unauthenticated("failed to validate one time prekey bundle signature")
        })?;

        self.storage.add_user(ik, spk_proto).await?;
        self.storage.add_opks(&ik, pre_keys).await?;
        let num_keys = Some(self.storage.get_one_time_prekey_count(&ik).await?);
        Ok(RegisterPreKeyBundleResponse { num_keys })
    }

    async fn handle_send_message(&self, request: SendMessageRequest) -> Result<()> {
        let message_proto: MessageProto = request
            .message
            .ok_or(Status::invalid_argument("request missing message"))?;

        let recipient = parse_verifying_key(
            &request
                .recipient_identity_key
                .ok_or(Status::invalid_argument("missing recipient identity key"))?,
        )
        .map_err(|_| Status::invalid_argument("invalid recipient identity key"))?;

        // Do some basic validation on the message before persisting it or sending it to the
        // recipient.
        let _message = protocol::x3dh::Message::try_from(message_proto.clone())?;

        #[allow(deprecated)]
        let addr = base64::encode(recipient.as_bytes());
        info!("Received request to send message to: \"{}\".", addr);

        let tx = self.receivers.lock().unwrap().get(&recipient).cloned();
        if let Some(tx) = tx {
            if let Ok(()) = tx.send(Ok(message_proto.clone())).await {
                return Ok(());
            }
        }

        self.storage.add_message(&recipient, message_proto).await?;
        Ok(())
    }
}

#[tonic::async_trait]
impl BrongnalService for BrongnalController {
    async fn register_pre_key_bundle(
        &self,
        request: Request<RegisterPreKeyBundleRequest>,
    ) -> Result<Response<RegisterPreKeyBundleResponse>> {
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
        request: Request<PreKeyBundleRequest>,
    ) -> Result<Response<PreKeyBundleProto>> {
        let request = request.into_inner();

        let ik = parse_verifying_key(
            &request
                .identity_key
                .ok_or(Status::invalid_argument("missing recipient identity key"))?,
        )
        .map_err(|_| Status::invalid_argument("invalid recipient identity key"))?;
        #[allow(deprecated)]
        let ik_str = base64::encode(ik.as_bytes());

        info!("Retrieving PreKeyBundle for \"{}\".", ik_str);

        let (spk, opk) = tokio::join!(
            self.storage.get_current_spk(&ik),
            // TODO(https://github.com/brongan/brongnal/issues/26) - Prevent one time key pop abuse.
            self.storage.pop_opk(&ik)
        );
        let spk = spk?;
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
        request: Request<Streaming<SendMessageRequest>>,
    ) -> Result<Response<SendMessageResponse>> {
        let mut stream = request.into_inner();
        while let Some(request) = stream.next().await {
            let request = request.inspect_err(|e| error!("SendMessageRequest failed: {e}"))?;
            self.handle_send_message(request)
                .await
                .inspect_err(|e| error!("Failed to send message: {e}"))?;
        }
        Ok(Response::new(SendMessageResponse {}))
    }

    type RetrieveMessagesStream = ReceiverStream<Result<MessageProto>>;
    async fn retrieve_messages(
        &self,
        request: Request<RetrieveMessagesRequest>,
    ) -> Result<Response<Self::RetrieveMessagesStream>> {
        let request = request.into_inner();

        let ik = parse_verifying_key(
            &request
                .identity_key
                .ok_or(Status::invalid_argument("missing recipient identity key"))?,
        )
        .map_err(|_| Status::invalid_argument("invalid recipient identity key"))?;

        #[allow(deprecated)]
        let ik_str = base64::encode(ik.as_bytes());
        info!("Retrieving key=\"{ik_str}\"'s messages.");

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
