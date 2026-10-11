//! Signs with the upload key using the YubiKey daemon's ssh-agent socket.

use pkcs1::der::Decode;
use ssh_agent_lib::agent::Session;
use ssh_agent_lib::client::Client;
use ssh_agent_lib::error::AgentError;
use ssh_agent_lib::proto::{PublicCredential, SignRequest, RSA_SHA2_256};
use ssh_agent_lib::ssh_key::public::{KeyData, RsaPublicKey};
use ssh_agent_lib::ssh_key::{Algorithm, HashAlg, Mpint};
use std::path::PathBuf;
use tokio::net::UnixStream;
use x509_cert::Certificate;

const SOCKET: &str = "com.brongan.age-plugin-yubikey-daemon/signing.sock";

#[derive(Debug, thiserror::Error)]
pub enum SignError {
    #[error("the upload certificate has no RSA key: {0}")]
    NotRsa(#[source] pkcs1::Error),
    #[error("the upload certificate's RSA key is malformed: {0}")]
    MalformedKey(#[source] ssh_agent_lib::ssh_key::Error),
    #[error("no runtime directory found, so the signing socket is unavailable")]
    NoRuntimeDir,
    #[error("could not connect to {path}: {source}")]
    Connect {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not get the agent's keys: {0}")]
    Identities(#[source] AgentError),
    #[error("the agent does not have the upload key (is the daemon running with --sign-slot?)")]
    MissingKey,
    #[error("the agent refused to sign: {0}")]
    Sign(#[source] AgentError),
    #[error("the agent returned a {0} signature instead of rsa-sha2-256")]
    WrongAlgorithm(Algorithm),
}

/// Returns a SHA256withRSA PKCS#1 v1.5 signature for `data`, made by the key
/// in `certificate`.
pub async fn sign(certificate: &Certificate, data: Vec<u8>) -> Result<Vec<u8>, SignError> {
    let key = ssh_key(certificate)?;
    let path = dirs::runtime_dir()
        .ok_or(SignError::NoRuntimeDir)?
        .join(SOCKET);
    let stream = UnixStream::connect(&path)
        .await
        .map_err(|source| SignError::Connect { path, source })?;
    let mut agent = Client::new(stream);

    let identities = agent
        .request_identities()
        .await
        .map_err(SignError::Identities)?;
    let credential = identities
        .into_iter()
        .map(|identity| identity.credential)
        .find(|credential| matches!(credential, PublicCredential::Key(k) if *k == key))
        .ok_or(SignError::MissingKey)?;
    let signature = agent
        .sign(SignRequest {
            credential,
            data,
            flags: RSA_SHA2_256,
        })
        .await
        .map_err(SignError::Sign)?;
    match signature.algorithm() {
        Algorithm::Rsa {
            hash: Some(HashAlg::Sha256),
        } => Ok(signature.as_bytes().to_vec()),
        other => Err(SignError::WrongAlgorithm(other)),
    }
}

fn ssh_key(certificate: &Certificate) -> Result<KeyData, SignError> {
    let spki = &certificate.tbs_certificate.subject_public_key_info;
    let key = pkcs1::RsaPublicKey::from_der(spki.subject_public_key.raw_bytes())
        .map_err(|e| SignError::NotRsa(e.into()))?;
    Ok(KeyData::Rsa(RsaPublicKey {
        e: Mpint::from_positive_bytes(key.public_exponent.as_bytes())
            .map_err(SignError::MalformedKey)?,
        n: Mpint::from_positive_bytes(key.modulus.as_bytes()).map_err(SignError::MalformedKey)?,
    }))
}
