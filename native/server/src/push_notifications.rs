use base64::{engine::general_purpose::STANDARD as base64, Engine as _};
use gcp_auth::{CustomServiceAccount, TokenProvider};
use serde_json::json;
use tracing::{info, instrument};

pub struct FirebaseCloudMessagingClient {
    client: reqwest::Client,
    service_account: CustomServiceAccount,
}

impl FirebaseCloudMessagingClient {
    pub async fn new(service_account_key: &str) -> anyhow::Result<Self> {
        let client = reqwest::Client::new();
        let service_account = CustomServiceAccount::from_json(service_account_key)?;

        Ok(FirebaseCloudMessagingClient {
            client,
            service_account,
        })
    }

    #[instrument(name = "push", skip(self, fcm_token, message))]
    pub async fn notify(&self, fcm_token: &str, message: &[u8]) -> anyhow::Result<()> {
        let payload = if message.len() <= 2048 {
            base64.encode(message)
        } else {
            String::from("")
        };
        let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
        let url = "https://fcm.googleapis.com/v1/projects/961697365248/messages:send";
        let message = json!({
            "message": {
                "token": fcm_token,
                "notification": {
                    "title": "Brongnal Message",
                },
                "data": {
                    "payload": payload
                }
            }
        });
        let _response = self
            .client
            .post(url)
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    self.service_account.token(scopes).await?.as_str()
                ),
            )
            .header("Content-Type", "application/json")
            .body(message.to_string())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
