use application::contents::ContentType;
use application::{Contents, Sender};
use ed25519_dalek::{Signature, VerifyingKey};
use protocol::x3dh::Message as X3DHMessage;
use protocol::x3dh::PreKeyBundle;
use protocol::x3dh::SignedPreKey;
use protocol::x3dh::SignedPreKeys;
use service::Message as MessageProto;
use service::PreKeyBundle as PreKeyBundleProto;
use service::SignedPreKey as SignedPreKeyProto;
use service::SignedPreKeys as SignedPreKeysProto;
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

pub mod service {
    tonic::include_proto!("service.v1");
}

pub mod application {
    tonic::include_proto!("application.v1");
}

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("service_descriptor");

impl From<SignedPreKey> for SignedPreKeyProto {
    fn from(val: SignedPreKey) -> Self {
        SignedPreKeyProto {
            pre_key: Some(val.pre_key.to_bytes().to_vec()),
            signature: Some(val.signature.to_bytes().to_vec()),
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
            signature: Some(val.signature.to_bytes().to_vec()),
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
            ik: sender_ik,
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
            sender_identity_key: Some(val.ik.to_bytes().to_vec()),
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

pub struct RatchetHeader {
    ratchet_key: X25519PublicKey,
    message_number: u32,
    chain_length: u32,
}

pub struct RatchetMessage {
    pub header: Option<RatchetHeader>,
    pub message: ApplicationMessage,
}

#[derive(Debug, Clone)]
pub struct ApplicationMessage {
    pub sender: String,
    pub text: String,
}

impl TryInto<RatchetHeader> for application::ratchet_message::Header {
    type Error = tonic::Status;
    fn try_into(self) -> Result<RatchetHeader, Self::Error> {
        let ratchet_key = parse_x25519_public_key(
            &self
                .ratchet_key
                .ok_or(Status::invalid_argument("Missing ratchet key."))?,
        )
        .map_err(|_e| Status::invalid_argument("Ratchet key was not valid."))?;
        let message_number = self
            .message_number
            .ok_or(Status::invalid_argument("Missing message_number."))?;
        let chain_length = self
            .chain_length
            .ok_or(Status::invalid_argument("Missing chain_length."))?;
        Ok(RatchetHeader {
            ratchet_key,
            message_number,
            chain_length,
        })
    }
}

impl From<RatchetHeader> for application::ratchet_message::Header {
    fn from(val: RatchetHeader) -> Self {
        Self {
            ratchet_key: Some(val.ratchet_key.as_bytes().to_vec()),
            message_number: Some(val.message_number),
            chain_length: Some(val.chain_length),
        }
    }
}

impl TryInto<ApplicationMessage> for application::Message {
    type Error = tonic::Status;
    fn try_into(self) -> Result<ApplicationMessage, Self::Error> {
        let sender =
            self.sender
                .and_then(|sender| sender.username)
                .ok_or(Status::invalid_argument(
                    "ApplicationMessage missing sender.",
                ))?;
        let contents = self
            .contents
            .and_then(|contents| contents.content_type)
            .ok_or(Status::invalid_argument(
                "ApplicationMessage missing contents.",
            ))?;
        let text = match contents {
            ContentType::Text(text) => text,
            _ => return Err(Status::unimplemented("Only text is supported.")),
        };
        Ok(ApplicationMessage { sender, text })
    }
}

impl From<ApplicationMessage> for application::Message {
    fn from(val: ApplicationMessage) -> Self {
        Self {
            sender: Some(Sender {
                username: Some(val.sender),
            }),
            contents: Some(Contents {
                content_type: Some(ContentType::Text(val.text)),
            }),
        }
    }
}

impl TryInto<RatchetMessage> for application::RatchetMessage {
    type Error = tonic::Status;
    fn try_into(self) -> Result<RatchetMessage, Self::Error> {
        let header = self.header.map(|header| header.try_into()).transpose()?;
        let message = self
            .message
            .ok_or(Status::invalid_argument("RatchetMessage missing message."))?
            .try_into()?;
        Ok(RatchetMessage { header, message })
    }
}

impl From<RatchetMessage> for application::RatchetMessage {
    fn from(val: RatchetMessage) -> Self {
        Self {
            header: val.header.map(|header| header.into()),
            message: Some(val.message.into()),
        }
    }
}
