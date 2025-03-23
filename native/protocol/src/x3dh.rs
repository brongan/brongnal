use crate::aead::{decrypt_data, encrypt_data, AeadError};
use crate::bundle::*;
use chacha20poly1305::{
    aead::{KeyInit, Payload},
    ChaCha20Poly1305,
};
use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;
use x25519_dalek::{
    PublicKey as X25519PublicKey, ReusableSecret as X25519ReusableSecret,
    StaticSecret as X25519StaticSecret,
};

/*
    Glossary - See https://signal.org/docs/specifications/x3dh/
    Alice is sending an end-to-end encrypted (E2EE) message to Bob.
    SK - Shared Secret. This protocol is used to result in Alice and Bob deriving the same sk which
    they can use for Symmetric encryptic encryption and ratcheting.
    EK - Ephemeral Key. This key is created when Alice sends message to Bob. She uses the private
    key to calculate `sk`, forgets `ek`, and puts the `ek` public key in the encoded message.
    IK - Identity Key. A long-term ED25519 key that identifies as user in the protocol.
    SPK - Signed Pre Key. A medium-term X25519 key signed by the user's IK.
    OPK - One-Time Pre Key. A short-term X25519 key that can only be used once by a client. The
    public key should only be vended by the server once to avoid failures.
    DH - Elliptic-curve Diffie–Hellman (ECDH) is a key agreement protocol that allows two parties,
    each having an elliptic-curve public–private key pair, to establish a shared secret over an insecure channel
    IKA - Alice's Identity Key
*/

/// A partipant in the X3DH protocol's prekey and the signature over it using their identity key.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedPreKey {
    pub pre_key: X25519PublicKey,
    pub signature: Signature,
}

/// A signature over multiple signed prekeys. Useful for one-time prekeys.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedPreKeys {
    pub pre_keys: Vec<X25519PublicKey>,
    pub signature: Signature,
}

/// The X3DH key agreement protocol results in:
/// * an epheremeral key `ek` that the recipient needs to decrypt encrypted message.
/// * the 32-bytes shared secret key SK.
#[derive(Debug, PartialEq)]
pub struct X3DHSendKeyAgreement {
    pub ek: X25519PublicKey,
    pub sk: [u8; 32],
}

/// A `Message` in this packages implementation of the X3DH protocol.
/// This struct is the output of `Alice` sending a message to `Bob`.
/// * `sender_ik` is the identity key of the sender.
/// * `ek` is the ephemeral key generated to encrypt the message.
/// * `pre_key` is the identifier (currently public key) of Bob's prekey that was used.
/// * `opk` is Bob's one time prekey that (may) have been used.
/// * `ciphertext` is the encrypted message.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub sender_ik: VerifyingKey,
    pub ek: X25519PublicKey,
    pub pre_key: X25519PublicKey,
    pub opk: Option<X25519PublicKey>,
    pub ciphertext: Vec<u8>,
}

#[allow(deprecated)]
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "From:   {}\n\
            EK:      {}\n\
            OPK:     {}\n\
            Payload: {}\n",
            base64::encode(self.sender_ik),
            base64::encode(self.ek),
            self.opk
                .map(base64::encode)
                .unwrap_or(String::from("(None)")),
            base64::encode(&self.ciphertext)
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreKeyBundle {
    pub ik: VerifyingKey,
    pub opk: Option<X25519PublicKey>,
    pub spk: SignedPreKey,
}

// KDF = Key Derivation Function
// HKDF (https://en.wikipedia.org/wiki/HKDF) is an HMAC based KDF construction defined in https://datatracker.ietf.org/doc/html/rfc5869.
// KDF(KM) represents 32 bytes of output from the HKDF algorithm [3] with inputs:
//    HKDF input key material = F || KM, where KM is an input byte sequence containing secret key material, and F is a byte sequence containing 32 0xFF bytes if curve is X25519, and 57 0xFF bytes if curve is X448. F is used for cryptographic domain separation with XEdDSA [2].
//    HKDF salt = A zero-filled byte sequence with length equal to the hash output length.
//    HKDF info = An ASCII string identifying the application.
fn kdf(km: &[u8]) -> [u8; 32] {
    let salt = [0; 32];
    let f = [0xFF, 32];
    let ikm = [&f, km].concat();
    let hk = Hkdf::<Sha256>::new(Some(&salt), &ikm);
    let mut okm = [0u8; 32];
    hk.expand(b"Brongnal", &mut okm).unwrap();
    okm
}

/// Convert Ed25519 private key to its birationally equivalent X25519 key.
fn to_x25519(key: &SigningKey) -> X25519StaticSecret {
    X25519StaticSecret::from(key.to_scalar_bytes())
}

/// Convert Ed25519 public key to its birationally equivalent X25519 key.
fn to_x25519_pub(key: &VerifyingKey) -> X25519PublicKey {
    X25519PublicKey::from(key.to_montgomery().to_bytes())
}

#[derive(Error, Debug, Serialize, Deserialize, PartialEq)]
pub enum X3DHError {
    #[error("Signature failed to validate.")]
    SignatureValidation,
    #[error("Aead routine failed.")]
    Aead(#[from] AeadError),
}

// DH(PK1, PK2) represents a byte sequence which is the shared secret output from an Elliptic Curve Diffie-Hellman function involving the key pairs represented by public keys PK1 and PK2. The Elliptic Curve Diffie-Hellman function will be either the X25519 or X448 function from [1], depending on the curve parameter.
// If the bundle does not contain a one-time prekey, she calculates:
//    DH1 = DH(IKA, SPKB)
//    DH2 = DH(EKA, IKB)
//    DH3 = DH(EKA, SPKB)
//    SK = KDF(DH1 || DH2 || DH3)
//If the bundle does contain a one-time prekey, the calculation is modified to include an additional DH:
//    DH4 = DH(EKA, OPKB)
//    SK = KDF(DH1 || DH2 || DH3 || DH4)
fn initiate_send_get_sk(
    recipient_ik: VerifyingKey,
    spk: &SignedPreKey,
    opk: Option<X25519PublicKey>,
    sender_ik: &SigningKey,
) -> Result<X3DHSendKeyAgreement, X3DHError> {
    // It might be tempting to observe that mutual authentication and forward secrecy are achieved by the DH calculations, and omit the prekey signature.
    // However, this would allow a "weak forward secrecy" attack:
    // A malicious server could provide Alice a prekey bundle with forged prekeys, and later compromise Bob's IKB to calculate SK.
    verify_bundle(&recipient_ik, &[spk.pre_key], &spk.signature)
        .map_err(|_| X3DHError::SignatureValidation)?;

    let ek = X25519ReusableSecret::random();
    let dh1 = to_x25519(sender_ik).diffie_hellman(&spk.pre_key);
    let dh2 = ek.diffie_hellman(&to_x25519_pub(&recipient_ik));
    let dh3 = ek.diffie_hellman(&spk.pre_key);

    let sk = match opk {
        Some(one_time_prekey) => {
            let dh4 = ek.diffie_hellman(&one_time_prekey);
            kdf(&[
                dh1.to_bytes(),
                dh2.to_bytes(),
                dh3.to_bytes(),
                dh4.to_bytes(),
            ]
            .concat())
        }
        None => kdf(&[dh1.to_bytes(), dh2.to_bytes(), dh3.to_bytes()].concat()),
    };

    // After a successful protocol run Alice and Bob will share a 32-byte secret key SK.
    // This key may be used within some post-X3DH secure communication protocol.
    Ok(X3DHSendKeyAgreement {
        ek: X25519PublicKey::from(&ek),
        sk,
    })
}

// Alice then sends Bob an initial message containing:
//    Alice's identity key IKA
//    Alice's ephemeral key EKA
//    Identifiers stating which of Bob's prekeys Alice used
//    An initial ciphertext encrypted with some AEAD encryption scheme [4] using AD as associated data
//    and using an encryption key which is either SK or the output from some cryptographic PRF keyed by SK.
pub fn initiate_send(
    prekey_bundle: PreKeyBundle,
    sender_ik: &SigningKey,
    message: &[u8],
) -> Result<([u8; 32], Message), X3DHError> {
    let X3DHSendKeyAgreement { ek, sk } = initiate_send_get_sk(
        prekey_bundle.ik,
        &prekey_bundle.spk,
        prekey_bundle.opk,
        sender_ik,
    )?;
    // Alice then calculates an "associated data" byte sequence AD that contains identity information for both parties:
    //   AD = Encode(IKA) || Encode(IKB)
    // Alice may optionally append additional information to AD, such as Alice and Bob's usernames, certificates, or other identifying information.
    let associated_data = [
        sender_ik.verifying_key().to_bytes(),
        prekey_bundle.ik.to_bytes(),
    ]
    .concat();

    // The initial ciphertext is typically the first message in some post-X3DH communication protocol.
    // In other words, this ciphertext typically has two roles, serving as the first message within some post-X3DH protocol, and as part of Alice's X3DH initial message.
    // After sending this, Alice may continue using SK or keys derived from SK within the post-X3DH protocol for communication with Bob
    let ciphertext = encrypt_data(
        Payload {
            msg: message,
            aad: &associated_data,
        },
        &ChaCha20Poly1305::new_from_slice(&sk).unwrap(),
    )?;

    Ok((
        sk,
        Message {
            sender_ik: sender_ik.verifying_key(),
            ek,
            pre_key: prekey_bundle.spk.pre_key,
            opk: prekey_bundle.opk,
            ciphertext,
        },
    ))
}

fn initiate_recv_get_sk(
    sender_ik: &VerifyingKey,
    ek: X25519PublicKey,
    opk: Option<X25519StaticSecret>,
    receiver_ik: &SigningKey,
    spk: &X25519StaticSecret,
) -> [u8; 32] {
    let dh1 = spk.diffie_hellman(&to_x25519_pub(sender_ik));
    let dh2 = to_x25519(receiver_ik).diffie_hellman(&ek);
    let dh3 = spk.diffie_hellman(&ek);

    if let Some(opk) = opk {
        let dh4 = opk.diffie_hellman(&ek);
        kdf(&[
            dh1.to_bytes(),
            dh2.to_bytes(),
            dh3.to_bytes(),
            dh4.to_bytes(),
        ]
        .concat())
    } else {
        kdf(&[dh1.to_bytes(), dh2.to_bytes(), dh3.to_bytes()].concat())
    }
}

/// Bob deletes any one-time prekey private key that was used, for forward secrecy.
/// Caller must delete sk on error and the opk must be wiped.
pub fn initiate_recv(
    receiver_ik: &SigningKey,
    receiver_spk: &X25519StaticSecret,
    sender_ik: &VerifyingKey,
    ek: X25519PublicKey,
    receiver_opk: Option<X25519StaticSecret>,
    ciphertext: &[u8],
) -> Result<([u8; 32], Vec<u8>), X3DHError> {
    // Upon receiving Alice's initial message, Bob retrieves Alice's identity key and ephemeral key from the message.
    // Bob also loads his identity private key, and the private key(s) corresponding to whichever signed prekey and one-time prekey (if any) Alice used.
    // Using these keys, Bob repeats the DH and KDF calculations from the previous section to derive SK, and then deletes the DH values.
    let sk = initiate_recv_get_sk(sender_ik, ek, receiver_opk, receiver_ik, receiver_spk);

    // Bob then constructs the AD byte sequence using IKA and IKB, as described in the previous section.
    // AD = Encode(IKA) || Encode(IKB)
    let ad = [sender_ik.to_bytes(), receiver_ik.verifying_key().to_bytes()].concat();

    // Bob may then continue using SK or keys derived from SK within the post-X3DH protocol for communication with Alice.
    // Finally, Bob attempts to decrypt the initial ciphertext using SK and AD.
    let cipher = ChaCha20Poly1305::new_from_slice(&sk).unwrap();
    Ok((sk, decrypt_data(ciphertext, &ad, &cipher)?))
}

#[cfg(test)]
mod tests {
    use crate::aead::AeadError;
    use crate::x3dh::X3DHError;

    use super::PreKeyBundle;
    use super::{
        create_prekey_bundle, initiate_recv, initiate_recv_get_sk, initiate_send,
        initiate_send_get_sk, SignedPreKey, X3DHSendKeyAgreement,
    };
    use anyhow::Result;
    use chacha20poly1305::aead::OsRng;
    use ed25519_dalek::SigningKey;
    use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

    // 1. Bob publishes his identity key and prekeys to a server.
    // 2. Alice fetches a "prekey bundle" from the server, and uses it to send an initial message to Bob.
    // 3. Bob receives and processes Alice's initial message.
    #[test]
    fn x3dh_key_agreement_opk() -> Result<()> {
        let bob_ik = SigningKey::generate(&mut OsRng);
        let bob_spk = create_prekey_bundle(&bob_ik, 1);
        let bob_spk_secret = bob_spk.bundle[0].clone().0;
        let bob_spk = SignedPreKey {
            pre_key: bob_spk.bundle[0].1,
            signature: bob_spk.signature,
        };
        let alice_ik = SigningKey::generate(&mut OsRng);

        let opk = X25519StaticSecret::random_from_rng(OsRng);
        let opk_pub = X25519PublicKey::from(&opk);

        let X3DHSendKeyAgreement {
            ek: ephemeral_key,
            sk: secret_key,
        } = initiate_send_get_sk(bob_ik.verifying_key(), &bob_spk, Some(opk_pub), &alice_ik)?;

        let recv_sk = initiate_recv_get_sk(
            &alice_ik.verifying_key(),
            ephemeral_key,
            Some(opk),
            &bob_ik,
            &bob_spk_secret,
        );
        assert_eq!(secret_key, recv_sk);
        Ok(())
    }

    #[test]
    fn x3dh_key_agreement() -> Result<()> {
        let bob_ik = SigningKey::generate(&mut OsRng);
        let bob_spk = create_prekey_bundle(&bob_ik, 1);
        let bob_spk_secret = bob_spk.bundle[0].clone().0;
        let bob_spk = SignedPreKey {
            pre_key: bob_spk.bundle[0].1,
            signature: bob_spk.signature,
        };
        let alice_ik = SigningKey::generate(&mut OsRng);

        let X3DHSendKeyAgreement { ek, sk } =
            initiate_send_get_sk(bob_ik.verifying_key(), &bob_spk, None, &alice_ik)?;

        let recv_sk = initiate_recv_get_sk(
            &alice_ik.verifying_key(),
            ek,
            None,
            &bob_ik,
            &bob_spk_secret,
        );
        assert_eq!(sk, recv_sk);

        Ok(())
    }

    #[test]
    fn x3dh_send_recv_opk() -> Result<()> {
        // 1. Bob publishes his identity key and prekeys to a server.
        let bob_ik = SigningKey::generate(&mut OsRng);
        let bob_spk = create_prekey_bundle(&bob_ik, 1);
        let bob_spk_secret = bob_spk.bundle[0].clone().0;
        let bob_spk = SignedPreKey {
            pre_key: bob_spk.bundle[0].1,
            signature: bob_spk.signature,
        };
        let bob_opk_priv = X25519StaticSecret::random_from_rng(OsRng);
        let bob_opk_pub = X25519PublicKey::from(&bob_opk_priv);

        let alice_ik = SigningKey::generate(&mut OsRng);

        let plaintext = "Hello Bob!";
        // 2. Alice fetches a "prekey bundle" from the server, and uses it to send an initial message to Bob.
        let bundle = PreKeyBundle {
            ik: bob_ik.verifying_key(),
            opk: Some(bob_opk_pub),
            spk: bob_spk.clone(),
        };
        let (send_sk, message) = initiate_send(bundle, &alice_ik, plaintext.as_bytes())?;

        // 3. Bob receives and processes Alice's initial message.
        let (recv_sk, decrypted) = initiate_recv(
            &bob_ik,
            &bob_spk_secret,
            &message.sender_ik,
            message.ek,
            Some(bob_opk_priv),
            &message.ciphertext,
        )?;
        assert_eq!(send_sk, recv_sk);
        assert_eq!("Hello Bob!", String::from_utf8(decrypted)?);

        Ok(())
    }

    #[test]
    fn x3dh_send_recv() -> Result<()> {
        // 1. Bob publishes his identity key and prekeys to a server.
        let bob_ik = SigningKey::generate(&mut OsRng);
        let bob_spk = create_prekey_bundle(&bob_ik, 1);
        let bob_spk_secret = bob_spk.bundle[0].clone().0;
        let bob_spk = SignedPreKey {
            pre_key: bob_spk.bundle[0].1,
            signature: bob_spk.signature,
        };
        let alice_ik = SigningKey::generate(&mut OsRng);

        // 2. Alice fetches a "prekey bundle" from the server, and uses it to send an initial message to Bob.
        let bundle = PreKeyBundle {
            ik: bob_ik.verifying_key(),
            opk: None,
            spk: bob_spk.clone(),
        };
        let (send_sk, message) = initiate_send(bundle, &alice_ik, b"Hello Bob!")?;

        // 3. Bob receives and processes Alice's initial message.
        let (recv_sk, decrypted) = initiate_recv(
            &bob_ik,
            &bob_spk_secret,
            &message.sender_ik,
            message.ek,
            None,
            &message.ciphertext,
        )?;
        assert_eq!(send_sk, recv_sk);
        assert_eq!("Hello Bob!", String::from_utf8(decrypted)?);

        Ok(())
    }

    #[test]
    fn x3dh_invalid_bundle_signature() -> Result<()> {
        let bob_spk = create_prekey_bundle(&SigningKey::generate(&mut OsRng), 1);
        let bob_spk = SignedPreKey {
            pre_key: bob_spk.bundle[0].1,
            signature: bob_spk.signature,
        };

        let bundle = PreKeyBundle {
            ik: SigningKey::generate(&mut OsRng).verifying_key(),
            opk: None,
            spk: bob_spk.clone(),
        };
        assert_eq!(
            initiate_send(bundle, &SigningKey::generate(&mut OsRng), b"Hello Bob!"),
            Err(X3DHError::SignatureValidation)
        );

        Ok(())
    }

    #[test]
    fn x3dh_invalid_ciphertext() -> Result<()> {
        let bob_ik = SigningKey::generate(&mut OsRng);
        let bob_spk = create_prekey_bundle(&bob_ik, 1);
        let bob_spk_secret = bob_spk.bundle[0].clone().0;
        let bob_spk = SignedPreKey {
            pre_key: bob_spk.bundle[0].1,
            signature: bob_spk.signature,
        };
        let alice_ik = SigningKey::generate(&mut OsRng);

        let bundle = PreKeyBundle {
            ik: bob_ik.verifying_key(),
            opk: None,
            spk: bob_spk.clone(),
        };
        let (_, message) = initiate_send(bundle, &alice_ik, b"Hello Bob!")?;

        assert_eq!(
            initiate_recv(
                &bob_ik,
                &bob_spk_secret,
                &message.sender_ik,
                message.ek,
                None,
                b"invalid ciphertext",
            ),
            Err(X3DHError::Aead(AeadError::Tag(b'i')))
        );

        Ok(())
    }
}
