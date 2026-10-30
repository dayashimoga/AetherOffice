//! Cryptographic digest algorithms and safe verification.

use crate::error::CryptoSecurityError;
use sha2::{Digest, Sha256, Sha384, Sha512};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

impl HashAlgorithm {
    pub fn oid(&self) -> &'static str {
        match self {
            HashAlgorithm::Sha256 => "2.16.840.1.101.3.4.2.1",
            HashAlgorithm::Sha384 => "2.16.840.1.101.3.4.2.2",
            HashAlgorithm::Sha512 => "2.16.840.1.101.3.4.2.3",
        }
    }

    pub fn from_oid(oid: &str) -> Option<Self> {
        match oid {
            "2.16.840.1.101.3.4.2.1" => Some(HashAlgorithm::Sha256),
            "2.16.840.1.101.3.4.2.2" => Some(HashAlgorithm::Sha384),
            "2.16.840.1.101.3.4.2.3" => Some(HashAlgorithm::Sha512),
            _ => None,
        }
    }

    pub fn xmldsig_uri(&self) -> &'static str {
        match self {
            HashAlgorithm::Sha256 => "http://www.w3.org/2001/04/xmlenc#sha256",
            HashAlgorithm::Sha384 => "http://www.w3.org/2001/04/xmldsig-more#sha384",
            HashAlgorithm::Sha512 => "http://www.w3.org/2001/04/xmlenc#sha512",
        }
    }

    pub fn from_xmldsig_uri(uri: &str) -> Option<Self> {
        match uri {
            "http://www.w3.org/2001/04/xmlenc#sha256" => Some(HashAlgorithm::Sha256),
            "http://www.w3.org/2001/04/xmldsig-more#sha384" => Some(HashAlgorithm::Sha384),
            "http://www.w3.org/2001/04/xmlenc#sha512" => Some(HashAlgorithm::Sha512),
            _ => None,
        }
    }

    pub fn digest_len(&self) -> usize {
        match self {
            HashAlgorithm::Sha256 => 32,
            HashAlgorithm::Sha384 => 48,
            HashAlgorithm::Sha512 => 64,
        }
    }
}

/// Compute cryptographic digest over the input data.
pub fn compute_digest(alg: HashAlgorithm, data: &[u8]) -> Vec<u8> {
    match alg {
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        HashAlgorithm::Sha384 => {
            let mut hasher = Sha384::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        HashAlgorithm::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
    }
}

/// Verify that data computes to expected digest with constant-time comparison.
pub fn verify_digest(
    alg: HashAlgorithm,
    data: &[u8],
    expected: &[u8],
) -> Result<(), CryptoSecurityError> {
    let computed = compute_digest(alg, data);
    if constant_time_eq(&computed, expected) {
        Ok(())
    } else {
        Err(CryptoSecurityError::DigestMismatch {
            expected: to_hex_string(expected),
            calculated: to_hex_string(&computed),
        })
    }
}

/// Constant-time byte slice comparison to mitigate timing side-channels.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Convert byte slice to lower-case hexadecimal string.
pub fn to_hex_string(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}

/// Parse hexadecimal string into byte vector.
pub fn from_hex_string(hex: &str) -> Result<Vec<u8>, CryptoSecurityError> {
    let cleaned: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
    if !cleaned.len().is_multiple_of(2) {
        return Err(CryptoSecurityError::BufferError(
            "Hex string length must be even".to_string(),
        ));
    }
    let mut out = Vec::with_capacity(cleaned.len() / 2);
    for i in (0..cleaned.len()).step_by(2) {
        let chunk = &cleaned[i..i + 2];
        match u8::from_str_radix(chunk, 16) {
            Ok(b) => out.push(b),
            Err(e) => {
                return Err(CryptoSecurityError::BufferError(format!(
                    "Invalid hex byte '{chunk}': {e}"
                )));
            }
        }
    }
    Ok(out)
}
