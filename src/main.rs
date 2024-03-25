use anyhow::{anyhow, Context, Result};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};

use std::sync::{mpsc, Arc};
use std::thread;

const NTHREADS: i32 = 16;
const NONCE_LEN: usize = 12;

fn encrypt_data(message: &[u8], cipher: &ChaCha20Poly1305) -> Result<String> {
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, &*message)
        .map_err(|e| anyhow!("encrypt failed: {e}"))?;

    Ok(format!(
        "{}{}{}",
        "v1",
        hex::encode(&nonce),
        hex::encode(&ciphertext)
    ))
}

fn decrypt_data(ciphertext: String, cipher: &ChaCha20Poly1305) -> Result<Vec<u8>> {
    let version = &ciphertext[0..2];
    if version != "v1" {
        return Err(anyhow!("Invalid version."));
    }

    let nonce_bytes = hex::decode(&ciphertext[2..(NONCE_LEN * 2 + 2)])
        .map_err(|e| anyhow!("Failed to decode nonce: {e}."))?;
    let encrypted = hex::decode(&ciphertext[(2 + NONCE_LEN * 2)..])
        .map_err(|e| anyhow!("Failed to decode ciphertext: {e}."))?;
    cipher
        .decrypt(&Nonce::from_slice(&nonce_bytes), &*encrypted)
        .map_err(|e| anyhow!("decrypt failed: {e}"))
}

fn threads_test() -> Result<()> {
    let mut children = Vec::new();
    let key = ChaCha20Poly1305::generate_key(&mut OsRng);
    let cipher = Arc::new(ChaCha20Poly1305::new(&key));

    let (tx, rx) = mpsc::channel();
    for id in 0..NTHREADS {
        let thread_tx = tx.clone();
        let cipher = cipher.clone();
        let child = thread::spawn(move || {
            let ciphertext: String =
                encrypt_data(format!("Hello I am thread: {id}").as_bytes(), &cipher).unwrap();
            thread_tx.send(ciphertext).unwrap();
        });

        children.push(child);
    }

    let mut ciphertexts: Vec<String> = Vec::with_capacity(NTHREADS as usize);
    for _ in 0..NTHREADS {
        ciphertexts.push(rx.recv().unwrap());
    }

    for child in children {
        child.join().expect("oops! the child thread panicked");
    }

    let cipher = ChaCha20Poly1305::new(&key);
    for ciphertext in ciphertexts {
        let decrypted_data = decrypt_data(ciphertext, &cipher).context("decryption failed.")?;
        println!(
            "Received: {}",
            String::from_utf8(decrypted_data.to_vec()).unwrap()
        );
    }
    Ok(())
}

fn main() {
    threads_test().unwrap();
}
