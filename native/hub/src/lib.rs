use client::{User, X3DHClient};
use proto::gossamer::gossamer_service_client::GossamerServiceClient as GossamerClient;
use proto::service::brongnal_service_client::BrongnalServiceClient as BrongnalClient;
use proto::ApplicationMessage;
use rinf::debug_print;
use rinf::{DartSignal, RustSignal};
use signals::*;
use std::{path::PathBuf, sync::Arc};
use tokio_rusqlite::Connection;
use tokio_stream::StreamExt;

mod signals;

rinf::write_interface!();

async fn await_rust_startup() -> Option<(PathBuf, Option<String>)> {
    let receiver = RustStartup::get_dart_signal_receiver();
    if let Some(dart_signal) = receiver.recv().await {
        let message = dart_signal.message;
        return Some((
            PathBuf::from(message.database_directory.to_owned()),
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
        debug_print!("Received request to register {}", message.username);
        return Some(message.username);
    }
    debug_print!("Lost message connection to flutter!");
    None
}

#[tokio::main]
async fn main() {
    // TODO(https://github.com/brongan/brongnal/issues/36): gracefully handle a lack of network connection.
    let addr = "https://signal.brongan.com:443";
    let brongnal = BrongnalClient::connect(addr).await.unwrap();
    let gossamer = GossamerClient::connect(addr).await.unwrap();

    let (db_dir, username) = await_rust_startup()
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

    let user = if username.is_none() {
        loop {
            let username = await_register_widget().await.unwrap();
            match User::new(
                brongnal.clone(),
                gossamer.clone(),
                client.clone(),
                username.clone(),
            )
            .await
            {
                Ok(user) => {
                    debug_print!("Registered from register widget: {username}");
                    RegisterUserResponse { username }.send_signal_to_dart();
                    break user;
                }
                Err(e) => {
                    debug_print!("Failed to register username: {e}");
                }
            }
        }
    } else {
        User::new(brongnal, gossamer, client, username.clone().unwrap())
            .await
            .unwrap()
    };

    let subscriber = user.get_messages().await.unwrap();
    let message_stream = subscriber.into_stream();
    tokio::pin!(message_stream);
    let receiver = SendMessage::get_dart_signal_receiver();

    loop {
        tokio::select! {
            decrypted = message_stream.next() => {
                match decrypted {
                    Some(Ok(msg)) => {
                        let ApplicationMessage {
                            sender,
                            text,
                        } = msg;
                        debug_print!("[Received Message] from {sender}: {text}",);
                        ReceivedMessage {
                            message: text,
                            sender: sender,
                        }
                        .send_signal_to_dart();

                    },
                    Some(Err(e)) => {
                        debug_print!("[Failed to Decrypt Message]: {e}");
                    },
                    None => {
                        debug_print!("Lost decrypted message stream.");
                        break;
                    },
                }
            },
            dart_signal = receiver.recv() => {
                match dart_signal {
                    Some(dart_signal) => {
                        let SendMessage {
                            sender,
                            message,
                            recipient
                        } = dart_signal.message;
                        debug_print!("Rust received message from flutter({sender})!: {}", &message);

                        if let Err(e) = user.send_message(&recipient, message).await {
                            debug_print!("Failed to query keys for user: {recipient}: {e}");
                        }
                    },
                    None => {
                        debug_print!("Lost message connection to flutter!");
                        break;
                    },
                }
            }
        }
    }

    debug_print!("rinf::dart_shutdown().await");
    rinf::dart_shutdown().await;
}
