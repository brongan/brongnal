use proto::ApplicationMessage;
use rinf::{DartSignal, RustSignal, SignalPiece};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, strum_macros::Display, Serialize)]
#[repr(u8)]
enum State {
    Sending,
    Sent,
    Delivered,
}

fn time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Serialize, RustSignal)]
struct PersistedConversations {
    vec: PersistedMessage,
}

#[derive(Serialize, RustSignal, SignalPiece)]
struct PersistedMessage {
    sender: String,
    receiver: String,
    creation_time: SystemTime,
    state: State,
    text: String,
}

fn create_tables(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute(
        "CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY,
            created_at INTEGER NOT NULL,
            name TEXT,
            profile_pic BLOB,
            )",
        (),
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS messages (
        message_id INTEGER PRIMARY KEY,
        sender TEXT NOT NULL,
        receiver TEXT NOT NULL,
        creation_time INTEGER NOT NULL,
        state INTEGER NOT NULL,
        text TEXT,
        FOREIGN KEY(sender) REFERENCES users(username),
        FOREIGN KEY(receiver) REFERENCES users(username),
     )",
        (),
    )?;

    Ok(())
}

type MessageId = i64;
pub fn add_user(
    connection: &Connection,
    username: &str,
    name: Option<String>,
    profile_pic: Vec<u8>,
) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT OR IGNORE INTO users (username, created_at, name, profile_pic) VALUES ($1, $2)",
        params![username, time_now(), name, profile_pic],
    )?;
    Ok(())
}

pub fn receive(
    connection: &Connection,
    our_username: &str,
    message: ApplicationMessage,
) -> rusqlite::Result<()> {
    connection.execute("INSERT OR IGNORE INTO messages (sender, receiver, creation_time, state, text) VALUES ($1, $2, $3, $4, $5)", 
        params![
            message.sender, our_username, time_now(), State::Delivered as u8, message.text
        ]
    )?;
    Ok(())
}

pub fn send(
    connection: &Connection,
    sender: String,
    receiver: String,
    message: String,
) -> rusqlite::Result<()> {
    connection.execute("INSERT OR IGNORE INTO messages (sender, receiver, creation_time, state, text) VALUES ($1, $2, $3, $4, $5)", 
        params![
            sender, receiver, time_now(), State::Sending as u8, message
        ]
    )?;
    Ok(())
}

fn update_state(connection: &Connection, id: MessageId, state: State) -> rusqlite::Result<()> {
    // Returns the first row updated so that a missing message results in an error.
    let _: u8 = connection.query_row(
        "UPDATE messages SET state = ?2 WHERE id = ?1 RETURNING state",
        params![id, state as u8],
        |row| row.get(0),
    )?;
    Ok(())
}

fn get_conversations(
    connection: &Connection,
    peer: String,
) -> rusqlite::Result<PersistedConversations> {
    let mut stmt = connection
        .prepare("SELECT (sender, receiver, creation_time, state, text) FROM messages")?;
    for 
}
