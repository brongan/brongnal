use client::{User, X3DHClient};
use identity::gossamer::GossamerServiceHandler;
use identity::persistence::GossamerStorage;
use server::brongnal::BrongnalController;
use server::persistence::SqliteStorage;
use std::sync::Arc;
use tokio_rusqlite::Connection;
use tonic::transport::Server;
use proto::gossamer::gossamer_service_server::GossamerServiceServer as GossamerServer;
use proto::service::brongnal_service_server::BrongnalServiceServer as BrongnalServer;
use tokio_stream::StreamExt;

#[tokio::test]
async fn test_split_services_flow() {
    // Bypass attestation and TLS for local testing
    std::env::set_var("BYPASS_ATTESTATION", "1");

    let identity_addr = "http://127.0.0.1:50052";
    let mailbox_addr = "http://127.0.0.1:50051";

    // 1. Spawn Identity Service
    let identity_conn = Connection::open_in_memory().await.unwrap();
    let identity_storage = GossamerStorage::new(identity_conn).await.unwrap();
    let identity_handler = GossamerServiceHandler::new(identity_storage);
    
    tokio::spawn(async move {
        Server::builder()
            .add_service(GossamerServer::new(identity_handler))
            .serve("127.0.0.1:50052".parse().unwrap())
            .await
            .unwrap();
    });

    // 2. Spawn Mailbox Service
    let mailbox_conn = Connection::open_in_memory().await.unwrap();
    let mailbox_storage = SqliteStorage::new(mailbox_conn).await.unwrap();
    let mailbox_controller = BrongnalController::new(mailbox_storage, None);

    tokio::spawn(async move {
        Server::builder()
            .add_service(BrongnalServer::new(mailbox_controller))
            .serve("127.0.0.1:50051".parse().unwrap())
            .await
            .unwrap();
    });

    // Give servers a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 3. Client 1: Alice registers
    let alice_db = Connection::open_in_memory().await.unwrap();
    let alice_x3dh = Arc::new(X3DHClient::new(alice_db).await.unwrap());
    let mut alice = User::new(
        mailbox_addr.to_string(),
        identity_addr.to_string(),
        alice_x3dh,
        "alice".to_string()
    ).await.expect("Failed to create Alice");
    
    alice.register(None).await.expect("Alice registration failed");

    // 4. Client 2: Bob registers
    let bob_db = Connection::open_in_memory().await.unwrap();
    let bob_x3dh = Arc::new(X3DHClient::new(bob_db).await.unwrap());
    let mut bob = User::new(
        mailbox_addr.to_string(),
        identity_addr.to_string(),
        bob_x3dh,
        "bob".to_string()
    ).await.expect("Failed to create Bob");

    bob.register(None).await.expect("Bob registration failed");

    // 5. Alice sends message to Bob
    // This involves Alice looking up Bob's IK from Identity Service
    // and requesting Pre-Keys from Mailbox Service.
    alice.send_message("bob".to_string(), "Hello Bob!".to_string())
        .await
        .expect("Alice failed to send message");

    // 6. Bob receives and decrypts message
    let subscriber = bob.get_messages().await.expect("Bob failed to subscribe");
    let stream = subscriber.into_stream();
    tokio::pin!(stream);
    let msg = stream.next().await
        .expect("Stream ended early")
        .expect("Failed to receive message");
    
    assert_eq!(msg.sender, "alice");
    assert_eq!(msg.text, "Hello Bob!");
    
    println!("Integration test passed!");
}
