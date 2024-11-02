use chacha20poly1305::{
    aead::{Aead, AeadCore, OsRng, Payload},
    ChaCha20Poly1305, Nonce,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const NONCE_LEN: usize = 12;
const VERSION_TAG: u8 = 1;

#[derive(Error, Debug, Serialize, Deserialize, PartialEq)]
pub enum AeadError {
    #[error("Encryption Allocation Failed.")]
    Allocate,
    #[error("Decryption Failed.")]
    Decrypt,
    #[error("Unexpected tag: `{0}`")]
    Tag(u8),
    #[error("Invalid Ciphertext")]
    InvalidCiphertext,
}

pub fn encrypt_data(payload: Payload, cipher: &ChaCha20Poly1305) -> Result<Vec<u8>, AeadError> {
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, payload)
        .map_err(|_| AeadError::Allocate)?;

    Ok([vec![VERSION_TAG], nonce.to_vec(), ciphertext].concat())
}

pub fn decrypt_data(
    ciphertext: &[u8],
    aad: &[u8],
    cipher: &ChaCha20Poly1305,
) -> Result<Vec<u8>, AeadError> {
    if ciphertext.len() < NONCE_LEN + 1 {
        return Err(AeadError::InvalidCiphertext);
    }

    if ciphertext[0] != VERSION_TAG {
        return Err(AeadError::Tag(ciphertext[0]));
    }

    let nonce_bytes = &ciphertext[1..(NONCE_LEN + 1)];
    let msg = &ciphertext[(NONCE_LEN + 1)..];
    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), Payload { msg, aad })
        .map_err(|_| AeadError::Decrypt)
}

#[cfg(test)]
mod tests {
    use crate::aead::*;
    use anyhow::Result;
    use ary::ary;
    use chacha20poly1305::KeyInit;

    #[test]
    fn aead_roundtrip_success() -> Result<()> {
        let msg = b"Hello I am a plaintext.";
        let aad = &[];
        let cipher = ChaCha20Poly1305::new(&ChaCha20Poly1305::generate_key(&mut OsRng));

        let ciphertext = encrypt_data(Payload { msg, aad }, &cipher)?;
        let decrypted_data = decrypt_data(&ciphertext, aad, &cipher)?;

        assert_eq!(&msg[..], &decrypted_data);
        Ok(())
    }

    #[test]
    fn invalid_ciphertext() {
        let cipher = ChaCha20Poly1305::new(&ChaCha20Poly1305::generate_key(&mut OsRng));

        assert_eq!(
            decrypt_data(&ary![VERSION_TAG, in b"123456789"], &[], &cipher),
            Err(AeadError::InvalidCiphertext)
        );
    }

    #[test]
    fn invalid_tag() -> Result<()> {
        let msg = b"Hello I am a plaintext.";
        let cipher = ChaCha20Poly1305::new(&ChaCha20Poly1305::generate_key(&mut OsRng));

        let mut ciphertext = encrypt_data(Payload { msg, aad: &[] }, &cipher)?;
        *ciphertext.first_mut().unwrap() = 0;

        assert_eq!(
            decrypt_data(&ciphertext, &[], &cipher),
            Err(AeadError::Tag(0))
        );
        Ok(())
    }

    #[test]
    fn decryption_failure() -> Result<()> {
        let msg = b"Hello I am a plaintext.";
        let aad = &[];
        let cipher = ChaCha20Poly1305::new(&ChaCha20Poly1305::generate_key(&mut OsRng));

        let mut ciphertext = encrypt_data(Payload { msg, aad }, &cipher)?;
        *ciphertext.last_mut().unwrap() = 0;
        assert_eq!(
            decrypt_data(&ciphertext, aad, &cipher),
            Err(AeadError::Decrypt),
        );
        Ok(())
    }
}
