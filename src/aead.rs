use anyhow::{anyhow, Result};
use chacha20poly1305::{
    aead::{Aead, AeadCore, OsRng, Payload},
    ChaCha20Poly1305, Nonce,
};

const NONCE_LEN: usize = 12;

pub fn encrypt_data(payload: Payload, cipher: &ChaCha20Poly1305) -> Result<String> {
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, payload)
        .map_err(|e| anyhow!("encrypt failed: {e}"))?;

    Ok(format!(
        "{}{}{}",
        "v1",
        hex::encode(&nonce),
        hex::encode(&ciphertext)
    ))
}

pub fn decrypt_data(ciphertext: &str, aad: &[u8], cipher: &ChaCha20Poly1305) -> Result<Vec<u8>> {
    let version = &ciphertext[0..2];
    if version != "v1" {
        return Err(anyhow!("Invalid version."));
    }

    let nonce_bytes = hex::decode(&ciphertext[2..(NONCE_LEN * 2 + 2)])
        .map_err(|e| anyhow!("Failed to decode nonce: {e}."))?;
    let msg = hex::decode(&ciphertext[(2 + NONCE_LEN * 2)..])
        .map_err(|e| anyhow!("Failed to decode ciphertext: {e}."))?;
    cipher
        .decrypt(&Nonce::from_slice(&nonce_bytes), Payload { msg: &msg, aad })
        .map_err(|e| anyhow!("decrypt failed: {e}"))
}

#[cfg(test)]
mod tests {
    use crate::aead::*;
    use anyhow::Context;
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
