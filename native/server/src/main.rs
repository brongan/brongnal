#![feature(duration_constructors)]
use proto::service::brongnal_service_server::BrongnalServiceServer as BrongnalServer;
use proto::FILE_DESCRIPTOR_SET;
use sentry::ClientInitGuard;
use server::brongnal::BrongnalController;
use server::persistence::{clean_mailboxes, SqliteStorage};
use server::push_notifications::FirebaseCloudMessagingClient;
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
        .add_service(reflection_service)
        .serve(server_addr)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use client::{User, X3DHClient};
    use gossamer::persistence::GossamerStorage;
    use gossamer::service::Service;
    use proto::gossamer::gossamer_service_server::GossamerServiceServer as GossamerServer;
    use proto::service::brongnal_service_server::BrongnalServiceServer as BrongnalServer;
    use std::sync::Arc;
    use tokio_rusqlite::Connection;
    use tokio_stream::StreamExt;
    use tonic::transport::Server;

    #[tokio::test]
    async fn test_split_services_flow() {
        // 1. Spawn Identity Service (OS-assigned port)
        let identity_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let identity_port = identity_listener.local_addr().unwrap().port();
        let identity_addr = format!("http://127.0.0.1:{identity_port}");

        let identity_conn = Connection::open_in_memory().await.unwrap();
        let identity_storage = GossamerStorage::new(identity_conn).await.unwrap();
        let identity_handler = Service::new(identity_storage);

        tokio::spawn(async move {
            Server::builder()
                .add_service(GossamerServer::new(identity_handler))
                .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(
                    identity_listener,
                ))
                .await
                .unwrap();
        });

        // 2. Spawn Mailbox Service (OS-assigned port)
        let mailbox_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mailbox_port = mailbox_listener.local_addr().unwrap().port();
        let mailbox_addr = format!("http://127.0.0.1:{mailbox_port}");

        let mailbox_conn = Connection::open_in_memory().await.unwrap();
        let mailbox_storage = SqliteStorage::new(mailbox_conn).await.unwrap();
        let mailbox_controller = BrongnalController::new(mailbox_storage, None);

        tokio::spawn(async move {
            Server::builder()
                .add_service(BrongnalServer::new(mailbox_controller))
                .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(
                    mailbox_listener,
                ))
                .await
                .unwrap();
        });

        // Give servers a moment to start
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // 3. Client 1: Alice registers
        let alice_db = Connection::open_in_memory().await.unwrap();
        let alice_x3dh = Arc::new(X3DHClient::new(alice_db).await.unwrap());
        let mut alice = User::new(
            mailbox_addr.clone(),
            identity_addr.clone(),
            alice_x3dh,
            "alice".to_string(),
        )
        .expect("Failed to create Alice");

        alice
            .register(None)
            .await
            .expect("Alice registration failed");

        // 4. Client 2: Bob registers
        let bob_db = Connection::open_in_memory().await.unwrap();
        let bob_x3dh = Arc::new(X3DHClient::new(bob_db).await.unwrap());
        let mut bob = User::new(
            mailbox_addr.clone(),
            identity_addr.clone(),
            bob_x3dh,
            "bob".to_string(),
        )
        .expect("Failed to create Bob");

        bob.register(None).await.expect("Bob registration failed");

        // 5. Alice sends message to Bob
        alice
            .send_message("bob".to_string(), "Hello Bob!".to_string())
            .await
            .expect("Alice failed to send message");

        // 6. Bob receives and decrypts message
        let subscriber = bob.get_messages().await.expect("Bob failed to subscribe");
        let stream = subscriber.into_stream();
        tokio::pin!(stream);
        let msg = stream
            .next()
            .await
            .expect("Stream ended early")
            .expect("Failed to receive message");

        assert_eq!(msg.sender, "alice");
        assert_eq!(msg.text, "Hello Bob!");
    }
}
