//! Signs an app bundle with the YubiKey upload key and uploads it to closed
//! testing (alpha). Play requires production access for production releases.
//!
//! Uses the caller’s `gcloud auth application-default login` credentials to
//! act as the play-publisher service account, so no key file is needed.

mod agent;
mod jar;

use google_androidpublisher3::api::{Track, TrackRelease};
use google_androidpublisher3::hyper_rustls::HttpsConnectorBuilder;
use google_androidpublisher3::hyper_util::client::legacy::Client;
use google_androidpublisher3::hyper_util::rt::TokioExecutor;
use google_androidpublisher3::yup_oauth2::client::CustomHyperClientBuilder;
use google_androidpublisher3::yup_oauth2::{
    read_authorized_user_secret, ServiceAccountImpersonationAuthenticator,
};
use google_androidpublisher3::AndroidPublisher;
use std::io::Cursor;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::info;
use tracing_subscriber::EnvFilter;
use x509_cert::der::DecodePem;
use x509_cert::Certificate;

const PACKAGE: &str = "com.brongan.brongnal_app";
const SERVICE_ACCOUNT: &str = "play-publisher@brongnal.iam.gserviceaccount.com";
const DEFAULT_BUNDLE: &str = "build/app/outputs/bundle/release/app-release.aab";
const TRACK: &str = "alpha";

#[derive(Debug, thiserror::Error)]
enum PublishError {
    #[error("could not read {path}: {source}")]
    ReadBundle {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not read {path} (run `ykman piv certificates export 9c {path}`): {source}")]
    ReadCertificate {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse {path}: {source}")]
    Certificate {
        path: PathBuf,
        #[source]
        source: x509_cert::der::Error,
    },
    #[error(transparent)]
    Jar(#[from] jar::JarError),
    #[error(transparent)]
    Sign(#[from] agent::SignError),
    #[error("no user config directory found, so credentials are unavailable")]
    NoConfigDir,
    #[error("could not read {path} (run `gcloud auth application-default login`): {source}")]
    ReadCredentials {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not load TLS root certificates: {0}")]
    TlsRoots(#[source] std::io::Error),
    #[error("could not build the service account authenticator: {0}")]
    Authenticator(#[source] std::io::Error),
    #[error("could not open an edit: {0}")]
    OpenEdit(#[source] Box<google_androidpublisher3::Error>),
    #[error("Play returned an edit with no id")]
    MissingEditId,
    #[error("could not upload the bundle: {0}")]
    UploadBundle(#[source] Box<google_androidpublisher3::Error>),
    #[error("Play returned an uploaded bundle with no version code")]
    MissingVersionCode,
    #[error("could not assign the release to {TRACK}: {0}")]
    AssignTrack(#[source] Box<google_androidpublisher3::Error>),
    #[error("could not commit the edit: {0}")]
    CommitEdit(#[source] Box<google_androidpublisher3::Error>),
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .with_writer(std::io::stderr)
        .with_target(false)
        .without_time()
        .init();
    match publish().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn publish() -> Result<(), PublishError> {
    let path: PathBuf = std::env::args()
        .nth(1)
        .unwrap_or(DEFAULT_BUNDLE.into())
        .into();
    info!("reading {}", path.display());
    let bundle = tokio::fs::read(&path)
        .await
        .map_err(|source| PublishError::ReadBundle { path, source })?;
    let bundle = sign(&bundle).await?;

    let play = publisher().await?;
    let edits = play.edits();
    info!("opening an edit for {PACKAGE}");
    let (_, edit) = edits
        .insert(Default::default(), PACKAGE)
        .doit()
        .await
        .map_err(|e| PublishError::OpenEdit(e.into()))?;
    let edit = edit.id.ok_or(PublishError::MissingEditId)?;

    info!("uploading {} MB", bundle.len() / 1_000_000);
    let (_, uploaded) = edits
        .bundles_upload(PACKAGE, &edit)
        .upload_resumable(Cursor::new(bundle), mime::APPLICATION_OCTET_STREAM)
        .await
        .map_err(|e| PublishError::UploadBundle(e.into()))?;
    let version_code = uploaded
        .version_code
        .ok_or(PublishError::MissingVersionCode)?;

    info!("assigning versionCode {version_code} to {TRACK}");
    edits
        .tracks_update(release(version_code.into()), PACKAGE, &edit, TRACK)
        .doit()
        .await
        .map_err(|e| PublishError::AssignTrack(e.into()))?;
    info!("committing the edit");
    edits
        .commit(PACKAGE, &edit)
        .doit()
        .await
        .map_err(|e| PublishError::CommitEdit(e.into()))?;
    info!("published {version_code} to {TRACK}");
    println!("{version_code}");
    Ok(())
}

async fn sign(bundle: &[u8]) -> Result<Vec<u8>, PublishError> {
    let path = dirs::config_dir()
        .ok_or(PublishError::NoConfigDir)?
        .join("play-publish/upload-certificate.pem");
    let pem = match tokio::fs::read(&path).await {
        Ok(pem) => pem,
        Err(source) => return Err(PublishError::ReadCertificate { path, source }),
    };
    let certificate =
        Certificate::from_pem(&pem).map_err(|source| PublishError::Certificate { path, source })?;
    let files = jar::signature_files(bundle)?;
    let signature = agent::sign(&certificate, files.sf.clone()).await?;
    let block = jar::signature_block(certificate, &signature)?;
    Ok(jar::signed_bundle(bundle, &files, &block)?)
}

async fn publisher(
) -> Result<AndroidPublisher<impl google_androidpublisher3::common::Connector>, PublishError> {
    let path = dirs::config_dir()
        .ok_or(PublishError::NoConfigDir)?
        .join("gcloud/application_default_credentials.json");
    let secret = read_authorized_user_secret(&path)
        .await
        .map_err(|source| PublishError::ReadCredentials { path, source })?;

    let connector = HttpsConnectorBuilder::new()
        .with_native_roots()
        .map_err(PublishError::TlsRoots)?
        .https_only()
        .enable_http2()
        .build();
    // The authenticator and hub send different body types, so each needs a
    // client using the shared connector.
    let auth = ServiceAccountImpersonationAuthenticator::with_client(
        secret,
        SERVICE_ACCOUNT,
        CustomHyperClientBuilder::from(
            Client::builder(TokioExecutor::new()).build(connector.clone()),
        ),
    )
    .build()
    .await
    .map_err(PublishError::Authenticator)?;
    Ok(AndroidPublisher::new(
        Client::builder(TokioExecutor::new()).build(connector),
        auth,
    ))
}

/// Replaces all releases on the track with `version_code`, marked as fully rolled out.
fn release(version_code: i64) -> Track {
    Track {
        track: Some(TRACK.into()),
        releases: Some(vec![TrackRelease {
            version_codes: Some(vec![version_code]),
            status: Some("completed".into()),
            ..Default::default()
        }]),
    }
}
