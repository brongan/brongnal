use reqwest::Url;
use sha2::{Digest, Sha256};

/// Compute the SHA-256 digest of an OCI image manifest or arbitrary bytes.
/// This is the content-addressable hash that identifies the container.
pub fn measure_image(manifest_bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(manifest_bytes);
    hasher.finalize().into()
}

/// Helper method to fetch the manifest for a given image reference and compute its digest.
/// For actual deployment, the launcher would just read the local extracted image manifest,
/// but pulling allows us to verify remote matches for deterministic builds.
pub async fn measure_remote_image(_image_ref: &str) -> anyhow::Result<[u8; 32]> {
    // MOCK: bypassing oci-distribution client for local development
    Ok(measure_image(b"mock_container_manifest"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_image_deterministic() {
        let manifest = b"{\"schemaVersion\": 2, \"layers\": []}";
        let digest1 = measure_image(manifest);
        let digest2 = measure_image(manifest);
        assert_eq!(digest1, digest2);
    }
    
    #[test]
    fn test_measure_image_different() {
        let digest1 = measure_image(b"{\"schemaVersion\": 2}");
        let digest2 = measure_image(b"{\"schemaVersion\": 1}");
        assert_ne!(digest1, digest2);
    }
}
