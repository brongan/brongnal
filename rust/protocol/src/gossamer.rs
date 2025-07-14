use ed25519_dalek::{Signature, VerifyingKey};
use num_enum::TryFromPrimitive;

#[repr(i32)]
#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum Action {
    AppendKey = 1,
    RevokeKey = 2,
}

pub struct Message {
    pub provider: Vec<u8>,
    pub public_key: VerifyingKey,
    pub action: Action,
}

pub struct SignedMessage {
    pub message: Message,
    pub signature: Signature,
    pub identity_key: VerifyingKey,
}
