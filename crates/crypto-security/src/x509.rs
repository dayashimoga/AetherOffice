//! X.509 Public Key Certificate model and validation engine.

use crate::asn1::DerValue;
use crate::base64::{decode_base64, encode_base64};
use crate::digest::{compute_digest, to_hex_string, HashAlgorithm};
use crate::error::CryptoSecurityError;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DistinguishedName {
    pub common_name: Option<String>,
    pub organization: Option<String>,
    pub organizational_unit: Option<String>,
    pub country: Option<String>,
    pub state_or_province: Option<String>,
    pub raw_entries: Vec<(String, String)>,
}

impl DistinguishedName {
    pub fn rfc4514_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(cn) = &self.common_name {
            parts.push(format!("CN={cn}"));
        }
        if let Some(o) = &self.organization {
            parts.push(format!("O={o}"));
        }
        if let Some(ou) = &self.organizational_unit {
            parts.push(format!("OU={ou}"));
        }
        if let Some(c) = &self.country {
            parts.push(format!("C={c}"));
        }
        if let Some(st) = &self.state_or_province {
            parts.push(format!("ST={st}"));
        }
        if parts.is_empty() {
            "CN=Unknown".to_string()
        } else {
            parts.join(", ")
        }
    }

    pub fn matches(&self, other: &DistinguishedName) -> bool {
        if self.common_name != other.common_name {
            return false;
        }
        if self.organization != other.organization {
            return false;
        }
        if self.country != other.country {
            return false;
        }
        true
    }

    pub fn to_der(&self) -> DerValue {
        let mut rdn_seq = Vec::new();
        if let Some(cn) = &self.common_name {
            rdn_seq.push(create_rdn("2.5.4.3", cn));
        }
        if let Some(o) = &self.organization {
            rdn_seq.push(create_rdn("2.5.4.10", o));
        }
        if let Some(ou) = &self.organizational_unit {
            rdn_seq.push(create_rdn("2.5.4.11", ou));
        }
        if let Some(c) = &self.country {
            rdn_seq.push(create_rdn("2.5.4.6", c));
        }
        DerValue::Sequence(rdn_seq)
    }

    pub fn parse(der: &DerValue) -> Result<Self, CryptoSecurityError> {
        let items = match der {
            DerValue::Sequence(seq) => seq,
            _ => {
                return Err(CryptoSecurityError::CertificateError(
                    "DN must be SEQUENCE".to_string(),
                ))
            }
        };

        let mut dn = DistinguishedName::default();
        for rdn_set in items {
            let set_items = match rdn_set {
                DerValue::Set(s) => s,
                _ => continue,
            };
            for attr in set_items {
                if let DerValue::Sequence(kv) = attr {
                    if kv.len() >= 2 {
                        if let DerValue::Oid(oid) = &kv[0] {
                            let val_str = match &kv[1] {
                                DerValue::Utf8String(s)
                                | DerValue::PrintableString(s)
                                | DerValue::Ia5String(s) => s.clone(),
                                _ => continue,
                            };
                            dn.raw_entries.push((oid.clone(), val_str.clone()));
                            match oid.as_str() {
                                "2.5.4.3" => dn.common_name = Some(val_str),
                                "2.5.4.10" => dn.organization = Some(val_str),
                                "2.5.4.11" => dn.organizational_unit = Some(val_str),
                                "2.5.4.6" => dn.country = Some(val_str),
                                "2.5.4.8" => dn.state_or_province = Some(val_str),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        Ok(dn)
    }
}

fn create_rdn(oid: &str, value: &str) -> DerValue {
    DerValue::Set(vec![DerValue::Sequence(vec![
        DerValue::Oid(oid.to_string()),
        DerValue::Utf8String(value.to_string()),
    ])])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidityPeriod {
    pub not_before: u64,
    pub not_after: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectPublicKeyInfo {
    pub algorithm_oid: String,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct X509Extension {
    pub oid: String,
    pub critical: bool,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct X509Certificate {
    pub version: u32,
    pub serial_number: Vec<u8>,
    pub signature_algorithm: String,
    pub issuer: DistinguishedName,
    pub validity: ValidityPeriod,
    pub subject: DistinguishedName,
    pub public_key_info: SubjectPublicKeyInfo,
    pub extensions: Vec<X509Extension>,
    pub signature_value: Vec<u8>,
    pub raw_der: Vec<u8>,
    pub tbs_bytes: Vec<u8>,
}

impl X509Certificate {
    pub fn from_der(bytes: &[u8]) -> Result<Self, CryptoSecurityError> {
        let (root, _) = DerValue::parse(bytes)?;
        let top_seq = match root {
            DerValue::Sequence(seq) => seq,
            _ => {
                return Err(CryptoSecurityError::CertificateError(
                    "Top-level certificate must be SEQUENCE".to_string(),
                ))
            }
        };

        if top_seq.len() < 3 {
            return Err(CryptoSecurityError::CertificateError(
                "Certificate SEQUENCE must have at least 3 elements".to_string(),
            ));
        }

        let tbs_val = &top_seq[0];
        let tbs_bytes = tbs_val.encode();

        let tbs_seq = match tbs_val {
            DerValue::Sequence(s) => s,
            _ => {
                return Err(CryptoSecurityError::CertificateError(
                    "TBSCertificate must be SEQUENCE".to_string(),
                ))
            }
        };

        let mut idx = 0;
        let mut version = 1u32;
        if idx < tbs_seq.len() {
            if let DerValue::ContextSpecific { tag: 0, data, .. } = &tbs_seq[idx] {
                if let Ok((DerValue::Integer(v_bytes), _)) = DerValue::parse(data) {
                    if !v_bytes.is_empty() {
                        version = (v_bytes[0] as u32) + 1;
                    }
                }
                idx += 1;
            }
        }

        let serial_number = if idx < tbs_seq.len() {
            if let DerValue::Integer(bytes) = &tbs_seq[idx] {
                bytes.clone()
            } else {
                vec![0x01]
            }
        } else {
            vec![0x01]
        };
        idx += 1;

        let signature_algorithm = if idx < tbs_seq.len() {
            extract_algorithm_oid(&tbs_seq[idx])?
        } else {
            "1.2.840.113549.1.1.11".to_string()
        };
        idx += 1;

        let issuer = if idx < tbs_seq.len() {
            DistinguishedName::parse(&tbs_seq[idx])?
        } else {
            DistinguishedName::default()
        };
        idx += 1;

        let validity = if idx < tbs_seq.len() {
            parse_validity(&tbs_seq[idx])?
        } else {
            ValidityPeriod {
                not_before: 0,
                not_after: u64::MAX,
            }
        };
        idx += 1;

        let subject = if idx < tbs_seq.len() {
            DistinguishedName::parse(&tbs_seq[idx])?
        } else {
            DistinguishedName::default()
        };
        idx += 1;

        let public_key_info = if idx < tbs_seq.len() {
            parse_spki(&tbs_seq[idx])?
        } else {
            SubjectPublicKeyInfo {
                algorithm_oid: "1.2.840.113549.1.1.1".to_string(),
                public_key: Vec::new(),
            }
        };

        let signature_value = match &top_seq[2] {
            DerValue::BitString { data, .. } => data.clone(),
            _ => Vec::new(),
        };

        Ok(X509Certificate {
            version,
            serial_number,
            signature_algorithm,
            issuer,
            validity,
            subject,
            public_key_info,
            extensions: Vec::new(),
            signature_value,
            raw_der: bytes.to_vec(),
            tbs_bytes,
        })
    }

    pub fn from_pem(pem: &str) -> Result<Self, CryptoSecurityError> {
        let lines: Vec<&str> = pem.lines().collect();
        let mut in_cert = false;
        let mut b64 = String::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed == "-----BEGIN CERTIFICATE-----" {
                in_cert = true;
            } else if trimmed == "-----END CERTIFICATE-----" {
                break;
            } else if in_cert {
                b64.push_str(trimmed);
            }
        }

        if b64.is_empty() {
            return Err(CryptoSecurityError::CertificateError(
                "No PEM certificate boundaries found".to_string(),
            ));
        }

        let der = decode_base64(&b64)?;
        Self::from_der(&der)
    }

    pub fn to_pem(&self) -> String {
        let b64 = encode_base64(&self.raw_der);
        let mut pem = String::from("-----BEGIN CERTIFICATE-----\n");
        for chunk in b64.as_bytes().chunks(64) {
            if let Ok(line) = std::str::from_utf8(chunk) {
                pem.push_str(line);
                pem.push('\n');
            }
        }
        pem.push_str("-----END CERTIFICATE-----\n");
        pem
    }

    pub fn fingerprint_sha256(&self) -> String {
        let digest = compute_digest(HashAlgorithm::Sha256, &self.raw_der);
        to_hex_string(&digest)
    }

    pub fn is_valid_at(&self, epoch: u64) -> Result<(), CryptoSecurityError> {
        if epoch < self.validity.not_before || epoch > self.validity.not_after {
            return Err(CryptoSecurityError::CertificateValidityError {
                not_before: self.validity.not_before,
                not_after: self.validity.not_after,
                evaluated_at: epoch,
            });
        }
        Ok(())
    }

    pub fn is_self_signed(&self) -> bool {
        self.issuer.matches(&self.subject)
    }

    /// Construct a valid mock X.509 certificate for testing and signature synthesis.
    pub fn create_mock(
        subject_cn: &str,
        issuer_cn: &str,
        serial: u64,
        not_before: u64,
        not_after: u64,
    ) -> Self {
        let issuer = DistinguishedName {
            common_name: Some(issuer_cn.to_string()),
            organization: Some("AetherOffice Trust Services".to_string()),
            country: Some("US".to_string()),
            ..Default::default()
        };
        let subject = DistinguishedName {
            common_name: Some(subject_cn.to_string()),
            organization: Some("AetherOffice Trust Services".to_string()),
            country: Some("US".to_string()),
            ..Default::default()
        };
        let validity = ValidityPeriod {
            not_before,
            not_after,
        };
        let spki = SubjectPublicKeyInfo {
            algorithm_oid: "1.2.840.113549.1.1.1".to_string(),
            public_key: vec![0x30, 0x12, 0x02, 0x01, 0x00],
        };

        let tbs = DerValue::Sequence(vec![
            DerValue::ContextSpecific {
                tag: 0,
                constructed: true,
                data: DerValue::Integer(vec![0x02]).encode(),
            },
            DerValue::Integer(serial.to_be_bytes().to_vec()),
            DerValue::Sequence(vec![
                DerValue::Oid("1.2.840.113549.1.1.11".to_string()),
                DerValue::Null,
            ]),
            issuer.to_der(),
            DerValue::Sequence(vec![
                DerValue::UtcTime(not_before),
                DerValue::UtcTime(not_after),
            ]),
            subject.to_der(),
            DerValue::Sequence(vec![
                DerValue::Sequence(vec![
                    DerValue::Oid(spki.algorithm_oid.clone()),
                    DerValue::Null,
                ]),
                DerValue::BitString {
                    unused_bits: 0,
                    data: spki.public_key.clone(),
                },
            ]),
        ]);

        let tbs_bytes = tbs.encode();
        let sig_val = compute_digest(HashAlgorithm::Sha256, &tbs_bytes);

        let cert_der = DerValue::Sequence(vec![
            tbs,
            DerValue::Sequence(vec![
                DerValue::Oid("1.2.840.113549.1.1.11".to_string()),
                DerValue::Null,
            ]),
            DerValue::BitString {
                unused_bits: 0,
                data: sig_val.clone(),
            },
        ])
        .encode();

        X509Certificate {
            version: 3,
            serial_number: serial.to_be_bytes().to_vec(),
            signature_algorithm: "1.2.840.113549.1.1.11".to_string(),
            issuer,
            validity,
            subject,
            public_key_info: spki,
            extensions: Vec::new(),
            signature_value: sig_val,
            raw_der: cert_der,
            tbs_bytes,
        }
    }
}

pub struct CertificateChainValidator;

impl CertificateChainValidator {
    pub fn validate_chain(
        leaf: &X509Certificate,
        intermediates: &[X509Certificate],
        trusted_roots: &[X509Certificate],
        evaluated_at: u64,
    ) -> Result<(), CryptoSecurityError> {
        leaf.is_valid_at(evaluated_at)?;

        if leaf.is_self_signed() {
            let found_root = trusted_roots
                .iter()
                .find(|r| r.fingerprint_sha256() == leaf.fingerprint_sha256());
            if found_root.is_some() {
                return Ok(());
            } else {
                return Err(CryptoSecurityError::ChainVerificationError(
                    "Self-signed certificate is not in trusted roots store".to_string(),
                ));
            }
        }

        let mut current = leaf.clone();
        let mut depth = 0;
        const MAX_DEPTH: usize = 10;

        while depth < MAX_DEPTH {
            if let Some(root) = trusted_roots
                .iter()
                .find(|r| r.subject.matches(&current.issuer))
            {
                root.is_valid_at(evaluated_at)?;
                return Ok(());
            }

            if let Some(inter) = intermediates
                .iter()
                .find(|i| i.subject.matches(&current.issuer))
            {
                inter.is_valid_at(evaluated_at)?;
                current = inter.clone();
                depth += 1;
            } else {
                return Err(CryptoSecurityError::ChainVerificationError(format!(
                    "Missing certificate for issuer '{}'",
                    current.issuer.rfc4514_string()
                )));
            }
        }

        Err(CryptoSecurityError::ChainVerificationError(
            "Certificate chain depth exceeded maximum allowed length".to_string(),
        ))
    }
}

fn extract_algorithm_oid(der: &DerValue) -> Result<String, CryptoSecurityError> {
    if let DerValue::Sequence(seq) = der {
        if !seq.is_empty() {
            if let DerValue::Oid(oid) = &seq[0] {
                return Ok(oid.clone());
            }
        }
    }
    Err(CryptoSecurityError::CertificateError(
        "Invalid AlgorithmIdentifier SEQUENCE".to_string(),
    ))
}

fn parse_validity(der: &DerValue) -> Result<ValidityPeriod, CryptoSecurityError> {
    if let DerValue::Sequence(seq) = der {
        if seq.len() >= 2 {
            let nb = match &seq[0] {
                DerValue::UtcTime(t) | DerValue::GeneralizedTime(t) => *t,
                _ => 0,
            };
            let na = match &seq[1] {
                DerValue::UtcTime(t) | DerValue::GeneralizedTime(t) => *t,
                _ => u64::MAX,
            };
            return Ok(ValidityPeriod {
                not_before: nb,
                not_after: na,
            });
        }
    }
    Err(CryptoSecurityError::CertificateError(
        "Invalid Validity SEQUENCE".to_string(),
    ))
}

fn parse_spki(der: &DerValue) -> Result<SubjectPublicKeyInfo, CryptoSecurityError> {
    if let DerValue::Sequence(seq) = der {
        if seq.len() >= 2 {
            let alg_oid = extract_algorithm_oid(&seq[0])?;
            let pub_key = match &seq[1] {
                DerValue::BitString { data, .. } => data.clone(),
                _ => Vec::new(),
            };
            return Ok(SubjectPublicKeyInfo {
                algorithm_oid: alg_oid,
                public_key: pub_key,
            });
        }
    }
    Err(CryptoSecurityError::CertificateError(
        "Invalid SubjectPublicKeyInfo SEQUENCE".to_string(),
    ))
}
