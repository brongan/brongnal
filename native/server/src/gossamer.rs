use ed25519_dalek::VerifyingKey;
use proto::gossamer::gossamer_service_server::GossamerService;
use proto::gossamer::{ActionRequest, ActionResponse, GetLedgerRequest, Ledger, SignedMessage};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};
use tracing::{error, info};

pub struct InMemoryGossamer {
    _provider: Arc<Mutex<HashMap<String, VerifyingKey>>>,
    messages: Arc<Mutex<Vec<SignedMessage>>>,
}

impl Default for InMemoryGossamer {
    fn default() -> Self {
        Self {
            _provider: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl InMemoryGossamer {
    fn handle_action(&self, message: SignedMessage) -> tonic::Result<()> {
        let signed_message: protocol::gossamer::SignedMessage = message.try_into()?;

        match signed_message.message.action {
            protocol::gossamer::Action::AppendKey => {
                // Verify username is not claimed or if it is, the signer is already registered.
            }
            protocol::gossamer::Action::RevokeKey => {
                // Verify key is claimed by key?
            }
        }
        todo!()
    }
}

#[tonic::async_trait]
impl GossamerService for InMemoryGossamer {
    async fn action(
        &self,
        request: Request<ActionRequest>,
    ) -> Result<Response<ActionResponse>, Status> {
        info!("Received Action Request.");
        match request.into_inner().message {
            Some(signed_message) => {
                let _: () = self
                    .handle_action(signed_message)
                    .inspect_err(|e| error!("Failed to ingest action: {e}"))?;
                Ok(Response::new(ActionResponse {}))
            }
            _ => Err(Status::invalid_argument("Empty Gossamer Action.")),
        }
    }

    async fn get_ledger(
        &self,
        request: Request<GetLedgerRequest>,
    ) -> Result<Response<Ledger>, Status> {
        let request = request.into_inner();
        info!("Received Ledger Request: {request:?}");

        // TODO return ledger
        Ok(Response::new(Ledger { users: Vec::new() }))
    }
}
