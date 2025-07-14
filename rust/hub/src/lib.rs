use client::{client::MessageModel, User, X3DHClient};
use proto::gossamer::gossamer_service_client::GossamerServiceClient as GossamerClient;
use proto::service::brongnal_service_client::BrongnalServiceClient as BrongnalClient;
use std::{path::PathBuf, sync::Arc};
use tokio_rusqlite::Connection;
use tokio_stream::StreamExt;

pub async fn flutter_init_user(
    database_directory: String,
    username: String,
    fcm_token: Option<String>,
) -> User {
    // TODO(https://github.com/brongan/brongnal/issues/36): gracefully handle a lack of network connection.
    let addr = "https://signal.brongan.com:443";
    let brongnal = BrongnalClient::connect(addr).await.unwrap();
    let gossamer = GossamerClient::connect(addr).await.unwrap();
    let database_directory = PathBuf::from(database_directory);
    let db_path = database_directory.join("keys.sqlite");

    // println!("Flutter persisted username: {username:?}");

    // println!("Database Path: {db_path:?}");
    let client = Arc::new(
        X3DHClient::new(Connection::open(db_path).await.expect("open database"))
            .await
            .expect("init database"),
    );

    User::new(brongnal, gossamer, client, username, fcm_token)
        .await
        .unwrap()
}

pub fn register_username(username: String) -> User {
    todo!()
}

#[flutter_rust_bridge::frb(init)]
async fn main() {
    flutter_rust_bridge::setup_default_user_utils();
    /*
    println!("Registering as: {username}");

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
                        println!("[Received Message] from {sender}: {text}");
                    },
                    Some(Err(e)) => {
                        println!("[Failed to Decrypt Message]: {e}");
                    },
                    None => {
                        println!("Lost decrypted message stream.");
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
                        println!("Rust received message from flutter({sender})!: {}", &message);

                        match user.send_message(recipient.clone(), message).await {
                            Ok(id) => {
                                match user.get_message(id).await {
                                    Ok(msg) => msg.send_signal_to_dart(),
                                    Err(e) => println!("Failed to retrieve sent message from DB."),
                                }
                            },
                            Err(e) => println!("Failed to query keys for user: {recipient}: {e}"),
                        }
                    },
                    None => {
                        println!("Lost message connection to flutter!");
                        break;
                    },
                }
            }
        }
    }
    */
}
