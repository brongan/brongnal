use crate::{ClientError, ClientResult};
use base64::{engine::general_purpose::STANDARD as base64, Engine as _};
use chacha20poly1305::aead::OsRng;
use chrono::DateTime;
use ed25519_dalek::{SigningKey, VerifyingKey};
use proto::ApplicationMessage;
use protocol::bundle::{create_prekey_bundle, sign_bundle};
use protocol::x3dh;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use strum_macros::FromRepr;
use tracing::info;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};
use x3dh::{SignedPreKey, SignedPreKeys};

fn time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

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

fn create_tables(connection: &Connection) -> rusqlite::Result<()> {
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "normal")?;
    connection.pragma_update(None, "foreign_keys", "on")?;

    connection.execute_batch(
        "
                    BEGIN;
                    CREATE TABLE IF NOT EXISTS keys (
                        public_key BLOB PRIMARY KEY,
                        private_key BLOB NOT NULL,
                        key_type INTEGER NOT NULL,
                        creation_time INTEGER NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS users (
                        username TEXT PRIMARY KEY,
                        created_at INTEGER NOT NULL,
                        name TEXT,
                        profile_pic BLOB
                    );
                    CREATE TABLE IF NOT EXISTS messages (
                        sender TEXT NOT NULL,
                        receiver TEXT NOT NULL,
                        creation_time INTEGER NOT NULL,
                        state INTEGER NOT NULL,
                        text TEXT,
                        FOREIGN KEY(sender) REFERENCES users(username),
                        FOREIGN KEY(receiver) REFERENCES users(username)
                    );
                    COMMIT;",
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

#[derive(Clone, Copy, strum_macros::Display, Serialize, FromRepr)]
#[repr(u8)]
pub enum MessageState {
    Sending,
    Sent,
    Delivered,
    Read,
}

#[derive(Serialize)]
pub struct MessagesModel {
    pub messages: Vec<MessageModel>,
}

#[derive(Serialize)]
pub struct MessageModel {
    pub sender: String,
    pub receiver: String,
    pub db_recv_time: i64,
    pub state: MessageState,
    pub text: String,
}

impl std::fmt::Display for MessageModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &MessageModel {
            ref sender,
            ref receiver,
            db_recv_time,
            state,
            ref text,
        } = self;
        let db_recv_time = DateTime::from_timestamp(db_recv_time as i64, 0).unwrap();
        write!(
            f,
            "From: {sender} To: {receiver} at {db_recv_time} with {state}\n{text}"
        )?;
        Ok(())
    }
}

type MessageId = i64;

pub fn add_user(
    connection: &Connection,
    username: &str,
    name: Option<String>,
    profile_pic: Option<Vec<u8>>,
) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT OR IGNORE INTO users (username, created_at, name, profile_pic) VALUES ($1, $2, $3, $4)",
        params![username, time_now(), name, profile_pic],
    )?;
    Ok(())
}

pub fn set_received(
    connection: &Connection,
    our_username: &str,
    message: ApplicationMessage,
) -> rusqlite::Result<()> {
    connection.execute("INSERT OR IGNORE INTO messages (sender, receiver, creation_time, state, text) VALUES ($1, $2, $3, $4, $5)", 
        params![
            message.sender, our_username, time_now(), MessageState::Delivered as u8, message.text
        ]
    )?;
    Ok(())
}

fn persist_state(
    connection: &Connection,
    sender: &str,
    receiver: &str,
    message: &str,
    state: MessageState,
) -> rusqlite::Result<MessageId> {
    Ok(connection.query_row("INSERT OR IGNORE INTO messages (sender, receiver, creation_time, state, text) VALUES ($1, $2, $3, $4, $5) RETURNING rowid", 
        params![
            sender, receiver, time_now(), state as u8, message
        ],
        |row| row.get(0),
    )?)
}

fn update_state(
    connection: &Connection,
    id: MessageId,
    state: MessageState,
) -> rusqlite::Result<()> {
    // Returns the first row updated so that a missing message results in an error.
    let _: u8 = connection.query_row(
        "UPDATE messages SET state = ?2 WHERE rowid = ?1 RETURNING state",
        params![id, state as u8],
        |row| row.get(0),
    )?;
    Ok(())
}

fn get_message(connection: &Connection, id: MessageId) -> rusqlite::Result<MessageModel> {
    connection.query_row(
        "SELECT sender, receiver, creation_time, state, text FROM messages WHERE rowid = ?1",
        params![id],
        |row| {
            Ok(MessageModel {
                sender: row.get(0)?,
                receiver: row.get(1)?,
                db_recv_time: row.get(2)?,
                state: MessageState::from_repr(row.get(3)?).unwrap(),
                text: row.get(4)?,
            })
        },
    )
}

fn get_conversations(connection: &Connection) -> rusqlite::Result<MessagesModel> {
    let mut stmt =
        connection.prepare("SELECT sender, receiver, creation_time, state, text FROM messages")?;
    let mut message_iter = stmt.query_map([], |row| {
        Ok(MessageModel {
            sender: row.get(0)?,
            receiver: row.get(1)?,
            db_recv_time: row.get(2)?,
            state: MessageState::from_repr(row.get(3)?).unwrap(),
            text: row.get(4)?,
        })
    })?;
    Ok(MessagesModel {
        messages: message_iter.try_collect()?,
    })
}

impl X3DHClient {
    pub async fn new(connection: tokio_rusqlite::Connection) -> ClientResult<X3DHClient> {
        let ik = connection
            .call(|connection| {
                create_tables(connection)?;
                lazy_init_pre_key(connection)?;
                Ok(lazy_init_identity_key(connection)?)
            })
            .await
            .map_err(ClientError::TokioSqlite)?;

        let sqlite_client = X3DHClient { connection, ik };
        Ok(sqlite_client)
    }

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
                let pubkey = base64.encode(pre_key.to_bytes());
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

    pub async fn persist_message(
        &self,
        sender: String,
        receiver: String,
        message: String,
        state: MessageState,
    ) -> ClientResult<MessageId> {
        self.connection
            .call(move |connection| {
                add_user(connection, &sender, None, None)?;
                add_user(connection, &receiver, None, None)?;
                Ok(persist_state(
                    connection, &sender, &receiver, &message, state,
                )?)
            })
            .await
            .map_err(ClientError::TokioSqlite)
    }

    pub async fn persist_message_state(
        &self,
        message_id: MessageId,
        state: MessageState,
    ) -> ClientResult<()> {
        self.connection
            .call(move |connection| Ok(update_state(connection, message_id, state)?))
            .await
            .map_err(ClientError::TokioSqlite)
    }

    pub async fn get_message(&self, id: MessageId) -> ClientResult<MessageModel> {
        self.connection
            .call(move |connection| Ok(get_message(connection, id)?))
            .await
            .map_err(ClientError::TokioSqlite)
    }

    pub async fn get_messages(&self) -> ClientResult<MessagesModel> {
        self.connection
            .call(move |connection| Ok(get_conversations(connection)?))
            .await
            .map_err(ClientError::TokioSqlite)
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
        create_tables(&connection)?;
        assert_eq!(load_identity_key(&connection)?, None);
        Ok(())
    }

    #[test]
    fn load_pre_key_not_found() -> Result<()> {
        let connection = Connection::open_in_memory()?;
        create_tables(&connection)?;
        assert!(load_pre_key(&connection)?.is_none());
        Ok(())
    }

    #[test]
    fn init_identity_key() -> Result<()> {
        let connection = Connection::open_in_memory()?;
        create_tables(&connection)?;
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
        create_tables(&connection)?;
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
