use blake2::{Blake2b512, Digest};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

pub fn sign_bundle(
    signing_key: &SigningKey,
    key_pairs: &[(X25519StaticSecret, X25519PublicKey)],
) -> Signature {
    let mut hasher = Blake2b512::new();
    hasher.update(key_pairs.len().to_be_bytes());
    for key_pair in key_pairs {
        hasher.update(key_pair.1.as_bytes());
    }
    signing_key.sign(&hasher.finalize())
}

pub fn verify_bundle(
    verifying_key: &VerifyingKey,
    public_keys: &[X25519PublicKey],
    signature: &Signature,
) -> Result<(), ed25519_dalek::ed25519::Error> {
    let mut hasher = Blake2b512::new();
    hasher.update(public_keys.len().to_be_bytes());
    for public_key in public_keys {
        hasher.update(public_key.as_bytes());
    }
    verifying_key.verify_strict(&hasher.finalize(), signature)
}

pub struct X3DHPreKeyBundle {
    pub bundle: Vec<(X25519StaticSecret, X25519PublicKey)>,
    pub signature: Signature,
}

pub fn create_prekey_bundle(signing_key: &SigningKey, num_keys: u32) -> X3DHPreKeyBundle {
    let bundle: Vec<_> = (0..num_keys)
        .map(|_| {
            let pkey = X25519StaticSecret::random();
            let pubkey = X25519PublicKey::from(&pkey);
            (pkey, pubkey)
        })
        .collect();
    let signature = sign_bundle(signing_key, &bundle);
    X3DHPreKeyBundle { signature, bundle }
}

#[cfg(test)]
mod tests {
    use crate::bundle::*;
    use anyhow::Result;
    use chacha20poly1305::aead::OsRng;

    #[test]
    fn create_verify_bundle_success() -> Result<()> {
        let key = SigningKey::generate(&mut OsRng);
        for bundle_size in [0, 1, 4] {
            let signed_bundle = create_prekey_bundle(&key, bundle_size);
            let bundle_keys: Vec<X25519PublicKey> = signed_bundle
                .bundle
                .into_iter()
                .map(|pair| pair.1)
                .collect();
            assert_eq!(
                verify_bundle(
                    &VerifyingKey::from(&key),
                    &bundle_keys,
                    &signed_bundle.signature
                )?,
                ()
            );

            let other_key = SigningKey::generate(&mut OsRng);
            assert!(verify_bundle(
                &VerifyingKey::from(&other_key),
                &bundle_keys,
                &signed_bundle.signature,
            )
            .is_err());
        }
        Ok(())
    }
}
