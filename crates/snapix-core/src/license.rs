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
