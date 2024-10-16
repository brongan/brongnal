use crate::messages::*;
use client::{listen, message, register, sqlite_client::SqliteClient, DecryptedMessage};
use proto::service::brongnal_client::BrongnalClient;
use rinf::debug_print;
use std::path::PathBuf;
use std::sync::Arc;
use tokio;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use tonic::transport::Channel;

mod messages;

rinf::write_interface!();

async fn register_and_listen(
    mut stub: BrongnalClient<Channel>,
    client: Arc<Mutex<SqliteClient>>,
    tx: Sender<DecryptedMessage>,
    username: String,
) {
    match register(&mut stub, client.clone(), username.clone()).await {
        Ok(_) => {
            debug_print!("Registered {username}");
        }
        Err(e) => {
            debug_print!("Failed to register {username} with error: {e}");
            return;
        }
    }
    if let Err(e) = listen(stub, client, username, tx).await {
        debug_print!("Listen terminated with: {e}");
    }
}

async fn handle_register_user(
    stub: BrongnalClient<Channel>,
    client: Arc<Mutex<SqliteClient>>,
    tx: Sender<DecryptedMessage>,
) {
    let receiver = RegisterUserRequest::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let message: RegisterUserRequest = dart_signal.message;
        match message.username {
            Some(name) => {
                debug_print!("Received request to register {name}");
                tokio::spawn(register_and_listen(
                    stub.clone(),
                    client.clone(),
                    tx.clone(),
                    name.clone(),
                ));
                RegisterUserResponse {
                    username: Some(name),
                }
                .send_signal_to_dart();
            }
            None => {
                debug_print!("Received empty register request.");
            }
        }
    }
}

async fn handle_send_message(mut stub: BrongnalClient<Channel>, client: Arc<Mutex<SqliteClient>>) {
    let receiver = SendMessage::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let req: SendMessage = dart_signal.message;
        match message(
            &mut stub,
            client.clone(),
            req.sender().to_owned(),
            req.receiver(),
            req.message(),
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                debug_print!("Failed to message: {e}");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let stub = BrongnalClient::connect("https://signal.brongan.com:443")
        .await
        .unwrap();

    let receiver = RustStartup::get_dart_signal_receiver();
    let (db_dir, username) = loop {
        if let Some(dart_signal) = receiver.recv().await {
            let message = dart_signal.message;
            break (
                PathBuf::from(message.database_directory().to_owned()),
                message.username,
            );
        }
    };

    debug_print!("Database Directory: {db_dir:?}");
    let identity_key_path = db_dir.join("identity_key");
    let db_path = db_dir.join("keys.sqlite");
    let client = Arc::new(Mutex::new(
        SqliteClient::new(&identity_key_path, &db_path).unwrap(),
    ));

    let (tx, mut rx) = mpsc::channel(100);
    if let Some(username) = username {
        register_and_listen(stub.clone(), client.clone(), tx, username).await;
    } else {
        tokio::spawn(handle_register_user(stub.clone(), client.clone(), tx));
    }
    tokio::spawn(handle_send_message(stub.clone(), client.clone()));

    while let Some(decrypted) = rx.recv().await {
        let message = String::from_utf8(decrypted.message).ok();
        if let Some(message) = &message {
            debug_print!(
                "[Received Message] {}: {message}",
                decrypted.sender_identity
            );
        }
        ReceivedMessage {
            sender: Some(decrypted.sender_identity),
            message,
        }
        .send_signal_to_dart();
    }
    rinf::dart_shutdown().await;
}
