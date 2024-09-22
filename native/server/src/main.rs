use crate::gossamer::InMemoryGossamer;
use brongnal::BrongnalController;
use memory_brongnal::MemoryStorage;
use server::proto::gossamer::gossamer_server::GossamerServer;
use server::proto::service::brongnal_server::BrongnalServer;
use server::proto::FILE_DESCRIPTOR_SET;
use std::net::{IpAddr, Ipv4Addr};
use tonic::transport::Server;
use tonic_reflection::server::Builder;

mod brongnal;
mod gossamer;
mod memory_brongnal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let server_addr = (IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080).into();

    println!("Brongnal Server listening at: {server_addr}");

    let controller = BrongnalController::new(Box::new(MemoryStorage::default()));

    Server::builder()
        .add_service(BrongnalServer::new(controller))
        .add_service(GossamerServer::new(InMemoryGossamer::default()))
        .add_service(reflection_service)
        .serve(server_addr)
        .await?;

    Ok(())
}
