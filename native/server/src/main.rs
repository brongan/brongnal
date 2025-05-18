#![feature(duration_constructors)]
#![feature(duration_constructors_lite)]
use crate::gossamer::InMemoryGossamer;
use crate::push_notifications::FirebaseCloudMessagingClient;
use brongnal::BrongnalController;
use persistence::{clean_mailboxes, SqliteStorage};
use proto::gossamer::gossamer_service_server::GossamerServiceServer as GossamerServer;
use proto::service::brongnal_service_server::BrongnalServiceServer as BrongnalServer;
use proto::FILE_DESCRIPTOR_SET;
use sentry::ClientInitGuard;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::time::Duration;
use tokio_rusqlite::Connection;
use tonic::transport::Server;
use tonic_reflection::server::Builder;
use tracing::{info, warn, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

mod brongnal;
mod gossamer;
mod persistence;
mod push_notifications;

pub async fn db_cleanup(connection: tokio_rusqlite::Connection) {
    let mut interval = tokio::time::interval(Duration::from_hours(1));
    loop {
        interval.tick().await;
        match clean_mailboxes(&connection, Duration::from_days(30)).await {
            Ok(num) => info!("Cleaned up {num} items from mailboxes."),
            Err(e) => warn!("Failed to clean mailboxes: {e}"),
        }
    }
}

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

    let _guard: Option<ClientInitGuard> = if let Ok(dsn) = std::env::var("SENTRY_DSN") {
        info!("Creating Sentry guard.");
        Some(sentry::init((
            dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        )))
    } else {
        warn!("Not creating Sentry guard.");
        None
    };

    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let server_addr = (IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080).into();

    let fcm_client: Option<FirebaseCloudMessagingClient> =
        if let Ok(service_account_key) = std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
            info!("Creating Firebase Cloud Messaging Client");
            Some(FirebaseCloudMessagingClient::new(&service_account_key).await?)
        } else {
            warn!("GOOGLE_APPLICATION_CREDENTIALS is unset. Push notifications are unsupported.");
            None
        };

    let xdg_dirs = xdg::BaseDirectories::with_prefix("brongnal")?;
    let db_path: PathBuf = if let Ok(db_dir) = std::env::var("DB") {
        [&db_dir, "brongnal.db3"].iter().collect()
    } else {
        xdg_dirs.place_data_file("brongnal_server.db3").unwrap()
    };
    info!("Database Path: {}", db_path.display());
    let connection = Connection::open(db_path).await?;
    tokio::spawn(db_cleanup(connection.clone()));

    let controller = BrongnalController::new(SqliteStorage::new(connection).await?, fcm_client);

    info!("Brongnal Server listening at: {server_addr}");

    Server::builder()
        .add_service(BrongnalServer::new(controller))
        .add_service(GossamerServer::new(InMemoryGossamer::default()))
        .add_service(reflection_service)
        .serve(server_addr)
        .await?;

    Ok(())
}
