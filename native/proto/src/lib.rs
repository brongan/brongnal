use ed25519_dalek::{Signature, VerifyingKey};
use prost::Message;
use thiserror::Error;
use tonic::Status;
use x25519_dalek::PublicKey as X25519PublicKey;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Key was not a valid ED25519 point.")]
    InvalidEd25519Key,
    #[error("Key was not a valid X25519 point.")]
    InvalidX25519Key,
}

pub fn parse_verifying_key(key: &[u8]) -> Result<VerifyingKey, ClientError> {
    VerifyingKey::from_bytes(&key.try_into().map_err(|_| ClientError::InvalidEd25519Key)?)
        .map_err(|_| ClientError::InvalidEd25519Key)
}

pub fn parse_x25519_public_key(key: &[u8]) -> Result<X25519PublicKey, ClientError> {
    let key: [u8; 32] = key.try_into().map_err(|_| ClientError::InvalidX25519Key)?;
    Ok(X25519PublicKey::from(key))
}
pub mod gossamer {
    tonic::include_proto!("gossamer");
}
pub mod service {
    tonic::include_proto!("service");
}
pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("service_descriptor");

impl Into<service::SignedPreKey> for protocol::x3dh::SignedPreKey {
    fn into(self) -> service::SignedPreKey {
        service::SignedPreKey {
            pre_key: Some(self.pre_key.to_bytes().to_vec()),
            signature: Some(self.signature.to_vec()),
        }
    }
}

impl Into<service::SignedPreKeys> for protocol::x3dh::SignedPreKeys {
    fn into(self) -> service::SignedPreKeys {
        service::SignedPreKeys {
            pre_keys: self
                .pre_keys
                .into_iter()
                .map(|key| key.to_bytes().to_vec())
                .collect(),
            signature: Some(self.signature.to_vec()),
        }
    }
}

impl TryFrom<service::SignedPreKey> for protocol::x3dh::SignedPreKey {
    type Error = tonic::Status;

    fn try_from(value: service::SignedPreKey) -> Result<Self, Self::Error> {
        let signature = value.signature();

        let pre_key = parse_x25519_public_key(value.pre_key())
            .map_err(|e| Status::invalid_argument(format!("Invalid SignedPreKey: {e}")))?;
        let signature = Signature::from_slice(&signature)
            .map_err(|_| Status::invalid_argument("Pre Key has an invalid X25519 Signature"))?;
        Ok(protocol::x3dh::SignedPreKey { pre_key, signature })
    }
}

impl TryFrom<service::Message> for protocol::x3dh::Message {
    type Error = tonic::Status;

    fn try_from(value: service::Message) -> Result<Self, Self::Error> {
        let sender_identity = value.sender_identity().to_owned();
        let sender_identity_key = parse_verifying_key(value.sender_identity_key())
            .map_err(|e| Status::invalid_argument(format!("Invalid sender_identity_key: {e}")))?;

        let ephemeral_key = parse_x25519_public_key(&value.ephemeral_key())
            .map_err(|e| Status::invalid_argument(format!("Invalid ephemeral_key: {e}")))?;

        let one_time_key = if let Some(otk) = value.one_time_key {
            Some(
                parse_x25519_public_key(&otk)
                    .map_err(|e| Status::invalid_argument(format!("Invalid one_time_key: {e}")))?,
            )
        } else {
            None
        };
        Ok(protocol::x3dh::Message {
            sender_identity,
            sender_identity_key,
            ephemeral_key,
            one_time_key,
            ciphertext: value
                .ciphertext
                .ok_or(Status::invalid_argument("request missing ciphertext"))?
                .to_vec(),
        })
    }
}

impl Into<service::Message> for protocol::x3dh::Message {
    fn into(self) -> service::Message {
        service::Message {
            sender_identity: Some(self.sender_identity),
            sender_identity_key: Some(self.sender_identity_key.to_bytes().to_vec()),
            ephemeral_key: Some(self.ephemeral_key.to_bytes().to_vec()),
            one_time_key: self.one_time_key.map(|otk| otk.to_bytes().to_vec()),
            ciphertext: Some(self.ciphertext),
        }
    }
}

impl TryInto<protocol::x3dh::PreKeyBundle> for service::PreKeyBundle {
    type Error = tonic::Status;

    fn try_into(self) -> Result<protocol::x3dh::PreKeyBundle, Self::Error> {
        let identity_key = parse_verifying_key(self.identity_key())
            .map_err(|_| Status::invalid_argument("PreKeyBundle invalid identity_key"))?;

        let one_time_key = if let Some(otk) = self.one_time_key {
            Some(
                parse_x25519_public_key(&otk)
                    .map_err(|e| Status::invalid_argument(format!("Invalid one_time_key: {e}")))?,
            )
        } else {
            None
        };

        let signed_pre_key = self
            .signed_pre_key
            .ok_or(Status::invalid_argument("PreKeyBundle missing spk."))?
            .try_into()?;

        Ok(protocol::x3dh::PreKeyBundle {
            identity_key,
            one_time_key,
            signed_pre_key,
        })
    }
}

#[allow(dead_code)]
struct SignedMessage {
    message: gossamer::Message,
    signature: Signature,
    provider: String,
    public_key: VerifyingKey,
}

impl TryInto<SignedMessage> for gossamer::SignedMessage {
    type Error = tonic::Status;
    fn try_into(self) -> Result<SignedMessage, Self::Error> {
        let signature = Signature::from_slice(self.signature()).map_err(|_| {
            Status::invalid_argument("SignedMessage has an invalid X25519 Signature")
        })?;
        let public_key = parse_verifying_key(&self.public_key()).map_err(|e| {
            Status::invalid_argument(format!(
                "SignedMessage has invalid sender_identity_key: {e}"
            ))
        })?;
        let contents = self.contents();
        public_key
            .verify_strict(&contents, &signature)
            .map_err(|_| Status::unauthenticated("SignedMessage signature invalid."))?;

        let message = gossamer::Message::decode(&*contents)
            .map_err(|_| Status::invalid_argument("contents are not serialized message."))?;

        Ok(SignedMessage {
            message,
            public_key,
            signature,
            provider: self
                .provider
                .ok_or(Status::invalid_argument("Missing provider."))?,
        })
    }
}
