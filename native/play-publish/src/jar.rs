//! v1 JAR signature: the format `jarsigner` creates and Play accepts for app bundles.

use base64::prelude::{Engine, BASE64_STANDARD};
use cms::cert::{CertificateChoices, IssuerAndSerialNumber};
use cms::content_info::{CmsVersion, ContentInfo};
use cms::signed_data::{
    CertificateSet, EncapsulatedContentInfo, SignedData, SignerIdentifier, SignerInfo, SignerInfos,
};
use const_oid::db::rfc5911::{ID_DATA, ID_SIGNED_DATA};
use const_oid::db::rfc5912::{ID_SHA_256, SHA_256_WITH_RSA_ENCRYPTION};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read, Write};
use x509_cert::der::asn1::{OctetString, SetOfVec};
use x509_cert::der::{Any, Encode};
use x509_cert::spki::AlgorithmIdentifierOwned;
use x509_cert::Certificate;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const MANIFEST: &str = "META-INF/MANIFEST.MF";
const SIGNATURE_FILE: &str = "META-INF/CERT.SF";
const SIGNATURE_BLOCK: &str = "META-INF/CERT.RSA";

#[derive(Debug, thiserror::Error)]
pub enum JarError {
    #[error("could not read the bundle: {0}")]
    Read(#[source] zip::result::ZipError),
    #[error("could not read {name} from the bundle: {source}")]
    ReadEntry {
        name: String,
        #[source]
        source: std::io::Error,
    },
    #[error("the bundle is already signed: it contains {0}")]
    AlreadySigned(String),
    #[error("could not encode the signature block: {0}")]
    Encode(#[source] x509_cert::der::Error),
    #[error("could not write the signed bundle: {0}")]
    Write(#[source] zip::result::ZipError),
    #[error("could not write {name} to the signed bundle: {source}")]
    WriteEntry {
        name: &'static str,
        #[source]
        source: std::io::Error,
    },
}

/// `MANIFEST.MF` and `CERT.SF` for a bundle. The signature covers `sf`.
pub struct SignatureFiles {
    pub manifest: Vec<u8>,
    pub sf: Vec<u8>,
}

/// Creates a digest for every file in `bundle`, in archive order.
pub fn signature_files(bundle: &[u8]) -> Result<SignatureFiles, JarError> {
    let mut archive = ZipArchive::new(Cursor::new(bundle)).map_err(JarError::Read)?;
    let mut manifest = String::new();
    header(&mut manifest, "Manifest-Version", "1.0");
    manifest.push_str("\r\n");
    let mut sections = String::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(JarError::Read)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().map_err(JarError::Read)?.into_owned();
        if name.starts_with("META-INF/") {
            return Err(JarError::AlreadySigned(name));
        }
        let mut contents = Vec::new();
        entry
            .read_to_end(&mut contents)
            .map_err(|source| JarError::ReadEntry {
                name: name.clone(),
                source,
            })?;

        let mut section = String::new();
        header(&mut section, "Name", &name);
        header(&mut section, "SHA-256-Digest", &digest(&contents));
        section.push_str("\r\n");
        header(&mut sections, "Name", &name);
        header(&mut sections, "SHA-256-Digest", &digest(section.as_bytes()));
        sections.push_str("\r\n");
        manifest.push_str(&section);
    }

    let mut sf = String::new();
    header(&mut sf, "Signature-Version", "1.0");
    header(
        &mut sf,
        "SHA-256-Digest-Manifest",
        &digest(manifest.as_bytes()),
    );
    sf.push_str("\r\n");
    sf.push_str(&sections);
    Ok(SignatureFiles {
        manifest: manifest.into_bytes(),
        sf: sf.into_bytes(),
    })
}

/// `CERT.RSA`: a detached CMS SignedData containing `certificate` and its
/// SHA256withRSA `signature` of `CERT.SF`, with no signed attributes.
pub fn signature_block(certificate: Certificate, signature: &[u8]) -> Result<Vec<u8>, JarError> {
    let sha256 = AlgorithmIdentifierOwned {
        oid: ID_SHA_256,
        parameters: None,
    };
    let signer = SignerInfo {
        version: CmsVersion::V1,
        sid: SignerIdentifier::IssuerAndSerialNumber(IssuerAndSerialNumber {
            issuer: certificate.tbs_certificate.issuer.clone(),
            serial_number: certificate.tbs_certificate.serial_number.clone(),
        }),
        digest_alg: sha256.clone(),
        signed_attrs: None,
        signature_algorithm: AlgorithmIdentifierOwned {
            oid: SHA_256_WITH_RSA_ENCRYPTION,
            parameters: Some(Any::null()),
        },
        signature: OctetString::new(signature).map_err(JarError::Encode)?,
        unsigned_attrs: None,
    };
    let signed_data = SignedData {
        version: CmsVersion::V1,
        digest_algorithms: SetOfVec::try_from(vec![sha256]).map_err(JarError::Encode)?,
        encap_content_info: EncapsulatedContentInfo {
            econtent_type: ID_DATA,
            econtent: None,
        },
        certificates: Some(CertificateSet(
            SetOfVec::try_from(vec![CertificateChoices::Certificate(certificate)])
                .map_err(JarError::Encode)?,
        )),
        crls: None,
        signer_infos: SignerInfos(SetOfVec::try_from(vec![signer]).map_err(JarError::Encode)?),
    };
    ContentInfo {
        content_type: ID_SIGNED_DATA,
        content: Any::encode_from(&signed_data).map_err(JarError::Encode)?,
    }
    .to_der()
    .map_err(JarError::Encode)
}

/// Copies `bundle` as is after the three signature entries.
pub fn signed_bundle(
    bundle: &[u8],
    files: &SignatureFiles,
    block: &[u8],
) -> Result<Vec<u8>, JarError> {
    let mut archive = ZipArchive::new(Cursor::new(bundle)).map_err(JarError::Read)?;
    let mut writer = ZipWriter::new(Cursor::new(Vec::with_capacity(bundle.len())));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, contents) in [
        (MANIFEST, files.manifest.as_slice()),
        (SIGNATURE_FILE, files.sf.as_slice()),
        (SIGNATURE_BLOCK, block),
    ] {
        writer.start_file(name, options).map_err(JarError::Write)?;
        writer
            .write_all(contents)
            .map_err(|source| JarError::WriteEntry { name, source })?;
    }
    for i in 0..archive.len() {
        let entry = archive.by_index_raw(i).map_err(JarError::Read)?;
        writer.raw_copy_file(entry).map_err(JarError::Write)?;
    }
    Ok(writer.finish().map_err(JarError::Write)?.into_inner())
}

fn digest(bytes: &[u8]) -> String {
    BASE64_STANDARD.encode(Sha256::digest(bytes))
}

/// Adds a manifest header, split into lines of at most 72 bytes; continuation
/// lines start with a space.
fn header(out: &mut String, name: &str, value: &str) {
    let line = format!("{name}: {value}");
    let mut rest = line.as_str();
    let mut width = 72;
    loop {
        let mut end = rest.len().min(width);
        while !rest.is_char_boundary(end) {
            end -= 1;
        }
        out.push_str(&rest[..end]);
        out.push_str("\r\n");
        rest = &rest[end..];
        if rest.is_empty() {
            return;
        }
        out.push(' ');
        width = 71;
    }
}

#[cfg(test)]
#[path = "jar_tests.rs"]
mod tests;
