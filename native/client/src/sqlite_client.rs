use crate::X3DHClient;
use anyhow::{anyhow, Context, Result};
use chacha20poly1305::aead::OsRng;
use ed25519_dalek::SigningKey;
use protocol::bundle::{create_prekey_bundle, sign_bundle};
use protocol::x3dh;
use rusqlite::Connection;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};
use x3dh::{SignedPreKey, SignedPreKeys};

enum KeyType {
    PreKey = 1,
    OneTimeKey = 2,
}

pub struct SqliteClient {
    identity_key: SigningKey,
    connection: Connection,
}

fn read_identity_key(path: &Path) -> Result<SigningKey> {
    let key_bytes = std::fs::read(path)?;
    let key_bytes: &[u8] = &key_bytes;
    SigningKey::from_keypair_bytes(
        key_bytes
            .try_into()
            .map_err(|_| anyhow!("invalidly sized key"))?,
    )
    .map_err(|_| anyhow!("invalid key"))
}

fn create_identity_key(path: &Path) -> Result<SigningKey> {
    let key = SigningKey::generate(&mut OsRng);
    std::fs::write(path, key.to_keypair_bytes())
        .context("Failed to write identity key to disk.")?;
    Ok(key)
}

impl SqliteClient {
    pub fn new(identity_key_path: &Path, db_path: &Path) -> Result<SqliteClient> {
        let identity_key = match read_identity_key(&identity_key_path) {
            Ok(key) => key,
            Err(_) => create_identity_key(&identity_key_path)
                .context("Failed to create new identity key.")?,
        };

        let connection = Connection::open(db_path).context("Failed to open db_path.")?;
        connection
            .execute(
                "create table if not exists keys (
             public_key blob primary key,
             private_key blob not null,
             key_type integer not null,
             creation_time integer not null
         )",
                (),
            )
            .context("Creating initial table failed.")?;

        let pre_key = X25519StaticSecret::random_from_rng(OsRng);
        let sqlite_client = SqliteClient {
            identity_key,
            connection,
        };
        sqlite_client.insert(pre_key, KeyType::PreKey)?;
        Ok(sqlite_client)
    }

    fn insert(&self, key: X25519StaticSecret, key_type: KeyType) -> Result<()> {
        let _ = self.connection.execute(
            "INSERT INTO keys (public_key, private_key, key_type, creation_time) VALUES (?1, ?2, ?3, ?4)",
            (
                X25519PublicKey::from(&key).to_bytes(),
                key.to_bytes(),
                key_type as u32,
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            ),
        ).context("Failed to insert key.");
        Ok(())
    }
}

impl X3DHClient for SqliteClient {
    fn fetch_wipe_one_time_secret_key(
        &mut self,
        one_time_key: &X25519PublicKey,
    ) -> Result<X25519StaticSecret, anyhow::Error> {
        let serialized = one_time_key.to_bytes();
        let mut stmt = self
            .connection
            .prepare("DELETE from keys WHERE public_key=?1;")?;
        let key: Vec<u8> = stmt.query_row(serialized, |row| row.get(0))?;
        let key: [u8; 32] = key.try_into().map_err(|_| anyhow!("oop"))?;
        Ok(X25519StaticSecret::from(key))
    }

    fn get_identity_key(&self) -> Result<SigningKey, anyhow::Error> {
        Ok(self.identity_key.clone())
    }

    fn get_pre_key(&self) -> Result<X25519StaticSecret, anyhow::Error> {
        let mut stmt = self.connection.prepare(
            "SELECT private_key FROM keys WHERE key_type = 1 ORDER BY creation_time DESC LIMIT 1",
        ).context("Failed to prepare statement.")?;
        let key = stmt
            .query_row([], |row| {
                let key: Vec<u8> = row.get(0)?;
                return Ok(key);
            })
            .context("Failed to find pre_key.")?;
        let key: [u8; 32] = key.try_into().map_err(|_| anyhow!("oop"))?;
        Ok(X25519StaticSecret::from(key))
    }

    fn get_spk(&self) -> Result<SignedPreKey, anyhow::Error> {
        let pre_key = self
            .get_pre_key()
            .context("Failed to get pre_key for spk.")?;
        Ok(SignedPreKey {
            pre_key: X25519PublicKey::from(&pre_key),
            signature: sign_bundle(
                &self.identity_key,
                &[(pre_key.clone(), X25519PublicKey::from(&pre_key))],
            ),
        })
    }

    fn add_one_time_keys(&mut self, num_keys: u32) -> Result<SignedPreKeys> {
        let otks = create_prekey_bundle(&self.identity_key, num_keys);
        let pre_keys = otks.bundle.iter().map(|(_, _pub)| *_pub).collect();
        for otk in otks.bundle {
            self.insert(otk.0, KeyType::OneTimeKey)
                .context("Failed to add one time keys.")?;
        }
        Ok(SignedPreKeys {
            pre_keys,
            signature: otks.signature,
        })
    }
}
