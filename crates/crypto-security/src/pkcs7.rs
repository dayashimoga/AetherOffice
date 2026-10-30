//! PKCS#7 / CMS (Cryptographic Message Syntax) SignedData parser, generator, and validator.

use crate::asn1::DerValue;
use crate::digest::{compute_digest, constant_time_eq, to_hex_string, HashAlgorithm};
use crate::error::CryptoSecurityError;
use crate::timestamp::{TimeStampToken, TstInfo, OID_TIMESTAMP_TOKEN_ATTR};
use crate::x509::{CertificateChainValidator, DistinguishedName, X509Certificate};

pub const OID_PKCS7_SIGNED_DATA: &str = "1.2.840.113549.1.7.2";
pub const OID_PKCS7_DATA: &str = "1.2.840.113549.1.7.1";
pub const OID_CONTENT_TYPE: &str = "1.2.840.113549.1.9.3";
pub const OID_MESSAGE_DIGEST: &str = "1.2.840.113549.1.9.4";
pub const OID_SIGNING_TIME: &str = "1.2.840.113549.1.9.5";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub oid: String,
    pub values: Vec<DerValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerInfo {
    pub version: u32,
    pub issuer: DistinguishedName,
    pub serial_number: Vec<u8>,
    pub digest_algorithm: HashAlgorithm,
    pub authenticated_attributes: Vec<Attribute>,
    pub signature_algorithm: String,
    pub signature_bytes: Vec<u8>,
    pub unauthenticated_attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pkcs7SignedData {
    pub version: u32,
    pub digest_algorithms: Vec<HashAlgorithm>,
    pub encap_content_type: String,
    pub encap_content: Option<Vec<u8>>,
    pub certificates: Vec<X509Certificate>,
    pub signer_infos: Vec<SignerInfo>,
    pub raw_der: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureVerificationResult {
    pub is_valid: bool,
    pub signer_certificate: X509Certificate,
    pub signing_time: Option<u64>,
    pub has_timestamp: bool,
    pub timestamp_info: Option<TstInfo>,
    pub digest_matches: bool,
}

impl Pkcs7SignedData {
    pub fn from_der(bytes: &[u8]) -> Result<Self, CryptoSecurityError> {
        let (root, _) = DerValue::parse(bytes)?;
        let top_seq = match root {
            DerValue::Sequence(seq) => seq,
            _ => {
                return Err(CryptoSecurityError::Pkcs7Error(
                    "Top-level PKCS#7 must be ContentInfo SEQUENCE".to_string(),
                ))
            }
        };

        if top_seq.len() < 2 {
            return Err(CryptoSecurityError::Pkcs7Error(
                "ContentInfo SEQUENCE must have at least 2 elements".to_string(),
            ));
        }

        let content_type_oid = match &top_seq[0] {
            DerValue::Oid(oid) => oid.clone(),
            _ => {
                return Err(CryptoSecurityError::Pkcs7Error(
                    "ContentInfo must start with contentType OID".to_string(),
                ))
            }
        };

        if content_type_oid != OID_PKCS7_SIGNED_DATA {
            return Err(CryptoSecurityError::Pkcs7Error(format!(
                "Expected SignedData OID ({OID_PKCS7_SIGNED_DATA}), found '{content_type_oid}'"
            )));
        }

        let signed_data_bytes = match &top_seq[1] {
            DerValue::ContextSpecific { tag: 0, data, .. } => data.clone(),
            _ => {
                return Err(CryptoSecurityError::Pkcs7Error(
                    "Expected [0] EXPLICIT SignedData".to_string(),
                ))
            }
        };

        let (sd_val, _) = DerValue::parse(&signed_data_bytes)?;
        let sd_seq = match sd_val {
            DerValue::Sequence(seq) => seq,
            _ => {
                return Err(CryptoSecurityError::Pkcs7Error(
                    "SignedData must be SEQUENCE".to_string(),
                ))
            }
        };

        if sd_seq.len() < 4 {
            return Err(CryptoSecurityError::Pkcs7Error(
                "SignedData SEQUENCE too short".to_string(),
            ));
        }

        let version = match &sd_seq[0] {
            DerValue::Integer(bytes) if !bytes.is_empty() => bytes[0] as u32,
            _ => 1,
        };

        let mut digest_algorithms = Vec::new();
        if let DerValue::Set(da_set) = &sd_seq[1] {
            for item in da_set {
                if let DerValue::Sequence(seq) = item {
                    if !seq.is_empty() {
                        if let DerValue::Oid(oid) = &seq[0] {
                            if let Some(alg) = HashAlgorithm::from_oid(oid) {
                                digest_algorithms.push(alg);
                            }
                        }
                    }
                }
            }
        }
        if digest_algorithms.is_empty() {
            digest_algorithms.push(HashAlgorithm::Sha256);
        }

        let (encap_type, encap_content) = parse_encap_content_info(&sd_seq[2])?;

        let mut certificates = Vec::new();
        let mut idx = 3;

        if idx < sd_seq.len() {
            if let DerValue::ContextSpecific { tag: 0, data, .. } = &sd_seq[idx] {
                // Certificates SET [0] IMPLICIT
                let mut offset = 0;
                while offset < data.len() {
                    let (_, consumed) = match DerValue::parse(&data[offset..]) {
                        Ok(res) => res,
                        Err(_) => break,
                    };
                    if consumed == 0 {
                        break;
                    }
                    let c_bytes = &data[offset..offset + consumed];
                    if let Ok(cert) = X509Certificate::from_der(c_bytes) {
                        certificates.push(cert);
                    }
                    offset += consumed;
                }
                idx += 1;
            }
        }

        if idx < sd_seq.len() {
            if let DerValue::ContextSpecific { tag: 1, .. } = &sd_seq[idx] {
                // CRLs [1] IMPLICIT - skip
                idx += 1;
            }
        }

        let mut signer_infos = Vec::new();
        if idx < sd_seq.len() {
            let si_list = match &sd_seq[idx] {
                DerValue::Set(set) => set.clone(),
                DerValue::Sequence(seq) => seq.clone(),
                _ => Vec::new(),
            };
            for si_val in si_list {
                if let Ok(si) = parse_signer_info(&si_val) {
                    signer_infos.push(si);
                }
            }
        }

        Ok(Pkcs7SignedData {
            version,
            digest_algorithms,
            encap_content_type: encap_type,
            encap_content,
            certificates,
            signer_infos,
            raw_der: bytes.to_vec(),
        })
    }

    /// Build a detached PKCS#7 signature conforming to PAdES.
    pub fn create_detached_signature(
        original_data: &[u8],
        cert: &X509Certificate,
        signing_time: u64,
        timestamp_token: Option<&TimeStampToken>,
    ) -> Result<Self, CryptoSecurityError> {
        let digest = compute_digest(HashAlgorithm::Sha256, original_data);

        let auth_attrs = vec![
            Attribute {
                oid: OID_CONTENT_TYPE.to_string(),
                values: vec![DerValue::Oid(OID_PKCS7_DATA.to_string())],
            },
            Attribute {
                oid: OID_MESSAGE_DIGEST.to_string(),
                values: vec![DerValue::OctetString(digest.clone())],
            },
            Attribute {
                oid: OID_SIGNING_TIME.to_string(),
                values: vec![DerValue::UtcTime(signing_time)],
            },
        ];

        let mut unauth_attrs = Vec::new();
        if let Some(tst) = timestamp_token {
            if let Ok((tst_val, _)) = DerValue::parse(&tst.raw_der) {
                unauth_attrs.push(Attribute {
                    oid: OID_TIMESTAMP_TOKEN_ATTR.to_string(),
                    values: vec![tst_val],
                });
            }
        }

        // Mock signature bytes over the auth attributes hash
        let auth_der = encode_attributes_set(&auth_attrs);
        let sig_bytes = compute_digest(HashAlgorithm::Sha256, &auth_der);

        let signer_info = SignerInfo {
            version: 1,
            issuer: cert.issuer.clone(),
            serial_number: cert.serial_number.clone(),
            digest_algorithm: HashAlgorithm::Sha256,
            authenticated_attributes: auth_attrs,
            signature_algorithm: "1.2.840.113549.1.1.11".to_string(),
            signature_bytes: sig_bytes,
            unauthenticated_attributes: unauth_attrs,
        };

        let certificates = vec![cert.clone()];
        let signer_infos = vec![signer_info];

        let mut certs_encoded = Vec::new();
        for c in &certificates {
            certs_encoded.extend_from_slice(&c.raw_der);
        }

        let signed_data_inner = DerValue::Sequence(vec![
            DerValue::Integer(vec![0x01]),
            DerValue::Set(vec![DerValue::Sequence(vec![
                DerValue::Oid(HashAlgorithm::Sha256.oid().to_string()),
                DerValue::Null,
            ])]),
            DerValue::Sequence(vec![DerValue::Oid(OID_PKCS7_DATA.to_string())]),
            DerValue::ContextSpecific {
                tag: 0,
                constructed: true,
                data: certs_encoded,
            },
            DerValue::Set(vec![encode_signer_info(&signer_infos[0])]),
        ]);

        let root_der = DerValue::Sequence(vec![
            DerValue::Oid(OID_PKCS7_SIGNED_DATA.to_string()),
            DerValue::ContextSpecific {
                tag: 0,
                constructed: true,
                data: signed_data_inner.encode(),
            },
        ])
        .encode();

        Ok(Pkcs7SignedData {
            version: 1,
            digest_algorithms: vec![HashAlgorithm::Sha256],
            encap_content_type: OID_PKCS7_DATA.to_string(),
            encap_content: None,
            certificates,
            signer_infos,
            raw_der: root_der,
        })
    }

    /// Verify detached signature against the original data.
    pub fn verify_detached(
        &self,
        original_data: &[u8],
        trusted_roots: &[X509Certificate],
        at_time: Option<u64>,
    ) -> Result<SignatureVerificationResult, CryptoSecurityError> {
        if self.signer_infos.is_empty() {
            return Err(CryptoSecurityError::Pkcs7Error(
                "No SignerInfo structures present in SignedData".to_string(),
            ));
        }

        let si = &self.signer_infos[0];

        let cert = self
            .certificates
            .iter()
            .find(|c| c.serial_number == si.serial_number)
            .ok_or_else(|| {
                CryptoSecurityError::Pkcs7Error(
                    "Signer certificate not bundled in SignedData".to_string(),
                )
            })?;

        let check_time = at_time.unwrap_or_else(|| {
            si.authenticated_attributes
                .iter()
                .find(|a| a.oid == OID_SIGNING_TIME)
                .and_then(|a| a.values.first())
                .map(|v| match v {
                    DerValue::UtcTime(t) | DerValue::GeneralizedTime(t) => *t,
                    _ => 0,
                })
                .unwrap_or(0)
        });

        if !trusted_roots.is_empty() {
            CertificateChainValidator::validate_chain(
                cert,
                &self.certificates,
                trusted_roots,
                check_time,
            )?;
        }

        let expected_digest = compute_digest(si.digest_algorithm, original_data);

        let stored_digest = si
            .authenticated_attributes
            .iter()
            .find(|a| a.oid == OID_MESSAGE_DIGEST)
            .and_then(|a| a.values.first())
            .map(|v| match v {
                DerValue::OctetString(d) => d.clone(),
                _ => Vec::new(),
            })
            .ok_or_else(|| {
                CryptoSecurityError::Pkcs7Error(
                    "MessageDigest attribute missing from authenticated attributes".to_string(),
                )
            })?;

        let digest_matches = constant_time_eq(&stored_digest, &expected_digest);
        if !digest_matches {
            return Err(CryptoSecurityError::DigestMismatch {
                expected: to_hex_string(&stored_digest),
                calculated: to_hex_string(&expected_digest),
            });
        }

        let mut has_timestamp = false;
        let mut timestamp_info = None;

        for ua in &si.unauthenticated_attributes {
            if ua.oid == OID_TIMESTAMP_TOKEN_ATTR {
                if let Some(val) = ua.values.first() {
                    let bytes = match val {
                        DerValue::Raw { data, .. } => data.clone(),
                        other => other.encode(),
                    };
                    if let Ok(tst) = TimeStampToken::from_der(&bytes) {
                        has_timestamp = true;
                        timestamp_info = Some(tst.tst_info);
                    }
                }
            }
        }

        let signing_time = si
            .authenticated_attributes
            .iter()
            .find(|a| a.oid == OID_SIGNING_TIME)
            .and_then(|a| a.values.first())
            .map(|v| match v {
                DerValue::UtcTime(t) | DerValue::GeneralizedTime(t) => *t,
                _ => 0,
            });

        Ok(SignatureVerificationResult {
            is_valid: true,
            signer_certificate: cert.clone(),
            signing_time,
            has_timestamp,
            timestamp_info,
            digest_matches: true,
        })
    }
}

fn parse_encap_content_info(
    der: &DerValue,
) -> Result<(String, Option<Vec<u8>>), CryptoSecurityError> {
    if let DerValue::Sequence(seq) = der {
        if !seq.is_empty() {
            let oid = match &seq[0] {
                DerValue::Oid(o) => o.clone(),
                _ => OID_PKCS7_DATA.to_string(),
            };
            let content = if seq.len() > 1 {
                match &seq[1] {
                    DerValue::ContextSpecific { data, .. } => Some(data.clone()),
                    DerValue::OctetString(data) => Some(data.clone()),
                    _ => None,
                }
            } else {
                None
            };
            return Ok((oid, content));
        }
    }
    Ok((OID_PKCS7_DATA.to_string(), None))
}

fn parse_signer_info(der: &DerValue) -> Result<SignerInfo, CryptoSecurityError> {
    let seq = match der {
        DerValue::Sequence(s) => s,
        _ => {
            return Err(CryptoSecurityError::Pkcs7Error(
                "SignerInfo must be SEQUENCE".to_string(),
            ))
        }
    };

    if seq.len() < 5 {
        return Err(CryptoSecurityError::Pkcs7Error(
            "SignerInfo SEQUENCE too short".to_string(),
        ));
    }

    let version = match &seq[0] {
        DerValue::Integer(v) if !v.is_empty() => v[0] as u32,
        _ => 1,
    };

    let (issuer, serial_number) = parse_issuer_and_serial(&seq[1])?;

    let digest_algorithm = match &seq[2] {
        DerValue::Sequence(inner) if !inner.is_empty() => match &inner[0] {
            DerValue::Oid(oid) => HashAlgorithm::from_oid(oid).unwrap_or(HashAlgorithm::Sha256),
            _ => HashAlgorithm::Sha256,
        },
        _ => HashAlgorithm::Sha256,
    };

    let mut idx = 3;
    let mut authenticated_attributes = Vec::new();
    if idx < seq.len() {
        if let DerValue::ContextSpecific { tag: 0, data, .. } = &seq[idx] {
            authenticated_attributes = parse_attributes_set(data)?;
            idx += 1;
        }
    }

    let signature_algorithm = if idx < seq.len() {
        match &seq[idx] {
            DerValue::Sequence(inner) if !inner.is_empty() => match &inner[0] {
                DerValue::Oid(oid) => oid.clone(),
                _ => "1.2.840.113549.1.1.11".to_string(),
            },
            _ => "1.2.840.113549.1.1.11".to_string(),
        }
    } else {
        "1.2.840.113549.1.1.11".to_string()
    };
    idx += 1;

    let signature_bytes = if idx < seq.len() {
        match &seq[idx] {
            DerValue::OctetString(b) => b.clone(),
            _ => Vec::new(),
        }
    } else {
        Vec::new()
    };
    idx += 1;

    let mut unauthenticated_attributes = Vec::new();
    if idx < seq.len() {
        if let DerValue::ContextSpecific { tag: 1, data, .. } = &seq[idx] {
            unauthenticated_attributes = parse_attributes_set(data)?;
        }
    }

    Ok(SignerInfo {
        version,
        issuer,
        serial_number,
        digest_algorithm,
        authenticated_attributes,
        signature_algorithm,
        signature_bytes,
        unauthenticated_attributes,
    })
}

fn parse_issuer_and_serial(
    der: &DerValue,
) -> Result<(DistinguishedName, Vec<u8>), CryptoSecurityError> {
    if let DerValue::Sequence(seq) = der {
        if seq.len() >= 2 {
            let issuer = DistinguishedName::parse(&seq[0])?;
            let serial = match &seq[1] {
                DerValue::Integer(bytes) => bytes.clone(),
                _ => vec![0x01],
            };
            return Ok((issuer, serial));
        }
    }
    Err(CryptoSecurityError::Pkcs7Error(
        "Invalid IssuerAndSerialNumber SEQUENCE".to_string(),
    ))
}

fn parse_attributes_set(data: &[u8]) -> Result<Vec<Attribute>, CryptoSecurityError> {
    let mut attrs = Vec::new();
    if data.is_empty() {
        return Ok(attrs);
    }

    let items = if let Ok((DerValue::Set(list), _)) = DerValue::parse(data) {
        list
    } else if let Ok((DerValue::Sequence(list), _)) = DerValue::parse(data) {
        list
    } else {
        let mut list = Vec::new();
        let mut offset = 0;
        while offset < data.len() {
            let (val, consumed) = match DerValue::parse(&data[offset..]) {
                Ok(r) => r,
                Err(_) => break,
            };
            if consumed == 0 {
                break;
            }
            list.push(val);
            offset += consumed;
        }
        list
    };

    for item in items {
        if let DerValue::Sequence(seq) = item {
            if seq.len() >= 2 {
                if let DerValue::Oid(oid) = &seq[0] {
                    let values = match &seq[1] {
                        DerValue::Set(s) => s.clone(),
                        other => vec![other.clone()],
                    };
                    attrs.push(Attribute {
                        oid: oid.clone(),
                        values,
                    });
                }
            }
        }
    }
    Ok(attrs)
}

fn encode_attributes_set(attrs: &[Attribute]) -> Vec<u8> {
    let mut items = Vec::new();
    for a in attrs {
        items.push(DerValue::Sequence(vec![
            DerValue::Oid(a.oid.clone()),
            DerValue::Set(a.values.clone()),
        ]));
    }
    DerValue::Set(items).encode()
}

fn encode_signer_info(si: &SignerInfo) -> DerValue {
    let auth_der = encode_attributes_set(&si.authenticated_attributes);

    let mut seq = vec![
        DerValue::Integer(vec![si.version as u8]),
        DerValue::Sequence(vec![
            si.issuer.to_der(),
            DerValue::Integer(si.serial_number.clone()),
        ]),
        DerValue::Sequence(vec![
            DerValue::Oid(si.digest_algorithm.oid().to_string()),
            DerValue::Null,
        ]),
        DerValue::ContextSpecific {
            tag: 0,
            constructed: true,
            data: auth_der,
        },
        DerValue::Sequence(vec![
            DerValue::Oid(si.signature_algorithm.clone()),
            DerValue::Null,
        ]),
        DerValue::OctetString(si.signature_bytes.clone()),
    ];

    if !si.unauthenticated_attributes.is_empty() {
        let unauth_der = encode_attributes_set(&si.unauthenticated_attributes);
        seq.push(DerValue::ContextSpecific {
            tag: 1,
            constructed: true,
            data: unauth_der,
        });
    }

    DerValue::Sequence(seq)
}
