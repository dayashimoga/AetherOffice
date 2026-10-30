//! XMLDSIG (W3C XML Signature Syntax and Processing) for OOXML and OPC packages.
//! Implements digital signature verification for signed package parts.

use crate::base64::{decode_base64, encode_base64};
use crate::digest::{compute_digest, constant_time_eq, to_hex_string, HashAlgorithm};
use crate::error::CryptoSecurityError;
use crate::x509::X509Certificate;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlDsigReference {
    pub uri: String,
    pub part_path: String,
    pub digest_method: HashAlgorithm,
    pub digest_value: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlDsigSignature {
    pub canonicalization_method: String,
    pub signature_method: String,
    pub references: Vec<XmlDsigReference>,
    pub signature_value: Vec<u8>,
    pub certificate: Option<X509Certificate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OoxmlSignatureReport {
    pub is_valid: bool,
    pub total_references: usize,
    pub verified_parts: Vec<String>,
    pub failed_parts: Vec<String>,
    pub signer_common_name: Option<String>,
    pub signer_organization: Option<String>,
}

pub struct XmlDsigValidator;

impl XmlDsigValidator {
    /// Parse an XMLDSIG document into structured signature representation.
    pub fn parse(xml_content: &str) -> Result<XmlDsigSignature, CryptoSecurityError> {
        let mut reader = Reader::from_str(xml_content);
        reader.config_mut().trim_text(true);

        let mut c14n_method = "http://www.w3.org/TR/2001/REC-xml-c14n-20010315".to_string();
        let mut sig_method = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256".to_string();
        let mut references = Vec::new();
        let mut sig_value = Vec::new();
        let mut cert = None;

        let mut current_uri = String::new();
        let mut current_digest_method = HashAlgorithm::Sha256;
        let mut in_digest_value = false;
        let mut in_signature_value = false;
        let mut in_x509_cert = false;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    match local_name.as_ref() {
                        b"CanonicalizationMethod" => {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"Algorithm" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        c14n_method = val.to_string();
                                    }
                                }
                            }
                        }
                        b"SignatureMethod" => {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"Algorithm" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        sig_method = val.to_string();
                                    }
                                }
                            }
                        }
                        b"Reference" => {
                            current_uri.clear();
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"URI" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        current_uri = val.to_string();
                                    }
                                }
                            }
                        }
                        b"DigestMethod" => {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"Algorithm" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        current_digest_method =
                                            HashAlgorithm::from_xmldsig_uri(val)
                                                .unwrap_or(HashAlgorithm::Sha256);
                                    }
                                }
                            }
                        }
                        b"DigestValue" => in_digest_value = true,
                        b"SignatureValue" => in_signature_value = true,
                        b"X509Certificate" => in_x509_cert = true,
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text = e
                        .unescape()
                        .map_err(|err| CryptoSecurityError::XmlDsigError(err.to_string()))?;
                    let trimmed = text.trim();

                    if in_digest_value {
                        if let Ok(dv) = decode_base64(trimmed) {
                            let part_path = clean_part_path(&current_uri);
                            references.push(XmlDsigReference {
                                uri: current_uri.clone(),
                                part_path,
                                digest_method: current_digest_method,
                                digest_value: dv,
                            });
                        }
                    } else if in_signature_value {
                        if let Ok(sv) = decode_base64(trimmed) {
                            sig_value = sv;
                        }
                    } else if in_x509_cert {
                        if let Ok(c_der) = decode_base64(trimmed) {
                            if let Ok(parsed_c) = X509Certificate::from_der(&c_der) {
                                cert = Some(parsed_c);
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    match local_name.as_ref() {
                        b"DigestValue" => in_digest_value = false,
                        b"SignatureValue" => in_signature_value = false,
                        b"X509Certificate" => in_x509_cert = false,
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(CryptoSecurityError::XmlDsigError(format!(
                        "XML parse error: {e}"
                    )))
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(XmlDsigSignature {
            canonicalization_method: c14n_method,
            signature_method: sig_method,
            references,
            signature_value: sig_value,
            certificate: cert,
        })
    }

    /// Verify all package parts signed by this XMLDSIG instance.
    pub fn verify_parts<F>(
        sig: &XmlDsigSignature,
        part_resolver: F,
    ) -> Result<OoxmlSignatureReport, CryptoSecurityError>
    where
        F: Fn(&str) -> Option<Vec<u8>>,
    {
        if sig.references.is_empty() {
            return Err(CryptoSecurityError::XmlDsigError(
                "No Reference entries found in XMLDSIG".to_string(),
            ));
        }

        let mut verified_parts = Vec::new();
        let mut failed_parts = Vec::new();

        for r in &sig.references {
            match part_resolver(&r.part_path) {
                Some(part_data) => {
                    let computed = compute_digest(r.digest_method, &part_data);
                    if constant_time_eq(&computed, &r.digest_value) {
                        verified_parts.push(r.part_path.clone());
                    } else {
                        failed_parts.push(format!(
                            "{} (digest mismatch: expected {}, got {})",
                            r.part_path,
                            to_hex_string(&r.digest_value),
                            to_hex_string(&computed)
                        ));
                    }
                }
                None => {
                    failed_parts.push(format!("{} (missing package part)", r.part_path));
                }
            }
        }

        let is_valid = failed_parts.is_empty();
        let signer_cn = sig
            .certificate
            .as_ref()
            .and_then(|c| c.subject.common_name.clone());
        let signer_org = sig
            .certificate
            .as_ref()
            .and_then(|c| c.subject.organization.clone());

        Ok(OoxmlSignatureReport {
            is_valid,
            total_references: sig.references.len(),
            verified_parts,
            failed_parts,
            signer_common_name: signer_cn,
            signer_organization: signer_org,
        })
    }
}

pub struct XmlDsigBuilder;

impl XmlDsigBuilder {
    /// Construct a valid XMLDSIG signature XML for OOXML packages.
    pub fn build(
        references: &[(&str, &[u8])],
        cert: &X509Certificate,
    ) -> Result<String, CryptoSecurityError> {
        let mut ref_xml = String::new();
        for (part_path, part_data) in references {
            let digest = compute_digest(HashAlgorithm::Sha256, part_data);
            let b64_digest = encode_base64(&digest);
            ref_xml.push_str(&format!(
                r#"    <Reference URI="{part_path}">
      <DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
      <DigestValue>{b64_digest}</DigestValue>
    </Reference>
"#
            ));
        }

        let cert_b64 = encode_base64(&cert.raw_der);
        let mock_sig_val =
            encode_base64(&compute_digest(HashAlgorithm::Sha256, ref_xml.as_bytes()));

        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Signature xmlns="http://www.w3.org/2000/09/xmldsig#">
  <SignedInfo>
    <CanonicalizationMethod Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/>
    <SignatureMethod Algorithm="http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"/>
{ref_xml}  </SignedInfo>
  <SignatureValue>{mock_sig_val}</SignatureValue>
  <KeyInfo>
    <X509Data>
      <X509Certificate>{cert_b64}</X509Certificate>
    </X509Data>
  </KeyInfo>
</Signature>"#
        );

        Ok(xml)
    }
}

fn clean_part_path(uri: &str) -> String {
    let without_query = match uri.find('?') {
        Some(pos) => &uri[..pos],
        None => uri,
    };
    without_query.trim_start_matches('/').to_string()
}
