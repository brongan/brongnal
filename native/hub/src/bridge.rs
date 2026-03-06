pub use client::client::{MessageModel, MessageState};
use client::{User, X3DHClient};
use flutter_rust_bridge::frb;
use proto::gossamer::gossamer_service_client::GossamerServiceClient as GossamerClient;
use proto::service::brongnal_service_client::BrongnalServiceClient as BrongnalClient;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tokio_rusqlite::Connection;
use tokio_stream::StreamExt;
use tracing::error;

use crate::frb_generated::StreamSink;

#[frb(mirror(MessageModel))]
struct _MessageModel {
    pub sender: String,
    pub receiver: String,
    pub db_recv_time: i64,
    pub state: MessageState,
    pub text: String,
}

#[frb(mirror(MessageState))]
enum _MessageState {
    Sending,
    Sent,
    Delivered,
    Read,
}

pub enum BridgeError {
    RegistrationFailed(String),
    InitializationFailed(String),
    MessageSendFailed(String),
}

#[frb(ignore)]
pub struct HubState {
    pub user: Arc<Mutex<Option<User>>>,
}

lazy_static::lazy_static! {
    static ref STATE: HubState = HubState {
        user: Arc::new(Mutex::new(None)),
    };
}

#[frb(init)]
pub fn init_app() {
    std::env::set_var("RUST_LOG", "info,rustls=error,hyper=error,h2=error");
    flutter_rust_bridge::setup_default_user_utils();
}

pub async fn start_hub(
    database_directory: String,
    username: Option<String>,
    fcm_token: Option<String>,
    backend_address: Option<String>,
) -> Result<(), BridgeError> {
    let addr = backend_address.unwrap_or_else(|| "https://signal.brongan.com:443".to_string());

    let brongnal = BrongnalClient::connect(addr.clone())
        .await
        .map_err(|e| BridgeError::InitializationFailed(e.to_string()))?;
    let gossamer = GossamerClient::connect(addr)
        .await
        .map_err(|e| BridgeError::InitializationFailed(e.to_string()))?;

    let db_path = PathBuf::from(database_directory).join("keys.sqlite");
    let client = Arc::new(
        X3DHClient::new(
            Connection::open(db_path)
                .await
                .map_err(|e| BridgeError::InitializationFailed(e.to_string()))?,
        )
        .await
        .map_err(|e| BridgeError::InitializationFailed(e.to_string()))?,
    );

    if let Some(uname) = username {
        let mut user = User::new(brongnal, gossamer, client, uname);
        
        let mut state_user = STATE.user.lock().await;
        *state_user = Some(user.clone());

        tokio::spawn(async move {
            if let Err(e) = user.register(fcm_token).await {
                error!("Background user registration failed: {}", e);
            }
        });
    }

    Ok(())
}

pub async fn register_user(
    username: String,
    fcm_token: Option<String>,
    backend_address: String,
    database_directory: String,
) -> Result<(), BridgeError> {
    let brongnal = BrongnalClient::connect(backend_address.clone())
        .await
        .map_err(|e| BridgeError::RegistrationFailed(e.to_string()))?;
    let gossamer = GossamerClient::connect(backend_address)
        .await
        .map_err(|e| BridgeError::RegistrationFailed(e.to_string()))?;

    let db_path = PathBuf::from(database_directory).join("keys.sqlite");
    let client = Arc::new(
        X3DHClient::new(
            Connection::open(db_path)
                .await
                .map_err(|e| BridgeError::RegistrationFailed(e.to_string()))?,
        )
        .await
        .map_err(|e| BridgeError::RegistrationFailed(e.to_string()))?,
    );

    let mut user = User::new(brongnal, gossamer, client, username);
    user.register(fcm_token)
        .await
        .map_err(|e| BridgeError::RegistrationFailed(e.to_string()))?;

    let mut state_user = STATE.user.lock().await;
    *state_user = Some(user);

    Ok(())
}

pub async fn send_message(recipient: String, text: String) -> Result<MessageModel, BridgeError> {
    let state_user = STATE.user.lock().await;
    let user = state_user.as_ref().ok_or(BridgeError::MessageSendFailed(
        "User not initialized".to_string(),
    ))?;

    let id = user
        .send_message(recipient.clone(), text)
        .await
        .map_err(|e| BridgeError::MessageSendFailed(e.to_string()))?;

    let msg = user.get_message(id).await.map_err(|e| {
        BridgeError::MessageSendFailed(format!("Failed to retrieve sent message: {e}"))
    })?;

    Ok(msg)
}

pub async fn get_all_messages() -> Result<Vec<MessageModel>, BridgeError> {
    let state_user = STATE.user.lock().await;
    let user = state_user
        .as_ref()
        .ok_or(BridgeError::InitializationFailed(
            "User not initialized".to_string(),
        ))?;

    let history = user
        .get_message_history()
        .await
        .map_err(|e| BridgeError::InitializationFailed(e.to_string()))?;

    Ok(history)
}

pub async fn subscribe_messages(sink: StreamSink<MessageModel>) -> Result<(), BridgeError> {
    let state_user = STATE.user.lock().await;
    let user = state_user.as_ref().ok_or(BridgeError::InitializationFailed(
        "User not initialized".to_string(),
    ))?;

    let subscriber = user
        .get_messages()
        .await
        .map_err(|e| BridgeError::InitializationFailed(e.to_string()))?;
    let message_stream = subscriber.into_stream();

    tokio::spawn(async move {
        tokio::pin!(message_stream);
        while let Some(msg) = message_stream.next().await {
            match msg {
                Ok(m) => {
                    let _ = sink.add(m);
                }
                Err(e) => {
                    error!("Stream error: {e}");
                    break;
                }
            }
        }
    });

    Ok(())
}
pub async fn start_mock_server(port: u16) -> Result<(), BridgeError> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = crate::mock_server::bind(&addr)
        .await
        .map_err(|e| BridgeError::InitializationFailed(e.to_string()))?;
    tokio::spawn(crate::mock_server::serve(listener, std::future::pending()));
    
    Ok(())
}
