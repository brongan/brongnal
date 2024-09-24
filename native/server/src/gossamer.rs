use ed25519_dalek::VerifyingKey;
use proto::gossamer::gossamer_server::Gossamer;
use proto::gossamer::{ActionRequest, ActionResponse, SignedMessage};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};

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

#[tonic::async_trait]
impl Gossamer for InMemoryGossamer {
    async fn perform(
        &self,
        request: Request<ActionRequest>,
    ) -> Result<Response<ActionResponse>, Status> {
        if let Some(message) = request.into_inner().message {
            // TODO verify signature and parse contents.
            self.messages.lock().unwrap().push(message);
            Ok(Response::new(ActionResponse {}))
        } else {
            Err(Status::invalid_argument("Empty Gossamer Action."))
        }
    }
}
