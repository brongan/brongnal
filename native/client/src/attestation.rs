use crate::trusted_digests::TRUSTED_IDENTITY_IMAGE_DIGESTS;
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, Algorithm, DecodingKey, Validation};
use proto::gossamer::AttestationResponse;
use serde::Deserialize;
use thiserror::Error;
use tracing::info;

/// GCA JWKS endpoint for verifying token signatures.
const GCA_JWKS_URL: &str =
    "https://www.googleapis.com/service_accounts/v1/metadata/jwk/signer@confidentialspace-sign.iam.gserviceaccount.com";

/// Expected issuer for GCA tokens.
const GCA_ISSUER: &str = "https://confidentialcomputing.googleapis.com";

#[derive(Error, Debug)]
pub enum AttestationError {
    #[error("Missing attestation field: {0}")]
    MissingField(&'static str),
    #[error("Container image digest is untrusted: {0}")]
    UntrustedContainer(String),
    #[error("Mismatched TLS binding: expected {expected:?}, got {actual:?}")]
    TlsBindingMismatch { expected: Vec<u8>, actual: Vec<u8> },
    #[error("GCA token verification failed: {0}")]
    TokenVerification(String),
    #[error("Invalid hardware model: {0}")]
    InvalidHardware(String),
    #[error("Secure boot not enabled")]
    InsecureBoot,
    #[error("Debug enabled on CVM")]
    DebugEnabled(String),
    #[error("JWKS fetch failed: {0}")]
    JwksFetch(String),
}

/// Nested claims for the running container (Confidential Space).
/// See: https://cloud.google.com/confidential-computing/confidential-vm/docs/token-claims
#[derive(Debug, Deserialize)]
pub struct ContainerClaims {
    /// The container image digest as `sha256:<hex>`, e.g. "sha256:abc123..."
    pub image_digest: Option<String>,
}

/// Top-level submods block in the GCA token.
#[derive(Debug, Deserialize)]
pub struct Submods {
    pub container: Option<ContainerClaims>,
}

/// Claims from a GCA OIDC token.
/// See: https://cloud.google.com/confidential-computing/confidential-vm/docs/token-claims
#[derive(Debug, Deserialize)]
pub struct GcaClaims {
    pub iss: String,
    pub eat_nonce: Vec<String>,
    pub secboot: bool,
    pub hwmodel: String,
    pub dbgstat: String,
    /// Workload-specific sub-claims (container digest lives here).
    pub submods: Option<Submods>,
}

pub struct AttestationVerifier;

impl AttestationVerifier {
    /// Verify the GCA OIDC JWT:
    /// 1. Verify JWT signature via GCA JWKS endpoint (Google-signed)
    /// 2. Check iss == confidentialcomputing.googleapis.com, hwmodel, secboot, dbgstat
    /// 3. Check submods.container.image_digest ∈ TRUSTED_DIGESTS (from the signed JWT)
    /// 4. Check eat_nonce in the JWT contains SHA-256(observed TLS cert pubkey) (aTLS binding)
    ///
    /// The client does NOT parse raw vTPM quotes or SNP reports.
    /// All hardware evidence verification is delegated to Google Cloud Attestation.
    /// The container digest is read from the signed JWT, not from the server's response.
    pub async fn verify(
        &self,
        response: &AttestationResponse,
        actual_tls_pubkey_hash: &[u8],
    ) -> Result<(), AttestationError> {
        // 1. Verify GCA JWT (signature + standard claims)
        let token = response
            .gca_token
            .as_ref()
            .ok_or(AttestationError::MissingField("gca_token"))?;

        let claims = self.verify_gca_jwt(token).await?;

        // 2. Check hardware claims
        if claims.hwmodel != "GCP_AMD_SEV" {
            return Err(AttestationError::InvalidHardware(claims.hwmodel));
        }
        if !claims.secboot {
            return Err(AttestationError::InsecureBoot);
        }
        if claims.dbgstat != "disabled-since-boot" {
            return Err(AttestationError::DebugEnabled(claims.dbgstat));
        }

        // 3. Container digest from the signed JWT (not from the server's response field)
        let image_digest = claims
            .submods
            .as_ref()
            .and_then(|s| s.container.as_ref())
            .and_then(|c| c.image_digest.as_deref())
            .ok_or(AttestationError::MissingField("submods.container.image_digest"))?;

        // Digest is "sha256:<hex>"; strip the prefix before comparing.
        let hex_digest = image_digest
            .strip_prefix("sha256:")
            .unwrap_or(image_digest);
        if !TRUSTED_IDENTITY_IMAGE_DIGESTS
            .iter()
            .any(|d| hex::encode(d) == hex_digest)
        {
            return Err(AttestationError::UntrustedContainer(image_digest.to_string()));
        }

        // 4. aTLS binding: eat_nonce in the JWT must contain SHA-256(observed TLS cert pubkey).
        //    The launcher sets this nonce when requesting the GCA token, binding the
        //    attestation to the active TLS session.
        let tls_hash_hex = hex::encode(actual_tls_pubkey_hash);
        let tls_hash_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            actual_tls_pubkey_hash,
        );
        if !claims
            .eat_nonce
            .iter()
            .any(|n| n == &tls_hash_hex || n == &tls_hash_b64)
        {
            return Err(AttestationError::TlsBindingMismatch {
                expected: actual_tls_pubkey_hash.to_vec(),
                actual: claims.eat_nonce.join(",").into_bytes(),
            });
        }

        Ok(())
    }

    async fn verify_gca_jwt(&self, token: &str) -> Result<GcaClaims, AttestationError> {
        // Fetch JWKS
        let jwks: JwkSet = reqwest::get(GCA_JWKS_URL)
            .await
            .map_err(|e| AttestationError::JwksFetch(e.to_string()))?
            .json()
            .await
            .map_err(|e| AttestationError::JwksFetch(e.to_string()))?;

        // Find the right key by kid
        let header = decode_header(token)
            .map_err(|e| AttestationError::TokenVerification(e.to_string()))?;

        let kid = header
            .kid
            .ok_or_else(|| AttestationError::TokenVerification("missing kid in JWT header".into()))?;

        let jwk = jwks
            .find(&kid)
            .ok_or_else(|| {
                AttestationError::TokenVerification(format!("kid {} not found in JWKS", kid))
            })?;

        let decoding_key = DecodingKey::from_jwk(jwk)
            .map_err(|e| AttestationError::TokenVerification(e.to_string()))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[GCA_ISSUER]);
        // We don't validate audience since we control what audience we request
        validation.validate_aud = false;

        let token_data = decode::<GcaClaims>(token, &decoding_key, &validation)
            .map_err(|e| AttestationError::TokenVerification(e.to_string()))?;

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verify_missing_gca_token() {
        let verifier = AttestationVerifier;
        let p_hash = vec![1, 2, 3, 4];
        let resp = AttestationResponse {
            gca_token: None,
        };

        let result = verifier.verify(&resp, &p_hash).await;
        assert!(matches!(result, Err(AttestationError::MissingField("gca_token"))));
    }
}
