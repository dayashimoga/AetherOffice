//! Sub-Phase 49.2.2 Integration Tests:
//! Cryptographic PAdES-LTV Signing, Redaction Stream Purging & PDF/A-2b Export
//!
//! Validates:
//! - PAdES detached digital signature creation with exact ByteRange allocation
//! - Long-Term Validation (LTV) `/DSS` dictionary injection
//! - Complete digital signature discovery and cryptographic integrity validation
//! - Graceful handling of unsigned documents (empty signature list)
//! - Irreversible physical string purging from PDF content streams
//! - Visual opaque solid fill rectangle stamping over redacted coordinates
//! - Forensic sanitization verification across multi-page content streams
//! - Standard ISO 19005-2 PDF/A-2b compliant XMP metadata packet injection
//! - DeviceRGB OutputIntent identifier injection for PDF/A compliance
//! - Stripping of forbidden PDF/A elements (e.g. JavaScript, Additional Actions)

use crypto_security::x509::X509Certificate;
use pdf_engine::diff::DiffBoundingBox;
use pdf_engine::security_export::{
    PdfRedactionTarget, PdfSecurityExportEngine, PdfSigningOptions, PdfaExportOptions,
};

fn create_test_pdf(content_streams: &[&str]) -> Vec<u8> {
    let mut doc_bytes = Vec::new();
    doc_bytes.extend_from_slice(b"%PDF-1.7\n");

    let num_pages = content_streams.len();
    let total_objs = 2 + num_pages * 3;
    let mut offsets = vec![0usize; total_objs + 1];

    // Catalog (1 0 obj)
    offsets[1] = doc_bytes.len();
    doc_bytes.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // Kids array for Pages dict
    let mut kids_str = String::from("[ ");
    for i in 0..num_pages {
        let page_id = 3 + i * 3;
        kids_str.push_str(&format!("{page_id} 0 R "));
    }
    kids_str.push(']');

    // Pages (2 0 obj)
    offsets[2] = doc_bytes.len();
    let pages_obj = format!(
        "2 0 obj\n<< /Type /Pages /Kids {} /Count {} /MediaBox [ 0 0 612 792 ] >>\nendobj\n",
        kids_str, num_pages
    );
    doc_bytes.extend_from_slice(pages_obj.as_bytes());

    for (i, content_text) in content_streams.iter().enumerate() {
        let page_id = 3 + i * 3;
        let content_id = 4 + i * 3;
        let font_id = 5 + i * 3;

        // Font obj
        offsets[font_id] = doc_bytes.len();
        let font_obj = format!(
            "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
            font_id
        );
        doc_bytes.extend_from_slice(font_obj.as_bytes());

        // Page obj
        offsets[page_id] = doc_bytes.len();
        let page_dict = format!(
            "{} 0 obj\n<< /Type /Page /Parent 2 0 R /Contents {} 0 R /Resources << /Font << /F1 {} 0 R >> >> /MediaBox [ 0 0 612 792 ] >>\nendobj\n",
            page_id, content_id, font_id
        );
        doc_bytes.extend_from_slice(page_dict.as_bytes());

        // Content stream
        let stream_body = format!("BT /F1 12 Tf 50 700 Td ({}) Tj ET\n", content_text);
        offsets[content_id] = doc_bytes.len();
        let stream_obj = format!(
            "{} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
            content_id,
            stream_body.len(),
            stream_body
        );
        doc_bytes.extend_from_slice(stream_obj.as_bytes());
    }

    // XRef & Trailer
    let xref_offset = doc_bytes.len();
    let num_entries = total_objs + 1;
    let mut xref_str = format!("xref\n0 {num_entries}\n0000000000 65535 f \r\n");
    for off in &offsets[1..=total_objs] {
        xref_str.push_str(&format!("{:010} 00000 n \r\n", off));
    }
    xref_str.push_str(&format!(
        "trailer\n<< /Size {num_entries} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF"
    ));
    doc_bytes.extend_from_slice(xref_str.as_bytes());

    doc_bytes
}

fn create_test_cert_pem() -> String {
    let cert = X509Certificate::create_mock(
        "AetherOffice Official Signer",
        "AetherOffice Security CA",
        42,
        1700000000,
        1800000000,
    );
    cert.to_pem()
}

#[test]
fn test_01_pades_signing_generates_valid_bytedict() {
    let pdf = create_test_pdf(&["Contract for enterprise licensing agreement"]);
    let cert_pem = create_test_cert_pem();

    let options = PdfSigningOptions {
        signer_name: "Corporate Officer".to_string(),
        reason: Some("Authorized Executive Signature".to_string()),
        location: Some("San Francisco, CA".to_string()),
        contact_info: None,
        add_ltv_dss: false,
        signing_time: Some(1750000000),
    };

    let signed_bytes = PdfSecurityExportEngine::sign_pades(&pdf, &cert_pem, "", Some(&options))
        .expect("PAdES signing must succeed");

    let signed_str = String::from_utf8_lossy(&signed_bytes);
    assert!(signed_str.contains("/Type /Sig"));
    assert!(signed_str.contains("/Filter /Adobe.PPKLite"));
    assert!(signed_str.contains("/SubFilter /adbe.pkcs7.detached"));
    assert!(signed_str.contains("/ByteRange ["));
    assert!(signed_str.contains("/Contents <"));
}

#[test]
fn test_02_pades_signing_with_ltv_dss() {
    let pdf = create_test_pdf(&["Standard Non-Disclosure Terms"]);
    let cert_pem = create_test_cert_pem();

    let options = PdfSigningOptions {
        signer_name: "Legal Counsel".to_string(),
        reason: Some("Approval".to_string()),
        location: None,
        contact_info: None,
        add_ltv_dss: true,
        signing_time: Some(1750000000),
    };

    let signed_bytes = PdfSecurityExportEngine::sign_pades(&pdf, &cert_pem, "", Some(&options))
        .expect("PAdES-LTV signing");

    let signed_str = String::from_utf8_lossy(&signed_bytes);
    assert!(
        signed_str.contains("/Type /DSS"),
        "Must contain /DSS dictionary"
    );
    assert!(
        signed_str.contains("/VRI"),
        "Must contain /VRI validation record"
    );
}

#[test]
fn test_03_validate_signed_pdf_success() {
    let pdf = create_test_pdf(&["Financial Disclosure Document"]);
    let cert_pem = create_test_cert_pem();

    let signed = PdfSecurityExportEngine::sign_pades(&pdf, &cert_pem, "", None)
        .expect("Sign for validation");

    let reports =
        PdfSecurityExportEngine::validate_signatures(&signed).expect("Validate signatures");

    assert_eq!(reports.len(), 1);
    let r = &reports[0];
    if !r.is_valid {
        eprintln!("VALIDATION ERROR: {:?}", r.error_message);
    }
    assert!(r.is_valid, "Signature must be cryptographically valid");
    assert_eq!(r.signer_common_name, "AetherOffice Official Signer");
    assert!(r.has_ltv_dss, "Should detect LTV DSS dictionary");
}

#[test]
fn test_04_validate_unsigned_pdf_returns_empty() {
    let pdf = create_test_pdf(&["Plain unsigned memorandum"]);

    let reports =
        PdfSecurityExportEngine::validate_signatures(&pdf).expect("Validate unsigned document");

    assert!(
        reports.is_empty(),
        "Unsigned document must return empty signature list"
    );
}

#[test]
fn test_05_redact_sensitive_strings_physically_purged() {
    let secret = "SSN: 999-88-7777";
    let pdf = create_test_pdf(&[&format!("Confidential Client Info {}", secret)]);

    let redactions = vec![PdfRedactionTarget {
        page_index: 0,
        bbox: None,
        text_pattern: Some(secret.to_string()),
        fill_color_rgb: None,
        overlay_text: Some("[REDACTED]".to_string()),
    }];

    let (redacted_bytes, report) =
        PdfSecurityExportEngine::apply_redactions(&pdf, &redactions).expect("Apply redactions");

    assert!(report.total_redactions_applied >= 1);
    assert!(
        report.forensically_sanitized,
        "Document must pass forensic verification"
    );
    assert!(report.purged_strings.contains(&secret.to_string()));

    let redacted_str = String::from_utf8_lossy(&redacted_bytes);
    assert!(
        !redacted_str.contains(secret),
        "Sensitive SSN must be completely absent from raw bytes"
    );
}

#[test]
fn test_06_redact_with_bounding_box_renders_opaque_rectangle() {
    let pdf = create_test_pdf(&["Patent Claim Coordinates"]);

    let redactions = vec![PdfRedactionTarget {
        page_index: 0,
        bbox: Some(DiffBoundingBox {
            x: 72.0,
            y: 700.0,
            width: 150.0,
            height: 25.0,
        }),
        text_pattern: None,
        fill_color_rgb: None,
        overlay_text: None,
    }];

    let (redacted_bytes, report) =
        PdfSecurityExportEngine::apply_redactions(&pdf, &redactions).expect("Redact bounding box");

    assert!(report.total_redactions_applied >= 1);

    let redacted_str = String::from_utf8_lossy(&redacted_bytes);
    // Verifies opaque black box path operator '0 0 0 rg 72.00 700.00 150.00 25.00 re f'
    assert!(redacted_str.contains("0 0 0 rg"));
    assert!(redacted_str.contains("72.00 700.00 150.00 25.00 re f"));
}

#[test]
fn test_07_redact_multiple_occurrences_and_pages() {
    let token = "CLASSIFIED_PROJECT_OMEGA";
    let pdf = create_test_pdf(&[
        &format!("Page 1 mentions {}", token),
        &format!("Page 2 also references {}", token),
    ]);

    let redactions = vec![PdfRedactionTarget {
        page_index: 0,
        bbox: None,
        text_pattern: Some(token.to_string()),
        fill_color_rgb: None,
        overlay_text: None,
    }];

    let (redacted_bytes, report) = PdfSecurityExportEngine::apply_redactions(&pdf, &redactions)
        .expect("Multi-occurrence redaction");

    assert!(report.total_redactions_applied >= 2);
    assert_eq!(report.pages_redacted.len(), 1); // target specified page 0
    let out_str = String::from_utf8_lossy(&redacted_bytes);
    assert!(!out_str.contains(token));
}

#[test]
fn test_08_pdfa_export_xmp_metadata_packet() {
    let pdf = create_test_pdf(&["Standard Document Content"]);

    let options = PdfaExportOptions {
        conformance: "2b".to_string(),
        title: Some("Permanent Enterprise Record".to_string()),
        author: Some("AetherOffice Legal Archive".to_string()),
        output_condition: Some("sRGB IEC61966-2.1".to_string()),
    };

    let pdfa_bytes =
        PdfSecurityExportEngine::export_pdfa(&pdf, Some(&options)).expect("Export PDF/A-2b");

    let pdfa_str = String::from_utf8_lossy(&pdfa_bytes);
    assert!(pdfa_str.contains("<?xpacket begin="));
    assert!(pdfa_str.contains("<pdfaid:part>2</pdfaid:part>"));
    assert!(pdfa_str.contains("<pdfaid:conformance>B</pdfaid:conformance>"));
    assert!(pdfa_str.contains("<dc:title><rdf:Alt><rdf:li xml:lang=\"x-default\">Permanent Enterprise Record</rdf:li></rdf:Alt></dc:title>"));
    assert!(pdfa_str.contains("<?xpacket end=\"w\"?>"));
}

#[test]
fn test_09_pdfa_export_output_intent_injected() {
    let pdf = create_test_pdf(&["Archival Specimen"]);

    let pdfa_bytes = PdfSecurityExportEngine::export_pdfa(&pdf, None).expect("Export PDF/A");

    let pdfa_str = String::from_utf8_lossy(&pdfa_bytes);
    assert!(pdfa_str.contains("/Type /OutputIntent"));
    assert!(pdfa_str.contains("/S /GTS_PDFA1"));
    assert!(pdfa_str.contains("/OutputConditionIdentifier (sRGB IEC61966-2.1)"));
}

#[test]
fn test_10_pdfa_export_strips_forbidden_javascript() {
    let pdf = create_test_pdf(&["Interactive Form Scripting /JavaScript /AA"]);
    let pdfa_bytes =
        PdfSecurityExportEngine::export_pdfa(&pdf, None).expect("Export sanitized PDF/A");

    let pdfa_str = String::from_utf8_lossy(&pdfa_bytes);
    assert!(!pdfa_str.contains("/JavaScript"));
    assert!(!pdfa_str.contains("/AA"));
}
