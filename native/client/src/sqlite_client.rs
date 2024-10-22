use crate::{ClientError, ClientResult, X3DHClient};
use chacha20poly1305::aead::OsRng;
use ed25519_dalek::{SigningKey, VerifyingKey};
use protocol::bundle::{create_prekey_bundle, sign_bundle};
use protocol::x3dh;
use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};
use x3dh::{SignedPreKey, SignedPreKeys};

#[derive(Clone, Copy, strum_macros::Display)]
#[repr(u32)]
enum KeyType {
    IdentityKey = 0,
    PreKey = 1,
    OneTimePreKey = 2,
}

pub struct SqliteClient {
    connection: Connection,
}

fn create_key_table(connection: &Connection) -> ClientResult<()> {
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

fn insert_identity_key(identity_key: &SigningKey, connection: &Connection) -> ClientResult<()> {
    connection.execute("INSERT INTO keys (public_key, private_key, key_type, creation_time) VALUES (?1, ?2, ?3, ?4)", params![
            VerifyingKey::from(identity_key).to_bytes(),
            identity_key.to_bytes(),
            KeyType::IdentityKey as u32,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() ]).map_err(|e| ClientError::InsertIdentityKey(e))?;
    Ok(())
}

fn load_identity_key(connection: &Connection) -> ClientResult<Option<SigningKey>> {
    let key: Option<[u8; 32]> = match connection.query_row(
        "SELECT private_key FROM keys WHERE key_type = ?1",
        params![KeyType::IdentityKey as u32],
        |row| row.get(0),
    ) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(ClientError::GetIdentityKey(e)),
    }?;
    Ok(key.map(|key| SigningKey::from(key)))
}

fn load_pre_key(connection: &Connection) -> ClientResult<Option<X25519StaticSecret>> {
    let key: Option<[u8; 32]> = match connection.query_row(
        "SELECT private_key FROM keys WHERE key_type = ?1 ORDER BY creation_time DESC LIMIT 1",
        params![KeyType::PreKey as u32],
        |row| row.get(0),
    ) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(ClientError::GetIdentityKey(e)),
    }?;
    Ok(key.map(|key| X25519StaticSecret::from(key)))
}

#[allow(deprecated)]
fn insert_pre_keys(
    keys: &[X25519StaticSecret],
    key_type: KeyType,
    connection: &Connection,
) -> ClientResult<()> {
    let mut stmt = connection.prepare(
            "INSERT INTO keys (public_key, private_key, key_type, creation_time) VALUES (?1, ?2, ?3, ?4)")?;
    for key in keys {
        let pre_key = X25519PublicKey::from(key).to_bytes();
        eprintln!("Inserting pre key: {}", base64::encode(pre_key));
        stmt.execute((
            pre_key,
            key.to_bytes(),
            key_type as u32,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        ))
        .map_err(|e| ClientError::InsertPreKey(e))?;
    }
    Ok(())
}

fn lazy_init_identity_key(connection: &Connection) -> ClientResult<()> {
    if let Some(_) = load_identity_key(&connection)? {
        return Ok(());
    }
    eprintln!("Creating initial identity key.");
    let identity_key = SigningKey::generate(&mut OsRng);
    insert_identity_key(&identity_key, &connection)?;
    Ok(())
}

fn lazy_init_pre_key(connection: &Connection) -> ClientResult<()> {
    if let Some(_) = load_pre_key(&connection)? {
        return Ok(());
    }
    eprintln!("Creating initial pre key.");
    insert_pre_keys(
        &[X25519StaticSecret::random()],
        KeyType::PreKey,
        &connection,
    )?;
    Ok(())
}

#[allow(dead_code)]
fn opk_count(connection: &Connection) -> ClientResult<u32> {
    connection
        .query_row(
            "SELECT COUNT(*) FROM keys WHERE key_type = ?1",
            params![KeyType::OneTimePreKey as u32],
            |row| Ok(row.get(0)?),
        )
        .map_err(|e| ClientError::GetPreKey(e))
}

impl SqliteClient {
    pub fn new(connection: Connection) -> ClientResult<SqliteClient> {
        create_key_table(&connection)?;
        lazy_init_identity_key(&connection)?;
        lazy_init_pre_key(&connection)?;

        let sqlite_client = SqliteClient { connection };
        Ok(sqlite_client)
    }
}

#[allow(deprecated)]
impl X3DHClient for SqliteClient {
    fn fetch_wipe_opk(
        &mut self,
        one_time_prekey: &X25519PublicKey,
    ) -> ClientResult<X25519StaticSecret> {
        eprintln!(
            "Using one time pre key '{}'",
            base64::encode(one_time_prekey.to_bytes())
        );
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
        eprintln!("Loading identity key.");
        Ok(load_identity_key(&self.connection)?.unwrap())
    }

    fn get_pre_key(&self, pre_key: &X25519PublicKey) -> ClientResult<X25519StaticSecret> {
        eprintln!("Loading pre key: {}", base64::encode(pre_key.to_bytes()));
        let key: [u8; 32] = self.connection.query_row("SELECT private_key FROM keys WHERE public_key = ?1 ORDER BY creation_time DESC LIMIT 1", params![pre_key.to_bytes()], |row| Ok(row.get(0)?)).map_err(|e| ClientError::GetPreKey(e))?;
        Ok(X25519StaticSecret::from(key))
    }

    fn get_spk(&self) -> ClientResult<SignedPreKey> {
        let pre_key = load_pre_key(&self.connection)?.unwrap();
        eprintln!("Signing pre key: {}", base64::encode(pre_key.to_bytes()));
        Ok(SignedPreKey {
            pre_key: X25519PublicKey::from(&pre_key),
            signature: sign_bundle(
                &load_identity_key(&self.connection)?.unwrap(),
                &[(pre_key.clone(), X25519PublicKey::from(&pre_key))],
            ),
        })
    }

    fn create_opks(&mut self, num_keys: u32) -> ClientResult<SignedPreKeys> {
        eprintln!("Creating {num_keys} one time pre keys!");
        let opks = create_prekey_bundle(&load_identity_key(&self.connection)?.unwrap(), num_keys);
        let pre_keys = opks.bundle.iter().map(|(_, _pub)| *_pub).collect();
        let persisted_pre_keys: Vec<X25519StaticSecret> =
            opks.bundle.into_iter().map(|opk| opk.0).collect();
        insert_pre_keys(
            &persisted_pre_keys,
            KeyType::OneTimePreKey,
            &self.connection,
        )?;
        Ok(SignedPreKeys {
            pre_keys,
            signature: opks.signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::sqlite_client::*;
    use anyhow::anyhow;
    use anyhow::Result;

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
        assert_eq!(load_pre_key(&connection)?.is_none(), true);
        Ok(())
    }

    #[test]
    fn init_identity_key() -> Result<()> {
        let connection = Connection::open_in_memory()?;
        create_key_table(&connection)?;
        lazy_init_identity_key(&connection)?;

        let key = load_identity_key(&connection)?;
        assert_eq!(key.is_some(), true);

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
        assert_eq!(key.is_some(), true);

        assert_eq!(
            load_pre_key(&connection)?
                .ok_or(anyhow!("no key"))?
                .to_bytes(),
            key.ok_or(anyhow!("no key"))?.to_bytes()
        );
        Ok(())
    }

    #[test]
    fn fetch_pre_key() -> Result<()> {
        let connection = Connection::open_in_memory()?;
        create_key_table(&connection)?;
        lazy_init_pre_key(&connection)?;
        let key1 = load_pre_key(&connection)?;
        assert_eq!(key1.is_some(), true);
        let key1 = key1.unwrap();

        let client = SqliteClient::new(connection)?;
        assert_eq!(
            client
                .get_pre_key(&X25519PublicKey::from(&key1))?
                .to_bytes(),
            key1.to_bytes()
        );

        Ok(())
    }

    #[test]
    fn client_stuff() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        let client = SqliteClient::new(conn)?;
        let _ik = client.get_ik()?;
        let _spk = client.get_spk()?;

        Ok(())
    }
}
