use client::{listen, MemoryClient};
use messages::brongnal::{
    brongnal_action::Action, BrongnalAction, BrongnalResult, ReceivedMessage,
};
use server::proto::service::brongnal_client::BrongnalClient;
use std::sync::Arc;
// TODO replace with tokio;
use tokio_with_wasm::tokio::{self, sync::mpsc, sync::Mutex};

mod messages;

rinf::write_interface!();

async fn main() {
    let mut receiver = BrongnalAction::get_dart_signal_receiver();
    let stub = BrongnalClient::connect("https://signal.brongan.com:443")
        .await
        .unwrap();
    let client = Arc::new(Mutex::new(MemoryClient::new()));
    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        while let Some(decrypted) = rx.recv().await {
            let message = String::from_utf8(decrypted).ok();
            ReceivedMessage { message }.send_signal_to_dart();
        }
    });

    while let Some(dart_signal) = receiver.recv().await {
        let message: BrongnalAction = dart_signal.message;
        match message.action.unwrap() {
            Action::RegisterName(name) => {
                let client = client.clone();
                let stub = stub.clone();
                let listen_name = name.clone();
                let tx = tx.clone();
                tokio::spawn(async move { listen(stub, client, listen_name, tx).await });
                BrongnalResult {
                    registered_name: Some(name),
                }
                .send_signal_to_dart();
            }
        }
    }
}
