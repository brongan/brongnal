use ::server::{MemoryServer, X3DHServer};
use futures::prelude::*;
use std::net::{IpAddr, Ipv6Addr};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_addr = (IpAddr::V6(Ipv6Addr::UNSPECIFIED), 8080);

    Ok(())
}
