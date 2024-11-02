use chacha20poly1305::{
    aead::{Aead, AeadCore, OsRng, Payload},
    ChaCha20Poly1305, KeyInit, Nonce,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const NONCE_LEN: usize = 12;
const VERSION_TAG: u8 = 2;

#[derive(Error, Debug, Serialize, Deserialize, PartialEq)]
pub enum AeadError {
    #[error("Encryption Failed.")]
    Encrypt,
    #[error("Unexpected tag: `{0}`")]
    Tag(u8),
}

fn encrypt_data(payload: Payload, cipher: &ChaCha20Poly1305) -> Result<Vec<u8>, AeadError> {
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, payload)
        .map_err(|_| AeadError::Encrypt)?;

    Ok([vec![VERSION_TAG], nonce.to_vec(), ciphertext].concat())
}

/// Encrypts a message using the AEAD scheme suitable in this application.
/// The AAD consists of sender_ik + receiver_ik + sender_identity + receiver_identity.
pub fn encrypt_brongnal(
    sk: &[u8; 32],
    message: &[u8],
    sender_ik: &[u8; 32],
    receiver_ik: &[u8; 32],
    sender: &str,
    receiver: &str,
) -> Result<Vec<u8>, AeadError> {
    let associated_data = [
        sender_ik,
        receiver_ik,
        sender.as_bytes(),
        receiver.as_bytes(),
    ]
    .concat();
    let payload = Payload {
        msg: message,
        aad: &associated_data,
    };
    let cipher = ChaCha20Poly1305::new_from_slice(sk).unwrap();
    encrypt_data(payload, &cipher)
}

fn decrypt_data(
    ciphertext: &[u8],
    aad: &[u8],
    cipher: &ChaCha20Poly1305,
) -> Result<Vec<u8>, AeadError> {
    if ciphertext[0] != VERSION_TAG {
        return Err(AeadError::Tag(ciphertext[0]));
    }
    let nonce_bytes = &ciphertext[1..(NONCE_LEN + 1)];
    let msg = &ciphertext[(NONCE_LEN + 1)..];
    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), Payload { msg, aad })
        .map_err(|_| AeadError::Encrypt)
}

/// Decrypts a message using the AEAD scheme suitable in this application.
/// The AAD consists of sender_ik + receiver_ik + sender_identity + receiver_identity.
pub fn decrypt_brongnal(
    sk: &[u8; 32],
    ciphertext: &[u8],
    sender_ik: &[u8; 32],
    receiver_ik: &[u8; 32],
    sender: &str,
    receiver: &str,
) -> Result<Vec<u8>, AeadError> {
    let associated_data = [
        sender_ik,
        receiver_ik,
        sender.as_bytes(),
        receiver.as_bytes(),
    ]
    .concat();
    decrypt_data(
        ciphertext,
        &associated_data,
        &ChaCha20Poly1305::new_from_slice(sk).unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use crate::aead::*;
    use anyhow::{Context, Result};
    use chacha20poly1305::KeyInit;
    use rand::Rng;

    #[test]
    fn aead() -> Result<()> {
        let key = ChaCha20Poly1305::generate_key(&mut OsRng);
        let msg = b"Hello I am a string.";
        let cipher = ChaCha20Poly1305::new(&key);
        let ciphertext = encrypt_data(Payload { msg, aad: &[] }, &cipher)?;
        let decrypted_data =
            decrypt_data(&ciphertext, &[], &cipher).context("decryption failed.")?;
        assert_eq!(msg, &decrypted_data);
        Ok(())
    }

    #[test]
    fn brongnal_aead() -> Result<()> {
        let msg = b"Hello I am a string.";
        let mut rng = rand::thread_rng();
        let sk = rng.gen();
        let alice_ik = rng.gen();
        let bob_ik = rng.gen();
        let ciphertext = encrypt_brongnal(sk, msg, alice_ik, bob_ik, "alice", "bo")?;
        let decrypted = decrypt_brongnal(sk, &ciphertext, alice_ik, bob_ik, "alice", "bob")?;
        assert_eq!(msg, decrypted);
        Ok(())
    }
}
