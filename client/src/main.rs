use anyhow::{Context, Result};
use chacha20poly1305::aead::OsRng;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use ed25519_dalek::SigningKey;
use futures::executor::block_on;
use protocol::bundle::{create_prekey_bundle, sign_bundle};
use protocol::x3dh::{x3dh_initiate_recv, x3dh_initiate_send, SignedPreKey, SignedPreKeys};
use rustls::pki_types::ServerName;
use server::X3DHServerClient;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tarpc::tokio_serde::formats::Bincode;
use tarpc::{client, context};
use tokio::net::TcpStream;
use tokio::runtime::Handle;
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

use tokio::sync::OnceCell;
static STUB: OnceCell<X3DHServerClient> = OnceCell::const_new();

async fn create_client() -> X3DHServerClient {
    let stream = connect_tcp("signal.brongan.com".to_string(), 8080)
        .await
        .unwrap();
    let transport = tarpc::serde_transport::Transport::from((stream, Bincode::default()));
    return X3DHServerClient::new(client::Config::default(), transport).spawn();
}

struct MyApp {
    client: Arc<Mutex<MemoryClient>>,
    name: String,
    stub: &'static X3DHServerClient,
}

impl MyApp {
    fn new(stub: &'static X3DHServerClient) -> Result<Self> {
        Ok(MyApp {
            client: Arc::new(Mutex::new(MemoryClient::default())),
            name: String::default(),
            stub,
        })
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Brongnal Desktop");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            if ui.button("Register").clicked() {
                let name = self.name.clone();
                let client = self.client.lock().unwrap();
                let ik = client.get_identity_key().unwrap().verifying_key();
                let spk = client.get_spk().unwrap();
                let stub = self.stub;

                tokio::spawn(async move {
                    eprintln!("Registering: {name}");
                    stub.set_spk(context::current(), name.clone(), ik, spk)
                        .await
                        .unwrap()
                        .unwrap();
                    eprintln!("Registered: {name}");
                });
            }
            ui.label(format!("Hello '{}'", self.name));
        });
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<(), eframe::Error> {
    let stub = STUB.get_or_init(create_client).await;
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Brongnal Desktop",
        native_options,
        Box::new(|_cc| Box::new(MyApp::new(stub).unwrap())),
    )
    .into()

    /*
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
    */
}
