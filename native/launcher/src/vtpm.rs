use anyhow::Result;

pub struct VtpmQuote {
    pub quote: Vec<u8>,
    pub signature: Vec<u8>,
    pub pcrs: Vec<u8>,
}

/// Extend a vTPM PCR with a measurement.
/// On real hardware: uses /dev/tpmrm0 via tss-esapi.
/// In tests: records the extension in a mock.
pub trait Vtpm {
    fn extend_pcr(&mut self, pcr: u32, digest: &[u8; 32]) -> Result<()>;
    fn get_quote(&self, nonce: &[u8]) -> Result<VtpmQuote>;
}

pub struct RealVtpm;

impl RealVtpm {
    pub fn new() -> Self {
        Self
    }
}

impl Vtpm for RealVtpm {
    fn extend_pcr(&mut self, _pcr: u32, _digest: &[u8; 32]) -> Result<()> {
        // TODO: tss-esapi integration for actual CVMs
        anyhow::bail!("Real vTPM not implemented yet");
    }

    fn get_quote(&self, _nonce: &[u8]) -> Result<VtpmQuote> {
        // TODO: tss-esapi integration for actual CVMs
        anyhow::bail!("Real vTPM not implemented yet");
    }
}

pub struct MockVtpm {
    pub extensions: Vec<(u32, [u8; 32])>,
}

impl MockVtpm {
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
        }
    }
}

impl Vtpm for MockVtpm {
    fn extend_pcr(&mut self, pcr: u32, digest: &[u8; 32]) -> Result<()> {
        self.extensions.push((pcr, *digest));
        Ok(())
    }

    fn get_quote(&self, _nonce: &[u8]) -> Result<VtpmQuote> {
        // In local mode, return mock data identical to the new AttestationVerifier mock expectations
        Ok(VtpmQuote {
            quote: vec![0x02; 100],
            signature: vec![0x03; 64],
            pcrs: vec![0x04; 256],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_vtpm_extend() {
        let mut vtpm = MockVtpm::new();
        let digest = [0xAA; 32];
        vtpm.extend_pcr(14, &digest).unwrap();
        assert_eq!(vtpm.extensions.len(), 1);
        assert_eq!(vtpm.extensions[0].0, 14);
        assert_eq!(vtpm.extensions[0].1, digest);
    }

    #[test]
    fn test_mock_vtpm_quote() {
        let vtpm = MockVtpm::new();
        let quote = vtpm.get_quote(b"nonce").unwrap();
        assert_eq!(quote.quote, vec![0x02; 100]);
    }
}
