use base64::{engine::general_purpose::STANDARD as base64, Engine as _};
use ed25519_dalek::VerifyingKey;
use prost::Message;
use proto::service::Message as MessageProto;
use proto::service::SignedPreKey as SignedPreKeyProto;
use rusqlite::params;
use rusqlite::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::Status;
use tracing::info;
use x25519_dalek::PublicKey as X25519PublicKey;

pub struct SqliteStorage(tokio_rusqlite::Connection);

fn time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl SqliteStorage {
    pub async fn new(connection: tokio_rusqlite::Connection) -> tokio_rusqlite::Result<Self> {
        connection
            .call(|connection| {
                connection.pragma_update(None, "journal_mode", "WAL")?;
                connection.pragma_update(None, "synchronous", "normal")?;
                connection.pragma_update(None, "foreign_keys", "on")?;
                connection.execute(
                    "CREATE TABLE IF NOT EXISTS device (
                        ik BLOB PRIMARY KEY,
                        spk BLOB NOT NULL,
                        time INTEGER NOT NULL
                    )",
                    (),
                )?;
                connection.execute(
                    "CREATE TABLE IF NOT EXISTS opk_queue (
                        opk BLOB PRIMARY KEY,
                        ik BLOB NOT NULL,
                        time INTEGER NOT NULL,
                        FOREIGN KEY(ik) REFERENCES device(ik)
                    )",
                    (),
                )?;
                connection.execute(
                    "CREATE TABLE IF NOT EXISTS mailbox (
                        message BLOB PRIMARY KEY,
                        ik BLOB NOT NULL,
                        time integer NOT NULL,
                        FOREIGN KEY(ik) REFERENCES device(ik)
                    )",
                    (),
                )?;
                Ok(())
            })
            .await?;

        Ok(SqliteStorage(connection))
    }
}

impl SqliteStorage {
    /// Add a new identity to the storage.
    /// Attempts to overwrite an identity key returns an error.
    /// Updates to the user's signed pre key are allowed.
    pub async fn add_user(&self, ik: VerifyingKey, spk: SignedPreKeyProto) -> tonic::Result<()> {
        info!("Adding user \"{}\" to the database.", base64.encode(ik));
        let spk = spk.encode_to_vec();
        let ik = ik.to_bytes();

        let persisted_spk = {
            let serialized_spk = spk.clone();
            self.0
                .call(move |connection| {
                    connection.execute(
                        "INSERT OR IGNORE INTO device (ik, spk, time) VALUES ($1, $2, ?3)",
                        params![ik, serialized_spk, time_now()],
                    )?;
                    let persisted_spk: Vec<u8> = connection.query_row(
                        "SELECT spk FROM device where ik = ?1",
                        params![ik],
                        |row| row.get(0),
                    )?;
                    Ok(persisted_spk)
                })
                .await
                .map_err(|e| Status::internal(format!("Failed to register user: {e}")))?
        };
        if spk != persisted_spk {
            return Err(Status::already_exists(
                "A user already exists with this key.",
            ));
        }
        Ok(())
    }

    /// Replaces the signed pre key for a given identity.
    // TODO(https://github.com/brongan/brongnal/issues/27) -  Implement signed pre key rotation.
    #[allow(dead_code)]
    pub async fn update_spk(&self, ik: &VerifyingKey, spk: SignedPreKeyProto) -> tonic::Result<()> {
        info!("Updating spk for {} in the database.", base64.encode(ik));
        let ik_bytes = ik.to_bytes();

        self.0
            .call(move |connection| {
                let spk = spk;
                // Returns the first row updated so that a missing key results in an error.
                let _: Vec<u8> = connection.query_row(
                    "UPDATE device SET spk = ?2 WHERE ik = ?1 RETURNING ik",
                    params![&ik_bytes, spk.encode_to_vec()],
                    |row| row.get(0),
                )?;
                Ok(())
            })
            .await
            .map_err(|e| {
                Status::not_found(format!(
                    "pre key for identity_key={} not found: {e}",
                    base64.encode(ik)
                ))
            })
    }

    /// Appends new unburnt one time pre keys for others to message a given identity.
    pub async fn add_opks(
        &self,
        ik: &VerifyingKey,
        opks: Vec<X25519PublicKey>,
    ) -> tonic::Result<()> {
        info!(
            "Adding {} one time key(s) for ik=\"{}\" to the database.",
            opks.len(),
            base64.encode(ik)
        );

        let ik = ik.to_bytes();

        self.0
            .call(move |connection| {
                let mut stmt = connection
                    .prepare("INSERT INTO opk_queue (ik, opk, time) VALUES (?1, ?2, ?3)")
                    .unwrap();
                for opk in opks {
                    stmt.execute((ik, opk.to_bytes(), time_now()))?;
                }
                Ok(())
            })
            .await
            .map_err(|_| Status::internal("failed to insert one time key"))
    }

    /// Retrieves the identity key and signed pre key for a given identity.
    /// A client must first invoke this before messaging a peer.
    pub async fn get_current_spk(&self, ik: &VerifyingKey) -> tonic::Result<SignedPreKeyProto> {
        info!(
            "Retrieving pre keys for user \"{}\" from the database.",
            base64.encode(ik)
        );
        let ik = ik.to_bytes();

        self.0
            .call(move |connection| {
                let spk: Vec<u8> = connection.query_row(
                    "SELECT spk FROM device WHERE ik = ?1",
                    params![ik],
                    |row| Ok(row.get(0).unwrap()),
                )?;
                let spk = SignedPreKeyProto::decode(&*spk).unwrap();
                Ok(spk)
            })
            .await
            .map_err(|_| Status::not_found("user not found"))
    }

    /// Retrieve a one time pre key for a identity key.
    pub async fn pop_opk(&self, ik: &VerifyingKey) -> tonic::Result<Option<X25519PublicKey>> {
        let ik = ik.to_bytes();
        info!(
            "Popping one time key for ik \"{}\" from the database.",
            base64.encode(ik)
        );

        self.0
            .call(move |connection| {
                let key: Option<[u8;32]> = match connection.query_row(
                        "DELETE FROM opk_queue WHERE opk = ( SELECT opk FROM opk_queue WHERE ik = ?1 ORDER BY time LIMIT 1) RETURNING opk", 
                        params![ik],
                        |row| row.get(0)) {
                        Ok(value) => Ok(Some(value)),
                        Err(Error::QueryReturnedNoRows) => Ok(None),
                        Err(e) => Err(e),
                    }?;
                Ok(key.map(X25519PublicKey::from))
                })
            .await
            .map_err(|e| Status::not_found(format!("failed to query for pre_key: {e}")))
    }

    /// Enqueue a message for a given recipient.
    pub async fn add_message(
        &self,
        recipient: &VerifyingKey,
        message: MessageProto,
    ) -> tonic::Result<()> {
        info!(
            "Enqueueing message for recipient {} in database.",
            base64.encode(recipient)
        );
        let recipient = recipient.to_bytes();

        self.0
            .call(move |connection| {
                connection.execute(
                    "INSERT INTO mailbox (message, ik, time) VALUES (?1, ?2, ?3)",
                    (message.encode_to_vec(), recipient, time_now()),
                )?;
                Ok(())
            })
            .await
            .map_err(|e| Status::not_found(format!("Cannot enqueue message for unknown user: {e}")))
    }

    /// Retrieve enqueued messages for a given identity.
    // TODO - Consider changing this to an async stream.
    pub async fn get_messages(&self, recipient: &VerifyingKey) -> tonic::Result<Vec<MessageProto>> {
        info!(
            "Retrieving messages for \"{}\" from the database.",
            base64.encode(recipient)
        );
        let recipient = recipient.to_bytes();

        self.0
            .call(move |connection| {
                let mut stmt =
                    connection.prepare("DELETE FROM mailbox WHERE ik = ?1 RETURNING message")?;
                let message_iter = stmt
                    .query_map(params![recipient], |row| row.get(0))
                    .unwrap();
                let mut ret = Vec::new();
                for message in message_iter {
                    let message: Vec<u8> = message?;
                    ret.push(
                        MessageProto::decode(&*message).expect("We don't persist bad messages."),
                    );
                }
                Ok(ret)
            })
            .await
            .map_err(|e| Status::internal(format!("Failed to query messages: {e}")))
    }

    pub async fn get_one_time_prekey_count(&self, ik: &VerifyingKey) -> tonic::Result<u32> {
        info!(
            "Retrieving opk count for \"{}\" from the database.",
            base64.encode(ik)
        );
        let ik = ik.to_bytes();

        self.0
            .call(move |connection| {
                match connection.query_row(
                    "SELECT COUNT(*) FROM opk_queue WHERE ik = $1",
                    [ik],
                    |row| row.get(0),
                ) {
                    Ok(value) => Ok(value),
                    Err(Error::QueryReturnedNoRows) => Ok(0),
                    Err(e) => Err(tokio_rusqlite::Error::Rusqlite(e)),
                }
            })
            .await
            .map_err(|_| Status::internal("Failed to query opk count."))
    }
}

#[cfg(test)]
mod tests {
    use crate::persistence::*;
    use anyhow::Result;
    use chacha20poly1305::aead::OsRng;
    use client::X3DHClient;
    use ed25519_dalek::SigningKey;
    use tokio_rusqlite::Connection;
    use tonic::Code;

    #[tokio::test]
    async fn add_user_get_keys_success() -> Result<()> {
        let conn = Connection::open_in_memory().await?;
        let storage = SqliteStorage::new(conn.clone()).await?;
        let alice = X3DHClient::new(conn).await?;
        let alice_ik = VerifyingKey::from(&alice.get_ik().await.unwrap());
        let alice_spk: SignedPreKeyProto = alice.get_spk().await.unwrap().into();
        storage.add_user(alice_ik, alice_spk.clone()).await?;
        assert_eq!(storage.get_current_spk(&alice_ik).await?, alice_spk);
        Ok(())
    }

    #[tokio::test]
    async fn add_user_idempotent() -> Result<()> {
        let conn = Connection::open_in_memory().await?;
        let storage = SqliteStorage::new(conn.clone()).await?;
        let alice = X3DHClient::new(conn).await?;
        let alice_ik = VerifyingKey::from(&alice.get_ik().await.unwrap());
        let alice_spk: SignedPreKeyProto = alice.get_spk().await.unwrap().into();
        storage.add_user(alice_ik, alice_spk.clone()).await?;
        storage.add_user(alice_ik, alice_spk.clone()).await?;

        Ok(())
    }

    #[tokio::test]
    async fn add_user_overwrite_fails() -> Result<()> {
        let conn = Connection::open_in_memory().await?;
        let storage = SqliteStorage::new(conn.clone()).await?;
        let alice = X3DHClient::new(conn).await?;
        let alice_ik = VerifyingKey::from(&alice.get_ik().await.unwrap());
        let alice_spk: SignedPreKeyProto = alice.get_spk().await.unwrap().into();
        storage.add_user(alice_ik, alice_spk.clone()).await?;

        let alice2 = X3DHClient::new(Connection::open_in_memory().await?).await?;
        let alice2_spk: SignedPreKeyProto = alice2.get_spk().await.unwrap().into();

        assert_eq!(
            storage
                .add_user(alice_ik, alice2_spk)
                .await
                .err()
                .map(|e| e.code()),
            Some(Code::AlreadyExists)
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_keys_not_found() -> Result<()> {
        let identity_key = SigningKey::generate(&mut OsRng);
        let storage = SqliteStorage::new(Connection::open_in_memory().await?).await?;
        assert_eq!(
            storage
                .get_current_spk(&VerifyingKey::from(&identity_key))
                .await
                .err()
                .map(|e| e.code()),
            Some(Code::NotFound)
        );
        Ok(())
    }

    #[tokio::test]
    async fn pop_empty_opks_none() -> Result<()> {
        let identity_key = SigningKey::generate(&mut OsRng);
        let storage = SqliteStorage::new(Connection::open_in_memory().await?).await?;
        assert_eq!(
            storage.pop_opk(&VerifyingKey::from(&identity_key)).await?,
            None
        );
        Ok(())
    }

    #[tokio::test]
    async fn retrieve_opk() -> Result<()> {
        let conn = Connection::open_in_memory().await?;
        let storage = SqliteStorage::new(conn.clone()).await?;
        let bob = X3DHClient::new(conn).await?;
        let bob_ik = VerifyingKey::from(&bob.get_ik().await?);
        let keys = bob.create_opks(1).await?.pre_keys;
        storage
            .add_user((&bob.get_ik().await?).into(), bob.get_spk().await?.into())
            .await?;
        storage.add_opks(&bob_ik, keys.clone()).await?;
        assert_eq!(storage.pop_opk(&bob_ik).await?, Some(keys[0]));
        assert_eq!(storage.pop_opk(&bob_ik).await?, None);
        Ok(())
    }

    #[tokio::test]
    async fn updating_spk_user_not_found() -> Result<()> {
        let identity_key = VerifyingKey::from(&SigningKey::generate(&mut OsRng));
        let storage = SqliteStorage::new(Connection::open_in_memory().await?).await?;
        assert_eq!(
            storage
                .update_spk(&identity_key, SignedPreKeyProto::default())
                .await
                .err()
                .map(|e| e.code()),
            Some(Code::NotFound)
        );
        Ok(())
    }

    #[tokio::test]
    async fn update_spk_success() -> Result<()> {
        let conn = Connection::open_in_memory().await?;
        let storage = SqliteStorage::new(conn.clone()).await?;
        let bob = X3DHClient::new(conn).await?;
        let bob_ik = VerifyingKey::from(&bob.get_ik().await.unwrap());
        let bob_spk: SignedPreKeyProto = bob.get_spk().await.unwrap().into();
        storage.add_user(bob_ik, bob_spk.clone()).await?;

        // Create a different spk and overwrite it.
        let new_spk = SignedPreKeyProto {
            pre_key: Some(bob.create_opks(1).await?.pre_keys[0].to_bytes().to_vec()),
            signature: bob_spk.signature.clone(),
        };
        storage.update_spk(&bob_ik, new_spk.clone()).await?;

        assert_eq!(storage.get_current_spk(&bob_ik).await?, new_spk);
        Ok(())
    }

    #[tokio::test]
    async fn add_message_unknown_user() -> Result<()> {
        let identity_key = SigningKey::generate(&mut OsRng);
        let storage = SqliteStorage::new(Connection::open_in_memory().await?).await?;
        assert_eq!(
            storage
                .add_message(&VerifyingKey::from(&identity_key), MessageProto::default())
                .await
                .err()
                .map(|e| e.code()),
            Some(Code::NotFound)
        );
        Ok(())
    }

    #[tokio::test]
    async fn add_get_message() -> Result<()> {
        let conn = Connection::open_in_memory().await?;
        let storage = SqliteStorage::new(conn.clone()).await?;
        let bob = X3DHClient::new(conn).await?;
        let bob_ik = VerifyingKey::from(&bob.get_ik().await.unwrap());
        let bob_spk: protocol::x3dh::SignedPreKey = bob.get_spk().await.unwrap();
        storage.add_user(bob_ik, bob_spk.clone().into()).await?;

        let message_proto = MessageProto {
            sender_identity_key: Some(b"alice identity key".to_vec()),
            ephemeral_key: Some(b"alice ephemeral key".to_vec()),
            pre_key: Some(b"bob pre key".to_vec()),
            one_time_key: Some(b"bob one time key".to_vec()),
            ciphertext: Some(b"ciphertext".to_vec()),
        };
        storage.add_message(&bob_ik, message_proto.clone()).await?;
        assert_eq!(storage.get_messages(&bob_ik).await?, vec![message_proto]);

        Ok(())
    }
}
