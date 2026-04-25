use super::*;
use ed25519_dalek::SigningKey;
use proto::gossamer::SignedMessage;
use rand_core::OsRng;

async fn setup_db() -> GossamerStorage {
    let conn = Connection::open_in_memory().await.unwrap();
    GossamerStorage::new(conn).await.unwrap()
}

#[tokio::test]
async fn test_schema_initialization() {
    let db = setup_db().await;
    // If we can call methods, schema is initialized
    assert!(db.get_ledger().await.is_ok());
}

#[tokio::test]
async fn test_append_key_success() {
    let db = setup_db().await;
    let provider = b"alice".to_vec();
    let key = SigningKey::generate(&mut OsRng).verifying_key();

    let appended = db.append_key(provider.clone(), key).await.unwrap();
    assert!(appended);

    let ledger = db.get_ledger().await.unwrap();
    assert_eq!(ledger.len(), 1);
    assert!(ledger.contains_key(&provider));
}

#[tokio::test]
async fn test_append_key_idempotency() {
    let db = setup_db().await;
    let provider = b"alice".to_vec();
    let key = SigningKey::generate(&mut OsRng).verifying_key();

    db.append_key(provider.clone(), key).await.unwrap();
    let appended_second_time = db.append_key(provider.clone(), key).await.unwrap();

    // Should return false because it already exists (idempotent ignore)
    assert!(!appended_second_time);
}

#[tokio::test]
async fn test_append_multiple_keys_for_same_provider() {
    let db = setup_db().await;
    let provider = b"alice".to_vec();
    let key1 = SigningKey::generate(&mut OsRng).verifying_key();
    let key2 = SigningKey::generate(&mut OsRng).verifying_key();

    db.append_key(provider.clone(), key1).await.unwrap();
    db.append_key(provider.clone(), key2).await.unwrap();

    let ledger = db.get_ledger().await.unwrap();
    assert_eq!(ledger.get(&provider).unwrap().len(), 2);
}

#[tokio::test]
async fn test_has_key_authorized() {
    let db = setup_db().await;
    let provider = b"alice".to_vec();
    let key = SigningKey::generate(&mut OsRng).verifying_key();
    db.append_key(provider.clone(), key).await.unwrap();

    assert!(db.has_key(provider, key).await.unwrap());
}

#[tokio::test]
async fn test_has_key_unauthorized() {
    let db = setup_db().await;
    let provider = b"alice".to_vec();
    let key1 = SigningKey::generate(&mut OsRng).verifying_key();
    let key2 = SigningKey::generate(&mut OsRng).verifying_key();
    db.append_key(provider, key1).await.unwrap();

    let other_provider = b"bob".to_vec();
    assert!(!db.has_key(other_provider, key2).await.unwrap());
}

#[tokio::test]
async fn test_get_key_provider_mapping() {
    let db = setup_db().await;
    let provider = b"alice".to_vec();
    let key = SigningKey::generate(&mut OsRng).verifying_key();
    db.append_key(provider.clone(), key).await.unwrap();

    let found_provider = db.get_key_provider(key).await.unwrap();
    assert_eq!(found_provider, Some(provider));
}

#[tokio::test]
async fn test_revoke_key_success() {
    let db = setup_db().await;
    let provider = b"alice".to_vec();
    let key = SigningKey::generate(&mut OsRng).verifying_key();
    db.append_key(provider.clone(), key).await.unwrap();

    let revoked = db.revoke_key(provider.clone(), key).await.unwrap();
    assert!(revoked);

    assert!(!db.has_key(provider, key).await.unwrap());
}

#[tokio::test]
async fn test_revoke_key_not_found() {
    let db = setup_db().await;
    let provider = b"alice".to_vec();
    let key = SigningKey::generate(&mut OsRng).verifying_key();

    let revoked = db.revoke_key(provider, key).await.unwrap();
    assert!(!revoked);
}

#[tokio::test]
async fn test_append_message_fk_constraint_success() {
    let db = setup_db().await;
    let provider = b"alice".to_vec();
    let key = SigningKey::generate(&mut OsRng).verifying_key();
    db.append_key(provider.clone(), key).await.unwrap();

    let msg = SignedMessage {
        contents: Some(vec![1, 2, 3]),
        identity_key: Some(vec![4, 5, 6]),
        signature: Some(vec![7, 8, 9]),
    };

    assert!(db.append_message(provider, msg).await.is_ok());
}

#[tokio::test]
async fn test_append_message_fk_constraint_failure() {
    let db = setup_db().await;
    let provider = b"nonexistent".to_vec();

    let msg = SignedMessage {
        contents: Some(vec![1, 2, 3]),
        identity_key: Some(vec![4, 5, 6]),
        signature: Some(vec![7, 8, 9]),
    };

    // Fails because 'nonexistent' is not in gossamer_providers
    assert!(db.append_message(provider, msg).await.is_err());
}

#[tokio::test]
async fn test_get_ledger_grouping() {
    let db = setup_db().await;
    let alice = b"alice".to_vec();
    let bob = b"bob".to_vec();
    let key_a1 = SigningKey::generate(&mut OsRng).verifying_key();
    let key_a2 = SigningKey::generate(&mut OsRng).verifying_key();
    let key_b1 = SigningKey::generate(&mut OsRng).verifying_key();

    db.append_key(alice.clone(), key_a1).await.unwrap();
    db.append_key(alice.clone(), key_a2).await.unwrap();
    db.append_key(bob.clone(), key_b1).await.unwrap();

    let ledger = db.get_ledger().await.unwrap();
    assert_eq!(ledger.len(), 2);
    assert_eq!(ledger.get(&alice).unwrap().len(), 2);
    assert_eq!(ledger.get(&bob).unwrap().len(), 1);
}
