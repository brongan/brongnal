use ed25519_dalek::VerifyingKey;
use proto::service::Message as MessageProto;
use proto::service::SignedPreKey as SignedPreKeyProto;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use tonic::Status;
use x25519_dalek::PublicKey as X25519PublicKey;

use crate::brongnal::Storage;

#[derive(Clone, Debug)]
pub struct MemoryStorage {
    identity_key: Arc<Mutex<HashMap<String, VerifyingKey>>>,
    current_pre_key: Arc<Mutex<HashMap<String, SignedPreKeyProto>>>,
    one_time_pre_keys: Arc<Mutex<HashMap<String, Vec<X25519PublicKey>>>>,
    messages: Arc<Mutex<HashMap<String, Vec<MessageProto>>>>,
}

impl Storage for MemoryStorage {
    fn add_user(
        &self,
        identity: String,
        identity_key: VerifyingKey,
        signed_pre_key: SignedPreKeyProto,
    ) -> tonic::Result<()> {
        self.identity_key
            .lock()
            .unwrap()
            .insert(identity.clone(), identity_key);
        self.current_pre_key
            .lock()
            .unwrap()
            .insert(identity.clone(), signed_pre_key);
        self.one_time_pre_keys
            .lock()
            .unwrap()
            .insert(identity, Vec::new());
        Ok(())
    }

    fn update_pre_key(&self, identity: &str, mut pre_key: SignedPreKeyProto) -> tonic::Result<()> {
        self.current_pre_key
            .lock()
            .unwrap()
            .get_mut(identity)
            .replace(&mut pre_key);
        Ok(())
    }

    fn add_one_time_keys(
        &self,
        identity: &str,
        mut pre_keys: Vec<X25519PublicKey>,
    ) -> tonic::Result<()> {
        let mut one_time_pre_keys = self.one_time_pre_keys.lock().unwrap();
        one_time_pre_keys
            .get_mut(identity)
            .ok_or(Status::not_found("User not found."))?
            .append(&mut pre_keys);
        Ok(())
    }

    fn get_current_keys(&self, identity: &str) -> tonic::Result<(VerifyingKey, SignedPreKeyProto)> {
        let identity_key = *self
            .identity_key
            .lock()
            .unwrap()
            .get(identity)
            .ok_or(Status::not_found("User not found."))?;
        let signed_pre_key = self
            .current_pre_key
            .lock()
            .unwrap()
            .get(identity)
            .ok_or(Status::not_found("User not found."))?
            .to_owned();
        Ok((identity_key, signed_pre_key))
    }

    fn pop_one_time_key(&self, identity: &str) -> tonic::Result<Option<X25519PublicKey>> {
        let one_time_key =
            if let Some(one_time_keys) = self.one_time_pre_keys.lock().unwrap().get_mut(identity) {
                one_time_keys.pop()
            } else {
                None
            };
        Ok(one_time_key)
    }

    fn add_message(&self, recipient: &str, message: MessageProto) -> tonic::Result<()> {
        let mut messages = self.messages.lock().unwrap();
        if !messages.contains_key(recipient) {
            messages.insert(recipient.to_owned(), Vec::new());
        }
        messages.get_mut(recipient).unwrap().push(message);
        Ok(())
    }

    fn get_messages(&self, identity: &str) -> tonic::Result<Vec<MessageProto>> {
        Ok(self
            .messages
            .lock()
            .unwrap()
            .remove(identity)
            .unwrap_or(Vec::new()))
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        MemoryStorage {
            identity_key: Arc::new(Mutex::new(HashMap::new())),
            current_pre_key: Arc::new(Mutex::new(HashMap::new())),
            one_time_pre_keys: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
