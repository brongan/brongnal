use crate::{brongnal::BrongnalController, persistence::SqliteStorage};
use client::{client::MessageState, User, X3DHClient};
use prost::Message as _;
use proto::service::{
    brongnal_service_client::BrongnalServiceClient,
    brongnal_service_server::{BrongnalService, BrongnalServiceServer},
    IdentityKeyRequest, PreKeyBundleRequest, RegisterPreKeyBundleRequest, SendMessageRequest,
};
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpListener, task::JoinHandle, time::timeout};
use tokio_rusqlite::Connection;
use tokio_stream::{wrappers::TcpListenerStream, StreamExt};
use tonic::{transport::Server, Code, Request};

struct TestServer {
    address: String,
    task: JoinHandle<Result<(), tonic::transport::Error>>,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl TestServer {
    async fn start() -> anyhow::Result<Self> {
        let storage = SqliteStorage::new(Connection::open_in_memory().await?).await?;
        let controller = BrongnalController::new(storage, None);
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = format!("http://{}", listener.local_addr()?);
        let task = tokio::spawn(
            Server::builder()
                .add_service(BrongnalServiceServer::new(controller))
                .serve_with_incoming(TcpListenerStream::new(listener)),
        );
        Ok(Self { address, task })
    }

    async fn user(&self, name: &str) -> anyhow::Result<(User, Arc<X3DHClient>)> {
        let x3dh = Arc::new(X3DHClient::new(Connection::open_in_memory().await?).await?);
        let user = User::new(self.address.clone(), x3dh.clone(), name.to_owned())?;
        Ok((user, x3dh))
    }
}

#[tokio::test]
async fn registration_is_idempotent_and_rejects_another_key() -> anyhow::Result<()> {
    let server = TestServer::start().await?;
    let (mut alice, x3dh) = server.user("alice").await?;
    alice.register(None).await?;
    alice.register(None).await?;
    let (mut other, _) = server.user("alice").await?;
    assert!(
        matches!(other.register(None).await, Err(client::ClientError::Grpc(e)) if e.code() == Code::AlreadyExists)
    );
    let mut stub = BrongnalServiceClient::connect(server.address.clone()).await?;
    let response = stub
        .get_identity_key(IdentityKeyRequest {
            username: Some("alice".into()),
        })
        .await?
        .into_inner();
    assert_eq!(
        response.identity_key(),
        x3dh.get_ik().verifying_key().as_bytes()
    );
    assert_eq!(
        stub.get_identity_key(IdentityKeyRequest {
            username: Some("missing".into())
        })
        .await
        .unwrap_err()
        .code(),
        Code::NotFound
    );
    assert!(alice
        .send_message("missing".into(), "not sent".into())
        .await
        .is_err());
    assert!(alice.get_message_history().await?.is_empty());
    Ok(())
}

#[tokio::test]
async fn registration_validates_username_and_signatures() -> anyhow::Result<()> {
    let storage = SqliteStorage::new(Connection::open_in_memory().await?).await?;
    let controller = BrongnalController::new(storage, None);
    let x3dh = X3DHClient::new(Connection::open_in_memory().await?).await?;
    let valid = RegisterPreKeyBundleRequest {
        username: Some("alice".into()),
        identity_key: Some(x3dh.get_ik().verifying_key().to_bytes().to_vec()),
        signed_pre_key: Some(x3dh.get_spk().await?.into()),
        one_time_key_bundle: Some(x3dh.create_opks(0).await?.into()),
        fcm_token: None,
    };
    for username in [None, Some(String::new()), Some("a".repeat(49))] {
        let request = RegisterPreKeyBundleRequest {
            username,
            ..valid.clone()
        };
        assert_eq!(
            controller
                .register_pre_key_bundle(Request::new(request))
                .await
                .unwrap_err()
                .code(),
            Code::InvalidArgument
        );
    }
    let mut invalid = valid.clone();
    invalid.signed_pre_key.as_mut().unwrap().signature = Some(vec![0; 64]);
    assert_eq!(
        controller
            .register_pre_key_bundle(Request::new(invalid))
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    assert_eq!(
        controller
            .get_identity_key(Request::new(IdentityKeyRequest {
                username: Some("alice".into())
            }))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    controller
        .register_pre_key_bundle(Request::new(valid))
        .await?;
    Ok(())
}

#[tokio::test]
async fn registered_users_exchange_live_messages_and_reject_false_sender() -> anyhow::Result<()> {
    let server = TestServer::start().await?;
    let (mut bob, bob_keys) = server.user("bob").await?;
    bob.register(None).await?;
    // A sender registering after subscription must also be recognized.
    let messages = bob.get_messages().await?.into_stream();
    tokio::pin!(messages);
    let (mut alice, _) = server.user("alice").await?;
    alice.register(None).await?;

    let (_, attacker_keys) = server.user("attacker").await?;
    let mut stub = BrongnalServiceClient::connect(server.address.clone()).await?;
    let recipient = bob_keys.get_ik().verifying_key().to_bytes().to_vec();
    let bundle = stub
        .request_pre_keys(PreKeyBundleRequest {
            identity_key: Some(recipient.clone()),
        })
        .await?
        .into_inner()
        .try_into()?;
    let forged: proto::application::RatchetMessage = proto::RatchetMessage {
        header: None,
        message: proto::ApplicationMessage {
            sender: "alice".into(),
            text: "forged".into(),
        },
    }
    .into();
    let (_, encrypted) =
        protocol::x3dh::initiate_send(bundle, &attacker_keys.get_ik(), &forged.encode_to_vec())?;
    stub.send_message(tokio_stream::once(SendMessageRequest {
        recipient_identity_key: Some(recipient),
        message: Some(encrypted.into()),
    }))
    .await?;

    let id = alice.send_message("bob".into(), "hello".into()).await?;
    let received = timeout(Duration::from_secs(5), messages.next())
        .await?
        .unwrap()?;
    assert_eq!(
        (&*received.sender, &*received.receiver, &*received.text),
        ("alice", "bob", "hello")
    );
    assert!(matches!(received.state, MessageState::Delivered));
    assert!(matches!(
        alice.get_message(id).await?.state,
        MessageState::Sent
    ));
    let history = bob.get_message_history().await?;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].text, "hello");
    Ok(())
}

#[tokio::test]
async fn registered_users_receive_mailbox_messages() -> anyhow::Result<()> {
    let server = TestServer::start().await?;
    let (mut alice, _) = server.user("alice").await?;
    let (mut bob, _) = server.user("bob").await?;
    alice.register(None).await?;
    bob.register(None).await?;
    alice.send_message("bob".into(), "offline".into()).await?;
    let messages = bob.get_messages().await?.into_stream();
    tokio::pin!(messages);
    let received = timeout(Duration::from_secs(5), messages.next())
        .await?
        .unwrap()?;
    assert_eq!(received.text, "offline");
    assert_eq!(received.sender, "alice");
    Ok(())
}
