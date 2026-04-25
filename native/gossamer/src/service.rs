use crate::persistence::GossamerStorage;
use proto::gossamer::gossamer_service_server::GossamerService;
use proto::gossamer::{
    ActionRequest, ActionResponse, GetLedgerRequest, Ledger, SignedMessage, User,
};
use tonic::{Request, Response, Status};
use tracing::{error, info, instrument};

pub struct Service {
    storage: GossamerStorage,
}

impl Service {
    pub fn new(storage: GossamerStorage) -> Self {
        Self { storage }
    }

    async fn handle_action(&self, message: SignedMessage) -> tonic::Result<()> {
        // NOTE: The `try_into()` implementation for `SignedMessage` (in proto/src/lib.rs)
        // cryptographically verifies that the `identity_key` signed the `contents`.
        let signed_message: protocol::gossamer::SignedMessage = message.clone().try_into()?;

        let provider = signed_message.message.provider.clone();
        let public_key = signed_message.message.public_key;
        let identity_key = signed_message.identity_key;

        let is_authorized = self
            .storage
            .has_key(provider.clone(), identity_key)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let has_provider = self
            .storage
            .has_provider(provider.clone())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if !is_authorized && has_provider {
            return Err(Status::permission_denied(
                "The signing key is not authorized for this provider.",
            ));
        }

        if !is_authorized && !has_provider {
            if let Some(existing_provider) = self
                .storage
                .get_key_provider(identity_key)
                .await
                .map_err(|e| Status::internal(e.to_string()))?
            {
                return Err(Status::permission_denied(format!(
                    "This key is already associated with another provider: 0x{}",
                    hex::encode(existing_provider)
                )));
            }

            if identity_key != public_key {
                return Err(Status::permission_denied(
                    "New provider names must be claimed by the key being added.",
                ));
            }
        }

        match signed_message.message.action {
            protocol::gossamer::Action::AppendKey => {
                // Security Note: We do NOT require Proof of Possession (PoP) for the key being added
                // if the request is signed by an existing authorized key. The authorized user is
                // trusted to manage their own key set.
                if let Some(owner) = self
                    .storage
                    .get_key_provider(public_key)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?
                    && owner != provider {
                        return Err(Status::permission_denied(format!(
                            "The public key being added is already associated with another provider: 0x{}",
                            hex::encode(owner)
                        )));
                    }

                let _inserted = self
                    .storage
                    .append_key(provider.clone(), public_key)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;
            }
            protocol::gossamer::Action::RevokeKey => {
                let revoked = self
                    .storage
                    .revoke_key(provider.clone(), public_key)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;

                if !revoked {
                    return Err(Status::permission_denied(
                        "Cannot revoke key that is not registered or this key does not own the entry for this username",
                    ));
                }
            }
        }

        self.storage
            .append_message(provider, message)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(())
    }
}

#[tonic::async_trait]
impl GossamerService for Service {
    #[instrument(skip(self, request))]
    async fn action(
        &self,
        request: Request<ActionRequest>,
    ) -> Result<Response<ActionResponse>, Status> {
        info!("Handling Action");
        match request.into_inner().message {
            Some(signed_message) => {
                let _: () = self
                    .handle_action(signed_message)
                    .await
                    .inspect_err(|e| error!("Failed to ingest action: {e}"))?;
                Ok(Response::new(ActionResponse {}))
            }
            _ => Err(Status::invalid_argument("Empty Gossamer Action.")),
        }
    }

    #[instrument(skip(self, _request))]
    async fn get_ledger(
        &self,
        _request: Request<GetLedgerRequest>,
    ) -> Result<Response<Ledger>, Status> {
        info!("Returning Ledger.");
        let grouped_ledger = self
            .storage
            .get_ledger()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let users = grouped_ledger
            .into_iter()
            .map(|(provider, public_keys)| User {
                provider: Some(provider),
                public_keys: public_keys
                    .into_iter()
                    .map(|k| k.as_bytes().to_vec())
                    .collect(),
            })
            .collect();

        Ok(Response::new(Ledger { users }))
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
