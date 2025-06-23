use client::{client::MessageModel, User, X3DHClient};
use proto::gossamer::gossamer_service_client::GossamerServiceClient as GossamerClient;
use proto::service::brongnal_service_client::BrongnalServiceClient as BrongnalClient;
use rinf::{debug_print, DartSignal, RustSignal};
use signals::*;
use std::{path::PathBuf, sync::Arc};
use tokio_rusqlite::Connection;
use tokio_stream::StreamExt;

mod signals;

rinf::write_interface!();

async fn rust_startup() -> Option<RustStartup> {
    let receiver = RustStartup::get_dart_signal_receiver();
    if let Some(dart_signal) = receiver.recv().await {
        return Some(dart_signal.message);
    }
    debug_print!("Lost rust startup connection to flutter.");
    None
}

async fn register_widget() -> Option<RegisterUserRequest> {
    let receiver = RegisterUserRequest::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let message: RegisterUserRequest = dart_signal.message;
        debug_print!("Received request to register {}", message.username);
        return Some(message);
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

    let RustStartup {
        database_directory,
        username,
        fcm_token,
    } = rust_startup().await.expect("Rust startup message sent.");
    let database_directory = PathBuf::from(database_directory);
    let db_path = database_directory.join("keys.sqlite");

    debug_print!("Flutter persisted username: {username:?}");

    debug_print!("Database Path: {db_path:?}");
    let client = Arc::new(
        X3DHClient::new(Connection::open(db_path).await.expect("open database"))
            .await
            .expect("init database"),
    );

    let user = if username.is_none() {
        loop {
            if let Some(RegisterUserRequest { username }) = register_widget().await {
                debug_print!("Registering as: {username}");
                match User::new(
                    brongnal.clone(),
                    gossamer.clone(),
                    client.clone(),
                    username.clone(),
                    fcm_token.clone(),
                )
                .await
                {
                    Ok(user) => {
                        RegisterUserResponse { username }.send_signal_to_dart();
                        break user;
                    }
                    Err(e) => {
                        debug_print!("Failed to register username: {e}");
                    }
                }
            }
        }
    } else {
        User::new(brongnal, gossamer, client, username.clone().unwrap(), None)
            .await
            .unwrap()
    };

    for msg in user.get_message_history().await.unwrap().messages {
        msg.send_signal_to_dart();
    }

    let subscriber = user.get_messages().await.unwrap();
    let message_stream = subscriber.into_stream();
    tokio::pin!(message_stream);
    let receiver = SendMessage::get_dart_signal_receiver();

    loop {
        tokio::select! {
            decrypted = message_stream.next() => {
                match decrypted {
                    Some(Ok(msg)) => {
                        msg.send_signal_to_dart();
                        let MessageModel {sender,text, receiver: _, db_recv_time: _, state: _ } = msg;
                        debug_print!("[Received Message] from {sender}: {text}");
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

                        match user.send_message(recipient.clone(), message).await {
                            Ok(id) => {
                                match user.get_message(id).await {
                                    Ok(msg) => msg.send_signal_to_dart(),
                                    Err(e) => debug_print!("Failed to retrieve sent message from DB."),
                                }
                            },
                            Err(e) => debug_print!("Failed to query keys for user: {recipient}: {e}"),
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
