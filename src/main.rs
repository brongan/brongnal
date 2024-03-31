use anyhow::{anyhow, Context, Result};
use blake2::{Blake2b512, Digest};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use ed25519_dalek::{SecretKey, Signature, Signer, SigningKey, Verifier, VerifyingKey};
use std::sync::{mpsc, Arc};
use std::thread;
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

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

fn sign_bundle(signing_key: &SigningKey, key_pairs: &[(StaticSecret, PublicKey)]) -> Signature {
    let mut hasher = Blake2b512::new();
    hasher.update(key_pairs.len().to_be_bytes());
    for key_pair in key_pairs {
        hasher.update(key_pair.1.as_bytes());
    }
    signing_key.sign(&hasher.finalize())
}

fn verify_bundle(
    verifying_key: &VerifyingKey,
    public_keys: &[PublicKey],
    signature: &Signature,
) -> Result<(), ed25519_dalek::ed25519::Error> {
    let mut hasher = Blake2b512::new();
    hasher.update(public_keys.len().to_be_bytes());
    for public_key in public_keys {
        hasher.update(public_key.as_bytes());
    }
    verifying_key.verify(&hasher.finalize(), signature)
}

struct X3DHPreKey {
    signature: Signature,
    bundle: Vec<(StaticSecret, PublicKey)>,
}

fn x3dh_pre_key(signing_key: &SigningKey, num_keys: u32) -> X3DHPreKey {
    let bundle: Vec<_> = (0..num_keys)
        .map(|_| {
            let pkey = StaticSecret::random();
            let pubkey = PublicKey::from(&pkey);
            (pkey, pubkey)
        })
        .collect();
    let signature = sign_bundle(signing_key, &bundle);
    X3DHPreKey { signature, bundle }
}

struct SignedPreKey {
    signature: Signature,
    pre_key: PublicKey,
}

struct X3DHInitialResponse {
    identity_key: VerifyingKey,
    signed_pre_key: SignedPreKey,
    one_time_key: Option<PublicKey>,
}

struct X3DHInitiateResult {
    identity_key: VerifyingKey,
    ephemeral_key: EphemeralSecret,
    //secret_key:  
}

fn x3dh_initiate_send_get_sk(
    x3dh_initial_response: &X3DHInitialResponse,
    sender_key: &SecretKey,
) -> Result<()> {
    verify_bundle(
        &x3dh_initial_response.identity_key,
        &[x3dh_initial_response.signed_pre_key.pre_key],
        &x3dh_initial_response.signed_pre_key.signature,
    )
    .map_err(|e| anyhow!("Failed to verify bundle: {e}"));

    let ephemeral = EphemeralSecret::random();
    let sender_key = PublicKey::from(sender_key.into());
    // const DH1 = await sodium.crypto_scalarmult(senderX, signedPreKey);
    // const DH2 = await sodium.crypto_scalarmult(ephSecret, recipientX);
    // const DH3 = await sodium.crypto_scalarmult(ephSecret, signedPreKey);
    let dh1 = sender_key.diffie_hellman(&x3dh_initial_response.signed_pre_key.pre_key);
    let dh2 = ephemeral.diffie_hellman(&PublicKey::from(x3dh_initial_response.identity_key.to_montgomery().to_bytes()));
    let dh3 = ephemeral.diffie_hellman(&x3dh_initial_response.signed_pre_key.pre_key);

    let sk = if let Some(one_time_key) = x3dh_initial_response.one_time_key {
        let dh4 = ephemeral.diffie_hellman(&one_time_key)
        kdf(dh1, dh2, dh3, dh4)
    } else {
        kdf(dh1, dh2, dh3)
    };

}

fn main() {
    threads_test().unwrap();
}
