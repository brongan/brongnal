use client::{listen, message, register, DecryptedMessage, MemoryClient};
use messages::brongnal::{ReceivedMessage, RegisterUserRequest};
use rinf::debug_print;
use server::proto::service::brongnal_client::BrongnalClient;
use std::sync::Arc;
use tokio::{
    self,
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
};
use tonic::transport::Channel;

use crate::messages::brongnal::{RegisterUserResponse, SendMessage};

mod messages;

rinf::write_interface!();

async fn handle_register_user(
    mut stub: BrongnalClient<Channel>,
    client: Arc<Mutex<MemoryClient>>,
    tx: Sender<DecryptedMessage>,
) {
    let mut receiver = RegisterUserRequest::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let message: RegisterUserRequest = dart_signal.message;
        match message.username {
            Some(name) => {
                debug_print!("Received request to register {name}");
                match register(&mut stub, client.clone(), name.clone()).await {
                    Ok(_) => {
                        debug_print!("Registered {name}");
                    }
                    Err(e) => {
                        debug_print!("Failed to register {name} with error: {e}");
                    }
                }
                let client = client.clone();
                let stub = stub.clone();
                let listen_name = name.clone();
                let tx = tx.clone();
                tokio::spawn(listen(stub, client, listen_name, tx));
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

async fn handle_send_message(mut stub: BrongnalClient<Channel>, client: Arc<Mutex<MemoryClient>>) {
    let mut receiver = SendMessage::get_dart_signal_receiver();
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

async fn main() {
    let stub = BrongnalClient::connect("https://signal.brongan.com:443")
        .await
        .unwrap();
    let client = Arc::new(Mutex::new(MemoryClient::new()));

    let (tx, mut rx) = mpsc::channel(100);
    tokio::spawn(handle_register_user(stub.clone(), client.clone(), tx));
    tokio::spawn(handle_send_message(stub.clone(), client.clone()));

    tokio::spawn(async move {
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
    });
}
