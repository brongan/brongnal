use server::proto::gossamer_server::GossamerServer;
use server::proto::{brongnal_server::BrongnalServer, FILE_DESCRIPTOR_SET};
use server::MemoryServer;
use std::net::{IpAddr, Ipv4Addr};
use tonic::transport::Server;
use tonic_reflection::server::Builder;

use crate::gossamer::InMemoryGossamer;

mod gossamer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let server_addr = (IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080).into();

    println!("Brongnal Server listening at: {server_addr}");

    Server::builder()
        .add_service(BrongnalServer::new(MemoryServer::default()))
        .add_service(GossamerServer::new(InMemoryGossamer {}))
        .add_service(reflection_service)
        .serve(server_addr)
        .await?;

    Ok(())
}
