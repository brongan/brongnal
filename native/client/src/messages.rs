use std::time::{SystemTime, UNIX_EPOCH};

use proto::ApplicationMessage;
use rusqlite::{params, Connection};

/*
 *@UseRowClass(MessageModel)
class Messages extends Table {
  IntColumn get id => integer().autoIncrement()();
  TextColumn get sender => text()();
  TextColumn get receiver => text()();
  TextColumn get message => text()();
  DateTimeColumn get time => dateTime()();
  IntColumn get state => intEnum<MessageState>()();

  @override
  Set<Column<Object>>? get primaryKey => {id};
}
*/

#[derive(Clone, Copy, strum_macros::Display)]
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

// A message is sent by someone.
// possibly in a group
// the sender could be self or another user
// a recipient can either self or another user
// a message could be a part of a group chat
// a message can be sending, sent, delivered, or read
// contents can be a message, image, etc
// The primary key of a message is its hash?? How else to reference other messages with read
// receipts.
fn create_tables(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute(
        "CREATE TABLE IF NOT EXISTS users (username TEXT PRIMARY KEY)",
        (),
    )?;
    connection.execute(
        "CREATE TABLE IF NOT EXISTS messages (
        sender TEXT NOT NULL,
        receiver TEXT NOT NULL,
        text TEXT NOT NULL,
        time INTEGER NOT NULL,
        state INTEGER NOT NULL,
        group INTEGER,
        FOREIGN KEY(sender) REFERENCES users(username),
        FOREIGN KEY(receiver) REFERENCES users(username),
     )",
        (),
    )?;
    Ok(())
}

type MessageId = u32;

fn add_user(connection: &Connection, username: &str) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT OR IGNORE INTO users (username) VALUES ($1)",
        params![username],
    )?;
    Ok(())
}

fn receive(
    connection: &Connection,
    our_username: &str,
    message: ApplicationMessage,
) -> rusqlite::Result<()> {
    connection.execute("INSERT OR IGNORE INTO messages (sender, receiver, contents, time, state) VALUES ($1, $2, $3, $4, $5)", 
        params![
            message.sender, our_username, message.text, time_now(), State::Delivered as u8
        ]
    )?;
    Ok(())
}

fn send(
    connection: &Connection,
    sender: String,
    receiver: String,
    message: String,
) -> rusqlite::Result<()> {
    connection.execute("INSERT OR IGNORE INTO messages (sender, receiver, contents, time, state) VALUES ($1, $2, $3, $4, $5)", 
        params![
            sender, receiver, message, time_now(), State::Sending as u8
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
