use crate::gossamer::InMemoryGossamer;
use brongnal::BrongnalController;
use proto::gossamer::gossamer_server::GossamerServer;
use proto::service::brongnal_server::BrongnalServer;
use proto::FILE_DESCRIPTOR_SET;
use rusqlite::Connection;
use sqlite_brongnal::SqliteStorage;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use tonic::transport::Server;
use tonic_reflection::server::Builder;

mod brongnal;
mod gossamer;
mod memory_brongnal;
mod sqlite_brongnal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let server_addr = (IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080).into();

    println!("Brongnal Server listening at: {server_addr}");

    let mut db_path = PathBuf::from(std::env::var("DB").unwrap_or(String::from("db")));
    db_path.push("brongnal.db3");
    println!("Database Path: {}", db_path.display());
    let connection = Connection::open(db_path)?;
    let controller = BrongnalController::new(Box::new(SqliteStorage::new(connection)?));

    Server::builder()
        .add_service(BrongnalServer::new(controller))
        .add_service(GossamerServer::new(InMemoryGossamer::default()))
        .add_service(reflection_service)
        .serve(server_addr)
        .await?;

    Ok(())
}
