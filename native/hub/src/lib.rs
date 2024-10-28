use crate::messages::*;
use client::{listen, message, register, sqlite_client::SqliteClient};
use client::{DecryptedMessage, X3DHClient};
use proto::service::brongnal_client::BrongnalClient;
use rinf::debug_print;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;
use tokio;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tonic::transport::Channel;

mod messages;

rinf::write_interface!();

async fn await_rust_startup() -> Option<(PathBuf, Option<String>)> {
    let receiver = RustStartup::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
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

async fn send_messages(mut stub: BrongnalClient<Channel>, client: Arc<Mutex<SqliteClient>>) {
    let receiver = SendMessage::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let req: SendMessage = dart_signal.message;
        debug_print!("Rust received message from flutter!: {}", req.message());
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
    debug_print!("Lost message connection to flutter!");
}

async fn decrypt_messages(
    stub: BrongnalClient<Channel>,
    x3dh_client: Arc<Mutex<dyn X3DHClient + Send>>,
    name: String,
    tx: Sender<DecryptedMessage>,
) {
    if let Err(e) = listen(stub, x3dh_client, name, tx).await {
        debug_print!("Listen terminated with: {e}");
    }
}

async fn send_messages_to_flutter(mut rx: Receiver<DecryptedMessage>) {
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
    debug_print!("Lost decrypted message stream.");
}

#[tokio::main]
async fn main() {
    // TODO gracefully handle a lack of network connection.
    let mut stub = BrongnalClient::connect("https://signal.brongan.com:443")
        .await
        .unwrap();

    let (db_dir, mut username) = await_rust_startup()
        .await
        .expect("Rust startup message sent.");
    debug_print!("Flutter persisted username: {username:?}");

    let db_path = db_dir.join("keys.sqlite");
    debug_print!("Database Path: {db_path:?}");
    let client = Arc::new(Mutex::new(
        SqliteClient::new(Connection::open(db_path).expect("open database"))
            .expect("init database"),
    ));

    while let None = username {
        username = await_register_widget().await;
        debug_print!("Registered from register widget: {username:?}");
    }
    let username = username.unwrap();

    debug_print!("Registering with username: {}", username);
    match register(&mut stub, client.clone(), username.clone()).await {
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

    // TODO these really should not be separate async tasks.
    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(decrypt_messages(stub.clone(), client.clone(), username, tx));
    tokio::spawn(send_messages_to_flutter(rx));

    rinf::dart_shutdown().await;
}
