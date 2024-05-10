// TODO replace with tokio;
use tokio_with_wasm::tokio;

mod messages;

rinf::write_interface!();

async fn main() {
    // TODO tokio::spawn that processes messages from Dart.
}
