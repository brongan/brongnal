use chacha20poly1305::{
    aead::{Aead, AeadCore, OsRng, Payload},
    ChaCha20Poly1305, Nonce,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const NONCE_LEN: usize = 12;
const VERSION_TAG: u8 = 1;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AeadError {
    #[error("Encryption Failed.")]
    Encrypt,
    #[error("Unexpected tag: `{0}`")]
    Tag(u8),
}

pub fn encrypt_data(payload: Payload, cipher: &ChaCha20Poly1305) -> Result<Vec<u8>, AeadError> {
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, payload)
        .map_err(|_| AeadError::Encrypt)?;

    return Ok([vec![VERSION_TAG], nonce.to_vec(), ciphertext].concat());
}

pub fn decrypt_data(
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
        .decrypt(Nonce::from_slice(&nonce_bytes), Payload { msg: &msg, aad })
        .map_err(|_| AeadError::Encrypt)
}

#[cfg(test)]
mod tests {
    use crate::aead::*;
    use anyhow::{Context, Result};
    use chacha20poly1305::KeyInit;

    #[test]
    fn aead() -> Result<()> {
        let key = ChaCha20Poly1305::generate_key(&mut OsRng);
        let text = "Hello I am a string.";
        let cipher = ChaCha20Poly1305::new(&key);
        let ciphertext = encrypt_data(
            Payload {
                msg: text.as_bytes(),
                aad: &[],
            },
            &cipher,
        )
        .unwrap();
        let decrypted_data =
            decrypt_data(&ciphertext, &[], &cipher).context("decryption failed.")?;
        assert_eq!(text, String::from_utf8(decrypted_data)?);
        Ok(())
    }
}
