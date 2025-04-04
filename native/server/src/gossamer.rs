use ed25519_dalek::VerifyingKey;
use proto::gossamer::gossamer_service_server::GossamerService;
use proto::gossamer::{
    ActionRequest, ActionResponse, GetLedgerRequest, Ledger, SignedMessage, User,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};
use tracing::{error, info};

pub struct InMemoryGossamer {
    provider: Arc<Mutex<HashMap<Vec<u8>, VerifyingKey>>>,
    messages: Arc<Mutex<Vec<SignedMessage>>>,
}

impl Default for InMemoryGossamer {
    fn default() -> Self {
        Self {
            provider: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl InMemoryGossamer {
    fn handle_action(&self, message: SignedMessage) -> tonic::Result<()> {
        let signed_message: protocol::gossamer::SignedMessage = message.clone().try_into()?;

        let mut providers = self.provider.lock().unwrap();
        if signed_message.identity_key != signed_message.message.public_key {
            return Err(Status::unimplemented(
                "Multiple identity keys for a given username is not yet supported.",
            ));
        }

        match signed_message.message.action {
            protocol::gossamer::Action::AppendKey => {
                if let Some(key) = providers.get(&signed_message.message.provider) {
                    if key != &signed_message.message.public_key {
                        return Err(Status::already_exists(format!(
                            "Provider: {:?} is already registered.",
                            &signed_message.message.provider
                        )));
                    }
                }
                providers.insert(
                    signed_message.message.provider,
                    signed_message.message.public_key,
                );
            }
            protocol::gossamer::Action::RevokeKey => {
                if let Some(ik) = providers.get(&signed_message.message.provider) {
                    if ik == &signed_message.message.public_key {
                        providers.remove(&signed_message.message.provider);
                    } else {
                        return Err(Status::permission_denied(
                            "this key does not own the entry for this username",
                        ));
                    }
                } else {
                    return Err(Status::failed_precondition(
                        "Cannot revoke key that is not registered.",
                    ));
                }
            }
        }
        self.messages.lock().unwrap().push(message);
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
        info!("Received Ledger Request.");

        let providers = self.provider.lock().unwrap();
        let users = providers
            .iter()
            .map(|(provider, key)| User {
                provider: Some(provider.to_owned()),
                public_keys: vec![key.to_bytes().to_vec()],
            })
            .collect();

        Ok(Response::new(Ledger { users }))
    }
}
