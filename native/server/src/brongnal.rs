use ed25519_dalek::VerifyingKey;
use protocol::bundle::verify_bundle;
use protocol::x3dh::{Message, SignedPreKey};
use server::parse_verifying_key;
use server::proto::{
    brongnal_server::Brongnal, PreKeyBundle as PreKeyBundleProto, RegisterPreKeyBundleRequest,
    RegisterPreKeyBundleResponse, RequestPreKeysRequest, RetrieveMessagesRequest,
    SendMessageRequest, SendMessageResponse, X3dhMessage as MessageProto,
};
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use x25519_dalek::PublicKey as X25519PublicKey;

#[derive(Clone, Debug)]
pub struct InMemoryBrongnal {
    identity_key: Arc<Mutex<HashMap<String, VerifyingKey>>>,
    current_pre_key: Arc<Mutex<HashMap<String, SignedPreKey>>>,
    one_time_pre_keys: Arc<Mutex<HashMap<String, Vec<X25519PublicKey>>>>,
    messages: Arc<Mutex<HashMap<String, Vec<Message>>>>,
    receivers: Arc<Mutex<HashMap<String, mpsc::Sender<Result<MessageProto, Status>>>>>,
}

impl Default for InMemoryBrongnal {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryBrongnal {
    pub fn new() -> Self {
        InMemoryBrongnal {
            identity_key: Arc::new(Mutex::new(HashMap::new())),
            current_pre_key: Arc::new(Mutex::new(HashMap::new())),
            one_time_pre_keys: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
            receivers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tonic::async_trait]
impl Brongnal for InMemoryBrongnal {
    async fn register_pre_key_bundle(
        &self,
        request: Request<RegisterPreKeyBundleRequest>,
    ) -> Result<Response<RegisterPreKeyBundleResponse>, Status> {
        let request = request.into_inner();
        println!("Registering PreKeyBundle for {}", request.identity());
        let identity = request
            .identity
            .ok_or(Status::invalid_argument("request missing identity"))?;
        let ik = request
            .ik
            .ok_or(Status::invalid_argument("request missing ik"))?;
        let ik = parse_verifying_key(ik)?;
        let spk = SignedPreKey::try_from(
            request
                .spk
                .ok_or(Status::invalid_argument("Request Missing SPK."))?,
        )?;
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
        let request = request.into_inner();
        println!("RequestingPreKey Bundle for {}", request.identity());
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
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        let request = request.into_inner();
        println!("Sending a message to: {}", request.recipient_identity());
        let recipient_identity = request.recipient_identity.ok_or(Status::invalid_argument(
            "SendMessageRequest missing recipient_identity",
        ))?;
        let message: MessageProto = request
            .message
            .ok_or(Status::invalid_argument(
                "SendMessageRequest missing message.",
            ))?
            .into();

        let tx = self
            .receivers
            .lock()
            .unwrap()
            .get(&recipient_identity)
            .map(|tx| tx.to_owned());
        if let Some(tx) = tx {
            if let Ok(()) = tx.send(Ok(message.clone())).await {
                return Ok(Response::new(SendMessageResponse {}));
            } else {
                // Idk what can really be done about this race condition.
                self.receivers.lock().unwrap().remove(&recipient_identity);
            }
        }

        let mut messages = self.messages.lock().unwrap();
        if !messages.contains_key(&recipient_identity) {
            messages.insert(recipient_identity.clone(), Vec::new());
        }
        messages
            .get_mut(&recipient_identity)
            .unwrap()
            .push(message.try_into()?);
        Ok(Response::new(SendMessageResponse {}))
    }

    type RetrieveMessagesStream = ReceiverStream<Result<MessageProto, Status>>;
    async fn retrieve_messages(
        &self,
        request: Request<RetrieveMessagesRequest>,
    ) -> Result<Response<Self::RetrieveMessagesStream>, Status> {
        let request = request.into_inner();
        println!("Retrieving {}'s messages.", request.identity());
        let identity = request
            .identity
            .ok_or(Status::invalid_argument("request missing identity"))?;
        let (tx, rx) = mpsc::channel(4);

        let messages = self
            .messages
            .lock()
            .unwrap()
            .remove(&identity)
            .unwrap_or(Vec::new());

        for message in messages {
            // TODO handle result.
            let _ = tx.send(Ok(message.into())).await;
        }
        self.receivers.lock().unwrap().insert(identity, tx);

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
