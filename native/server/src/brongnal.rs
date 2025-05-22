use crate::persistence::SqliteStorage;
use crate::push_notifications::FirebaseCloudMessagingClient;
use base64::{engine::general_purpose::STANDARD as base64, Engine as _};
use ed25519_dalek::{Signature, VerifyingKey};
use prost::Message as _;
use proto::service::brongnal_service_server::BrongnalService;
use proto::service::{
    Message as MessageProto, PreKeyBundle as PreKeyBundleProto, PreKeyBundleRequest,
    RegisterPreKeyBundleRequest, RegisterPreKeyBundleResponse, RetrieveMessagesRequest,
    SendMessageRequest, SendMessageResponse, SignedPreKey as SignedPreKeyProto,
};
use proto::{parse_verifying_key, parse_x25519_public_key};
use protocol::bundle::verify_bundle;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{error, info, instrument, warn};
use x25519_dalek::PublicKey as X25519PublicKey;

pub struct BrongnalController {
    storage: SqliteStorage,
    receivers: Arc<Mutex<HashMap<VerifyingKey, Sender<Result<MessageProto>>>>>,
    fcm_client: Option<FirebaseCloudMessagingClient>,
}

impl BrongnalController {
    pub fn new(
        storage: SqliteStorage,
        fcm_client: Option<FirebaseCloudMessagingClient>,
    ) -> BrongnalController {
        BrongnalController {
            storage,
            receivers: Arc::new(Mutex::new(HashMap::new())),
            fcm_client,
        }
    }

    #[instrument(name="", skip(self, ik), fields(ik = base64.encode(ik)))]
    async fn handle_request_pre_keys(&self, ik: VerifyingKey) -> Result<PreKeyBundleProto> {
        let (spk, opk) = tokio::join!(
            self.storage.get_current_spk(&ik),
            // TODO(https://github.com/brongan/brongnal/issues/26) - Prevent one time key pop abuse.
            self.storage.pop_opk(&ik)
        );
        let spk = spk?;
        let opk = opk?;

        info!("Returning Pre Keys");

        Ok(PreKeyBundleProto {
            identity_key: Some(ik.as_bytes().into()),
            one_time_key: opk.map(|opk| opk.as_bytes().into()),
            signed_pre_key: Some(spk),
        })
    }

    #[instrument(name="", skip(self, ik, spk, pre_keys), fields(ik = base64.encode(ik), pre_keys = pre_keys.len()))]
    async fn handle_register_pre_key_bundle(
        &self,
        ik: &VerifyingKey,
        spk: SignedPreKeyProto,
        pre_keys: Vec<X25519PublicKey>,
        fcm_token: Option<String>,
    ) -> Result<RegisterPreKeyBundleResponse> {
        self.storage.add_user(ik, spk).await?;
        self.storage.add_opks(ik, pre_keys).await?;
        if let Some(fcm_token) = fcm_token {
            self.storage.set_fcm_token(ik, fcm_token).await?;
        }
        let num_keys = Some(self.storage.get_one_time_prekey_count(ik).await?);
        info!("Registered Device");
        Ok(RegisterPreKeyBundleResponse { num_keys })
    }

    #[instrument(name="",skip(self, recipient, message), fields(ik = base64.encode(recipient)))]
    async fn handle_send_message(
        &self,
        recipient: &VerifyingKey,
        message: MessageProto,
    ) -> Result<()> {
        info!("Sending message.");
        let tx = self.receivers.lock().unwrap().get(recipient).cloned();
        if let Some(tx) = tx {
            match tx.send(Ok(message.clone())).await {
                Ok(_) => {
                    info!("Delivered message to cached peer.");
                    return Ok(());
                }
                Err(_) => warn!("Failed to deliver message to cached peer."),
            }
        }

        let two_weeks = Duration::new(2 * 7 * 24 * 60 * 60, 0);

        // TODO: write to database and hit push notification API in parralel?
        match (
            self.storage.get_fcm_token(recipient, two_weeks).await?,
            &self.fcm_client,
        ) {
            (Some(fcm_token), Some(fcm_client)) => {
                match fcm_client
                    .notify(&fcm_token, &message.encode_to_vec())
                    .await
                {
                    Ok(()) => info!("Delivered Push Notification"),
                    Err(e) => error!("Failed to send push notification to FCM API: {e}"),
                }
            }
            (None, _) => info!("Recipient device does not have an active FCM token."),
            (_, None) => info!("Cannot notify: GOOGLE_APPLICATION_CREDENTIALS is unset"),
        }

        self.storage.add_message(recipient, message).await?;
        info!("Put message in mailbox.");
        Ok(())
    }

    #[instrument(name="", skip(self, ik), fields(ik = base64.encode(ik)))]
    async fn handle_retrieve_messages(
        &self,
        ik: VerifyingKey,
    ) -> Result<mpsc::Receiver<Result<MessageProto>>> {
        let (tx, rx) = mpsc::channel(100);

        // TODO(#14) - RetrieveMessages requires proof of possession
        for message in self.storage.get_messages(&ik).await.inspect_err(|e| {
            error!(
                %e,
                "Failed to retrieve messages from storage."
            )
        })? {
            // TODO handle result.
            match tx.send(Ok(message)).await {
                Ok(_) => info!("Sent message from mailbox."),
                Err(e) => error!(%e, "Failed to send message from mailbox"),
            }
        }
        self.receivers.lock().unwrap().insert(ik, tx);
        info!("Message Stream Open");
        Ok(rx)
    }
}

#[tonic::async_trait]
impl BrongnalService for BrongnalController {
    #[instrument(skip(self, request))]
    async fn register_pre_key_bundle(
        &self,
        request: Request<RegisterPreKeyBundleRequest>,
    ) -> Result<Response<RegisterPreKeyBundleResponse>> {
        let request = request.into_inner();
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
        let fcm_token = request.fcm_token;
        let response = self
            .handle_register_pre_key_bundle(&ik, spk_proto, pre_keys, fcm_token)
            .await
            .inspect_err(|e| error!(%e, "Failed to register pre key bundle"))?;
        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
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
        let reply = self.handle_request_pre_keys(ik).await?;

        Ok(Response::new(reply))
    }

    #[instrument(skip(self, request))]
    async fn send_message(
        &self,
        request: Request<Streaming<SendMessageRequest>>,
    ) -> Result<Response<SendMessageResponse>> {
        let mut stream = request.into_inner();
        while let Some(request) = stream.next().await {
            let request = request.inspect_err(|e| error!("SendMessageRequest failed: {e}"))?;
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
            let _message = protocol::x3dh::InitiationMessage::try_from(message_proto.clone())?;

            self.handle_send_message(&recipient, message_proto)
                .await
                .inspect_err(|e| error!("Failed to send message: {e}"))?;
        }
        Ok(Response::new(SendMessageResponse {}))
    }

    type RetrieveMessagesStream = ReceiverStream<Result<MessageProto>>;
    #[instrument(skip(self, request))]
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

        let rx = self.handle_retrieve_messages(ik).await?;

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
