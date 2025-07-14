use blake2::{Blake2b512, Digest};
use chacha20poly1305::aead::OsRng;
use x25519_dalek::{
    EphemeralSecret as X25519EphemeralSecret, PublicKey as X25519PublicKey,
    ReusableSecret as X25519ReusableSecret, SharedSecret, StaticSecret as X25519StaticSecret,
};

struct RootKey([u8; 32]);
struct ChainKey([u8; 32]);
struct MessageKey([u8; 32]);

// GENERATE_DH(): Returns a new Diffie-Hellman key pair.
fn generate_dh() -> X25519EphemeralSecret {
    return X25519EphemeralSecret::random_from_rng(OsRng);
}

// DH(dh_pair, dh_pub): Returns the output from the Diffie-Hellman calculation between the private key from the DH key pair dh_pair and the DH public key dh_pub. If the DH function rejects invalid public keys, then this function may raise an exception which terminates processing.
fn dh(dh_pair: X25519EphemeralSecret, dh_pub: &X25519PublicKey) -> SharedSecret {
    dh_pair.diffie_hellman(dh_pub)
}

impl RootKey {
    // KDF_RK(rk, dh_out): Returns a pair (32-byte root key, 32-byte chain key) as the output of applying a KDF keyed by a 32-byte root key rk to a Diffie-Hellman output dh_out.
    fn kdf_rk(self, dh_out: SharedSecret) -> (RootKey, ChainKey) {
        let digest = Blake2b512::new()
            .chain_update(b"RootKeyConstant")
            .chain_update(&self.0)
            .finalize();
        // TODO - figure out how to avoid the copies here?
        let (l, r) = digest.split_at(32);
        (
            RootKey(l.try_into().unwrap()),
            ChainKey(r.try_into().unwrap()),
        )
    }
}

// Symmetric Ratchet
impl ChainKey {
    // KDF_CK(ck): Returns a pair (32-byte chain key, 32-byte message key) as the output of applying a KDF keyed by a 32-byte chain key ck to some constant.
    fn kdf_ck(self) -> (Self, MessageKey) {
        let digest = Blake2b512::new()
            .chain_update(b"ChainKeyConstant?")
            .chain_update(&self.0)
            .finalize();
        // TODO - figure out how to avoid the copies here?
        let (l, r) = digest.split_at(32);
        (
            ChainKey(l.try_into().unwrap()),
            MessageKey(r.try_into().unwrap()),
        )
    }
}

// TODO(https://github.com/brongan/brongnal/issues/7) - Implement ratcheting.
// https://signal.org/docs/specifications/doubleratchet/
