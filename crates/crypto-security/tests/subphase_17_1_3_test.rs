//! Comprehensive integration tests for Subphase 17.1.3:
//! Sanitize, Privacy Audit & Forensic Redaction Engine.

use crypto_security::pii::{PiiKind, PiiScanner};
use crypto_security::redact::{PdfRedactor, RedactionBox};
use crypto_security::sanitize::{DocumentSanitizer, SanitizeOptions};

#[test]
fn test_01_pii_scanner_email_phone_ssn_detection() {
    let text =
        "Contact John Doe at john.doe@company.com or direct line (555) 234-5678. SSN: 123-45-6789.";
    let scanner = PiiScanner::new();
    let matches = scanner.scan(text);

    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].kind, PiiKind::Email);
    assert_eq!(matches[0].matched_text, "john.doe@company.com");

    assert_eq!(matches[1].kind, PiiKind::PhoneNumber);
    assert_eq!(matches[1].matched_text, "(555) 234-5678");

    assert_eq!(matches[2].kind, PiiKind::Ssn);
    assert_eq!(matches[2].matched_text, "123-45-6789");
}

#[test]
fn test_02_pii_scanner_credit_card_luhn_filter() {
    // Valid Visa test card (passes Luhn)
    let valid_card = "Card number: 4532 0150 0000 0007 on file";
    // Invalid card (fails Luhn)
    let invalid_card = "Random numbers: 4532 0150 0000 0009 on file";

    let scanner = PiiScanner::new();
    let valid_matches = scanner.scan(valid_card);
    assert_eq!(valid_matches.len(), 1);
    assert_eq!(valid_matches[0].kind, PiiKind::CreditCard);

    let invalid_matches = scanner.scan(invalid_card);
    assert!(invalid_matches.is_empty());
}

#[test]
fn test_03_pii_scanner_ip_and_confidential_keywords() {
    let text = "Server internal IP 192.168.1.105 holds TOP SECRET executive compensation and SALARY benchmarks.";
    let scanner = PiiScanner::new();
    let matches = scanner.scan(text);

    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].kind, PiiKind::IpAddress);
    assert_eq!(matches[0].matched_text, "192.168.1.105");

    assert_eq!(matches[1].kind, PiiKind::SensitiveKeyword);
    assert_eq!(matches[1].matched_text, "TOP SECRET");

    assert_eq!(matches[2].kind, PiiKind::SensitiveKeyword);
    assert_eq!(matches[2].matched_text, "SALARY");
}

#[test]
fn test_04_pii_redact_text_replacement() {
    let text = "Send wire to john.doe@bank.com at IP 10.0.0.1 immediately.";
    let scanner = PiiScanner::new();
    let (redacted, count) = scanner.redact(text);

    assert_eq!(count, 2);
    assert!(!redacted.contains("john.doe@bank.com"));
    assert!(!redacted.contains("10.0.0.1"));
    assert!(redacted.contains("[REDACTED: EMAIL]"));
    assert!(redacted.contains("[REDACTED: IP_ADDRESS]"));
}

#[test]
fn test_05_ooxml_privacy_inspection_detects_metadata_and_comments() {
    let core_xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:creator>Alice Corporate Officer</dc:creator>
  <cp:lastModifiedBy>Bob Consultant</cp:lastModifiedBy>
  <dc:title>Confidential Strategic Plan</dc:title>
</cp:coreProperties>"#;

    let comments_xml = br#"<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:comment w:id="1" w:author="Legal"><w:p><w:r><w:t>Review indemnification</w:t></w:r></w:p></w:comment>
  <w:comment w:id="2" w:author="CFO"><w:p><w:r><w:t>Cap liability at $5M</w:t></w:r></w:p></w:comment>
</w:comments>"#;

    let doc_xml =
        br#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:ins w:id="3" w:author="Alice"><w:r><w:t>Added clause 4.2</w:t></w:r></w:ins></w:p>
    <w:p><w:del w:id="4" w:author="Bob"><w:r><w:t>Removed obsolete text</w:t></w:r></w:del></w:p>
  </w:body>
</w:document>"#;

    let parts: &[(&str, &[u8])] = &[
        ("docProps/core.xml", core_xml),
        ("word/comments.xml", comments_xml),
        ("word/document.xml", doc_xml),
    ];

    let report = DocumentSanitizer::inspect_ooxml(parts);

    assert_eq!(report.author_metadata.len(), 3);
    assert_eq!(report.comments_count, 2);
    assert_eq!(report.tracked_changes_count, 2);
}

#[test]
fn test_06_ooxml_sanitize_strips_core_properties() {
    let core_xml = br#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:creator>Real Author Name</dc:creator>
  <cp:lastModifiedBy>Real Modifier Name</cp:lastModifiedBy>
</cp:coreProperties>"#;

    let parts: &[(&str, &[u8])] = &[("docProps/core.xml", core_xml)];

    let options = SanitizeOptions {
        strip_metadata: true,
        ..Default::default()
    };

    let cleaned = DocumentSanitizer::clean_ooxml(parts, &options);
    assert_eq!(cleaned.len(), 1);

    let cleaned_xml = String::from_utf8(cleaned[0].1.clone()).expect("UTF8 decode");
    assert!(!cleaned_xml.contains("Real Author Name"));
    assert!(!cleaned_xml.contains("Real Modifier Name"));
    assert!(cleaned_xml.contains("Anonymous"));
}

#[test]
fn test_07_ooxml_sanitize_removes_tracked_changes() {
    let doc_xml =
        br#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:ins w:id="1"><w:r><w:t>Accepted new text</w:t></w:r></w:ins>
      <w:del w:id="2"><w:r><w:t>Old deleted text</w:t></w:r></w:del>
    </w:p>
  </w:body>
</w:document>"#;

    let parts: &[(&str, &[u8])] = &[("word/document.xml", doc_xml)];
    let options = SanitizeOptions {
        strip_revisions: true,
        ..Default::default()
    };

    let cleaned = DocumentSanitizer::clean_ooxml(parts, &options);
    let cleaned_xml = String::from_utf8(cleaned[0].1.clone()).expect("UTF8 decode");

    assert!(cleaned_xml.contains("Accepted new text"));
    assert!(!cleaned_xml.contains("Old deleted text"));
    assert!(!cleaned_xml.contains("<w:ins"));
    assert!(!cleaned_xml.contains("<w:del"));
}

#[test]
fn test_08_ooxml_sanitize_removes_hidden_sheets_and_slides() {
    let workbook_xml =
        br#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheets>
    <sheet name="PublicReport" sheetId="1"/>
    <sheet name="SecretFinancials" sheetId="2" state="hidden"/>
    <sheet name="ExecutivePayroll" sheetId="3" state="veryHidden"/>
  </sheets>
</workbook>"#;

    let parts: &[(&str, &[u8])] = &[("xl/workbook.xml", workbook_xml)];
    let options = SanitizeOptions {
        remove_hidden_content: true,
        ..Default::default()
    };

    let cleaned = DocumentSanitizer::clean_ooxml(parts, &options);
    let cleaned_xml = String::from_utf8(cleaned[0].1.clone()).expect("UTF8 decode");

    assert!(!cleaned_xml.contains(r#"state="hidden""#));
    assert!(!cleaned_xml.contains(r#"state="veryHidden""#));
}

#[test]
fn test_09_pdf_forensic_redactor_purges_sensitive_strings() {
    let pdf_stream = "BT\n/F1 12 Tf\n72 712 Td\n(Confidential Settlement: $1,500,000) Tj\nET\n";
    let sensitive = ["$1,500,000"];

    let custom_box = RedactionBox::new(72.0, 710.0, 150.0, 14.0);
    let (redacted_stream, count) =
        PdfRedactor::redact_strings(pdf_stream, &sensitive, Some(custom_box));

    assert_eq!(count, 1);
    assert!(!redacted_stream.contains("$1,500,000"));
    assert!(redacted_stream.contains("0 0 0 rg"));
    assert!(redacted_stream.contains("re f"));
}

#[test]
fn test_10_pdf_forensic_verification_guarantees_zero_leak() {
    let clean_stream = "BT\n/F1 12 Tf\n72 712 Td\n(Disclosed Public Facts) Tj\nET\n";
    let sensitive = ["SecretKey99", "SSN-1234"];

    assert!(PdfRedactor::verify_purged(clean_stream, &sensitive));

    let dirty_stream = "BT\n/F1 12 Tf\n72 712 Td\n(Disclosed SecretKey99 Facts) Tj\nET\n";
    assert!(!PdfRedactor::verify_purged(dirty_stream, &sensitive));
}
