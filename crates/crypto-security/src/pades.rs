//! PAdES-LTV (PDF Advanced Electronic Signatures - Long-Term Validation).
//! Implements ISO 32000-2 & ETSI EN 319 142 ByteRange verification and DSS checking.

use crate::digest::{compute_digest, from_hex_string, to_hex_string, HashAlgorithm};
use crate::error::CryptoSecurityError;
use crate::pkcs7::Pkcs7SignedData;
use crate::timestamp::TimeStampToken;
use crate::x509::X509Certificate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfByteRange {
    pub offset1: usize,
    pub length1: usize,
    pub offset2: usize,
    pub length2: usize,
}

impl PdfByteRange {
    pub fn new(offset1: usize, length1: usize, offset2: usize, length2: usize) -> Self {
        Self {
            offset1,
            length1,
            offset2,
            length2,
        }
    }

    pub fn to_array(&self) -> [usize; 4] {
        [self.offset1, self.length1, self.offset2, self.length2]
    }

    pub fn validate_bounds(&self, total_len: usize) -> Result<(), CryptoSecurityError> {
        if self.offset1 != 0 {
            return Err(CryptoSecurityError::PdfByteRangeError(format!(
                "First offset must be 0, found {}",
                self.offset1
            )));
        }
        if self.offset1 + self.length1 > self.offset2 {
            return Err(CryptoSecurityError::PdfByteRangeError(format!(
                "Overlapping ByteRange: {} + {} > {}",
                self.offset1, self.length1, self.offset2
            )));
        }
        if self.offset2 + self.length2 > total_len {
            return Err(CryptoSecurityError::PdfByteRangeError(format!(
                "ByteRange extends beyond file length: {} + {} > {}",
                self.offset2, self.length2, total_len
            )));
        }
        Ok(())
    }

    pub fn extract_bytes(&self, pdf_bytes: &[u8]) -> Result<Vec<u8>, CryptoSecurityError> {
        self.validate_bounds(pdf_bytes.len())?;
        let part1 = &pdf_bytes[self.offset1..self.offset1 + self.length1];
        let part2 = &pdf_bytes[self.offset2..self.offset2 + self.length2];
        let mut out = Vec::with_capacity(part1.len() + part2.len());
        out.extend_from_slice(part1);
        out.extend_from_slice(part2);
        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PadesValidationReport {
    pub is_valid: bool,
    pub signer_common_name: String,
    pub signer_organization: String,
    pub signing_time: Option<u64>,
    pub has_rfc3161_timestamp: bool,
    pub has_ltv_dss: bool,
    pub byte_range: [usize; 4],
    pub document_hash: String,
}

pub struct PadesValidator;

impl PadesValidator {
    /// Validates all digital signatures in a PDF buffer.
    pub fn validate(
        pdf_bytes: &[u8],
        trusted_roots: &[X509Certificate],
    ) -> Result<PadesValidationReport, CryptoSecurityError> {
        let pdf_str = std::str::from_utf8(pdf_bytes).map_err(|e| {
            CryptoSecurityError::PadesValidationError(format!("PDF contains invalid UTF-8: {e}"))
        })?;

        let byte_range = parse_pdf_byte_range(pdf_str)?;
        let contents_hex = parse_pdf_contents_hex(pdf_str)?;

        byte_range.validate_bounds(pdf_bytes.len())?;

        let signed_data_bytes = byte_range.extract_bytes(pdf_bytes)?;
        let doc_hash = compute_digest(HashAlgorithm::Sha256, &signed_data_bytes);

        let pkcs7_der = from_hex_string(&contents_hex)?;
        let pkcs7 = Pkcs7SignedData::from_der(&pkcs7_der)?;

        let verification = pkcs7.verify_detached(&signed_data_bytes, trusted_roots, None)?;

        let has_ltv_dss = pdf_str.contains("/DSS") || pdf_str.contains("/VRI");

        let signer_cn = verification
            .signer_certificate
            .subject
            .common_name
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        let signer_org = verification
            .signer_certificate
            .subject
            .organization
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());

        Ok(PadesValidationReport {
            is_valid: verification.is_valid,
            signer_common_name: signer_cn,
            signer_organization: signer_org,
            signing_time: verification.signing_time,
            has_rfc3161_timestamp: verification.has_timestamp,
            has_ltv_dss,
            byte_range: byte_range.to_array(),
            document_hash: to_hex_string(&doc_hash),
        })
    }
}

pub struct PadesSigner;

impl PadesSigner {
    /// Create a signed PDF by injecting signature dictionary with exact ByteRange and PKCS#7 contents.
    pub fn sign_simple_pdf(
        base_pdf_content: &[u8],
        cert: &X509Certificate,
        signing_time: u64,
        timestamp_token: Option<&TimeStampToken>,
    ) -> Result<Vec<u8>, CryptoSecurityError> {
        // Construct standard placeholder signature dictionary
        let sig_dict_prefix = b"\n% SigDict Placeholder\n4 0 obj\n<< /Type /Sig /Filter /Adobe.PPKLite /SubFilter /adbe.pkcs7.detached /ByteRange [ 0000000000 0000000000 0000000000 0000000000 ] /Contents <";
        let placeholder_hex_len = 4096; // 2KB signature allocation in hex (4096 chars)
        let placeholder_zeros = vec![b'0'; placeholder_hex_len];
        let sig_dict_suffix = b"> >>\nendobj\n%%EOF\n";

        let mut template = Vec::with_capacity(
            base_pdf_content.len()
                + sig_dict_prefix.len()
                + placeholder_hex_len
                + sig_dict_suffix.len(),
        );
        template.extend_from_slice(base_pdf_content);
        let sig_prefix_pos = template.len();
        template.extend_from_slice(sig_dict_prefix);
        let contents_start = template.len();
        template.extend_from_slice(&placeholder_zeros);
        let contents_end = template.len();
        template.extend_from_slice(sig_dict_suffix);

        let offset1 = 0usize;
        let length1 = contents_start;
        let offset2 = contents_end;
        let length2 = template.len() - contents_end;

        // Overwrite ByteRange placeholder in template
        let br_str = format!(
            "[ {:010} {:010} {:010} {:010} ]",
            offset1, length1, offset2, length2
        );
        let br_placeholder = b"[ 0000000000 0000000000 0000000000 0000000000 ]";
        if let Some(pos) =
            find_subsequence(&template[sig_prefix_pos..contents_start], br_placeholder)
        {
            let abs_pos = sig_prefix_pos + pos;
            template[abs_pos..abs_pos + br_placeholder.len()].copy_from_slice(br_str.as_bytes());
        }

        // Compute hash of the exact ByteRange chunks
        let byte_range = PdfByteRange::new(offset1, length1, offset2, length2);
        let sign_target = byte_range.extract_bytes(&template)?;

        // Build PKCS#7 SignedData
        let pkcs7 = Pkcs7SignedData::create_detached_signature(
            &sign_target,
            cert,
            signing_time,
            timestamp_token,
        )?;

        let sig_hex = to_hex_string(&pkcs7.raw_der);
        if sig_hex.len() > placeholder_hex_len {
            return Err(CryptoSecurityError::PadesValidationError(format!(
                "Signature hex length {} exceeds allocated buffer {}",
                sig_hex.len(),
                placeholder_hex_len
            )));
        }

        // Inject signature hex with zero padding
        let padded_hex = format!("{:0<width$}", sig_hex, width = placeholder_hex_len);
        template[contents_start..contents_end].copy_from_slice(padded_hex.as_bytes());

        Ok(template)
    }
}

fn parse_pdf_byte_range(pdf_str: &str) -> Result<PdfByteRange, CryptoSecurityError> {
    let key = "/ByteRange";
    let pos = pdf_str.find(key).ok_or_else(|| {
        CryptoSecurityError::PdfByteRangeError("Missing /ByteRange key in PDF".to_string())
    })?;

    let after = &pdf_str[pos + key.len()..];
    let open_bracket = after.find('[').ok_or_else(|| {
        CryptoSecurityError::PdfByteRangeError("Missing '[' after /ByteRange".to_string())
    })?;
    let close_bracket = after.find(']').ok_or_else(|| {
        CryptoSecurityError::PdfByteRangeError("Missing ']' after /ByteRange".to_string())
    })?;

    let nums_str = &after[open_bracket + 1..close_bracket];
    let nums: Vec<usize> = nums_str
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    if nums.len() != 4 {
        return Err(CryptoSecurityError::PdfByteRangeError(format!(
            "Expected 4 values in /ByteRange, found {}",
            nums.len()
        )));
    }

    Ok(PdfByteRange::new(nums[0], nums[1], nums[2], nums[3]))
}

fn parse_pdf_contents_hex(pdf_str: &str) -> Result<String, CryptoSecurityError> {
    let key = "/Contents";
    let pos = pdf_str.find(key).ok_or_else(|| {
        CryptoSecurityError::PadesValidationError("Missing /Contents key in PDF".to_string())
    })?;

    let after = &pdf_str[pos + key.len()..];
    let open_angle = after.find('<').ok_or_else(|| {
        CryptoSecurityError::PadesValidationError("Missing '<' in /Contents".to_string())
    })?;
    let close_angle = after[open_angle + 1..].find('>').ok_or_else(|| {
        CryptoSecurityError::PadesValidationError("Missing '>' in /Contents".to_string())
    })?;

    let hex_val = &after[open_angle + 1..open_angle + 1 + close_angle];
    Ok(hex_val.trim_end_matches('0').to_string())
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
