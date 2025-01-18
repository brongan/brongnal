use crate::brongnal::CurrentKeys;
use ed25519_dalek::VerifyingKey;
use libsql::params;
use prost::Message;
use proto::parse_verifying_key;
use proto::service::Message as MessageProto;
use proto::service::SignedPreKey as SignedPreKeyProto;
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::Status;
use tracing::info;
use x25519_dalek::PublicKey as X25519PublicKey;

pub struct SqliteStorage(libsql::Connection);

impl SqliteStorage {
    pub async fn new(connection: libsql::Connection) -> libsql::Result<Self> {
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS user (
                        identity STRING PRIMARY KEY,
                        key BLOB NOT NULL,
                        current_pre_key BLOB NOT NULL,
                        creation_time INTEGER NOT NULL
                    )",
                (),
            )
            .await?;
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS pre_key (
                        key BLOB PRIMARY KEY,
                        user_identity STRING NOT NULL,
                        creation_time integer NOT NULL,
                        FOREIGN KEY(user_identity) REFERENCES user(identity)
                    )",
                (),
            )
            .await?;
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS message (
                        message BLOB PRIMARY KEY,
                        user_identity STRING NOT NULL,
                        creation_time integer NOT NULL,
                        FOREIGN KEY(user_identity) REFERENCES user(identity)
                    )",
                (),
            )
            .await?;
        Ok(SqliteStorage(connection))
    }
}

impl SqliteStorage {
    /// Add a new identity to the storage.
    /// Attempts to overwrite an identity key returns an error.
    pub async fn register_user(
        &self,
        identity: String,
        ik: VerifyingKey,
        spk: SignedPreKeyProto,
    ) -> tonic::Result<()> {
        info!("Adding user \"{identity}\" to the database.");
        println!("{identity}, {ik:?}");
        let connection = &self.0;
        connection.execute(
                    "INSERT OR IGNORE INTO user (identity, key, current_pre_key, creation_time) VALUES (?1, ?2, ?3, ?4)",
                    params![
                        identity.clone(),
                        ik.to_bytes(),
                        spk.encode_to_vec(),
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    ]
        ).await.map_err(|e| Status::internal(format!("Failed to register user: {e}")))?;
        let mut rows = connection
            .query(
                "SELECT key FROM user where identity = ?1",
                params![identity],
            )
            .await
            .map_err(|e| {
                Status::internal(format!("Unexpected error when registering user: {e}"))
            })?;
        let row = rows
            .next()
            .await
            .unwrap()
            .ok_or(Status::not_found("user not found"))?;

        let persisted_ik = parse_verifying_key(&row.get::<[u8; 32]>(0).unwrap()).unwrap();

        if ik != persisted_ik {
            return Err(Status::already_exists(
                "A user is already registered with username: {identity}",
            ));
        }
        Ok(())
    }

    /// Replaces the signed pre key for a given identity.
    // TODO(https://github.com/brongan/brongnal/issues/27) -  Implement signed pre key rotation.
    #[allow(dead_code)]
    pub async fn update_spk(&self, identity: String, spk: SignedPreKeyProto) -> tonic::Result<()> {
        info!("Updating pre key for user \"{identity}\" to the database.");

        let connection = &self.0;
        let rows_changed = connection
            .execute(
                "UPDATE user SET current_pre_key = ?2 WHERE identity = ?1",
                params![identity.clone(), spk.encode_to_vec()],
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to update spk: {e}")))?;
        if rows_changed != 1 {
            return Err(Status::not_found(format!(
                "pre key for user {identity} not found. rows changed: {rows_changed}"
            )));
        }
        Ok(())
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

        let connection = &self.0;
        let mut stmt = connection
            .prepare("INSERT INTO pre_key (user_identity, key, creation_time) VALUES (?1, ?2, ?3)")
            .await
            .unwrap();
        for opk in opks {
            stmt.execute(params![
                identity.clone(),
                opk.to_bytes(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ])
            .await
            .map_err(|_| Status::internal("failed to insert one time key"))?;
        }
        Ok(())
    }

    /// Retrieves the identity key and signed pre key for a given identity.
    /// A client must first invoke this before messaging a peer.
    pub async fn get_current_keys(&self, identity: String) -> tonic::Result<CurrentKeys> {
        info!("Retrieving pre keys for user \"{identity}\" from the database.");

        let connection = &self.0;
        let mut rows = connection
            .query(
                "SELECT key, current_pre_key FROM user WHERE identity = ?1",
                params![identity],
            )
            .await
            .unwrap();

        let row = rows
            .next()
            .await
            .unwrap()
            .ok_or(Status::not_found("user not found"))?;

        Ok((
            parse_verifying_key(&row.get::<[u8; 32]>(0).unwrap()).unwrap(),
            SignedPreKeyProto::decode(&*(row.get::<Vec<u8>>(1)).unwrap()).unwrap(),
        ))
    }

    /// Retrieve a one time pre key for an identity.
    pub async fn pop_opk(&self, identity: String) -> tonic::Result<Option<X25519PublicKey>> {
        info!("Popping one time key for user \"{identity}\" from the database.");

        let connection = &self.0;
        let mut rows = connection.query(
            "DELETE from pre_key WHERE key = ( SELECT key FROM pre_key WHERE user_identity = ?1 ORDER BY creation_time LIMIT 1) RETURNING key", 
            params![identity.to_owned()]).await
            .map_err(|e| Status::not_found(format!("failed to query for pre_key: {e}")))?;

        let row = rows
            .next()
            .await
            .unwrap()
            .map(|row| row.get::<[u8; 32]>(0).unwrap());
        Ok(row.map(X25519PublicKey::from))
    }

    /// Enqueue a message for a given recipient.
    pub async fn add_message(&self, recipient: String, message: MessageProto) -> tonic::Result<()> {
        info!("Enqueueing message for user {recipient} in database.");

        let connection = &self.0;
        connection
            .execute(
                "INSERT INTO message (message, user_identity, creation_time) VALUES (?1, ?2, ?3)",
                params![
                    message.encode_to_vec(),
                    recipient.clone(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                ],
            )
            .await
            .map_err(|_| {
                Status::not_found(format!(
                    "Cannot enqueue message for unknown user: {recipient}"
                ))
            })?;
        Ok(())
    }

    /// Retrieve enqueued messages for a given identity.
    // TODO: Maybe not delete the messages until they're sent to the user?
    pub async fn get_messages(&self, identity: String) -> tonic::Result<Vec<MessageProto>> {
        info!("Retrieving messages for \"{identity}\" from the database.");

        let connection = &self.0;
        let mut rows = connection
            .query(
                "DELETE from message WHERE user_identity = ?1 RETURNING message",
                params![identity],
            )
            .await
            .unwrap();

        let mut ret = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|_| Status::internal("Failed to query messages."))?
        {
            let message: Vec<u8> = row.get(0).unwrap();
            ret.push(MessageProto::decode(&*message).expect("We don't persist bad messages."));
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use crate::persistence::*;
    use anyhow::Result;
    use client::X3DHClient;
    use libsql::Builder;
    use tonic::Code;

    #[tokio::test]
    async fn register_user_get_keys_success() -> Result<()> {
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let storage = SqliteStorage::new(conn.clone()).await?;
        let alice = X3DHClient::new(conn).await?;
        let alice_ik = VerifyingKey::from(&alice.get_ik().await.unwrap());
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
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let storage = SqliteStorage::new(conn.clone()).await?;
        let alice = X3DHClient::new(conn).await?;
        let alice_ik = VerifyingKey::from(&alice.get_ik().await.unwrap());
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
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let storage = SqliteStorage::new(conn.clone()).await?;
        let alice = X3DHClient::new(conn).await?;
        let alice_ik = VerifyingKey::from(&alice.get_ik().await.unwrap());
        let alice_spk: SignedPreKeyProto = alice.get_spk().await.unwrap().into();
        storage
            .register_user(String::from("alice"), alice_ik, alice_spk.clone())
            .await?;

        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let alice2 = X3DHClient::new(conn).await?;
        let alice2_ik = VerifyingKey::from(&alice2.get_ik().await.unwrap());

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
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let storage = SqliteStorage::new(conn).await?;
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
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let storage = SqliteStorage::new(conn).await?;
        assert_eq!(storage.pop_opk(String::from("bob")).await?, None);
        Ok(())
    }

    #[tokio::test]
    async fn retrieve_opk() -> Result<()> {
        let bob_name = String::from("bob");
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let storage = SqliteStorage::new(conn.clone()).await?;
        let bob = X3DHClient::new(conn).await?;
        let keys = bob.create_opks(1).await?.pre_keys;
        storage
            .register_user(
                bob_name.clone(),
                (&bob.get_ik().await?).into(),
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
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let storage = SqliteStorage::new(conn).await?;
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
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let storage = SqliteStorage::new(conn.clone()).await?;
        let bob = X3DHClient::new(conn).await?;
        let bob_ik = VerifyingKey::from(&bob.get_ik().await.unwrap());
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
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let storage = SqliteStorage::new(conn).await?;
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
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let storage = SqliteStorage::new(conn.clone()).await?;
        let bob = X3DHClient::new(conn).await?;
        let bob_ik = VerifyingKey::from(&bob.get_ik().await.unwrap());
        let bob_spk: protocol::x3dh::SignedPreKey = bob.get_spk().await.unwrap();
        storage
            .register_user(bob_name.clone(), bob_ik, bob_spk.clone().into())
            .await?;

        let message_proto = MessageProto {
            sender_identity: Some(String::from("alice")),
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
