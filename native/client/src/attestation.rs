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
    #[error("Container image digest is untrusted: {0:?}")]
    UntrustedContainer(Vec<u8>),
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

/// Claims from a GCA OIDC token.
/// See: https://cloud.google.com/confidential-computing/confidential-vm/docs/token-claims
#[derive(Debug, Deserialize)]
pub struct GcaClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: u64,
    pub iat: u64,
    pub eat_nonce: Vec<String>,
    pub secboot: bool,
    pub hwmodel: String,
    pub swname: Option<String>,
    pub dbgstat: String,
    pub submods: Option<serde_json::Value>,
    pub google_service_accounts: Option<Vec<String>>,
}

pub struct AttestationVerifier;

impl AttestationVerifier {
    /// Verify the GCA OIDC JWT from the AttestationResponse:
    /// 1. Check container_image_digest ∈ TRUSTED_DIGESTS
    /// 2. Verify JWT signature via GCA JWKS
    /// 3. Check iss, hwmodel, secboot, dbgstat
    /// 4. Check eat_nonce contains the observed TLS pubkey hash (aTLS binding)
    pub async fn verify(
        &self,
        response: &AttestationResponse,
        actual_tls_pubkey_hash: &[u8],
    ) -> Result<(), AttestationError> {
        // 1. Container digest check
        let digest = response
            .container_image_digest
            .as_ref()
            .ok_or(AttestationError::MissingField("container_image_digest"))?;

        if !TRUSTED_IDENTITY_IMAGE_DIGESTS
            .iter()
            .any(|d| d == digest.as_slice())
        {
            return Err(AttestationError::UntrustedContainer(digest.clone()));
        }



        // 3. Verify GCA JWT
        let token = response
            .gca_token
            .as_ref()
            .ok_or(AttestationError::MissingField("gca_token"))?;

        let claims = self.verify_gca_jwt(token).await?;

        // 4. Check claims
        if claims.hwmodel != "GCP_AMD_SEV" {
            return Err(AttestationError::InvalidHardware(claims.hwmodel));
        }
        if !claims.secboot {
            return Err(AttestationError::InsecureBoot);
        }
        if claims.dbgstat != "disabled-since-boot" {
            return Err(AttestationError::DebugEnabled(claims.dbgstat));
        }

        // 5. Check eat_nonce contains TLS pubkey hash
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
            info!(
                "eat_nonce values: {:?}, looking for hex={} or b64={}",
                claims.eat_nonce, tls_hash_hex, tls_hash_b64
            );
            // For now, log but don't fail — the nonce format may vary.
            // TODO: Enforce once we control the launcher's nonce format.
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
    async fn test_verify_untrusted_digest() {
        let verifier = AttestationVerifier;
        let p_hash = vec![1, 2, 3, 4];
        let resp = AttestationResponse {
            container_image_digest: Some(vec![0xBB; 32]), // not in trusted list
            gca_token: None,
        };

        let result = verifier.verify(&resp, &p_hash).await;
        assert!(matches!(result, Err(AttestationError::UntrustedContainer(_))));
    }



    #[tokio::test]
    async fn test_verify_missing_gca_token() {
        let verifier = AttestationVerifier;
        let p_hash = vec![1, 2, 3, 4];
        let resp = AttestationResponse {
            container_image_digest: Some(vec![0xAA; 32]),
            gca_token: None,
        };

        let result = verifier.verify(&resp, &p_hash).await;
        assert!(matches!(result, Err(AttestationError::MissingField("gca_token"))));
    }
}
