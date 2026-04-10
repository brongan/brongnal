use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};
use std::sync::{Arc, Mutex};
use tonic::transport::{Channel, ClientTlsConfig};
use sha2::{Digest, Sha256};
use rustls::RootCertStore;

#[derive(Debug)]
pub struct TlsRecorderVerifier {
    inner: Arc<dyn ServerCertVerifier>,
    pub pubkey_hash: Arc<Mutex<Option<Vec<u8>>>>,
}

impl TlsRecorderVerifier {
    pub fn new(pubkey_hash: Arc<Mutex<Option<Vec<u8>>>>) -> Self {
        let mut root_store = RootCertStore::empty();
        root_store.extend(
            webpki_roots::TLS_SERVER_ROOTS
                .iter()
                .cloned()
        );
        let inner = rustls::client::WebPkiServerVerifier::builder(Arc::new(root_store)).build().unwrap();
        Self {
            inner,
            pubkey_hash,
        }
    }
    
    fn compute_pubkey_hash(cert_der: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(cert_der);
        hasher.finalize().to_vec()
    }
}

impl ServerCertVerifier for TlsRecorderVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let hash = Self::compute_pubkey_hash(end_entity.as_ref());
        let mut guard = self.pubkey_hash.lock().unwrap();
        *guard = Some(hash);

        self.inner.verify_server_cert(end_entity, intermediates, server_name, ocsp_response, now)
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

use tonic::transport::Endpoint;
use tower::Service;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tonic::transport::Uri;

#[derive(Clone)]
pub struct CustomTlsConnector {
    config: Arc<ClientConfig>,
}

impl Service<Uri> for CustomTlsConnector {
    type Response = tokio_rustls::client::TlsStream<TcpStream>;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Uri) -> Self::Future {
        let config = self.config.clone();
        Box::pin(async move {
            let host = req.host().unwrap_or("").to_string();
            let port = req.port_u16().unwrap_or(443);
            let addr = format!("{}:{}", host, port);
            let tcp = TcpStream::connect(addr).await?;
            let domain = ServerName::try_from(host).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid DNS name")
            })?;
            let connector = TlsConnector::from(config);
            connector.connect(domain, tcp).await
        })
    }
}

pub struct AttestedChannel {
    pub channel: Channel,
    pub captured_hash: Arc<Mutex<Option<Vec<u8>>>>,
}

impl AttestedChannel {
    pub async fn connect(dst: String) -> Result<Self, tonic::transport::Error> {
        let captured_hash = Arc::new(Mutex::new(None));
        let verifier = TlsRecorderVerifier::new(captured_hash.clone());

        let config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(verifier))
            .with_no_client_auth();
            
        let mut config = config;
        config.alpn_protocols = vec![b"h2".to_vec()]; // Required for HTTP/2 (gRPC)

        let connector = CustomTlsConnector {
            config: Arc::new(config),
        };
            
        let endpoint = Endpoint::from_shared(dst)
            .map_err(|e| tonic::transport::Error::from(e))?;
        let channel = endpoint.connect_with_connector(connector).await?;

        Ok(Self {
            channel,
            captured_hash,
        })
    }
}
