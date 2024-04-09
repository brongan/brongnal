use anyhow::{Context, Result};
use chacha20poly1305::aead::OsRng;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use ed25519_dalek::SigningKey;
use protocol::bundle::{create_prekey_bundle, sign_bundle};
use protocol::x3dh::{x3dh_initiate_recv, x3dh_initiate_send, SignedPreKey, SignedPreKeys};
use rustls::pki_types::ServerName;
use server::X3DHServerClient;
use std::collections::HashMap;
use tarpc::tokio_serde::formats::Bincode;
use tarpc::{client, context};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

pub trait X3DHClient {
    fn fetch_wipe_one_time_secret_key(
        &mut self,
        one_time_key: &X25519PublicKey,
    ) -> Result<X25519StaticSecret, anyhow::Error>;
    fn get_identity_key(&self) -> Result<SigningKey, anyhow::Error>;
    fn get_pre_key(&mut self) -> Result<X25519StaticSecret, anyhow::Error>;
    fn get_spk(&self) -> Result<SignedPreKey, anyhow::Error>;
    fn add_one_time_keys(&mut self, num_keys: u32) -> SignedPreKeys;
}

struct SessionKeys<T> {
    session_keys: HashMap<T, [u8; 32]>,
}

impl<Identity: Eq + std::hash::Hash> SessionKeys<Identity> {
    fn set_session_key(&mut self, recipient_identity: Identity, secret_key: &[u8; 32]) {
        self.session_keys.insert(recipient_identity, *secret_key);
    }

    fn get_encryption_key(&mut self, recipient_identity: &Identity) -> Result<ChaCha20Poly1305> {
        let key = self
            .session_keys
            .get(recipient_identity)
            .context("Session key not found.")?;
        Ok(ChaCha20Poly1305::new_from_slice(key).unwrap())
    }

    fn destroy_session_key(&mut self, peer: &Identity) {
        self.session_keys.remove(peer);
    }
}

pub struct MemoryClient {
    identity_key: SigningKey,
    pre_key: X25519StaticSecret,
    one_time_pre_keys: HashMap<X25519PublicKey, X25519StaticSecret>,
}

impl Default for MemoryClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryClient {
    pub fn new() -> Self {
        Self {
            identity_key: SigningKey::generate(&mut OsRng),
            pre_key: X25519StaticSecret::random_from_rng(OsRng),
            one_time_pre_keys: HashMap::new(),
        }
    }
}

impl X3DHClient for MemoryClient {
    fn fetch_wipe_one_time_secret_key(
        &mut self,
        one_time_key: &X25519PublicKey,
    ) -> Result<X25519StaticSecret> {
        self.one_time_pre_keys
            .remove(one_time_key)
            .context("Client failed to find pre key.")
    }

    fn get_identity_key(&self) -> Result<SigningKey> {
        Ok(self.identity_key.clone())
    }

    fn get_pre_key(&mut self) -> Result<X25519StaticSecret> {
        Ok(self.pre_key.clone())
    }

    fn get_spk(&self) -> Result<SignedPreKey> {
        Ok(SignedPreKey {
            pre_key: X25519PublicKey::from(&self.pre_key),
            signature: sign_bundle(
                &self.identity_key,
                &[(self.pre_key.clone(), X25519PublicKey::from(&self.pre_key))],
            ),
        })
    }

    fn add_one_time_keys(&mut self, num_keys: u32) -> SignedPreKeys {
        let otks = create_prekey_bundle(&self.identity_key, num_keys);
        let pre_keys = otks.bundle.iter().map(|(_, _pub)| *_pub).collect();
        for otk in otks.bundle {
            self.one_time_pre_keys.insert(otk.1, otk.0);
        }
        SignedPreKeys {
            pre_keys,
            signature: otks.signature,
        }
    }
}

async fn connect_tcp(domain: String, port: u16) -> Result<TlsStream<TcpStream>, std::io::Error> {
    use std::sync::Arc;
    let host = format!("{}:{}", &domain, port);

    let root_store =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let servername = ServerName::try_from(domain).unwrap();

    let stream = TcpStream::connect(host).await?;
    connector.connect(servername, stream).await
}

#[tokio::main]
async fn main() -> Result<()> {
    let stream = connect_tcp("brongnal.brongan.com".to_string(), 8080).await?;
    let transport = tarpc::serde_transport::Transport::from((stream, Bincode::default()));

    let rpc_client = X3DHServerClient::new(client::Config::default(), transport).spawn();

    let mut bob = MemoryClient::new();
    rpc_client
        .set_spk(
            context::current(),
            "Bob".to_owned(),
            bob.get_identity_key()?.verifying_key(),
            bob.get_spk()?,
        )
        .await??;

    rpc_client
        .publish_otk_bundle(
            context::current(),
            "Bob".to_owned(),
            bob.get_identity_key()?.verifying_key(),
            bob.add_one_time_keys(100),
        )
        .await??;

    let bundle = rpc_client
        .fetch_prekey_bundle(context::current(), "Bob".to_owned())
        .await??;

    let alice = MemoryClient::new();
    let (_send_sk, message) = x3dh_initiate_send(bundle, &alice.get_identity_key()?, b"Hi Bob")?;
    rpc_client
        .send_message(context::current(), "Bob".to_owned(), message)
        .await??;

    let messages = rpc_client
        .retrieve_messages(context::current(), "Bob".to_owned())
        .await?;
    let message = &messages.first().unwrap();

    let (_recv_sk, msg) = x3dh_initiate_recv(
        &bob.get_identity_key()?.clone(),
        &bob.get_pre_key()?.clone(),
        &message.sender_identity_key,
        message.ephemeral_key,
        message
            .otk
            .map(|otk_pub| bob.fetch_wipe_one_time_secret_key(&otk_pub).unwrap()),
        &message.ciphertext,
    )?;

    println!("Alice sent to Bob: {}", String::from_utf8(msg)?);

    Ok(())
}
