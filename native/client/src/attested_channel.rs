use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme};
use sha2::{Digest, Sha256};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::Service;

/// A TLS certificate verifier that delegates to the standard WebPKI verifier
/// but also records the SHA-256 hash of the leaf certificate's DER encoding.
///
/// This hash is used for aTLS (attested TLS) binding: the server includes
/// it in its GCA attestation token's `eat_nonce`, proving that the attested
/// workload controls the TLS private key.
#[derive(Debug)]
struct CertRecordingVerifier {
    inner: Arc<dyn ServerCertVerifier>,
    cert_hash: Arc<Mutex<Option<Vec<u8>>>>,
}

impl CertRecordingVerifier {
    fn new(cert_hash: Arc<Mutex<Option<Vec<u8>>>>) -> Self {
        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let inner = rustls::client::WebPkiServerVerifier::builder(Arc::new(root_store))
            .build()
            .unwrap();
        Self { inner, cert_hash }
    }
}

impl ServerCertVerifier for CertRecordingVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        // Record the leaf cert hash before delegating to standard verification.
        let hash = Sha256::digest(end_entity.as_ref()).to_vec();
        *self.cert_hash.lock().unwrap() = Some(hash);

        self.inner
            .verify_server_cert(end_entity, intermediates, server_name, ocsp_response, now)
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

/// Tower `Service<Uri>` that performs TLS over a raw TCP connection.
///
/// Required because tonic's built-in TLS doesn't expose the rustls
/// `ClientConfig` at a level where we can inject a custom `ServerCertVerifier`.
#[derive(Clone)]
struct TlsConnectorService {
    config: Arc<ClientConfig>,
}

impl Service<Uri> for TlsConnectorService {
    type Response = tokio_rustls::client::TlsStream<TcpStream>;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let config = self.config.clone();
        Box::pin(async move {
            let host = uri.host().unwrap_or("").to_string();
            let port = uri.port_u16().unwrap_or(443);
            let tcp = TcpStream::connect((host.as_str(), port)).await?;
            let domain = ServerName::try_from(host).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid DNS name")
            })?;
            TlsConnector::from(config).connect(domain, tcp).await
        })
    }
}

/// Connect to a gRPC endpoint over TLS while capturing the server's leaf
/// certificate hash for subsequent attestation verification.
///
/// Returns `(channel, cert_hash)` where `cert_hash` is the SHA-256 of the
/// DER-encoded leaf certificate observed during the TLS handshake.
pub async fn connect(dst: String) -> Result<(Channel, Vec<u8>), tonic::transport::Error> {
    let cert_hash: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));
    let verifier = CertRecordingVerifier::new(cert_hash.clone());

    // .dangerous() is required by rustls to set any custom ServerCertVerifier,
    // even though ours fully delegates to the standard WebPkiServerVerifier.
    // There's no "wrap the default with a hook" API — this is the only way to
    // inject the cert-hash recording side effect.
    let mut tls_config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth();
    tls_config.alpn_protocols = vec![b"h2".to_vec()];

    let connector = TlsConnectorService {
        config: Arc::new(tls_config),
    };

    let channel = Endpoint::from_shared(dst)?
        .connect_with_connector(connector)
        .await?;

    // The TLS handshake completed during connect_with_connector, so the hash
    // is guaranteed to be populated.
    let hash = cert_hash
        .lock()
        .unwrap()
        .take()
        .expect("TLS handshake completed but cert hash was not recorded");

    Ok((channel, hash))
}
