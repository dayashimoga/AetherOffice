//! RFC 3161 Internet X.509 Public Key Infrastructure Time-Stamp Protocol (TSP).

use crate::asn1::DerValue;
use crate::digest::{constant_time_eq, to_hex_string, HashAlgorithm};
use crate::error::CryptoSecurityError;
use crate::x509::X509Certificate;

pub const OID_TST_INFO: &str = "1.2.840.113549.1.9.16.1.4";
pub const OID_TIMESTAMP_TOKEN_ATTR: &str = "1.2.840.113549.1.9.16.2.14";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TstInfo {
    pub version: u32,
    pub policy: String,
    pub hash_algorithm: HashAlgorithm,
    pub hashed_message: Vec<u8>,
    pub serial_number: Vec<u8>,
    pub gen_time: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeStampToken {
    pub tst_info: TstInfo,
    pub tsa_certificate: Option<X509Certificate>,
    pub raw_der: Vec<u8>,
}

impl TimeStampToken {
    pub fn create_mock(
        digest: &[u8],
        alg: HashAlgorithm,
        gen_time: u64,
        tsa_cert: Option<&X509Certificate>,
    ) -> Self {
        let tst_info = TstInfo {
            version: 1,
            policy: "1.3.6.1.4.1.13762.3".to_string(),
            hash_algorithm: alg,
            hashed_message: digest.to_vec(),
            serial_number: vec![0x10, 0x20, 0x30],
            gen_time,
        };

        let tst_der = DerValue::Sequence(vec![
            DerValue::Integer(vec![tst_info.version as u8]),
            DerValue::Oid(tst_info.policy.clone()),
            DerValue::Sequence(vec![
                DerValue::Sequence(vec![DerValue::Oid(alg.oid().to_string()), DerValue::Null]),
                DerValue::OctetString(digest.to_vec()),
            ]),
            DerValue::Integer(tst_info.serial_number.clone()),
            DerValue::GeneralizedTime(gen_time),
        ])
        .encode();

        let token_der = DerValue::Sequence(vec![
            DerValue::Oid(OID_TST_INFO.to_string()),
            DerValue::ContextSpecific {
                tag: 0,
                constructed: true,
                data: tst_der,
            },
        ])
        .encode();

        TimeStampToken {
            tst_info,
            tsa_certificate: tsa_cert.cloned(),
            raw_der: token_der,
        }
    }

    pub fn from_der(bytes: &[u8]) -> Result<Self, CryptoSecurityError> {
        let (root, _) = DerValue::parse(bytes)?;
        let top_seq = match root {
            DerValue::Sequence(seq) => seq,
            _ => {
                return Err(CryptoSecurityError::TimestampError(
                    "TimeStampToken must be SEQUENCE".to_string(),
                ))
            }
        };

        if top_seq.is_empty() {
            return Err(CryptoSecurityError::TimestampError(
                "Empty TimeStampToken SEQUENCE".to_string(),
            ));
        }

        let mut tst_bytes = Vec::new();
        for item in &top_seq {
            if let DerValue::ContextSpecific { tag: 0, data, .. } = item {
                tst_bytes = data.clone();
                break;
            }
        }

        if tst_bytes.is_empty() {
            // Check direct TstInfo sequence
            tst_bytes = bytes.to_vec();
        }

        let (tst_val, _) = DerValue::parse(&tst_bytes)?;
        let tst_seq = match tst_val {
            DerValue::Sequence(seq) => seq,
            _ => {
                return Err(CryptoSecurityError::TimestampError(
                    "TSTInfo must be SEQUENCE".to_string(),
                ))
            }
        };

        if tst_seq.len() < 5 {
            return Err(CryptoSecurityError::TimestampError(
                "TSTInfo SEQUENCE too short".to_string(),
            ));
        }

        let version = match &tst_seq[0] {
            DerValue::Integer(v) if !v.is_empty() => v[0] as u32,
            _ => 1,
        };

        let policy = match &tst_seq[1] {
            DerValue::Oid(oid) => oid.clone(),
            _ => "unknown-policy".to_string(),
        };

        let (hash_alg, hashed_message) = parse_message_imprint(&tst_seq[2])?;

        let serial_number = match &tst_seq[3] {
            DerValue::Integer(v) => v.clone(),
            _ => vec![0x01],
        };

        let gen_time = match &tst_seq[4] {
            DerValue::GeneralizedTime(t) | DerValue::UtcTime(t) => *t,
            _ => 0,
        };

        Ok(TimeStampToken {
            tst_info: TstInfo {
                version,
                policy,
                hash_algorithm: hash_alg,
                hashed_message,
                serial_number,
                gen_time,
            },
            tsa_certificate: None,
            raw_der: bytes.to_vec(),
        })
    }

    pub fn verify(&self, expected_digest: &[u8]) -> Result<(), CryptoSecurityError> {
        if !constant_time_eq(&self.tst_info.hashed_message, expected_digest) {
            return Err(CryptoSecurityError::TimestampError(format!(
                "Timestamp message imprint mismatch: expected {}, got {}",
                to_hex_string(expected_digest),
                to_hex_string(&self.tst_info.hashed_message)
            )));
        }
        Ok(())
    }
}

fn parse_message_imprint(der: &DerValue) -> Result<(HashAlgorithm, Vec<u8>), CryptoSecurityError> {
    if let DerValue::Sequence(seq) = der {
        if seq.len() >= 2 {
            let alg_oid = match &seq[0] {
                DerValue::Sequence(inner) if !inner.is_empty() => match &inner[0] {
                    DerValue::Oid(oid) => oid.as_str(),
                    _ => "",
                },
                DerValue::Oid(oid) => oid.as_str(),
                _ => "",
            };
            let alg = HashAlgorithm::from_oid(alg_oid).unwrap_or(HashAlgorithm::Sha256);
            let msg = match &seq[1] {
                DerValue::OctetString(data) => data.clone(),
                _ => Vec::new(),
            };
            return Ok((alg, msg));
        }
    }
    Err(CryptoSecurityError::TimestampError(
        "Invalid MessageImprint SEQUENCE".to_string(),
    ))
}
