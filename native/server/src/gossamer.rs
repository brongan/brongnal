use ed25519_dalek::{Signature, VerifyingKey};
use prost::Message;
use proto::gossamer::gossamer_service_server::GossamerService;
use proto::gossamer::{
    ActionRequest, ActionResponse, AppendKey, GetLedgerRequest, GossamerMessage, Ledger, RevokeKey,
    SignedMessage,
};
use proto::parse_verifying_key;
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
        if message.contents().len() == 0 {
            return Err(Status::invalid_argument("empty action contents"));
        }

        let signature = Signature::from_slice(&message.signature())
            .map_err(|_| Status::invalid_argument("invalid signature"))?;

        let public_key = parse_verifying_key(message.public_key())
            .map_err(|_| Status::invalid_argument("invalid public key"))?;

        public_key
            .verify_strict(message.contents(), &signature)
            .map_err(|_| Status::invalid_argument("signature error"))?;

        let message = GossamerMessage::decode(&*message.contents())?;
        let action = match message.action {
            Some(action) => action,
            None => return Err(Status::invalid_argument("empty message")),
        };
        match action {
            AppendKey(provider, public_key, key_purpose) => {
                if provider.len() == 32 {
                    return Err(Status::invalid_argument("invalid action provider"));
                }
                todo!();
            }
            RevokeKey(provider, public_key, public_key) => {
                todo!();
            }
        }

        Ok(())
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
