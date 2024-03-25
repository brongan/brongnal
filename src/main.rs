use anyhow::{anyhow, Context, Result};
use ring::aead::Aad;
use ring::aead::LessSafeKey;
use ring::aead::Nonce;
use ring::aead::UnboundKey;
use ring::aead::CHACHA20_POLY1305;
use ring::aead::NONCE_LEN;
use ring::rand::SecureRandom;
use ring::rand::SystemRandom;
use ring::test::from_hex;
use std::sync::mpsc;
use std::thread;

const NTHREADS: i32 = 16;

fn encrypt_data(
    message: &[u8],
    sealing_key: &mut LessSafeKey,
    id: i32,
) -> Result<String, ring::error::Unspecified> {
    let mut encrypted = message.to_vec();
    let mut nonce_bytes = vec![0; NONCE_LEN];
    let bytes = id.to_be_bytes();
    nonce_bytes[8..].copy_from_slice(&bytes);
    let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)?;
    sealing_key
        .seal_in_place_append_tag(nonce, Aad::empty(), &mut encrypted)
        .unwrap();

    Ok(format!(
        "{}{}{}",
        "v1",
        hex::encode(&nonce_bytes),
        hex::encode(&encrypted)
    ))
}

fn decrypt_data(ciphertext: String, key: &mut LessSafeKey) -> Result<Vec<u8>> {
    let version = &ciphertext[0..2];
    if version != "v1" {
        return Err(anyhow!("Invalid version."));
    }

    let nonce_bytes = from_hex(&ciphertext[2..(NONCE_LEN * 2 + 2)])
        .map_err(|e| anyhow!("Failed to decode nonce: {e}."))?;
    let mut encrypted = from_hex(&ciphertext[(2 + NONCE_LEN * 2)..])
        .map_err(|e| anyhow!("Failed to decode ciphertext: {e}."))?;
    let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
        .map_err(|e| anyhow!("Failed to nonce: {e}"))?;
    Ok(key
        .open_in_place(nonce, Aad::empty(), &mut encrypted)
        .map_err(|e| anyhow!("Failed to open_in_Place: {e}"))?
        .to_owned())
}

fn do_stuff() -> Result<()> {
    let mut children = Vec::new();
    let rand = SystemRandom::new();
    let mut key = vec![0; CHACHA20_POLY1305.key_len()];
    rand.fill(&mut key)
        .map_err(|e| anyhow!("Failed to fill key: {e}"))?;

    let (tx, rx) = mpsc::channel();
    for id in 0..NTHREADS {
        let thread_tx = tx.clone();
        let key = key.clone();
        let child = thread::spawn(move || {
            let mut sealing_key =
                LessSafeKey::new(UnboundKey::new(&CHACHA20_POLY1305, &key).unwrap());
            let msg: String = encrypt_data(
                format!("Hello I am thread: {id}").as_bytes(),
                &mut sealing_key,
                id,
            )
            .unwrap();
            thread_tx.send(msg).unwrap();
            println!("thread {} finished", id);
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

    for ciphertext in ciphertexts {
        let mut opening_key = LessSafeKey::new(
            UnboundKey::new(&CHACHA20_POLY1305, &key)
                .map_err(|e| anyhow!("Failed to create opening key: {e}"))?,
        );
        let decrypted_data =
            decrypt_data(ciphertext, &mut opening_key).context("decryption failed.")?;
        println!(
            "Received: {}",
            String::from_utf8(decrypted_data.to_vec()).unwrap()
        );
    }

    Ok(())
}

fn main() {
    do_stuff().unwrap();
}
