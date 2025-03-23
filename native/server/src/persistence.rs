use crate::brongnal::CurrentKeys;
use ed25519_dalek::VerifyingKey;
use prost::Message;
use proto::parse_verifying_key;
use proto::service::Message as MessageProto;
use proto::service::SignedPreKey as SignedPreKeyProto;
use rusqlite::params;
use rusqlite::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::Status;
use tracing::info;
use x25519_dalek::PublicKey as X25519PublicKey;

pub struct SqliteStorage(tokio_rusqlite::Connection);

impl SqliteStorage {
    pub async fn new(connection: tokio_rusqlite::Connection) -> tokio_rusqlite::Result<Self> {
        connection
            .call(|connection| {
                connection.pragma_update(None, "journal_mode", "WAL")?;
                connection.pragma_update(None, "synchronous", "normal")?;
                connection.pragma_update(None, "foreign_keys", "on")?;
                connection.execute(
                    "CREATE TABLE IF NOT EXISTS user (
                        identity STRING PRIMARY KEY,
                        key BLOB NOT NULL,
                        current_pre_key BLOB NOT NULL,
                        creation_time INTEGER NOT NULL
                    )",
                    (),
                )?;
                connection.execute(
                    "CREATE TABLE IF NOT EXISTS pre_key (
                        key BLOB PRIMARY KEY,
                        user_identity STRING NOT NULL,
                        creation_time integer NOT NULL,
                        FOREIGN KEY(user_identity) REFERENCES user(identity)
                    )",
                    (),
                )?;
                connection.execute(
                    "CREATE TABLE IF NOT EXISTS message (
                        message BLOB PRIMARY KEY,
                        user_identity STRING NOT NULL,
                        creation_time integer NOT NULL,
                        FOREIGN KEY(user_identity) REFERENCES user(identity)
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
    pub async fn register_user(
        &self,
        identity: String,
        ik: VerifyingKey,
        spk: SignedPreKeyProto,
    ) -> tonic::Result<()> {
        info!("Adding user \"{identity}\" to the database.");
        let username = identity.clone();

        let persisted_ik = self
            .0
            .call(move |connection| {
                info!("Adding user \"{identity}\" to the database.");

                connection.execute(
                    "INSERT OR IGNORE INTO user (identity, key, current_pre_key, creation_time) VALUES (?1, ?2, ?3, ?4)",
                    params![
                        identity,
                        ik.to_bytes(),
                        spk.encode_to_vec(),
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    ],
                )?;
                let ik: [u8; 32] = connection.query_row("SELECT key FROM user where identity = ?1", params![identity], |row|  row.get(0))?;
                let ik = parse_verifying_key(&ik).unwrap();
                Ok(ik)
            })
            .await.map_err(|e| Status::internal(format!("Failed to register user: {e}")))?;
        if ik != persisted_ik {
            return Err(Status::already_exists(format!(
                "A user is already registered with username: {username}"
            )));
        }
        Ok(())
    }

    /// Replaces the signed pre key for a given identity.
    // TODO(https://github.com/brongan/brongnal/issues/27) -  Implement signed pre key rotation.
    #[allow(dead_code)]
    pub async fn update_spk(&self, identity: String, spk: SignedPreKeyProto) -> tonic::Result<()> {
        info!("Updating pre key for user \"{identity}\" to the database.");
        let identity_copy = identity.clone();

        self.0
            .call(move |connection| {
                let identity: &str = &identity;
                let spk = spk;
                // Returns the first row updated so missing key resullts in an error.
                let _: String = connection.query_row(
                    "UPDATE user SET current_pre_key = ?2 WHERE identity = ?1 RETURNING identity",
                    params![&identity, spk.encode_to_vec()],
                    |row| row.get(0),
                )?;
                Ok(())
            })
            .await
            .map_err(|_| Status::not_found(format!("pre key for user {identity_copy} not found")))
    }

    /// Appends new unburnt one time pre keys for others to message a given identity.
    pub async fn add_opks(
        &self,
        identity: String,
        opks: Vec<X25519PublicKey>,
    ) -> tonic::Result<()> {
        info!(
            "Adding {} one time keys for user \"{identity}\" to the database.",
            opks.len(),
        );

        self.0
            .call(move |connection| {
                let mut stmt = connection
                    .prepare("INSERT INTO pre_key (user_identity, key, creation_time) VALUES (?1, ?2, ?3)")
                    .unwrap();
                for opk in opks {
                    stmt.execute((
                            &identity,
                            opk.to_bytes(),
                            SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    ))?;
                }
                Ok(())
            })
            .await
            .map_err(|_| Status::internal("failed to insert one time key"))
    }

    /// Retrieves the identity key and signed pre key for a given identity.
    /// A client must first invoke this before messaging a peer.
    pub async fn get_current_keys(&self, identity: String) -> tonic::Result<CurrentKeys> {
        info!("Retrieving pre keys for user \"{identity}\" from the database.");

        self.0
            .call(move |connection| {
                let user: &str = &identity;
                let (ik, spk): (Vec<u8>, Vec<u8>) = connection.query_row(
                    "SELECT key, current_pre_key FROM user WHERE identity = ?1",
                    [user],
                    |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())),
                )?;
                let ik = parse_verifying_key(&ik).unwrap();
                let spk = SignedPreKeyProto::decode(&*spk).unwrap();
                Ok((ik, spk))
            })
            .await
            .map_err(|_| Status::not_found("user not found"))
    }

    /// Retrieve a one time pre key for an identity.
    pub async fn pop_opk(&self, identity: String) -> tonic::Result<Option<X25519PublicKey>> {
        info!("Popping one time key for user \"{identity}\" from the database.");

        self.0
            .call(move |connection| {
                let identity: &str = &identity;
                let key: Option<[u8;32]> = match connection.query_row(
                        "DELETE from pre_key WHERE key = ( SELECT key FROM pre_key WHERE user_identity = ?1 ORDER BY creation_time LIMIT 1) RETURNING key", 
                        [identity.to_owned()],
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
    pub async fn add_message(&self, recipient: String, message: MessageProto) -> tonic::Result<()> {
        info!("Enqueueing message for user {recipient} in database.");

        self.0
            .call(move |connection| {
                let recipient: &str = &recipient;
                connection.execute(
                    "INSERT INTO message (message, user_identity, creation_time) VALUES (?1, ?2, ?3)",
                    (
                        message.encode_to_vec(),
                        recipient,
                        SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    ),
                )?;
                Ok(())
            }
            )
            .await
            .map_err(|e| {
                Status::not_found(format!("Cannot enqueue message for unknown user: {e}"))
            })
    }

    /// Retrieve enqueued messages for a given identity.
    pub async fn get_messages(&self, identity: String) -> tonic::Result<Vec<MessageProto>> {
        info!("Retrieving messages for \"{identity}\" from the database.");

        self.0
            .call(move |connection| {
                let identity: &str = &identity;
                let mut stmt = connection
                    .prepare("DELETE from message WHERE user_identity = ?1 RETURNING message")?;
                let message_iter = stmt.query_map([identity], |row| row.get(0)).unwrap();
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
            .map_err(|_| Status::internal("Failed to query messages."))
    }

    pub async fn get_one_time_prekey_count(&self, identity: String) -> tonic::Result<u32> {
        info!("Retrieving opk count for \"{identity}\" from the database.");

        self.0
            .call(move |connection| {
                let identity: &str = &identity;
                match connection.query_row(
                    "SELECT COUNT(*) FROM pre_key WHERE user_identity = $1",
                    [identity.to_owned()],
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
    use client::X3DHClient;
    use tokio_rusqlite::Connection;
    use tonic::Code;

    #[tokio::test]
    async fn register_user_get_keys_success() -> Result<()> {
        let conn = Connection::open_in_memory().await?;
        let storage = SqliteStorage::new(conn.clone()).await?;
        let alice = X3DHClient::new(conn).await?;
        let alice_ik = VerifyingKey::from(&alice.get_ik());
        let alice_spk: SignedPreKeyProto = alice.get_spk().await.unwrap().into();
        storage
            .register_user(String::from("alice"), alice_ik, alice_spk.clone())
            .await?;
        assert_eq!(
            storage.get_current_keys(String::from("alice")).await?,
            (alice_ik, alice_spk)
        );
        Ok(())
    }

    #[tokio::test]
    async fn register_user_idempotent() -> Result<()> {
        let conn = Connection::open_in_memory().await?;
        let storage = SqliteStorage::new(conn.clone()).await?;
        let alice = X3DHClient::new(conn).await?;
        let alice_ik = VerifyingKey::from(&alice.get_ik());
        let alice_spk: SignedPreKeyProto = alice.get_spk().await.unwrap().into();
        storage
            .register_user(String::from("alice"), alice_ik, alice_spk.clone())
            .await?;
        storage
            .register_user(String::from("alice"), alice_ik, alice_spk.clone())
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn register_user_overwrite_fails() -> Result<()> {
        let conn = Connection::open_in_memory().await?;
        let storage = SqliteStorage::new(conn.clone()).await?;
        let alice = X3DHClient::new(conn).await?;
        let alice_ik = VerifyingKey::from(&alice.get_ik());
        let alice_spk: SignedPreKeyProto = alice.get_spk().await.unwrap().into();
        storage
            .register_user(String::from("alice"), alice_ik, alice_spk.clone())
            .await?;

        let alice2 = X3DHClient::new(Connection::open_in_memory().await?).await?;
        let alice2_ik = VerifyingKey::from(&alice2.get_ik());

        assert_eq!(
            storage
                .register_user(String::from("alice"), alice2_ik, alice_spk.clone())
                .await
                .err()
                .map(|e| e.code()),
            Some(Code::AlreadyExists)
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_keys_not_found() -> Result<()> {
        let storage = SqliteStorage::new(Connection::open_in_memory().await?).await?;
        assert_eq!(
            storage
                .get_current_keys(String::from("alice"))
                .await
                .err()
                .map(|e| e.code()),
            Some(Code::NotFound)
        );
        Ok(())
    }

    #[tokio::test]
    async fn pop_empty_opks_none() -> Result<()> {
        let storage = SqliteStorage::new(Connection::open_in_memory().await?).await?;
        assert_eq!(storage.pop_opk(String::from("bob")).await?, None);
        Ok(())
    }

    #[tokio::test]
    async fn retrieve_opk() -> Result<()> {
        let bob_name = String::from("bob");
        let conn = Connection::open_in_memory().await?;
        let storage = SqliteStorage::new(conn.clone()).await?;
        let bob = X3DHClient::new(conn).await?;
        let keys = bob.create_opks(1).await?.pre_keys;
        storage
            .register_user(
                bob_name.clone(),
                (&bob.get_ik()).into(),
                bob.get_spk().await?.into(),
            )
            .await?;
        storage.add_opks(bob_name.clone(), keys.clone()).await?;
        assert_eq!(storage.pop_opk(bob_name.clone()).await?, Some(keys[0]));
        assert_eq!(storage.pop_opk(bob_name).await?, None);
        Ok(())
    }

    #[tokio::test]
    async fn updating_spk_user_not_found() -> Result<()> {
        let storage = SqliteStorage::new(Connection::open_in_memory().await?).await?;
        assert_eq!(
            storage
                .update_spk(String::from("bob"), SignedPreKeyProto::default())
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
        let bob_ik = VerifyingKey::from(&bob.get_ik());
        let mut bob_spk: SignedPreKeyProto = bob.get_spk().await.unwrap().into();
        storage
            .register_user(String::from("bob"), bob_ik, bob_spk.clone())
            .await?;

        bob_spk.pre_key = Some(bob.create_opks(1).await?.pre_keys[0].to_bytes().to_vec());
        storage
            .update_spk(String::from("bob"), bob_spk.clone())
            .await?;

        assert_eq!(
            storage.get_current_keys(String::from("bob")).await?,
            (bob_ik, bob_spk)
        );
        Ok(())
    }

    #[tokio::test]
    async fn add_message_unknown_user() -> Result<()> {
        let storage = SqliteStorage::new(Connection::open_in_memory().await?).await?;
        assert_eq!(
            storage
                .add_message(String::from("bob"), MessageProto::default())
                .await
                .err()
                .map(|e| e.code()),
            Some(Code::NotFound)
        );
        Ok(())
    }

    #[tokio::test]
    async fn add_get_message() -> Result<()> {
        let bob_name = String::from("bob");
        let conn = Connection::open_in_memory().await?;
        let storage = SqliteStorage::new(conn.clone()).await?;
        let bob = X3DHClient::new(conn).await?;
        let bob_ik = VerifyingKey::from(&bob.get_ik());
        let bob_spk: protocol::x3dh::SignedPreKey = bob.get_spk().await.unwrap();
        storage
            .register_user(bob_name.clone(), bob_ik, bob_spk.clone().into())
            .await?;

        let message_proto = MessageProto {
            sender_identity_key: Some(b"alice identity key".to_vec()),
            ephemeral_key: Some(b"alice ephemeral key".to_vec()),
            pre_key: Some(b"bob pre key".to_vec()),
            one_time_key: Some(b"bob one time key".to_vec()),
            ciphertext: Some(b"ciphertext".to_vec()),
        };
        storage
            .add_message(bob_name.clone(), message_proto.clone())
            .await?;
        assert_eq!(storage.get_messages(bob_name).await?, vec![message_proto]);

        Ok(())
    }
}
