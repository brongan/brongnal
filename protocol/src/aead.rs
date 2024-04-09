use chacha20poly1305::{
    aead::{Aead, AeadCore, OsRng, Payload},
    ChaCha20Poly1305, Nonce,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const NONCE_LEN: usize = 12;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AeadError {
    #[error("Encryption Failed.")]
    Encrypt,
    #[error("Unexpected tag: `{0}`")]
    Tag(String),
    #[error("Ciphertext was not hex encoded.")]
    Encoding,
}

pub fn encrypt_data(payload: Payload, cipher: &ChaCha20Poly1305) -> Result<String, AeadError> {
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, payload)
        .map_err(|_| AeadError::Encrypt)?;

    Ok(format!(
        "{}{}{}",
        "v1",
        hex::encode(nonce),
        hex::encode(ciphertext)
    ))
}

pub fn decrypt_data(
    ciphertext: &str,
    aad: &[u8],
    cipher: &ChaCha20Poly1305,
) -> Result<Vec<u8>, AeadError> {
    let version = &ciphertext[0..2];
    if version != "v1" {
        return Err(AeadError::Tag(version.to_owned()));
    }

    let nonce_bytes =
        hex::decode(&ciphertext[2..(NONCE_LEN * 2 + 2)]).map_err(|_| AeadError::Encoding)?;
    let msg = hex::decode(&ciphertext[(2 + NONCE_LEN * 2)..]).map_err(|_| AeadError::Encoding)?;
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
        let ciphertext: String = encrypt_data(
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
