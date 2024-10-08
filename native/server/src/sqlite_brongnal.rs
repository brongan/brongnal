use crate::brongnal::Storage;
use anyhow::{Context, Result};
use ed25519_dalek::VerifyingKey;
use prost::Message;
use proto::parse_verifying_key;
use proto::service::Message as MessageProto;
use proto::service::SignedPreKey as SignedPreKeyProto;
use rusqlite::{params, Connection};
use std::sync::MutexGuard;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{sync::Arc, sync::Mutex};
use tonic::Status;
use x25519_dalek::PublicKey as X25519PublicKey;

#[derive(Debug)]
pub struct SqliteStorage(Arc<Mutex<Connection>>);

impl SqliteStorage {
    fn connection(&self) -> tonic::Result<MutexGuard<Connection>> {
        self.0
            .lock()
            .map_err(|_e| Status::internal("failed to access sqlite connection"))
    }
    pub fn new(connection: Connection) -> Result<Self> {
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "normal")?;
        connection.pragma_update(None, "foreign_keys", "on")?;

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
            .context("Creating user table failed.")?;
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
            .context("Creating pre_key table failed.")?;
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
            .context("Creating message table failed.")?;

        Ok(SqliteStorage(Arc::new(Mutex::new(connection))))
    }
}

impl Storage for SqliteStorage {
    fn register_user(
        &self,
        identity: String,
        ik: VerifyingKey,
        spk: SignedPreKeyProto,
    ) -> tonic::Result<()> {
        println!("Adding user \"{identity}\" to the database.");

        let _ = self.connection()?.execute(
            "INSERT INTO user (identity, key, current_pre_key, creation_time) VALUES (?1, ?2, ?3, ?4)",
            (
                identity, ik.to_bytes(), spk.encode_to_vec(),
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            ),
        ).context("failed to insert key.");
        Ok(())
    }

    fn update_spk(&self, identity: &str, spk: SignedPreKeyProto) -> tonic::Result<()> {
        println!("Updating pre key for user \"{identity}\" to the database.");

        let _: String = self
            .connection()?
            .query_row(
                "UPDATE user SET current_pre_key = ?2 WHERE identity = ?1 RETURNING identity",
                params![identity, spk.encode_to_vec()],
                |row| Ok(row.get(0)?),
            )
            .map_err(|_| Status::not_found("user not found"))?;
        Ok(())
    }

    fn add_opks(&self, identity: &str, opks: Vec<X25519PublicKey>) -> tonic::Result<()> {
        println!(
            "Adding {} one time keys for user \"{identity}\" to the database.",
            opks.len()
        );

        let connection = self.connection()?;
        let mut stmt = connection
            .prepare("INSERT INTO pre_key (user_identity, key, creation_time) VALUES (?1, ?2, ?3)")
            .unwrap();
        for opk in opks {
            stmt.execute((
                identity,
                opk.to_bytes(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ))
            .map_err(|_| Status::internal("failed to insert one time key"))?;
        }
        Ok(())
    }

    fn get_current_keys(&self, identity: &str) -> tonic::Result<(VerifyingKey, SignedPreKeyProto)> {
        println!("Retrieving pre keys for user \"{identity}\" from the database.");

        let (ik, spk): (Vec<u8>, Vec<u8>) = self
            .connection()?
            .query_row(
                "SELECT key, current_pre_key FROM user WHERE identity = ?1",
                [identity],
                |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())),
            )
            .map_err(|_| Status::not_found("user not found"))?;
        let ik = parse_verifying_key(&ik).unwrap();
        let spk = SignedPreKeyProto::decode(&*spk).unwrap();
        Ok((ik, spk))
    }

    fn pop_opk(&self, identity: &str) -> tonic::Result<Option<X25519PublicKey>> {
        println!("Popping one time key for user \"{identity}\" from the database.");

        let key: Option<[u8;32]> = match self.connection()?.query_row(
            "DELETE from pre_key WHERE key = ( SELECT key FROM pre_key WHERE user_identity = ?1 ORDER BY creation_time LIMIT 1) RETURNING key", 
            [identity.to_owned()],
            |row| row.get(0)) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Status::not_found(format!("failed to query for pre_key: {e}"))),
        }?;

        Ok(key.map(|key| X25519PublicKey::from(key)))
    }

    fn add_message(&self, recipient: &str, message: MessageProto) -> tonic::Result<()> {
        println!("Enqueueing message for user {recipient} in database.");

        let _: u64 = self
            .connection()?
            .query_row(
                "INSERT INTO message (message, user_identity, creation_time) VALUES (?1, ?2, ?3) RETURNING creation_time",
                (
                    message.encode_to_vec(),
                    recipient,
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                ),|row| Ok(row.get(0)?),
            )
            .map_err(|_| Status::not_found("user not found"))?;
        Ok(())
    }

    fn get_messages(&self, identity: &str) -> tonic::Result<Vec<MessageProto>> {
        println!("Retrieving messages for \"{identity}\" from the database.");

        let connection = self.connection()?;
        let mut stmt = connection
            .prepare("DELETE from message WHERE user_identity = ?1 RETURNING message")
            .map_err(|e| {
                Status::internal(format!("Failed to query message table for {identity}: {e}"))
            })?;
        let message_iter = stmt.query_map([identity], |row| Ok(row.get(0)?)).unwrap();
        let mut ret = Vec::new();
        for message in message_iter {
            // TODO wtf is happening here?
            let message: Vec<u8> = message.unwrap();
            ret.push(
                MessageProto::decode(&*message)
                    .map_err(|_| Status::internal("Failed to deserialize Message proto"))?,
            );
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use crate::sqlite_brongnal::*;
    use anyhow::Result;
    use client::{memory_client::MemoryClient, X3DHClient};
    use tonic::Code;

    #[test]
    fn register_user_get_keys_success() -> Result<()> {
        let storage = SqliteStorage::new(Connection::open_in_memory()?)?;
        let alice = MemoryClient::new();
        let alice_ik = VerifyingKey::from(&alice.get_ik().unwrap());
        let alice_spk: SignedPreKeyProto = alice.get_spk().unwrap().into();
        assert_eq!(
            storage.register_user(String::from("alice"), alice_ik, alice_spk.clone())?,
            ()
        );
        assert_eq!(storage.get_current_keys("alice")?, (alice_ik, alice_spk));
        Ok(())
    }

    #[test]
    fn get_keys_not_found() -> Result<()> {
        let storage = SqliteStorage::new(Connection::open_in_memory()?)?;
        assert_eq!(
            storage.get_current_keys("alice").err().map(|e| e.code()),
            Some(Code::NotFound)
        );
        Ok(())
    }

    #[test]
    fn pop_empty_opks_none() -> Result<()> {
        let storage = SqliteStorage::new(Connection::open_in_memory()?)?;
        assert_eq!(storage.pop_opk("bob")?, None);
        Ok(())
    }

    #[test]
    fn retrieve_opk() -> Result<()> {
        let storage = SqliteStorage::new(Connection::open_in_memory()?)?;
        let mut bob = MemoryClient::new();
        let keys = bob.create_opks(1)?.pre_keys;
        storage.register_user(
            String::from("bob"),
            (&bob.get_ik()?).into(),
            bob.get_spk()?.into(),
        )?;
        storage.add_opks("bob", keys.clone())?;
        assert_eq!(storage.pop_opk("bob")?, Some(keys[0]));
        assert_eq!(storage.pop_opk("bob")?, None);
        Ok(())
    }

    #[test]
    fn updating_spk_user_not_found() -> Result<()> {
        let storage = SqliteStorage::new(Connection::open_in_memory()?)?;
        assert_eq!(
            storage
                .update_spk("bob", SignedPreKeyProto::default())
                .err()
                .map(|e| e.code()),
            Some(Code::NotFound)
        );
        Ok(())
    }

    #[test]
    fn update_spk_success() -> Result<()> {
        let storage = SqliteStorage::new(Connection::open_in_memory()?)?;
        let mut bob = MemoryClient::new();
        let bob_ik = VerifyingKey::from(&bob.get_ik().unwrap());
        let mut bob_spk: SignedPreKeyProto = bob.get_spk().unwrap().into();
        storage.register_user(String::from("bob"), bob_ik, bob_spk.clone())?;

        bob_spk.pre_key = Some(bob.create_opks(1)?.pre_keys[0].to_bytes().to_vec());
        storage.update_spk("bob", bob_spk.clone())?;

        assert_eq!(storage.get_current_keys("bob")?, (bob_ik, bob_spk));
        Ok(())
    }

    #[test]
    fn add_message_unknown_user() -> Result<()> {
        let storage = SqliteStorage::new(Connection::open_in_memory()?)?;
        assert_eq!(
            storage
                .add_message("bob", MessageProto::default())
                .err()
                .map(|e| e.code()),
            Some(Code::NotFound)
        );
        Ok(())
    }

    #[test]
    fn add_get_message() -> Result<()> {
        let storage = SqliteStorage::new(Connection::open_in_memory()?)?;
        let bob = MemoryClient::new();
        let bob_ik = VerifyingKey::from(&bob.get_ik().unwrap());
        let bob_spk: protocol::x3dh::SignedPreKey = bob.get_spk().unwrap();
        storage.register_user(String::from("bob"), bob_ik, bob_spk.clone().into())?;

        let message_proto = MessageProto {
            sender_identity: Some(String::from("alice")),
            sender_identity_key: Some(b"alice identity key".to_vec()),
            ephemeral_key: Some(b"alice ephemeral key".to_vec()),
            one_time_key: Some(b"bob one time key".to_vec()),
            ciphertext: Some(b"ciphertext".to_vec()),
        };
        storage.add_message("bob", message_proto.clone())?;
        assert_eq!(storage.get_messages("bob")?, vec![message_proto]);

        Ok(())
    }
}
