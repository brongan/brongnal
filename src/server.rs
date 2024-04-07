use brongnal::traits::X3DHServer;
use brongnal::MemoryServer;
use futures::prelude::*;
use futures::{future, prelude::*};
use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    time::Duration,
};
use tarpc::{
    context,
    server::{self, incoming::Incoming, Channel},
    tokio_serde::formats::Json,
};
use tokio::time;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (_client_transport, server_transport) = tarpc::transport::channel::unbounded();
    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), 8080);
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // Limit channels to 1 per IP.
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        .map(|channel| {
            let server = MemoryServer::new();
            channel.execute(server.serve()).for_each(spawn)
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;
    Ok(())
}
