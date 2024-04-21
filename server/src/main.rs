use server::service::brongnal_server::BrongnalServer;
use server::MemoryServer;
use std::net::{IpAddr, Ipv6Addr};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr = (IpAddr::V6(Ipv6Addr::UNSPECIFIED), 8080);
    let service = MemoryServer::default();

    Server::builder()
        .add_service(BrongnalServer::new(service))
        .serve(server_addr.into())
        .await?;

    Ok(())
}
