use ed25519_dalek::{Signature, VerifyingKey};
use prost::Message;
use tonic::Status;
use x25519_dalek::PublicKey as X25519PublicKey;

pub mod proto {
    pub mod gossamer {
        tonic::include_proto!("gossamer");
    }
    pub mod service {
        tonic::include_proto!("service");
    }
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("service_descriptor");
}

pub fn parse_verifying_key(key: Vec<u8>) -> Result<VerifyingKey, Status> {
    VerifyingKey::from_bytes(
        &key.try_into()
            .map_err(|_| Status::invalid_argument("Key is invalid size."))?,
    )
    .map_err(|_| Status::invalid_argument("ED25519 key was invalid."))
}

pub fn parse_x25519_public_key(key: Vec<u8>) -> Result<X25519PublicKey, Status> {
    let key: [u8; 32] = key
        .try_into()
        .map_err(|_| Status::invalid_argument("X25519PublicKey was invalid."))?;
    Ok(X25519PublicKey::from(key))
}

impl Into<proto::service::SignedPreKey> for protocol::x3dh::SignedPreKey {
    fn into(self) -> proto::service::SignedPreKey {
        proto::service::SignedPreKey {
            pre_key: Some(self.pre_key.to_bytes().to_vec()),
            signature: Some(self.signature.to_vec()),
        }
    }
}

impl Into<proto::service::SignedPreKeys> for protocol::x3dh::SignedPreKeys {
    fn into(self) -> proto::service::SignedPreKeys {
        proto::service::SignedPreKeys {
            pre_keys: self
                .pre_keys
                .into_iter()
                .map(|key| key.to_bytes().to_vec())
                .collect(),
            signature: Some(self.signature.to_vec()),
        }
    }
}

impl TryFrom<proto::service::SignedPreKey> for protocol::x3dh::SignedPreKey {
    type Error = tonic::Status;

    fn try_from(value: proto::service::SignedPreKey) -> Result<Self, Self::Error> {
        let signature = value
            .signature
            .ok_or(Status::invalid_argument("request missing signature"))?;

        let pre_key = parse_x25519_public_key(
            value
                .pre_key
                .ok_or(Status::invalid_argument("request issing pre key"))?,
        )?;
        let signature = Signature::from_slice(&signature)
            .map_err(|_| Status::invalid_argument("Pre Key has an invalid X25519 Signature"))?;
        Ok(protocol::x3dh::SignedPreKey { pre_key, signature })
    }
}

impl TryFrom<proto::service::Message> for protocol::x3dh::Message {
    type Error = tonic::Status;

    fn try_from(value: proto::service::Message) -> Result<Self, Self::Error> {
        let sender_identity_key = parse_verifying_key(value.sender_ik.ok_or(
            Status::invalid_argument("request missing sender_identity_key"),
        )?)?;

        let ephemeral_key = parse_x25519_public_key(
            value
                .ephemeral_key
                .ok_or(Status::invalid_argument("request missing ephemeral_key"))?,
        )?;

        let otk = if let Some(otk) = value.otk {
            Some(parse_x25519_public_key(otk)?)
        } else {
            None
        };

        let ciphertext = String::from_utf8(
            value
                .ciphertext
                .ok_or(Status::invalid_argument("request missing ciphertext"))?
                .to_vec(),
        )
        .map_err(|_| Status::invalid_argument("Invalid ciphertext."))?;

        Ok(protocol::x3dh::Message {
            sender_identity_key,
            ephemeral_key,
            otk,
            ciphertext,
        })
    }
}

impl Into<proto::service::Message> for protocol::x3dh::Message {
    fn into(self) -> proto::service::Message {
        proto::service::Message {
            sender_ik: Some(self.sender_identity_key.to_bytes().to_vec()),
            ephemeral_key: Some(self.ephemeral_key.to_bytes().to_vec()),
            otk: self.otk.map(|otk| otk.to_bytes().to_vec()),
            ciphertext: Some(self.ciphertext.as_bytes().to_vec()),
        }
    }
}

impl TryInto<protocol::x3dh::PreKeyBundle> for proto::service::PreKeyBundle {
    type Error = tonic::Status;

    fn try_into(self) -> Result<protocol::x3dh::PreKeyBundle, Self::Error> {
        let identity_key = parse_verifying_key(
            self.identity_key
                .ok_or(Status::invalid_argument("PreKeyBundle missing ik."))?,
        )?;

        let otk = if let Some(otk) = self.otk {
            Some(parse_x25519_public_key(otk)?)
        } else {
            None
        };

        let spk = self
            .spk
            .ok_or(Status::invalid_argument("PreKeyBundle missing spk."))?
            .try_into()?;

        Ok(protocol::x3dh::PreKeyBundle {
            identity_key,
            otk,
            spk,
        })
    }
}

struct SignedMessage {
    message: proto::gossamer::Message,
    signature: Signature,
    provider: String,
    public_key: VerifyingKey,
}

impl TryInto<SignedMessage> for proto::gossamer::SignedMessage {
    type Error = tonic::Status;
    fn try_into(self) -> Result<SignedMessage, Self::Error> {
        let signature = Signature::from_slice(self.signature())
            .map_err(|_| Status::invalid_argument("Pre Key has an invalid X25519 Signature"))?;
        let public_key = parse_verifying_key(self.public_key.ok_or(Status::invalid_argument(
            "request missing sender_identity_key",
        ))?)?;
        let contents = self
            .contents
            .ok_or(Status::invalid_argument("Missing contents."))?;
        public_key
            .verify_strict(&contents, &signature)
            .map_err(|_| Status::unauthenticated("SignedMessage signature invalid."))?;

        let message = proto::gossamer::Message::decode(&*contents)
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
