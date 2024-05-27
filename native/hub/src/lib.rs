use client::{listen, register, MemoryClient};
use messages::brongnal::{
    brongnal_action::Action, BrongnalAction, BrongnalResult, ReceivedMessage,
};
use server::proto::service::brongnal_client::BrongnalClient;
use std::sync::Arc;
// TODO replace with tokio;
use rinf::debug_print;
use tokio_with_wasm::tokio::{
    self,
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
};

mod messages;

rinf::write_interface!();

async fn register_user(tx: Sender<Vec<u8>>) {
    let mut stub = BrongnalClient::connect("https://signal.brongan.com:443")
        .await
        .unwrap();
    let client = Arc::new(Mutex::new(MemoryClient::new()));

    let mut receiver = BrongnalAction::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let message: BrongnalAction = dart_signal.message;
        match message.action.unwrap() {
            Action::RegisterName(name) => {
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
                BrongnalResult {
                    registered_name: Some(name),
                }
                .send_signal_to_dart();
            }
        }
    }
}

async fn main() {
    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(register_user(tx));

    tokio::spawn(async move {
        while let Some(decrypted) = rx.recv().await {
            let message = String::from_utf8(decrypted).ok();
            if let Some(message) = &message {
                debug_print!("Received Brongnal message: {message}");
            }
            ReceivedMessage { message }.send_signal_to_dart();
        }
    });
}
