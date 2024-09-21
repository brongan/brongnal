use ed25519_dalek::VerifyingKey;
use proto::service::brongnal_server::Brongnal;
use protocol::bundle::verify_bundle;
use server::parse_verifying_key;
use server::proto;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use x25519_dalek::PublicKey as X25519PublicKey;

#[derive(Clone, Debug)]
pub struct MemoryBrongnal {
    identity_key: Arc<Mutex<HashMap<String, VerifyingKey>>>,
    current_pre_key: Arc<Mutex<HashMap<String, protocol::x3dh::SignedPreKey>>>,
    one_time_pre_keys: Arc<Mutex<HashMap<String, Vec<X25519PublicKey>>>>,
    messages: Arc<Mutex<HashMap<String, Vec<protocol::x3dh::Message>>>>,
    receivers: Arc<Mutex<HashMap<String, Sender<Result<proto::service::Message, Status>>>>>,
}

impl Default for MemoryBrongnal {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBrongnal {
    pub fn new() -> Self {
        MemoryBrongnal {
            identity_key: Arc::new(Mutex::new(HashMap::new())),
            current_pre_key: Arc::new(Mutex::new(HashMap::new())),
            one_time_pre_keys: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
            receivers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tonic::async_trait]
impl Brongnal for MemoryBrongnal {
    async fn register_pre_key_bundle(
        &self,
        request: Request<proto::service::RegisterPreKeyBundleRequest>,
    ) -> Result<Response<proto::service::RegisterPreKeyBundleResponse>, Status> {
        let request = request.into_inner();
        println!("Registering PreKeyBundle for {}", request.identity());
        let identity: String = request
            .identity
            .clone()
            .ok_or(Status::invalid_argument("request missing identity"))?;
        let ik = parse_verifying_key(&request.identity_key())
            .map_err(|_| Status::invalid_argument("PreKeyBundle invalid identity_key"))?;
        let spk = protocol::x3dh::SignedPreKey::try_from(
            request
                .signed_pre_key
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
        Ok(Response::new(
            proto::service::RegisterPreKeyBundleResponse {},
        ))
    }

    async fn request_pre_keys(
        &self,
        request: Request<proto::service::RequestPreKeysRequest>,
    ) -> Result<Response<proto::service::PreKeyBundle>, Status> {
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

        let reply = proto::service::PreKeyBundle {
            identity_key: Some(identity_key.as_bytes().into()),
            one_time_key: otk.map(|otk| otk.as_bytes().into()),
            signed_pre_key: Some(spk.into()),
        };
        Ok(Response::new(reply))
    }

    async fn send_message(
        &self,
        request: Request<proto::service::SendMessageRequest>,
    ) -> Result<Response<proto::service::SendMessageResponse>, Status> {
        let request = request.into_inner();
        println!("Sending a message to: {}", request.recipient_identity());
        let recipient_identity = request.recipient_identity.ok_or(Status::invalid_argument(
            "SendMessageRequest missing recipient_identity",
        ))?;
        let message: proto::service::Message = request
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
                println!("Sent a message");
                return Ok(Response::new(proto::service::SendMessageResponse {}));
            } else {
                eprintln!("Hit a race condition");
                // Idk what can really be done about this race condition.
                self.receivers.lock().unwrap().remove(&recipient_identity);
            }
        }
        println!("Response okayed");
        let mut messages = self.messages.lock().unwrap();
        if !messages.contains_key(&recipient_identity) {
            println!("Did not find recipient_identity, adding their message under new key");
            messages.insert(recipient_identity.clone(), Vec::new());
        }
        messages
            .get_mut(&recipient_identity)
            .unwrap()
            .push(message.try_into()?);
        print!("No issues with messages lock, sending okay response");
        Ok(Response::new(proto::service::SendMessageResponse {}))
    }

    type RetrieveMessagesStream = ReceiverStream<Result<proto::service::Message, Status>>;
    async fn retrieve_messages(
        &self,
        request: Request<proto::service::RetrieveMessagesRequest>,
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
