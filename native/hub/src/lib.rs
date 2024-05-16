use client::BrongnalUser;
use messages::brongnal::{brongnal_action::Action, BrongnalAction, BrongnalResult};
// TODO replace with tokio;
use tokio_with_wasm::tokio;

mod messages;

rinf::write_interface!();

async fn actions() {
    let mut user = BrongnalUser::memory_user().await.unwrap();
    let mut receiver = BrongnalAction::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let message: BrongnalAction = dart_signal.message;
        match message.action.unwrap() {
            Action::RegisterName(name) => {
                user.register(&name).await.unwrap();
                BrongnalResult {
                    registered_name: Some(name),
                }
                .send_signal_to_dart();
            }
        }
    }
}

async fn main() {
    tokio::spawn(actions());
}
