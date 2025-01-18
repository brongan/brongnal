use crate::{ClientError, ClientResult};
use chacha20poly1305::aead::OsRng;
use ed25519_dalek::{SigningKey, VerifyingKey};
use libsql::{params, Connection};
use protocol::bundle::{create_prekey_bundle, sign_bundle};
use protocol::x3dh;
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
    connection: libsql::Connection,
}

async fn create_key_table(connection: &Connection) -> libsql::Result<()> {
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS keys (
             public_key BLOB PRIMARY KEY,
             private_key BLOB NOT NULL,
             key_type INTEGER NOT NULL,
             creation_time INTEGER NOT NULL
         )",
            (),
        )
        .await?;
    Ok(())
}

async fn insert_identity_key(
    identity_key: &SigningKey,
    connection: &Connection,
) -> libsql::Result<()> {
    connection.execute("INSERT INTO keys (public_key, private_key, key_type, creation_time) VALUES (?1, ?2, ?3, ?4)", params![
            VerifyingKey::from(identity_key).to_bytes(),
            identity_key.to_bytes(),
            KeyType::Identity as u32,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
    ]).await?;
    Ok(())
}

async fn load_identity_key(connection: &Connection) -> libsql::Result<Option<SigningKey>> {
    let mut rows = connection
        .query(
            "SELECT private_key FROM keys WHERE key_type = ?1",
            params![KeyType::Identity as u32],
        )
        .await?;
    Ok(rows
        .next()
        .await
        .unwrap()
        .map(|row| SigningKey::from(row.get::<[u8; 32]>(0).unwrap())))
}

async fn load_pre_key(connection: &Connection) -> libsql::Result<Option<X25519StaticSecret>> {
    let mut rows = connection
        .query(
            "SELECT private_key FROM keys WHERE key_type = ?1",
            params![KeyType::Pre as u32],
        )
        .await?;
    Ok(rows
        .next()
        .await
        .unwrap()
        .map(|row| X25519StaticSecret::from(row.get::<[u8; 32]>(0).unwrap())))
}

async fn insert_pre_keys(
    keys: &[X25519StaticSecret],
    key_type: KeyType,
    connection: &Connection,
) -> libsql::Result<()> {
    let mut stmt = connection.prepare(
            "INSERT INTO keys (public_key, private_key, key_type, creation_time) VALUES (?1, ?2, ?3, ?4)").await?;
    for key in keys {
        let pre_key = X25519PublicKey::from(key).to_bytes();
        #[allow(deprecated)]
        let pubkey = base64::encode(pre_key);
        info!("Inserting pre key: {pubkey} of type: {key_type}");
        stmt.execute(params![
            pre_key,
            key.to_bytes(),
            key_type as u32,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        ])
        .await?;
    }
    Ok(())
}

async fn lazy_init_identity_key(connection: &Connection) -> libsql::Result<()> {
    if load_identity_key(connection).await?.is_some() {
        return Ok(());
    }
    info!("Creating initial identity key.");
    let identity_key = SigningKey::generate(&mut OsRng);
    insert_identity_key(&identity_key, connection).await?;
    Ok(())
}

async fn lazy_init_pre_key(connection: &Connection) -> libsql::Result<()> {
    if load_pre_key(connection).await?.is_some() {
        return Ok(());
    }
    info!("Creating initial pre key.");
    insert_pre_keys(&[X25519StaticSecret::random()], KeyType::Pre, connection).await?;
    Ok(())
}

#[allow(dead_code)]
async fn opk_count(connection: &Connection) -> libsql::Result<u32> {
    let mut rows = connection
        .query(
            "SELECT COUNT(*) FROM keys WHERE key_type = ?1",
            params![KeyType::OneTimePre as u32],
        )
        .await?;
    let row = rows.next().await?.unwrap();
    Ok(row.get(0)?)
}

impl X3DHClient {
    pub async fn new(connection: libsql::Connection) -> ClientResult<X3DHClient> {
        create_key_table(&connection).await?;
        lazy_init_identity_key(&connection).await?;
        lazy_init_pre_key(&connection).await?;
        let sqlite_client = X3DHClient { connection };
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
        info!("Using one time pre key '{pubkey}'",);
        let mut rows = self
            .connection
            .query(
                "DELETE from keys WHERE public_key=?1 RETURNING private_key",
                params![one_time_prekey.to_bytes()],
            )
            .await?;
        let row = rows
            .next()
            .await
            .unwrap()
            .ok_or(ClientError::WipeOpk(pubkey))?;
        let key: [u8; 32] = row.get(0).unwrap();
        Ok(X25519StaticSecret::from(key))
    }

    pub async fn get_ik(&self) -> ClientResult<SigningKey> {
        info!("Loading identity key.");
        load_identity_key(&self.connection)
            .await?
            .ok_or(ClientError::GetIdentityKey)
    }

    pub async fn get_pre_key(&self, pre_key: X25519PublicKey) -> ClientResult<X25519StaticSecret> {
        #[allow(deprecated)]
        let pubkey = base64::encode(pre_key.to_bytes());
        info!("Loading pre key: {pubkey}");
        let mut rows = self.connection.query("SELECT private_key FROM keys WHERE public_key = ?1 ORDER BY creation_time DESC LIMIT 1", params![pre_key.to_bytes()]).await?;
        let row = rows.next().await.unwrap().unwrap();
        let key: [u8; 32] = row.get(0).unwrap();
        Ok(X25519StaticSecret::from(key))
    }

    pub async fn get_spk(&self) -> ClientResult<SignedPreKey> {
        let pre_key = load_pre_key(&self.connection).await?.unwrap();
        #[allow(deprecated)]
        let pubkey = base64::encode(pre_key.to_bytes());
        info!("Signing pre key: {pubkey}");
        Ok(SignedPreKey {
            pre_key: X25519PublicKey::from(&pre_key),
            signature: sign_bundle(
                &load_identity_key(&self.connection).await?.unwrap(),
                &[(pre_key.clone(), X25519PublicKey::from(&pre_key))],
            ),
        })
        .map_err(ClientError::Sqlite)
    }

    pub async fn create_opks(&self, num_keys: u32) -> ClientResult<SignedPreKeys> {
        info!("Creating {num_keys} one time pre keys!");
        let ik: SigningKey = load_identity_key(&self.connection)
            .await?
            .expect("has identity key");
        let opks = create_prekey_bundle(&ik, num_keys);
        let pre_keys = opks.bundle.iter().map(|(_, _pub)| *_pub).collect();
        let persisted_pre_keys: Vec<X25519StaticSecret> =
            opks.bundle.into_iter().map(|opk| opk.0).collect();
        insert_pre_keys(&persisted_pre_keys, KeyType::OneTimePre, &self.connection).await?;

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
    use libsql::Builder;

    #[tokio::test]
    async fn load_identity_key_not_found() -> Result<()> {
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let connection = db.connect().unwrap();
        create_key_table(&connection).await?;
        assert_eq!(load_identity_key(&connection).await?, None);
        Ok(())
    }

    #[tokio::test]
    async fn load_pre_key_not_found() -> Result<()> {
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let connection = db.connect().unwrap();
        create_key_table(&connection).await?;
        assert!(load_pre_key(&connection).await?.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn init_identity_key() -> Result<()> {
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let connection = db.connect().unwrap();
        create_key_table(&connection).await?;
        lazy_init_identity_key(&connection).await?;

        let key = load_identity_key(&connection).await?;
        assert!(key.is_some());

        lazy_init_identity_key(&connection).await?;
        assert_eq!(load_identity_key(&connection).await?, key);
        Ok(())
    }

    #[tokio::test]
    async fn init_pre_key() -> Result<()> {
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let connection = db.connect().unwrap();
        create_key_table(&connection).await?;
        lazy_init_pre_key(&connection).await?;

        let key = load_pre_key(&connection).await?;
        assert!(key.is_some());

        assert_eq!(
            load_pre_key(&connection)
                .await?
                .ok_or(anyhow!("no key"))?
                .to_bytes(),
            key.ok_or(anyhow!("no key"))?.to_bytes()
        );
        Ok(())
    }

    #[tokio::test]
    async fn fetch_pre_key() -> Result<()> {
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let connection = db.connect().unwrap();
        let client = X3DHClient::new(connection).await?;
        let spk = client.get_spk().await?;
        client.get_pre_key(spk.pre_key).await?;
        Ok(())
    }

    #[tokio::test]
    async fn client_stuff() -> Result<()> {
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let client = X3DHClient::new(conn).await?;
        let _ik = client.get_ik().await?;
        let _spk = client.get_spk().await?;

        Ok(())
    }
}
