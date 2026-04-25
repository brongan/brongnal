use ed25519_dalek::VerifyingKey;
use prost::Message;
use proto::gossamer::SignedMessage;
use rusqlite::params;
use std::collections::HashMap;
use tokio_rusqlite::{Connection, Result};
use tracing::{info, instrument};

#[derive(Clone)]
pub struct GossamerStorage(Connection);

impl GossamerStorage {
    pub async fn new(connection: Connection) -> Result<Self> {
        info!("Creating SQlite Tables for Gossamer Service.");
        connection
            .call(|connection| {
                connection.pragma_update(None, "journal_mode", "WAL")?;
                connection.pragma_update(None, "synchronous", "normal")?;
                connection.pragma_update(None, "foreign_keys", "on")?;
                connection.execute_batch(
                    "
                    BEGIN;
                    CREATE TABLE IF NOT EXISTS gossamer_providers (
                        provider BLOB PRIMARY KEY
                    );
                    CREATE TABLE IF NOT EXISTS gossamer_keys (
                        public_key BLOB PRIMARY KEY,
                        provider BLOB NOT NULL,
                        FOREIGN KEY(provider) REFERENCES gossamer_providers(provider) ON DELETE CASCADE
                    );
                    CREATE TABLE IF NOT EXISTS gossamer_messages (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        provider BLOB NOT NULL,
                        signed_message BLOB NOT NULL,
                        FOREIGN KEY(provider) REFERENCES gossamer_providers(provider) ON DELETE CASCADE
                    );
                    COMMIT;",
                )?;
                Ok(())
            })
            .await?;
        Ok(GossamerStorage(connection))
    }

    /// Atomically registers a provider (if it doesn't already exist) and associates a public key with it.
    ///
    /// **Authorization Requirements**:
    /// - For existing providers, the request must be authenticated by any currently authorized
    ///   key to permit the addition of a new key.
    /// - For new providers, the entity must sign the request with the new key itself to
    ///   prove possession and claim the name.
    ///
    /// Returns `true` if the key was successfully added, or `false` if that specific key is already
    /// registered for the provider. Note that a single provider can have multiple associated keys.
    #[instrument(skip(self))]
    pub async fn append_key(&self, provider: Vec<u8>, public_key: VerifyingKey) -> Result<bool> {
        self.0
            .call(move |connection| {
                let tx = connection.transaction()?;
                tx.execute(
                    "INSERT OR IGNORE INTO gossamer_providers (provider) VALUES (?1)",
                    params![provider],
                )?;
                let affected = tx.execute(
                    "INSERT OR IGNORE INTO gossamer_keys (public_key, provider) VALUES (?1, ?2)",
                    params![public_key.as_bytes(), provider],
                )?;
                tx.commit()?;
                Ok(affected == 1)
            })
            .await
    }

    /// Returns `true` if the specific public key is currently associated with the given provider hash.
    #[instrument(skip(self))]
    pub async fn has_key(&self, provider: Vec<u8>, public_key: VerifyingKey) -> Result<bool> {
        self.0
            .call(move |connection| {
                let mut statement = connection.prepare(
                    "SELECT 1 FROM gossamer_keys WHERE provider = ?1 AND public_key = ?2",
                )?;
                let exists = statement.exists(params![provider, public_key.as_bytes()])?;
                Ok(exists)
            })
            .await
    }

    /// Returns the provider hash associated with a given public key, if it exists.
    #[instrument(skip(self))]
    pub async fn get_key_provider(&self, public_key: VerifyingKey) -> Result<Option<Vec<u8>>> {
        self.0
            .call(move |connection| {
                let mut statement = connection
                    .prepare("SELECT provider FROM gossamer_keys WHERE public_key = ?1")?;
                let mut rows = statement.query([public_key.as_bytes()])?;
                if let Some(row) = rows.next()? {
                    let provider: Vec<u8> = row.get(0)?;
                    Ok(Some(provider))
                } else {
                    Ok(None)
                }
            })
            .await
    }

    /// Returns `true` if the provider hash has already been registered in the system.
    /// Once registered, providers are considered permanent and cannot be deleted.
    #[instrument(skip(self))]
    pub async fn has_provider(&self, provider: Vec<u8>) -> Result<bool> {
        self.0
            .call(move |connection| {
                let mut statement =
                    connection.prepare("SELECT 1 FROM gossamer_providers WHERE provider = ?1")?;
                let exists = statement.exists(params![provider])?;
                Ok(exists)
            })
            .await
    }

    /// Removes a specific public key association for a provider.
    ///
    /// Returns `true` if the key was found and removed, or `false` if no such key exists for the provider.
    /// Revoking the last key of a provider does not delete the provider identity itself.
    #[instrument(skip(self))]
    pub async fn revoke_key(&self, provider: Vec<u8>, public_key: VerifyingKey) -> Result<bool> {
        self.0
            .call(move |connection| {
                let affected = connection.execute(
                    "DELETE FROM gossamer_keys WHERE provider = ?1 AND public_key = ?2",
                    params![provider, public_key.as_bytes()],
                )?;
                Ok(affected == 1)
            })
            .await
    }

    /// Appends a signed message to the audit log, associated with a specific provider.
    ///
    /// The `provider` hash must correspond to the identity that authored and signed the message.
    /// **Important**: Callers MUST validate the message signature and ensure the signing key is
    /// authorized for this provider before persisting the message.
    ///
    /// This method enforces a database foreign key constraint: the provider must already exist
    /// in the `gossamer_providers` table.
    #[instrument(skip(self, message))]
    pub async fn append_message(&self, provider: Vec<u8>, message: SignedMessage) -> Result<()> {
        let contents = message.encode_to_vec();
        self.0
            .call(move |connection| {
                connection.execute(
                    "INSERT INTO gossamer_messages (provider, signed_message) VALUES (?1, ?2)",
                    params![provider, contents],
                )?;
                Ok(())
            })
            .await
    }

    /// Retrieves the entire ledger of all providers and their respective identity keys, grouped by provider.
    #[instrument(skip(self))]
    pub async fn get_ledger(&self) -> Result<HashMap<Vec<u8>, Vec<VerifyingKey>>> {
        self.0
            .call(|connection| {
                let mut statement =
                    connection.prepare("SELECT provider, public_key FROM gossamer_keys")?;
                let rows = statement.query_map([], |row| {
                    let provider: Vec<u8> = row.get(0)?;
                    let key_bytes: Vec<u8> = row.get(1)?;
                    let key = VerifyingKey::try_from(key_bytes.as_slice()).map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            1,
                            "invalid ed25519 key".into(),
                            rusqlite::types::Type::Blob,
                        )
                    })?;
                    Ok((provider, key))
                })?;

                let mut ledger: HashMap<Vec<u8>, Vec<VerifyingKey>> = HashMap::new();
                for row in rows {
                    let (provider, key) = row?;
                    ledger.entry(provider).or_default().push(key);
                }
                Ok(ledger)
            })
            .await
    }
}

#[cfg(test)]
#[path = "persistence_tests.rs"]
mod tests;
