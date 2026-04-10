mod acme;
mod measure;
mod vtpm;

use anyhow::Result;
use std::fs;
use std::path::Path;
use vtpm::{MockVtpm, Vtpm};

async fn run() -> Result<()> {
    println!("Starting Brongnal CVM Launcher");
    
    // 1. Read config from instance metadata (or args for local)
    // 2. Measure image
    // For local dev mock flow:
    let container_digest = measure::measure_image(b"mock_container_manifest");
    
    // 3. Obtain TLS cert
    let (tls_cert, _tls_key) = acme::obtain_certificate(
        "gossamer.brongan.com", 
        "mock_kid", 
        "mock_hmac", 
        "mock_proj"
    ).await?;
    
    let tls_pubkey_hash = measure::measure_image(&tls_cert); // Simplified hash for mock

    // 4. Extend vTPM PCR[14]
    let mut vtpm = MockVtpm::new();
    vtpm.extend_pcr(14, &container_digest)?;
    // (mocking extending for the tls pubkey as well)
    vtpm.extend_pcr(14, &tls_pubkey_hash)?;

    // 5. Request vTPM quote over PCRs, fetch Google Shielded VM certs
    let _quote = vtpm.get_quote(b"nonce")?;
    let _google_shielded_certs = vec![vec![0x01; 100]]; // Mock cert chain
    
    // 6. Write attestation artifacts
    let attestation_dir = Path::new("/tmp/run_attestation");
    fs::create_dir_all(attestation_dir).unwrap_or_default();
    fs::write(attestation_dir.join("quote.bin"), &_quote.quote)?;
    fs::write(attestation_dir.join("quote_sig.bin"), &_quote.signature)?;

    println!("MOCK: Extracted Quote written to {:?}", attestation_dir);

    // 7. Write TLS cert/key to /run/tls/
    let tls_dir = Path::new("/tmp/run_tls");
    fs::create_dir_all(tls_dir).unwrap_or_default();
    fs::write(tls_dir.join("cert.pem"), &tls_cert)?;

    println!("MOCK: TLS Certs written to {:?}", tls_dir);

    // 8. Mount data disk at /mnt/data
    // 9. exec() identity service binary
    println!("MOCK: Exec identity service");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_boot_flow_mock() {
        assert!(run().await.is_ok());
    }
}
