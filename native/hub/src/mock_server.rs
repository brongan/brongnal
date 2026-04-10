use proto::gossamer::gossamer_service_server::{GossamerService, GossamerServiceServer};
use proto::gossamer::{
    ActionRequest, ActionResponse, AttestationRequest, AttestationResponse, GetLedgerRequest,
    Ledger, User as UserProto,
};
use proto::service::brongnal_service_server::{BrongnalService, BrongnalServiceServer};
use proto::service::{
    Message as MessageProto, RegisterPreKeyBundleResponse, PreKeyBundle, PreKeyBundleRequest,
    RegisterPreKeyBundleRequest, RetrieveMessagesRequest, SendMessageRequest, SendMessageResponse,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use prost::Message;

#[derive(Default)]
struct InnerState {
    users: HashMap<Vec<u8>, UserProto>,
    messages: HashMap<Vec<u8>, Vec<MessageProto>>,
}

#[derive(Clone, Default)]
pub struct MockBackend {
    state: Arc<Mutex<InnerState>>,
}

#[tonic::async_trait]
impl GossamerService for MockBackend {
    async fn action(&self, request: Request<ActionRequest>) -> Result<Response<ActionResponse>, Status> {
        let req = request.into_inner();
        let signed = req.message.ok_or(Status::invalid_argument("missing message"))?;

        let contents = proto::gossamer::Message::decode(&*signed.contents.ok_or(Status::invalid_argument("missing contents"))?)
            .map_err(|e| Status::internal(e.to_string()))?;

        let ik = contents.public_key.clone().ok_or(Status::invalid_argument("missing public_key in message"))?;

        let mut state = self.state.lock().unwrap();
        let provider = contents.provider.as_ref().ok_or(Status::invalid_argument("missing provider"))?;
        state.users.entry(provider.clone()).or_insert(UserProto {
            provider: Some(provider.clone()),
            public_keys: vec![ik],
        });

        Ok(Response::new(ActionResponse {}))
    }

    async fn get_ledger(&self, _request: Request<GetLedgerRequest>) -> Result<Response<Ledger>, Status> {
        let state = self.state.lock().unwrap();
        Ok(Response::new(Ledger {
            users: state.users.values().cloned().collect(),
        }))
    }

    async fn get_attestation(
        &self,
        _request: Request<AttestationRequest>,
    ) -> Result<Response<AttestationResponse>, Status> {
        Ok(Response::new(AttestationResponse {
            gca_token: Some("mock.jwt.token".to_string()),
        }))
    }
}

#[tonic::async_trait]
impl BrongnalService for MockBackend {
    async fn register_pre_key_bundle(
        &self,
        _request: Request<RegisterPreKeyBundleRequest>,
    ) -> Result<Response<RegisterPreKeyBundleResponse>, Status> {
        Ok(Response::new(RegisterPreKeyBundleResponse { 
            num_keys: Some(100)
        }))
    }

    async fn request_pre_keys(
        &self,
        request: Request<PreKeyBundleRequest>,
    ) -> Result<Response<PreKeyBundle>, Status> {
        let req = request.into_inner();
        let ik = req.identity_key.ok_or(Status::invalid_argument("missing identity key"))?;
        Ok(Response::new(PreKeyBundle {
            identity_key: Some(ik),
            signed_pre_key: Some(proto::service::SignedPreKey {
                pre_key: Some(vec![0u8; 32]),
                signature: Some(vec![1u8; 64]),
            }),
            one_time_key: Some(vec![2u8; 32]),
        }))
    }

    async fn send_message(
        &self,
        request: Request<tonic::Streaming<SendMessageRequest>>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        let mut stream = request.into_inner();
        while let Some(req) = stream.message().await? {
            let recipient = req.recipient_identity_key.ok_or(Status::invalid_argument("missing recipient"))?;
            let message = req.message.ok_or(Status::invalid_argument("missing message"))?;
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
        let ik = req.identity_key.ok_or(Status::invalid_argument("missing identity key"))?;
        
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
        .add_service(GossamerServiceServer::new(mock.clone()))
        .add_service(BrongnalServiceServer::new(mock))
        .serve_with_incoming_shutdown(
            tokio_stream::wrappers::TcpListenerStream::new(listener),
            shutdown,
        )
        .await
}
