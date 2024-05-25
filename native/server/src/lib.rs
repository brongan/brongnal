use ed25519_dalek::{Signature, VerifyingKey};
use proto::{
    brongnal_server::Brongnal, PreKeyBundle as PreKeyBundleProto, RegisterPreKeyBundleRequest,
    RegisterPreKeyBundleResponse, RequestPreKeysRequest, RetrieveMessagesRequest,
    SendMessageRequest, SendMessageResponse, SignedPreKey as SignedPreKeyProto,
    SignedPreKeys as SignedPreKeysProto, X3dhMessage as MessageProto,
};
use protocol::bundle::verify_bundle;
use protocol::x3dh::{Message, PreKeyBundle, SignedPreKey, SignedPreKeys};
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use x25519_dalek::PublicKey as X25519PublicKey;

pub mod proto {
    tonic::include_proto!("service");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("service_descriptor");
}

#[derive(Clone, Debug)]
pub struct MemoryServer {
    identity_key: Arc<Mutex<HashMap<String, VerifyingKey>>>,
    current_pre_key: Arc<Mutex<HashMap<String, SignedPreKey>>>,
    one_time_pre_keys: Arc<Mutex<HashMap<String, Vec<X25519PublicKey>>>>,
    messages: Arc<Mutex<HashMap<String, Vec<Message>>>>,
    receivers: Arc<Mutex<HashMap<String, mpsc::Sender<Result<MessageProto, Status>>>>>,
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
            receivers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn spawn(fut: impl futures::Future<Output = ()> + Send + 'static) {
        tokio::spawn(fut);
    }
}

impl Into<SignedPreKeyProto> for SignedPreKey {
    fn into(self) -> SignedPreKeyProto {
        SignedPreKeyProto {
            pre_key: Some(self.pre_key.to_bytes().to_vec()),
            signature: Some(self.signature.to_vec()),
        }
    }
}

impl Into<SignedPreKeysProto> for SignedPreKeys {
    fn into(self) -> SignedPreKeysProto {
        SignedPreKeysProto {
            pre_keys: self
                .pre_keys
                .into_iter()
                .map(|key| key.to_bytes().to_vec())
                .collect(),
            signature: Some(self.signature.to_vec()),
        }
    }
}

fn parse_verifying_key(key: Vec<u8>) -> Result<VerifyingKey, Status> {
    VerifyingKey::from_bytes(
        &key.try_into()
            .map_err(|_| Status::invalid_argument("Key is invalid size."))?,
    )
    .map_err(|_| Status::invalid_argument("ED25519 key was invalid."))
}

fn parse_x25519_public_key(key: Vec<u8>) -> Result<X25519PublicKey, Status> {
    let key: [u8; 32] = key
        .try_into()
        .map_err(|_| Status::invalid_argument("X25519PublicKey was invalid."))?;
    Ok(X25519PublicKey::from(key))
}

impl TryFrom<SignedPreKeyProto> for SignedPreKey {
    type Error = tonic::Status;

    fn try_from(value: SignedPreKeyProto) -> Result<Self, Self::Error> {
        let signature = value
            .signature
            .ok_or(Status::invalid_argument("request missing signature"))?;

        let pre_key = parse_x25519_public_key(
            value
                .pre_key
                .ok_or(Status::invalid_argument("request issing pre key"))?,
        )?;
        let signature = Signature::from_slice(&signature)
            .map_err(|_| Status::invalid_argument("Pre Key has an invalid X25519 Signature"))?;
        Ok(SignedPreKey { pre_key, signature })
    }
}

impl TryFrom<MessageProto> for Message {
    type Error = tonic::Status;

    fn try_from(value: MessageProto) -> Result<Self, Self::Error> {
        let sender_identity_key = parse_verifying_key(value.sender_ik.ok_or(
            Status::invalid_argument("request missing sender_identity_key"),
        )?)?;

        let ephemeral_key = parse_x25519_public_key(
            value
                .ephemeral_key
                .ok_or(Status::invalid_argument("request missing ephemeral_key"))?,
        )?;

        let otk = if let Some(otk) = value.otk {
            Some(parse_x25519_public_key(otk)?)
        } else {
            None
        };

        let ciphertext = String::from_utf8(
            value
                .ciphertext
                .ok_or(Status::invalid_argument("request missing ciphertext"))?
                .to_vec(),
        )
        .map_err(|_| Status::invalid_argument("Invalid ciphertext."))?;

        Ok(Message {
            sender_identity_key,
            ephemeral_key,
            otk,
            ciphertext,
        })
    }
}

impl Into<MessageProto> for Message {
    fn into(self) -> MessageProto {
        MessageProto {
            sender_ik: Some(self.sender_identity_key.to_bytes().to_vec()),
            ephemeral_key: Some(self.ephemeral_key.to_bytes().to_vec()),
            otk: self.otk.map(|otk| otk.to_bytes().to_vec()),
            ciphertext: Some(self.ciphertext.as_bytes().to_vec()),
        }
    }
}

impl TryInto<PreKeyBundle> for PreKeyBundleProto {
    type Error = tonic::Status;

    fn try_into(self) -> Result<PreKeyBundle, Self::Error> {
        let identity_key = parse_verifying_key(
            self.identity_key
                .ok_or(Status::invalid_argument("PreKeyBundle missing ik."))?,
        )?;

        let otk = if let Some(otk) = self.otk {
            Some(parse_x25519_public_key(otk)?)
        } else {
            None
        };

        let spk = self
            .spk
            .ok_or(Status::invalid_argument("PreKeyBundle missing spk."))?
            .try_into()?;

        Ok(PreKeyBundle {
            identity_key,
            otk,
            spk,
        })
    }
}

#[tonic::async_trait]
impl Brongnal for MemoryServer {
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
