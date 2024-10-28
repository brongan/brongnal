use ed25519_dalek::{Signature, VerifyingKey};
use prost::Message;
use thiserror::Error;
use tonic::Status;
use x25519_dalek::PublicKey as X25519PublicKey;

#[derive(Error, Debug)]
pub enum KeyError {
    #[error("Key was not a valid ED25519 point.")]
    InvalidEd25519Key,
    #[error("Key was not a valid X25519 point.")]
    InvalidX25519Key,
}

pub fn parse_verifying_key(key: &[u8]) -> Result<VerifyingKey, KeyError> {
    VerifyingKey::from_bytes(&key.try_into().map_err(|_| KeyError::InvalidEd25519Key)?)
        .map_err(|_| KeyError::InvalidEd25519Key)
}

pub fn parse_x25519_public_key(key: &[u8]) -> Result<X25519PublicKey, KeyError> {
    let key: [u8; 32] = key.try_into().map_err(|_| KeyError::InvalidX25519Key)?;
    Ok(X25519PublicKey::from(key))
}
pub mod gossamer {
    tonic::include_proto!("gossamer");
}
pub mod service {
    tonic::include_proto!("service");
}
pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("service_descriptor");

use protocol::x3dh::Message as X3DHMessage;
use protocol::x3dh::PreKeyBundle;
use protocol::x3dh::SignedPreKey;
use protocol::x3dh::SignedPreKeys;
use service::Message as MessageProto;
use service::PreKeyBundle as PreKeyBundleProto;
use service::SignedPreKey as SignedPreKeyProto;
use service::SignedPreKeys as SignedPreKeysProto;

impl From<SignedPreKey> for SignedPreKeyProto {
    fn from(val: SignedPreKey) -> Self {
        SignedPreKeyProto {
            pre_key: Some(val.pre_key.to_bytes().to_vec()),
            signature: Some(val.signature.to_vec()),
        }
    }
}

impl From<SignedPreKeys> for SignedPreKeysProto {
    fn from(val: SignedPreKeys) -> Self {
        SignedPreKeysProto {
            pre_keys: val
                .pre_keys
                .into_iter()
                .map(|key| key.to_bytes().to_vec())
                .collect(),
            signature: Some(val.signature.to_vec()),
        }
    }
}

impl TryFrom<SignedPreKeyProto> for SignedPreKey {
    type Error = tonic::Status;

    fn try_from(value: SignedPreKeyProto) -> Result<Self, Self::Error> {
        let signature = value.signature();

        let pre_key = parse_x25519_public_key(value.pre_key())
            .map_err(|e| Status::invalid_argument(format!("Invalid SignedPreKey: {e}")))?;
        let signature = Signature::from_slice(signature)
            .map_err(|_| Status::invalid_argument("Pre Key has an invalid X25519 Signature"))?;
        Ok(SignedPreKey { pre_key, signature })
    }
}

impl TryFrom<MessageProto> for X3DHMessage {
    type Error = tonic::Status;

    fn try_from(value: MessageProto) -> Result<Self, Self::Error> {
        let sender_identity = value.sender_identity().to_owned();
        let sender_ik = parse_verifying_key(value.sender_identity_key())
            .map_err(|e| Status::invalid_argument(format!("Invalid sender_identity_key: {e}")))?;

        let ek = parse_x25519_public_key(value.ephemeral_key())
            .map_err(|e| Status::invalid_argument(format!("Invalid ephemeral_key: {e}")))?;

        let pre_key = parse_x25519_public_key(value.pre_key())
            .map_err(|e| Status::invalid_argument(format!("Invalid pre: {e}")))?;

        let opk = if let Some(opk) = value.one_time_key {
            Some(
                parse_x25519_public_key(&opk)
                    .map_err(|e| Status::invalid_argument(format!("Invalid one_time_key: {e}")))?,
            )
        } else {
            None
        };
        Ok(X3DHMessage {
            sender_identity,
            sender_ik,
            ek,
            pre_key,
            opk,
            ciphertext: value
                .ciphertext
                .ok_or(Status::invalid_argument("request missing ciphertext"))?
                .to_vec(),
        })
    }
}

impl From<X3DHMessage> for MessageProto {
    fn from(val: X3DHMessage) -> Self {
        MessageProto {
            sender_identity: Some(val.sender_identity),
            sender_identity_key: Some(val.sender_ik.to_bytes().to_vec()),
            ephemeral_key: Some(val.ek.to_bytes().to_vec()),
            pre_key: Some(val.pre_key.to_bytes().to_vec()),
            one_time_key: val.opk.map(|opk| opk.to_bytes().to_vec()),
            ciphertext: Some(val.ciphertext),
        }
    }
}

impl TryInto<PreKeyBundle> for PreKeyBundleProto {
    type Error = tonic::Status;

    fn try_into(self) -> Result<PreKeyBundle, Self::Error> {
        let ik = parse_verifying_key(self.identity_key())
            .map_err(|_| Status::invalid_argument("PreKeyBundle invalid identity_key"))?;

        let opk = if let Some(opk) = self.one_time_key {
            Some(
                parse_x25519_public_key(&opk)
                    .map_err(|e| Status::invalid_argument(format!("Invalid one_time_key: {e}")))?,
            )
        } else {
            None
        };

        let spk = self
            .signed_pre_key
            .ok_or(Status::invalid_argument("PreKeyBundle missing spk."))?
            .try_into()?;

        Ok(PreKeyBundle { ik, opk, spk })
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
        let public_key = parse_verifying_key(self.public_key()).map_err(|e| {
            Status::invalid_argument(format!(
                "SignedMessage has invalid sender_identity_key: {e}"
            ))
        })?;
        let contents = self.contents();
        public_key
            .verify_strict(contents, &signature)
            .map_err(|_| Status::unauthenticated("SignedMessage signature invalid."))?;

        let message = gossamer::Message::decode(contents)
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
