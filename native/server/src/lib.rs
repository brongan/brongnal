use ed25519_dalek::{Signature, VerifyingKey};
use proto::{
    PreKeyBundle as PreKeyBundleProto, SignedPreKey as SignedPreKeyProto,
    SignedPreKeys as SignedPreKeysProto, X3dhMessage as MessageProto,
};
use protocol::x3dh::{Message, PreKeyBundle, SignedPreKey, SignedPreKeys};
use tonic::Status;
use x25519_dalek::PublicKey as X25519PublicKey;

pub mod proto {
    tonic::include_proto!("service");
    tonic::include_proto!("gossamer");
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

impl Into<SignedPreKeyProto> for SignedPreKey {
    fn into(self) -> SignedPreKeyProto {
        SignedPreKeyProto {
            pre_key: Some(self.pre_key.to_bytes().to_vec()),
            signature: Some(self.signature.to_vec()),
        }
    }
}

impl Into<SignedPreKeysProto> for SignedPreKeys {
    fn into(self) -> SignedPreKeysProto {
        SignedPreKeysProto {
            pre_keys: self
                .pre_keys
                .into_iter()
                .map(|key| key.to_bytes().to_vec())
                .collect(),
            signature: Some(self.signature.to_vec()),
        }
    }
}

impl TryFrom<SignedPreKeyProto> for SignedPreKey {
    type Error = tonic::Status;

    fn try_from(value: SignedPreKeyProto) -> Result<Self, Self::Error> {
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
        Ok(SignedPreKey { pre_key, signature })
    }
}

impl TryFrom<MessageProto> for Message {
    type Error = tonic::Status;

    fn try_from(value: MessageProto) -> Result<Self, Self::Error> {
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

        Ok(Message {
            sender_identity_key,
            ephemeral_key,
            otk,
            ciphertext,
        })
    }
}

impl Into<MessageProto> for Message {
    fn into(self) -> MessageProto {
        MessageProto {
            sender_ik: Some(self.sender_identity_key.to_bytes().to_vec()),
            ephemeral_key: Some(self.ephemeral_key.to_bytes().to_vec()),
            otk: self.otk.map(|otk| otk.to_bytes().to_vec()),
            ciphertext: Some(self.ciphertext.as_bytes().to_vec()),
        }
    }
}

impl TryInto<PreKeyBundle> for PreKeyBundleProto {
    type Error = tonic::Status;

    fn try_into(self) -> Result<PreKeyBundle, Self::Error> {
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

        Ok(PreKeyBundle {
            identity_key,
            otk,
            spk,
        })
    }
}
