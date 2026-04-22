use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::entitlements::{Entitlements, Tier};

pub trait LicenseVerifier: Send + Sync {
    fn verify(&self, key: &str) -> Result<Entitlements>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LicenseClaims {
    pub tier: Tier,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub expires_at: Option<u64>,
}

impl LicenseClaims {
    fn into_entitlements(self) -> Result<Entitlements> {
        if let Some(expires_at) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now > expires_at {
                bail!("License key has expired.");
            }
        }

        Ok(match self.tier {
            Tier::Free => Entitlements::free(),
            Tier::Pro => Entitlements::pro(),
        })
    }
}

/// Verifies signed license strings in the form:
/// `SNAPIX-1-<claims_hex>-<signature_hex>`
///
/// `claims_hex` is the UTF-8 JSON bytes of [`LicenseClaims`] encoded as hex.
/// `signature_hex` is an Ed25519 signature over those original JSON bytes.
#[derive(Debug, Clone)]
pub struct Ed25519LicenseVerifier {
    public_key: VerifyingKey,
}

impl Ed25519LicenseVerifier {
    pub fn from_public_key_hex(hex: &str) -> Result<Self> {
        let bytes = decode_hex_fixed::<32>(hex.trim())
            .context("Ed25519 public key must be 32 bytes of hex")?;
        let public_key = VerifyingKey::from_bytes(&bytes)
            .map_err(|error| anyhow!("Invalid Ed25519 public key: {error}"))?;
        Ok(Self { public_key })
    }

    fn parse_signed_key<'a>(&self, key: &'a str) -> Result<(&'a str, &'a str)> {
        let Some(rest) = key.trim().strip_prefix("SNAPIX-1-") else {
            bail!("License key format is not recognized.");
        };

        let mut parts = rest.splitn(2, '-');
        let claims_hex = parts
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("License claims are missing."))?;
        let signature_hex = parts
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("License signature is missing."))?;
        Ok((claims_hex, signature_hex))
    }
}

impl LicenseVerifier for Ed25519LicenseVerifier {
    fn verify(&self, key: &str) -> Result<Entitlements> {
        let (claims_hex, signature_hex) = self.parse_signed_key(key)?;
        let claims_bytes = decode_hex(claims_hex).context("Failed to decode license claims")?;
        let signature_bytes =
            decode_hex_fixed::<64>(signature_hex).context("Failed to decode license signature")?;
        let signature = Signature::from_bytes(&signature_bytes);

        self.public_key
            .verify(&claims_bytes, &signature)
            .map_err(|_| anyhow!("License signature verification failed."))?;

        let claims: LicenseClaims =
            serde_json::from_slice(&claims_bytes).context("Failed to parse license claims")?;
        claims.into_entitlements()
    }
}

/// Development verifier kept for local activation and UI testing.
pub struct StubVerifier;

impl LicenseVerifier for StubVerifier {
    fn verify(&self, key: &str) -> Result<Entitlements> {
        if key.trim() == "SNAPIX-PRO-DEV" {
            Ok(Entitlements::pro())
        } else {
            Ok(Entitlements::free())
        }
    }
}

fn decode_hex(input: &str) -> Result<Vec<u8>> {
    let input = input.trim();
    if input.len() % 2 != 0 {
        bail!("Hex input must have an even number of characters");
    }

    let mut output = Vec::with_capacity(input.len() / 2);
    let bytes = input.as_bytes();
    for chunk in bytes.chunks_exact(2) {
        let hi = decode_hex_nibble(chunk[0])?;
        let lo = decode_hex_nibble(chunk[1])?;
        output.push((hi << 4) | lo);
    }
    Ok(output)
}

fn decode_hex_fixed<const N: usize>(input: &str) -> Result<[u8; N]> {
    let bytes = decode_hex(input)?;
    let actual_len = bytes.len();
    bytes
        .try_into()
        .map_err(|_| anyhow!("Expected {} bytes of hex data but got {}", N, actual_len))
}

fn decode_hex_nibble(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => bail!("Invalid hex character"),
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};

    use super::*;

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[7_u8; 32])
    }

    fn verifier() -> Ed25519LicenseVerifier {
        let signing_key = signing_key();
        Ed25519LicenseVerifier {
            public_key: signing_key.verifying_key(),
        }
    }

    fn encode_hex(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }
        output
    }

    fn signed_key(claims: &LicenseClaims) -> String {
        let bytes = serde_json::to_vec(claims).expect("claims json");
        let signature = signing_key().sign(&bytes);
        format!(
            "SNAPIX-1-{}-{}",
            encode_hex(&bytes),
            encode_hex(&signature.to_bytes())
        )
    }

    #[test]
    fn ed25519_verifier_accepts_valid_pro_key() {
        let verifier = verifier();
        let key = signed_key(&LicenseClaims {
            tier: Tier::Pro,
            subject: Some("test@example.com".into()),
            expires_at: None,
        });

        let ent = verifier.verify(&key).unwrap();
        assert!(ent.is_pro());
    }

    #[test]
    fn ed25519_verifier_rejects_invalid_signature() {
        let verifier = verifier();
        let claims = LicenseClaims {
            tier: Tier::Pro,
            subject: None,
            expires_at: None,
        };
        let mut key = signed_key(&claims);
        key.pop();
        key.push('0');

        let error = verifier.verify(&key).unwrap_err().to_string();
        assert!(error.contains("verification failed") || error.contains("decode"));
    }

    #[test]
    fn ed25519_verifier_rejects_expired_key() {
        let verifier = verifier();
        let key = signed_key(&LicenseClaims {
            tier: Tier::Pro,
            subject: None,
            expires_at: Some(1),
        });

        let error = verifier.verify(&key).unwrap_err().to_string();
        assert!(error.contains("expired"));
    }

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
