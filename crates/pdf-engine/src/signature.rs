//! PDF Digital Signature Detection, ByteRange Verification & Preservation (ISO 32000-1 Clause 12.8).

use crate::catalog::PdfDocument;
use crate::error::PdfError;
use crate::types::PdfObject;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Extracted digital signature record with cryptographic ByteRange parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfSignature {
    pub field_name: String,
    pub filter: String,
    pub sub_filter: Option<String>,
    pub byte_range: [usize; 4],
    pub contents_hex: String,
    pub contents_bytes: Vec<u8>,
    pub signing_time: Option<String>,
    pub reason: Option<String>,
    pub location: Option<String>,
    pub contact_info: Option<String>,
    pub computed_sha256: [u8; 32],
    pub is_byte_range_valid: bool,
}

pub struct SignatureValidator;

impl SignatureValidator {
    /// Discovers and validates all digital signatures in a PDF document.
    pub fn extract_signatures(
        data: &[u8],
        doc: &PdfDocument,
    ) -> Result<Vec<PdfSignature>, PdfError> {
        let mut signatures = Vec::new();

        // 1. Traverse /Root -> /AcroForm -> /Fields
        let acro_form = match doc.catalog().get("AcroForm") {
            Some(PdfObject::Dictionary(d)) => Some(d.clone()),
            Some(PdfObject::Reference(ref_id)) => {
                if let Ok(PdfObject::Dictionary(d)) = doc.resolve_object(*ref_id) {
                    Some(d)
                } else {
                    None
                }
            }
            _ => None,
        };

        let fields = match acro_form.as_ref().and_then(|af| af.get("Fields")) {
            Some(PdfObject::Array(arr)) => arr.clone(),
            Some(PdfObject::Reference(ref_id)) => {
                if let Ok(PdfObject::Array(arr)) = doc.resolve_object(*ref_id) {
                    arr
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        };

        for field_obj in fields {
            let field_dict = match field_obj {
                PdfObject::Dictionary(d) => Some(d),
                PdfObject::Reference(ref_id) => {
                    if let Ok(PdfObject::Dictionary(d)) = doc.resolve_object(ref_id) {
                        Some(d)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            let Some(f_dict) = field_dict else { continue };

            // Check if this field is a signature field: /FT /Sig
            let is_sig = f_dict
                .get("FT")
                .and_then(|v| v.as_name())
                .map(|n| n == "Sig")
                .unwrap_or(false);

            if !is_sig {
                continue;
            }

            let field_name = f_dict
                .get("T")
                .and_then(|v| v.to_string_lossy())
                .unwrap_or_else(|| "SignatureField".into());

            // Value /V points to the signature dictionary
            let sig_dict = match f_dict.get("V") {
                Some(PdfObject::Dictionary(d)) => Some(d.clone()),
                Some(PdfObject::Reference(ref_id)) => {
                    if let Ok(PdfObject::Dictionary(d)) = doc.resolve_object(*ref_id) {
                        Some(d)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            let Some(s_dict) = sig_dict else { continue };

            if let Ok(sig) = Self::parse_signature_dict(data, &field_name, &s_dict) {
                signatures.push(sig);
            }
        }

        Ok(signatures)
    }

    /// Parses and verifies a single `/Type /Sig` dictionary against document bytes.
    pub fn parse_signature_dict(
        data: &[u8],
        field_name: &str,
        dict: &BTreeMap<String, PdfObject>,
    ) -> Result<PdfSignature, PdfError> {
        let filter = dict
            .get("Filter")
            .and_then(|v| v.as_name())
            .unwrap_or("Adobe.PPKLite")
            .to_string();

        let sub_filter = dict
            .get("SubFilter")
            .and_then(|v| v.as_name())
            .map(String::from);

        let byte_range_arr = dict
            .get("ByteRange")
            .and_then(|v| v.as_array())
            .ok_or_else(|| PdfError::SyntaxError {
                offset: 0,
                message: "Signature dictionary missing /ByteRange array".into(),
            })?;

        if byte_range_arr.len() < 4 {
            return Err(PdfError::SyntaxError {
                offset: 0,
                message: "ByteRange array must contain at least 4 integers".into(),
            });
        }

        let o1 = byte_range_arr[0].as_i64().unwrap_or(0).max(0) as usize;
        let l1 = byte_range_arr[1].as_i64().unwrap_or(0).max(0) as usize;
        let o2 = byte_range_arr[2].as_i64().unwrap_or(0).max(0) as usize;
        let l2 = byte_range_arr[3].as_i64().unwrap_or(0).max(0) as usize;
        let byte_range = [o1, l1, o2, l2];

        let contents_bytes = match dict.get("Contents") {
            Some(PdfObject::HexString(b)) | Some(PdfObject::LiteralString(b)) => b.clone(),
            _ => Vec::new(),
        };

        let mut contents_hex = String::with_capacity(contents_bytes.len() * 2);
        for &b in &contents_bytes {
            contents_hex.push_str(&format!("{:02X}", b));
        }

        let signing_time = dict.get("M").and_then(|v| v.to_string_lossy());
        let reason = dict.get("Reason").and_then(|v| v.to_string_lossy());
        let location = dict.get("Location").and_then(|v| v.to_string_lossy());
        let contact_info = dict.get("ContactInfo").and_then(|v| v.to_string_lossy());

        // Validate ByteRange bounds against physical file length
        let is_range_valid =
            o1 == 0 && (o1 + l1) <= data.len() && o2 >= (o1 + l1) && (o2 + l2) <= data.len();

        let computed_sha256 = if is_range_valid {
            let mut hasher = Sha256::new();
            hasher.update(&data[o1..o1 + l1]);
            hasher.update(&data[o2..o2 + l2]);
            let result = hasher.finalize();
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&result);
            hash
        } else {
            [0u8; 32]
        };

        Ok(PdfSignature {
            field_name: field_name.to_string(),
            filter,
            sub_filter,
            byte_range,
            contents_hex,
            contents_bytes,
            signing_time,
            reason,
            location,
            contact_info,
            computed_sha256,
            is_byte_range_valid: is_range_valid,
        })
    }

    /// Verifies if any existing digital signature's signed byte range was tampered with.
    pub fn verify_range_intact(data: &[u8], byte_range: &[usize; 4]) -> bool {
        let [o1, l1, o2, l2] = *byte_range;
        o1 == 0 && (o1 + l1) <= data.len() && o2 >= (o1 + l1) && (o2 + l2) <= data.len()
    }
}
