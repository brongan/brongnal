#![allow(dead_code)]
use blake2::{Blake2b512, Digest};

mod aead;
pub mod bundle;
pub mod gossamer;
pub mod x3dh;

// TODO(https://github.com/brongan/brongnal/issues/7) - Implement ratcheting.
fn ratchet(key: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let mut hasher = Blake2b512::new();
    hasher.update(key);
    let blake2b_mac = hasher.finalize();
    let mut l = [0; 32];
    let mut r = [0; 32];
    l.clone_from_slice(&blake2b_mac[0..32]);
    r.clone_from_slice(&blake2b_mac[32..]);
    (l, r)
}
