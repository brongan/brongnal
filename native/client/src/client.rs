use crate::{ClientError, ClientResult};
use chacha20poly1305::aead::OsRng;
use ed25519_dalek::{SigningKey, VerifyingKey};
use protocol::bundle::{create_prekey_bundle, sign_bundle};
use protocol::x3dh;
use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};
use x3dh::{SignedPreKey, SignedPreKeys};

#[derive(Clone, Copy, strum_macros::Display)]
#[repr(u32)]
enum KeyType {
    Identity = 0,
    Pre = 1,
    OneTimePre = 2,
}

pub struct X3DHClient {
    connection: tokio_rusqlite::Connection,
    ik: SigningKey,
}

fn create_key_table(connection: &Connection) -> rusqlite::Result<()> {
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "normal")?;
    connection.pragma_update(None, "foreign_keys", "on")?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS keys (
             public_key BLOB PRIMARY KEY,
             private_key BLOB NOT NULL,
             key_type INTEGER NOT NULL,
             creation_time INTEGER NOT NULL
         )",
        (),
    )?;
    Ok(())
}

fn insert_identity_key(identity_key: &SigningKey, connection: &Connection) -> rusqlite::Result<()> {
    connection.execute("INSERT INTO keys (public_key, private_key, key_type, creation_time) VALUES (?1, ?2, ?3, ?4)", params![
            VerifyingKey::from(identity_key).to_bytes(),
            identity_key.to_bytes(),
            KeyType::Identity as u32,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
    ])?;
    Ok(())
}

fn load_identity_key(connection: &Connection) -> rusqlite::Result<Option<SigningKey>> {
    let key: Option<[u8; 32]> = match connection.query_row(
        "SELECT private_key FROM keys WHERE key_type = ?1",
        params![KeyType::Identity as u32],
        |row| row.get(0),
    ) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }?;
    Ok(key.map(SigningKey::from))
}

fn load_pre_key(connection: &Connection) -> rusqlite::Result<Option<X25519StaticSecret>> {
    let key: Option<[u8; 32]> = match connection.query_row(
        "SELECT private_key FROM keys WHERE key_type = ?1",
        params![KeyType::Pre as u32],
        |row| row.get(0),
    ) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }?;
    Ok(key.map(X25519StaticSecret::from))
}

fn insert_pre_keys(
    keys: &[X25519StaticSecret],
    key_type: KeyType,
    connection: &Connection,
) -> rusqlite::Result<()> {
    let mut stmt = connection.prepare(
            "INSERT INTO keys (public_key, private_key, key_type, creation_time) VALUES (?1, ?2, ?3, ?4)")?;
    for key in keys {
        let pre_key = X25519PublicKey::from(key).to_bytes();
        #[allow(deprecated)]
        let pubkey = base64::encode(pre_key);
        info!("Inserting pre key: {pubkey}");
        stmt.execute((
            pre_key,
            key.to_bytes(),
            key_type as u32,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        ))?;
    }
    Ok(())
}

fn lazy_init_identity_key(connection: &Connection) -> rusqlite::Result<SigningKey> {
    if let Some(ik) = load_identity_key(connection)? {
        return Ok(ik);
    }
    info!("Creating initial identity key.");
    let identity_key = SigningKey::generate(&mut OsRng);
    insert_identity_key(&identity_key, connection)?;
    Ok(identity_key)
}

fn lazy_init_pre_key(connection: &Connection) -> rusqlite::Result<()> {
    if load_pre_key(connection)?.is_some() {
        return Ok(());
    }
    info!("Creating initial pre key.");
    insert_pre_keys(&[X25519StaticSecret::random()], KeyType::Pre, connection)?;
    Ok(())
}

#[allow(dead_code)]
fn opk_count(connection: &Connection) -> rusqlite::Result<u32> {
    connection.query_row(
        "SELECT COUNT(*) FROM keys WHERE key_type = ?1",
        params![KeyType::OneTimePre as u32],
        |row| row.get(0),
    )
}

impl X3DHClient {
    pub async fn new(connection: tokio_rusqlite::Connection) -> ClientResult<X3DHClient> {
        let ik = connection
            .call(|connection| {
                create_key_table(connection)?;
                lazy_init_pre_key(connection)?;
                Ok(lazy_init_identity_key(connection)?)
            })
            .await
            .map_err(ClientError::TokioSqlite)?;

        let sqlite_client = X3DHClient { connection, ik };
        Ok(sqlite_client)
    }
}

impl X3DHClient {
    pub async fn fetch_wipe_opk(
        &self,
        one_time_prekey: X25519PublicKey,
    ) -> ClientResult<X25519StaticSecret> {
        #[allow(deprecated)]
        let pubkey = base64::encode(one_time_prekey.to_bytes());
        info!("Attempting to consume one time pre key '{pubkey}'",);
        let key: [u8; 32] = self
            .connection
            .call(move |connection| {
                Ok(connection.query_row(
                    "DELETE from keys WHERE public_key=?1 RETURNING private_key",
                    params![one_time_prekey.to_bytes()],
                    |row| row.get(0),
                )?)
            })
            .await
            .map_err(|_| ClientError::WipeOpk(pubkey))?;
        Ok(X25519StaticSecret::from(key))
    }

    pub fn get_ik(&self) -> SigningKey {
        self.ik.clone()
    }

    pub async fn get_pre_key(&self, pre_key: X25519PublicKey) -> ClientResult<X25519StaticSecret> {
        #[allow(deprecated)]
        let pubkey = base64::encode(pre_key.to_bytes());
        info!("Loading pre key: {pubkey}");
        let key: [u8; 32] = self.connection.call(move |connection| {
            Ok(connection.query_row("SELECT private_key FROM keys WHERE public_key = ?1 ORDER BY creation_time DESC LIMIT 1",
                params![pre_key.to_bytes()],
                |row| row.get(0))?)
        }).await?;
        Ok(X25519StaticSecret::from(key))
    }

    pub async fn get_spk(&self) -> ClientResult<SignedPreKey> {
        let ik = self.ik.clone();
        self.connection
            .call(move |connection| {
                let pre_key = load_pre_key(connection)?.unwrap();
                #[allow(deprecated)]
                let pubkey = base64::encode(pre_key.to_bytes());
                info!("Signing pre key: {pubkey}");
                Ok(SignedPreKey {
                    pre_key: X25519PublicKey::from(&pre_key),
                    signature: sign_bundle(
                        &ik,
                        &[(pre_key.clone(), X25519PublicKey::from(&pre_key))],
                    ),
                })
            })
            .await
            .map_err(ClientError::TokioSqlite)
    }

    pub async fn create_opks(&self, num_keys: u32) -> ClientResult<SignedPreKeys> {
        if num_keys != 0 {
            info!("Creating {num_keys} one time pre keys!");
        }
        let opks = create_prekey_bundle(&self.ik, num_keys);
        let pre_keys = opks.bundle.iter().map(|(_, _pub)| *_pub).collect();
        let persisted_pre_keys: Vec<X25519StaticSecret> =
            opks.bundle.into_iter().map(|opk| opk.0).collect();
        self.connection
            .call(move |connection| {
                Ok(insert_pre_keys(
                    &persisted_pre_keys,
                    KeyType::OneTimePre,
                    connection,
                )?)
            })
            .await?;

        Ok(SignedPreKeys {
            pre_keys,
            signature: opks.signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::client::*;
    use anyhow::anyhow;
    use anyhow::Result;
    use rusqlite::Connection;

    #[test]
    fn load_identity_key_not_found() -> Result<()> {
        let connection = Connection::open_in_memory()?;
        create_key_table(&connection)?;
        assert_eq!(load_identity_key(&connection)?, None);
        Ok(())
    }

    #[test]
    fn load_pre_key_not_found() -> Result<()> {
        let connection = Connection::open_in_memory()?;
        create_key_table(&connection)?;
        assert!(load_pre_key(&connection)?.is_none());
        Ok(())
    }

    #[test]
    fn init_identity_key() -> Result<()> {
        let connection = Connection::open_in_memory()?;
        create_key_table(&connection)?;
        lazy_init_identity_key(&connection)?;

        let key = load_identity_key(&connection)?;
        assert!(key.is_some());

        lazy_init_identity_key(&connection)?;
        assert_eq!(load_identity_key(&connection)?, key);
        Ok(())
    }

    #[test]
    fn init_pre_key() -> Result<()> {
        let connection = Connection::open_in_memory()?;
        create_key_table(&connection)?;
        lazy_init_pre_key(&connection)?;

        let key = load_pre_key(&connection)?;
        assert!(key.is_some());

        assert_eq!(
            load_pre_key(&connection)?
                .ok_or(anyhow!("no key"))?
                .to_bytes(),
            key.ok_or(anyhow!("no key"))?.to_bytes()
        );
        Ok(())
    }

    #[tokio::test]
    async fn fetch_pre_key() -> Result<()> {
        let connection = tokio_rusqlite::Connection::open_in_memory().await?;
        let client = X3DHClient::new(connection).await?;
        let spk = client.get_spk().await?;
        client.get_pre_key(spk.pre_key).await?;
        Ok(())
    }

    #[tokio::test]
    async fn client_stuff() -> Result<()> {
        let conn = tokio_rusqlite::Connection::open_in_memory().await?;
        let client = X3DHClient::new(conn).await?;
        let _spk = client.get_spk().await?;

        Ok(())
    }
}
