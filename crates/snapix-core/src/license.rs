use anyhow::Result;

use crate::entitlements::Entitlements;

pub trait LicenseVerifier: Send + Sync {
    fn verify(&self, key: &str) -> Result<Entitlements>;
}

/// Stub verifier used until real Ed25519 verification is wired up in M4.
pub struct StubVerifier;

impl LicenseVerifier for StubVerifier {
    fn verify(&self, key: &str) -> Result<Entitlements> {
        if key == "SNAPIX-PRO-DEV" {
            Ok(Entitlements::pro())
        } else {
            Ok(Entitlements::free())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_verifier_pro_key() {
        let verifier = StubVerifier;
        let ent = verifier.verify("SNAPIX-PRO-DEV").unwrap();
        assert!(ent.is_pro());
    }

    #[test]
    fn stub_verifier_invalid_key() {
        let verifier = StubVerifier;
        let ent = verifier.verify("invalid-key").unwrap();
        assert!(!ent.is_pro());
    }

    #[test]
    fn stub_verifier_empty_key() {
        let verifier = StubVerifier;
        let ent = verifier.verify("").unwrap();
        assert!(!ent.is_pro());
    }
}
