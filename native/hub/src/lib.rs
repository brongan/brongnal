use crate::messages::*;
use client::{get_messages, register_device, register_username, send_message, X3DHClient};
use ed25519_dalek::SigningKey;
use proto::gossamer::gossamer_service_client::GossamerServiceClient as GossamerClient;
use proto::service::brongnal_service_client::BrongnalServiceClient as BrongnalClient;
use proto::ApplicationMessage;
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
async fn send_messages(
    mut brongnal: BrongnalClient<Channel>,
    mut gossamer: GossamerClient<Channel>,
    ik: SigningKey,
    name: String,
) {
    let receiver = SendMessage::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let req: SendMessage = dart_signal.message;
        let message = req.message.unwrap();
        let recipient = req.receiver.unwrap();
        debug_print!("Rust received message from flutter!: {}", &message);

        let msg = ApplicationMessage {
            claimed_sender: name.clone(),
            text: message,
        };

        if let Err(e) =
            send_message(&mut brongnal, &mut gossamer, ik.clone(), &recipient, msg).await
        {
            debug_print!("Failed to query keys for user: {recipient}: {e}");
        }
    }
    debug_print!("Lost message connection to flutter!");
}

/// Async task that listens messages from the server, decrypts them, and sends them to flutter.
async fn receive_messages(stub: BrongnalClient<Channel>, x3dh_client: Arc<X3DHClient>) {
    let messages = get_messages(stub, x3dh_client);
    tokio::pin!(messages);
    while let Some(decrypted) = messages.next().await {
        if let Err(e) = decrypted {
            debug_print!("[Failed to Decrypt Message]: {e}");
            continue;
        }
        let ApplicationMessage {
            claimed_sender,
            text,
        } = decrypted.unwrap();
        debug_print!("[Received Message] from (claimed) {claimed_sender}: {text}",);
        // TODO validate sender claim
        ReceivedMessage {
            message: Some(text),
            sender: Some(claimed_sender),
        }
        .send_signal_to_dart();
    }
    debug_print!("Lost decrypted message stream.");
}

#[tokio::main]
async fn main() {
    // TODO(https://github.com/brongan/brongnal/issues/36): gracefully handle a lack of network connection.
    let addr = "https://signal.brongan.com:443";
    let mut brongnal = BrongnalClient::connect(addr).await.unwrap();
    let mut gossamer = GossamerClient::connect(addr).await.unwrap();

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
    let ik = client.get_ik();

    while username.is_none() {
        username = await_register_widget().await;
        let ik = client.get_ik();
        // TODO gracefully handle failure here >.<
        match register_username(&mut gossamer, ik, username.clone().unwrap()).await {
            Ok(_) => {
                debug_print!("Registered from register widget: {username:?}");
            }
            Err(e) => {
                debug_print!("Failed to register username: {e}");
            }
        }
    }
    let username = username.unwrap();

    debug_print!("Registering with username: {}", username);
    match register_device(&mut brongnal, &client.clone()).await {
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

    tokio::spawn(send_messages(brongnal.clone(), gossamer, ik, username));
    tokio::spawn(receive_messages(brongnal, client));

    rinf::dart_shutdown().await;
}
