use proto::gossamer::gossamer_service_server::GossamerServiceServer as GossamerServer;
use proto::FILE_DESCRIPTOR_SET;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use tokio_rusqlite::Connection;
use tonic::transport::Server;
use tonic_reflection::server::Builder;
use tracing::{info, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use xdg::BaseDirectories;

mod service;
mod persistence;

use service::Service;
use persistence::GossamerStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_level(true)
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .with_target(false)
        .without_time()
        .finish()
        .with(EnvFilter::from_default_env())
        .try_init()?;

    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let server_addr = (IpAddr::V4(Ipv4Addr::UNSPECIFIED), 50052).into();

    let xdg_dirs = BaseDirectories::with_prefix("brongnal")?;
    let db_path: PathBuf = if let Ok(db_path_env) = std::env::var("DB") {
        PathBuf::from(db_path_env)
    } else {
        xdg_dirs.place_data_file("gossamer.db").unwrap()
    };
    info!("Gossamer Service Database Path: {}", db_path.display());
    
    let connection = Connection::open(db_path).await?;
    let storage = GossamerStorage::new(connection).await?;
    let handler = Service::new(storage);

    info!("Gossamer Server answering at: {server_addr}");

    Server::builder()
        .add_service(GossamerServer::new(handler))
        .add_service(reflection_service)
        .serve(server_addr)
        .await?;

    Ok(())
}
