use crate::messages::*;
use client::{get_messages, register, send_message, X3DHClient};
use proto::service::brongnal_service_client::BrongnalServiceClient as BrongnalClient;
use rinf::debug_print;
use std::{path::PathBuf, sync::Arc};
use tokio_rusqlite::Connection;
use tokio_stream::StreamExt;
use tonic::transport::Channel;

mod messages;

rinf::write_interface!();

async fn await_rust_startup() -> Option<(PathBuf, Option<String>)> {
    let receiver = RustStartup::get_dart_signal_receiver();
    if let Some(dart_signal) = receiver.recv().await {
        let message = dart_signal.message;
        return Some((
            PathBuf::from(message.database_directory().to_owned()),
            message.username,
        ));
    }
    debug_print!("Lost rust startup connection to flutter.");
    None
}

async fn await_register_widget() -> Option<String> {
    let receiver = RegisterUserRequest::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let message: RegisterUserRequest = dart_signal.message;
        match message.username {
            Some(username) => {
                debug_print!("Received request to register {username}");
                return Some(username);
            }
            None => {
                debug_print!("Received empty register request.");
            }
        }
    }
    debug_print!("Lost message connection to flutter!");
    None
}

/// Async task that listens to signals from dart for messages and forwards them to the server.
async fn send_messages(mut stub: BrongnalClient<Channel>, client: Arc<X3DHClient>) {
    let receiver = SendMessage::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let req: SendMessage = dart_signal.message;
        debug_print!("Rust received message from flutter!: {}", req.message());
        match send_message(&mut stub, &client, req.receiver(), req.message()).await {
            Ok(_) => {}
            Err(e) => {
                debug_print!("Failed to message: {e}");
            }
        }
    }
    debug_print!("Lost message connection to flutter!");
}

/// Async task that listens messages from the server, decrypts them, and sends them to flutter.
async fn receive_messages(
    stub: BrongnalClient<Channel>,
    x3dh_client: Arc<X3DHClient>,
    name: String,
) {
    let messages = get_messages(stub, x3dh_client, name);
    tokio::pin!(messages);
    while let Some(decrypted) = messages.next().await {
        if let Err(e) = decrypted {
            debug_print!("[Failed to Decrypt Message]: {e}");
            continue;
        }
        let decrypted = decrypted.unwrap();
        let sender = decrypted.sender_identity;
        let message = String::from_utf8(decrypted.message);
        match &message {
            Ok(message) => {
                debug_print!("[Received Message] {sender}: {message}");
            }
            Err(_) => {
                debug_print!("Decrypted message was not UTF-8 encoded.");
            }
        }
        ReceivedMessage {
            sender: Some(sender),
            message: message.ok(),
        }
        .send_signal_to_dart();
    }
    debug_print!("Lost decrypted message stream.");
}

#[tokio::main]
async fn main() {
    // TODO(https://github.com/brongan/brongnal/issues/36): gracefully handle a lack of network connection.
    let mut stub = BrongnalClient::connect("https://signal.brongan.com:443")
        .await
        .unwrap();

    let (db_dir, mut username) = await_rust_startup()
        .await
        .expect("Rust startup message sent.");
    debug_print!("Flutter persisted username: {username:?}");

    let db_path = db_dir.join("keys.sqlite");
    debug_print!("Database Path: {db_path:?}");
    let client = Arc::new(
        X3DHClient::new(Connection::open(db_path).await.expect("open database"))
            .await
            .expect("init database"),
    );

    while username.is_none() {
        username = await_register_widget().await;
        debug_print!("Registered from register widget: {username:?}");
    }
    let username = username.unwrap();

    debug_print!("Registering with username: {}", username);
    match register(&mut stub, &client.clone(), username.clone()).await {
        Ok(_) => {
            debug_print!("Registered {username}");
            RegisterUserResponse {
                username: Some(username.clone()),
            }
            .send_signal_to_dart();
        }
        Err(e) => {
            debug_print!("Failed to register {username} with error: {e}");
        }
    }

    tokio::spawn(send_messages(stub.clone(), client.clone()));
    tokio::spawn(receive_messages(stub, client, username));

    rinf::dart_shutdown().await;
}
