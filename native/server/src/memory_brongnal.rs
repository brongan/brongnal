use async_trait::async_trait;
use ed25519_dalek::VerifyingKey;
use proto::service::Message as MessageProto;
use proto::service::SignedPreKey as SignedPreKeyProto;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use tonic::Status;
use x25519_dalek::PublicKey as X25519PublicKey;

use crate::brongnal::CurrentKeys;
use crate::brongnal::Storage;

#[derive(Clone, Debug)]
pub struct MemoryStorage {
    iks: Arc<Mutex<HashMap<String, VerifyingKey>>>,
    spks: Arc<Mutex<HashMap<String, SignedPreKeyProto>>>,
    opks: Arc<Mutex<HashMap<String, Vec<X25519PublicKey>>>>,
    messages: Arc<Mutex<HashMap<String, Vec<MessageProto>>>>,
}

impl Default for MemoryStorage {
    fn default() -> Self {
        MemoryStorage {
            iks: Arc::new(Mutex::new(HashMap::new())),
            spks: Arc::new(Mutex::new(HashMap::new())),
            opks: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn register_user(
        &self,
        identity: String,
        ik: VerifyingKey,
        spk: SignedPreKeyProto,
    ) -> tonic::Result<()> {
        self.iks.lock().unwrap().insert(identity.to_string(), ik);
        self.spks.lock().unwrap().insert(identity.to_string(), spk);
        self.opks
            .lock()
            .unwrap()
            .insert(identity.to_string(), Vec::new());
        Ok(())
    }

    async fn update_spk(
        &self,
        identity: String,
        mut pre_key: SignedPreKeyProto,
    ) -> tonic::Result<()> {
        self.spks
            .lock()
            .unwrap()
            .get_mut(&identity)
            .replace(&mut pre_key);
        Ok(())
    }

    async fn add_opks(
        &self,
        identity: String,
        mut pre_keys: Vec<X25519PublicKey>,
    ) -> tonic::Result<()> {
        let mut opks = self.opks.lock().unwrap();
        opks.get_mut(&identity)
            .ok_or(Status::not_found("User not found."))?
            .append(&mut pre_keys);
        Ok(())
    }

    async fn get_current_keys(&self, identity: String) -> tonic::Result<CurrentKeys> {
        let ik = *self
            .iks
            .lock()
            .unwrap()
            .get(&identity)
            .ok_or(Status::not_found("User not found."))?;
        let spk = self
            .spks
            .lock()
            .unwrap()
            .get(&identity)
            .ok_or(Status::not_found("User not found."))?
            .to_owned();
        Ok((ik, spk))
    }

    async fn pop_opk(&self, identity: String) -> tonic::Result<Option<X25519PublicKey>> {
        let opk = if let Some(opks) = self.opks.lock().unwrap().get_mut(&identity) {
            opks.pop()
        } else {
            None
        };
        Ok(opk)
    }

    async fn add_message(&self, recipient: String, message: MessageProto) -> tonic::Result<()> {
        let mut messages = self.messages.lock().unwrap();
        if !messages.contains_key(&recipient) {
            messages.insert(recipient.to_owned(), Vec::new());
        }
        messages.get_mut(&recipient).unwrap().push(message);
        Ok(())
    }

    async fn get_messages(&self, identity: String) -> tonic::Result<Vec<MessageProto>> {
        Ok(self
            .messages
            .lock()
            .unwrap()
            .remove(&identity)
            .unwrap_or_default())
    }
}
