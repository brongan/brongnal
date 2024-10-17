use crate::{ClientError, ClientResult, X3DHClient};
use chacha20poly1305::aead::OsRng;
use ed25519_dalek::SigningKey;
use protocol::bundle::{create_prekey_bundle, sign_bundle};
use protocol::x3dh;
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};
use x3dh::{SignedPreKey, SignedPreKeys};

#[derive(Clone, Copy, strum_macros::Display)]
#[repr(u32)]
enum KeyType {
    PreKey = 1,
    OneTimeKey = 2,
}

struct PreKey {
    pub_key: X25519PublicKey,
    priv_key: X25519StaticSecret,
    key_type: KeyType,
}

pub struct SqliteClient {
    identity_key: SigningKey,
    connection: Connection,
}

fn read_ik(path: &Path) -> ClientResult<SigningKey> {
    let key_bytes =
        std::fs::read(path).map_err(|_| ClientError::LoadIdentityKey("file missing"))?;
    let key_bytes: &[u8] = &key_bytes;
    SigningKey::from_keypair_bytes(
        key_bytes
            .try_into()
            .map_err(|_| ClientError::LoadIdentityKey("wrong size"))?,
    )
    .map_err(|_| ClientError::LoadIdentityKey("invalid key"))
}

fn init_ik(path: &Path) -> ClientResult<SigningKey> {
    let key = SigningKey::generate(&mut OsRng);
    std::fs::write(path, key.to_keypair_bytes()).map_err(|_| ClientError::SaveIdentityKey)?;
    Ok(key)
}

impl SqliteClient {
    pub fn new(identity_key_path: &Path, db_path: &Path) -> ClientResult<SqliteClient> {
        let identity_key = match read_ik(&identity_key_path) {
            Ok(key) => key,
            Err(_) => init_ik(&identity_key_path)?,
        };

        let connection = Connection::open(db_path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "normal")?;
        connection.pragma_update(None, "foreign_keys", "on")?;

        connection.execute(
            "create table if not exists keys (
             public_key blob primary key,
             private_key blob not null,
             key_type integer not null,
             creation_time integer not null
         )",
            (),
        )?;

        let pre_key = X25519StaticSecret::random_from_rng(OsRng);
        let sqlite_client = SqliteClient {
            identity_key,
            connection,
        };
        sqlite_client.insert(&[PreKey {
            pub_key: X25519PublicKey::from(&pre_key),
            priv_key: pre_key,
            key_type: KeyType::PreKey,
        }])?;
        Ok(sqlite_client)
    }

    fn insert(&self, keys: &[PreKey]) -> ClientResult<()> {
        let mut stmt = self.connection.prepare(
            "INSERT INTO keys (public_key, private_key, key_type, creation_time) VALUES (?1, ?2, ?3, ?4)")?;
        for key in keys {
            stmt.execute((
                key.pub_key.to_bytes(),
                key.priv_key.to_bytes(),
                key.key_type as u32,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ))
            .map_err(|e| ClientError::InsertPreKey(e))?;
        }
        Ok(())
    }
}

impl X3DHClient for SqliteClient {
    fn fetch_wipe_opk(
        &mut self,
        one_time_prekey: &X25519PublicKey,
    ) -> ClientResult<X25519StaticSecret> {
        let key: [u8; 32] = self
            .connection
            .query_row(
                "DELETE from keys WHERE public_key=?1 RETURNING private_key",
                params![one_time_prekey.to_bytes()],
                |row| Ok(row.get(0)?),
            )
            .map_err(|_| ClientError::WipeOpk(*one_time_prekey))?;
        Ok(X25519StaticSecret::from(key))
    }

    fn get_ik(&self) -> ClientResult<SigningKey> {
        Ok(self.identity_key.clone())
    }

    fn get_pre_key(&self) -> ClientResult<X25519StaticSecret> {
        let mut stmt = self.connection.prepare(
            "SELECT private_key FROM keys WHERE key_type = 1 ORDER BY creation_time DESC LIMIT 1",
        ).map_err(|e| ClientError::RetrievePreKey(e))?;
        let key = stmt
            .query_row([], |row| {
                let key: Vec<u8> = row.get(0)?;
                return Ok(key);
            })
            .map_err(|e| ClientError::RetrievePreKey(e))?;
        let key: [u8; 32] = key.try_into().unwrap();
        Ok(X25519StaticSecret::from(key))
    }

    fn get_spk(&self) -> ClientResult<SignedPreKey> {
        let pre_key = self.get_pre_key()?;
        Ok(SignedPreKey {
            pre_key: X25519PublicKey::from(&pre_key),
            signature: sign_bundle(
                &self.identity_key,
                &[(pre_key.clone(), X25519PublicKey::from(&pre_key))],
            ),
        })
    }

    fn create_opks(&mut self, num_keys: u32) -> ClientResult<SignedPreKeys> {
        let opks = create_prekey_bundle(&self.identity_key, num_keys);
        let pre_keys = opks.bundle.iter().map(|(_, _pub)| *_pub).collect();
        let persisted_pre_keys: Vec<PreKey> = opks
            .bundle
            .into_iter()
            .map(|opk| PreKey {
                priv_key: opk.0,
                pub_key: opk.1,
                key_type: KeyType::OneTimeKey,
            })
            .collect();
        self.insert(&persisted_pre_keys)?;
        Ok(SignedPreKeys {
            pre_keys,
            signature: opks.signature,
        })
    }
}
