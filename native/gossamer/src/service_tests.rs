use super::*;
use crate::persistence::GossamerStorage;
use crate::service::Service;
use ed25519_dalek::{Signer, SigningKey};
use prost::Message;
use proto::gossamer::{ActionRequest, GetLedgerRequest, SignedMessage};
use rand_core::OsRng;
use tokio_rusqlite::Connection;
use tonic::Request;

async fn setup_service() -> Service {
    let conn = Connection::open_in_memory().await.unwrap();
    let storage = GossamerStorage::new(conn).await.unwrap();
    Service::new(storage)
}

fn create_signed_action(
    signer: &SigningKey,
    provider: Vec<u8>,
    new_key: Vec<u8>,
    action: protocol::gossamer::Action,
) -> SignedMessage {
    let message = protocol::gossamer::Message {
        provider,
        public_key: ed25519_dalek::VerifyingKey::try_from(new_key.as_slice()).unwrap(),
        action,
    };
    
    let contents: proto::gossamer::Message = message.into();
    let encoded = contents.encode_to_vec();
    let signature = signer.sign(&encoded);
    
    SignedMessage {
        contents: Some(encoded),
        signature: Some(signature.to_vec()),
        identity_key: Some(signer.verifying_key().to_bytes().to_vec()),
    }
}

#[tokio::test]
async fn test_get_attestation_mock() {
    let service = setup_service().await;
    let response = service
        .get_attestation(Request::new(proto::gossamer::AttestationRequest {}))
        .await
        .unwrap()
        .into_inner();
    
    assert!(response.gca_token.is_some());
}

#[tokio::test]
async fn test_action_claim_new_provider_success() {
    let service = setup_service().await;
    let alice_key = SigningKey::generate(&mut OsRng);
    
    // New user claims 'alice' using their own key
    let action = create_signed_action(
        &alice_key,
        b"alice".to_vec(),
        alice_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    
    let res = service.action(Request::new(ActionRequest { message: Some(action) })).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_action_claim_denied_if_not_self_signed() {
    let service = setup_service().await;
    let attacker_key = SigningKey::generate(&mut OsRng);
    let victim_key = SigningKey::generate(&mut OsRng);
    
    // Attacker tries to claim 'victim' identity for the victims key, but signs with attackers key
    let action = create_signed_action(
        &attacker_key,
        b"victim".to_vec(),
        victim_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    
    let res = service.action(Request::new(ActionRequest { message: Some(action) })).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_action_append_key_success_with_existing_auth() {
    let service = setup_service().await;
    let alice_key1 = SigningKey::generate(&mut OsRng);
    let alice_key2 = SigningKey::generate(&mut OsRng);
    
    // 1. Initial claim
    let claim = create_signed_action(
        &alice_key1,
        b"alice".to_vec(),
        alice_key1.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    service.action(Request::new(ActionRequest { message: Some(claim) })).await.unwrap();
    
    // 2. Alice adds a second key, authorized by the first
    let add_key = create_signed_action(
        &alice_key1,
        b"alice".to_vec(),
        alice_key2.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    
    let res = service.action(Request::new(ActionRequest { message: Some(add_key) })).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_action_key_theft_denied() {
    let service = setup_service().await;
    let alice_key = SigningKey::generate(&mut OsRng);
    let bob_key = SigningKey::generate(&mut OsRng);
    
    // 1. Alice claims 'alice'
    let alice_claim = create_signed_action(
        &alice_key,
        b"alice".to_vec(),
        alice_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    service.action(Request::new(ActionRequest { message: Some(alice_claim) })).await.unwrap();
    
    // 2. Bob tries to claim 'bob' but uses Alice's key (Key Theft)
    let bob_theft_claim = create_signed_action(
        &alice_key,
        b"bob".to_vec(),
        alice_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    
    let res = service.action(Request::new(ActionRequest { message: Some(bob_theft_claim) })).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().message().contains("already associated with another provider"));
}

#[tokio::test]
async fn test_action_append_key_unauthorized_signer_denied() {
    let service = setup_service().await;
    let alice_key = SigningKey::generate(&mut OsRng);
    let attacker_key = SigningKey::generate(&mut OsRng);
    let new_key = SigningKey::generate(&mut OsRng);
    
    // 1. Alice claims 'alice'
    let alice_claim = create_signed_action(
        &alice_key,
        b"alice".to_vec(),
        alice_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    service.action(Request::new(ActionRequest { message: Some(alice_claim) })).await.unwrap();
    
    // 2. Attacker tries to add a key to Alice's account
    let attack_add = create_signed_action(
        &attacker_key,
        b"alice".to_vec(),
        new_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    
    let res = service.action(Request::new(ActionRequest { message: Some(attack_add) })).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_action_existing_user_theft_denied() {
    let service = setup_service().await;
    let alice_key = SigningKey::generate(&mut OsRng);
    let bob_key = SigningKey::generate(&mut OsRng);
    
    // 1. Alice claims 'alice'
    let alice_claim = create_signed_action(
        &alice_key,
        b"alice".to_vec(),
        alice_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    service.action(Request::new(ActionRequest { message: Some(alice_claim) })).await.unwrap();

    // 2. Bob claims 'bob'
    let bob_claim = create_signed_action(
        &bob_key,
        b"bob".to_vec(),
        bob_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    service.action(Request::new(ActionRequest { message: Some(bob_claim) })).await.unwrap();

    // 3. Bob tries to add Alice's key to his own account (Key Theft in AppendKey)
    let theft_append = create_signed_action(
        &bob_key,
        b"bob".to_vec(),
        alice_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    
    let res = service.action(Request::new(ActionRequest { message: Some(theft_append) })).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().message().contains("already associated with another provider"));
}

#[tokio::test]
async fn test_action_revoke_key_success() {
    let service = setup_service().await;
    let alice_key = SigningKey::generate(&mut OsRng);
    
    // 1. Claim
    let claim = create_signed_action(
        &alice_key,
        b"alice".to_vec(),
        alice_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    service.action(Request::new(ActionRequest { message: Some(claim) })).await.unwrap();
    
    // 2. Revoke
    let revoke = create_signed_action(
        &alice_key,
        b"alice".to_vec(),
        alice_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::RevokeKey,
    );
    
    let res = service.action(Request::new(ActionRequest { message: Some(revoke) })).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_action_revoke_key_unauthorized_signer_denied() {
    let service = setup_service().await;
    let alice_key = SigningKey::generate(&mut OsRng);
    let attacker_key = SigningKey::generate(&mut OsRng);
    
    // 1. Alice claims 'alice'
    let alice_claim = create_signed_action(
        &alice_key,
        b"alice".to_vec(),
        alice_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    service.action(Request::new(ActionRequest { message: Some(alice_claim) })).await.unwrap();
    
    // 2. Attacker tries to revoke Alice's key
    let attack_revoke = create_signed_action(
        &attacker_key,
        b"alice".to_vec(),
        alice_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::RevokeKey,
    );
    
    let res = service.action(Request::new(ActionRequest { message: Some(attack_revoke) })).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_action_revoke_nonexistent_key_denied() {
    let service = setup_service().await;
    let alice_key = SigningKey::generate(&mut OsRng);
    let fake_key = SigningKey::generate(&mut OsRng);
    
    // 1. Alice claims 'alice'
    let alice_claim = create_signed_action(
        &alice_key,
        b"alice".to_vec(),
        alice_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    service.action(Request::new(ActionRequest { message: Some(alice_claim) })).await.unwrap();
    
    // 2. Alice tries to revoke a key she doesn't actually have registered
    let bad_revoke = create_signed_action(
        &alice_key,
        b"alice".to_vec(),
        fake_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::RevokeKey,
    );
    
    let res = service.action(Request::new(ActionRequest { message: Some(bad_revoke) })).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_action_invalid_signature_denied() {
    let service = setup_service().await;
    let alice_key = SigningKey::generate(&mut OsRng);
    
    let mut action = create_signed_action(
        &alice_key,
        b"alice".to_vec(),
        alice_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    
    // Corrupt the signature
    if let Some(ref mut sig) = action.signature {
        if !sig.is_empty() {
            sig[0] ^= 0xFF;
        }
    }
    
    let res = service.action(Request::new(ActionRequest { message: Some(action) })).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().code(), tonic::Code::Unauthenticated);
}

#[tokio::test]
async fn test_get_ledger_mapping() {
    let service = setup_service().await;
    let alice_key = SigningKey::generate(&mut OsRng);
    
    let claim = create_signed_action(
        &alice_key,
        b"alice".to_vec(),
        alice_key.verifying_key().to_bytes().to_vec(),
        protocol::gossamer::Action::AppendKey,
    );
    service.action(Request::new(ActionRequest { message: Some(claim) })).await.unwrap();
    
    let ledger = service.get_ledger(Request::new(GetLedgerRequest {})).await.unwrap().into_inner();
    assert_eq!(ledger.users.len(), 1);
    assert_eq!(ledger.users[0].provider, Some(b"alice".to_vec()));
}
