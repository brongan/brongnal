use proto::service::brongnal_service_server::{BrongnalService, BrongnalServiceServer};
use proto::service::{
    IdentityKeyRequest, IdentityKeyResponse, Message as MessageProto, PreKeyBundle,
    PreKeyBundleRequest, RegisterPreKeyBundleRequest, RegisterPreKeyBundleResponse,
    RetrieveMessagesRequest, SendMessageRequest, SendMessageResponse,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Default)]
struct InnerState {
    users: HashMap<String, Vec<u8>>,
    bundles: HashMap<Vec<u8>, PreKeyBundle>,
    messages: HashMap<Vec<u8>, Vec<MessageProto>>,
}

#[derive(Clone, Default)]
pub struct MockBackend {
    state: Arc<Mutex<InnerState>>,
}

#[tonic::async_trait]
impl BrongnalService for MockBackend {
    async fn get_identity_key(
        &self,
        request: Request<IdentityKeyRequest>,
    ) -> Result<Response<IdentityKeyResponse>, Status> {
        let request = request.into_inner();
        let state = self.state.lock().unwrap();
        let key = state
            .users
            .get(request.username())
            .ok_or_else(|| Status::not_found("username not found"))?;
        Ok(Response::new(IdentityKeyResponse {
            identity_key: Some(key.clone()),
        }))
    }

    async fn register_pre_key_bundle(
        &self,
        request: Request<RegisterPreKeyBundleRequest>,
    ) -> Result<Response<RegisterPreKeyBundleResponse>, Status> {
        let request = request.into_inner();
        let username = request
            .username
            .filter(|name| !name.is_empty() && name.len() <= 48)
            .ok_or_else(|| Status::invalid_argument("username must contain 1 to 48 bytes"))?;
        let ik = request
            .identity_key
            .ok_or_else(|| Status::invalid_argument("missing identity key"))?;
        let spk = request
            .signed_pre_key
            .ok_or_else(|| Status::invalid_argument("missing signed prekey"))?;
        let mut state = self.state.lock().unwrap();
        if state.users.get(&username).is_some_and(|key| key != &ik) {
            return Err(Status::already_exists("username already registered"));
        }
        state.users.insert(username, ik.clone());
        state.bundles.insert(
            ik.clone(),
            PreKeyBundle {
                identity_key: Some(ik),
                signed_pre_key: Some(spk),
                one_time_key: None,
            },
        );
        Ok(Response::new(RegisterPreKeyBundleResponse {
            num_keys: Some(100),
        }))
    }

    async fn request_pre_keys(
        &self,
        request: Request<PreKeyBundleRequest>,
    ) -> Result<Response<PreKeyBundle>, Status> {
        let req = request.into_inner();
        let ik = req
            .identity_key
            .ok_or(Status::invalid_argument("missing identity key"))?;
        let state = self.state.lock().unwrap();
        let bundle = state
            .bundles
            .get(&ik)
            .ok_or_else(|| Status::not_found("identity key not found"))?;
        Ok(Response::new(bundle.clone()))
    }

    async fn send_message(
        &self,
        request: Request<tonic::Streaming<SendMessageRequest>>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        let mut stream = request.into_inner();
        while let Some(req) = stream.message().await? {
            let recipient = req
                .recipient_identity_key
                .ok_or(Status::invalid_argument("missing recipient"))?;
            let message = req
                .message
                .ok_or(Status::invalid_argument("missing message"))?;
            let mut state = self.state.lock().unwrap();
            state.messages.entry(recipient).or_default().push(message);
        }
        Ok(Response::new(SendMessageResponse {}))
    }

    type RetrieveMessagesStream = ReceiverStream<Result<MessageProto, Status>>;

    async fn retrieve_messages(
        &self,
        request: Request<RetrieveMessagesRequest>,
    ) -> Result<Response<Self::RetrieveMessagesStream>, Status> {
        let req = request.into_inner();
        let ik = req
            .identity_key
            .ok_or(Status::invalid_argument("missing identity key"))?;

        let (tx, rx) = tokio::sync::mpsc::channel(4);
        let state_arc = self.state.clone();

        tokio::spawn(async move {
            loop {
                let msgs = {
                    let mut state = state_arc.lock().unwrap();
                    state.messages.remove(&ik).unwrap_or_default()
                };
                for m in msgs {
                    if tx.send(Ok(m)).await.is_err() {
                        return;
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

/// Binds to the address and returns the listener.
/// Awaiting this ensures the port is reserved.
pub async fn bind(addr: &str) -> Result<tokio::net::TcpListener, Box<dyn std::error::Error>> {
    let addr: std::net::SocketAddr = addr.parse()?;
    Ok(tokio::net::TcpListener::bind(addr).await?)
}

/// Runs the gRPC server on the provided listener until the shutdown signal is received.
pub async fn serve(
    listener: tokio::net::TcpListener,
    shutdown: impl std::future::Future<Output = ()>,
) -> Result<(), tonic::transport::Error> {
    let mock = MockBackend::default();
    Server::builder()
        .add_service(BrongnalServiceServer::new(mock))
        .serve_with_incoming_shutdown(
            tokio_stream::wrappers::TcpListenerStream::new(listener),
            shutdown,
        )
        .await
}
