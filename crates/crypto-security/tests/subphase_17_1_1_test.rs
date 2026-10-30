//! Comprehensive integration tests for Subphase 17.1.1:
//! PAdES-LTV, X.509 & PKCS#7 Digital Signatures for PDF and OOXML.

use crypto_security::asn1::DerValue;
use crypto_security::base64::{decode_base64, encode_base64};
use crypto_security::digest::{compute_digest, constant_time_eq, verify_digest, HashAlgorithm};
use crypto_security::pades::{PadesSigner, PadesValidator};
use crypto_security::pkcs7::Pkcs7SignedData;
use crypto_security::timestamp::TimeStampToken;
use crypto_security::x509::{CertificateChainValidator, X509Certificate};
use crypto_security::xmldsig::{XmlDsigBuilder, XmlDsigValidator};

#[test]
fn test_01_sha_digest_family_and_constant_time_comparison() {
    let payload = b"Hello, AetherOffice Enterprise Cryptography!";

    let d256 = compute_digest(HashAlgorithm::Sha256, payload);
    assert_eq!(d256.len(), 32);
    assert!(verify_digest(HashAlgorithm::Sha256, payload, &d256).is_ok());

    let d384 = compute_digest(HashAlgorithm::Sha384, payload);
    assert_eq!(d384.len(), 48);
    assert!(verify_digest(HashAlgorithm::Sha384, payload, &d384).is_ok());

    let d512 = compute_digest(HashAlgorithm::Sha512, payload);
    assert_eq!(d512.len(), 64);
    assert!(verify_digest(HashAlgorithm::Sha512, payload, &d512).is_ok());

    // Tampered payload fails
    let tampered = b"Hello, AetherOffice Enterprise Cryptography?";
    assert!(verify_digest(HashAlgorithm::Sha256, tampered, &d256).is_err());

    // Constant-time comparison
    assert!(constant_time_eq(&d256, &d256));
    let mut modified = d256.clone();
    modified[0] ^= 0xFF;
    assert!(!constant_time_eq(&d256, &modified));
}

#[test]
fn test_02_asn1_der_primitives_roundtrip() {
    let original = DerValue::Sequence(vec![
        DerValue::Boolean(true),
        DerValue::Integer(vec![0x00, 0xFF, 0x12]),
        DerValue::OctetString(b"Secret Document Stream".to_vec()),
        DerValue::Oid("1.2.840.113549.1.1.11".to_string()),
        DerValue::Utf8String("Jane Doe - Chief Legal Officer".to_string()),
        DerValue::Set(vec![
            DerValue::PrintableString("US".to_string()),
            DerValue::UtcTime(1770000000),
        ]),
    ]);

    let der_bytes = original.encode();
    assert!(!der_bytes.is_empty());

    let (parsed, consumed) = DerValue::parse(&der_bytes).expect("DER parse failed");
    assert_eq!(consumed, der_bytes.len());
    assert_eq!(original, parsed);
}

#[test]
fn test_03_base64_encode_decode_roundtrip() {
    let test_cases: &[&[u8]] = &[
        b"",
        b"f",
        b"fo",
        b"foo",
        b"foob",
        b"fooba",
        b"foobar",
        b"Cryptographic Signatures in Rust with #![forbid(unsafe_code)]",
    ];

    for data in test_cases {
        let encoded = encode_base64(data);
        let decoded = decode_base64(&encoded).expect("Base64 decode failed");
        assert_eq!(*data, decoded.as_slice());
    }

    // Whitespace handling in PEM blocks
    let with_whitespace = "   Y\nWFt  Ym\r\nVy   ";
    let decoded = decode_base64(with_whitespace).expect("Whitespace base64 decode failed");
    assert_eq!(decoded, b"aamber");
}

#[test]
fn test_04_x509_certificate_mock_creation_and_pem_roundtrip() {
    let cert = X509Certificate::create_mock(
        "AetherOffice Signer",
        "AetherOffice Root CA",
        1001,
        1700000000,
        1800000000,
    );

    assert_eq!(cert.version, 3);
    assert_eq!(
        cert.subject.common_name.as_deref(),
        Some("AetherOffice Signer")
    );
    assert_eq!(
        cert.issuer.common_name.as_deref(),
        Some("AetherOffice Root CA")
    );
    assert!(!cert.is_self_signed());

    // Validity check
    assert!(cert.is_valid_at(1750000000).is_ok());
    assert!(cert.is_valid_at(1600000000).is_err());
    assert!(cert.is_valid_at(1900000000).is_err());

    // PEM roundtrip
    let pem = cert.to_pem();
    assert!(pem.contains("-----BEGIN CERTIFICATE-----"));
    assert!(pem.contains("-----END CERTIFICATE-----"));

    let parsed = X509Certificate::from_pem(&pem).expect("PEM cert parse failed");
    assert_eq!(
        parsed.subject.common_name.as_deref(),
        Some("AetherOffice Signer")
    );
    assert_eq!(parsed.fingerprint_sha256(), cert.fingerprint_sha256());
}

#[test]
fn test_05_x509_certificate_chain_validation() {
    let root = X509Certificate::create_mock(
        "AetherOffice Root CA",
        "AetherOffice Root CA",
        1,
        1700000000,
        1900000000,
    );
    assert!(root.is_self_signed());

    let intermediate = X509Certificate::create_mock(
        "AetherOffice Intermediate CA",
        "AetherOffice Root CA",
        2,
        1700000000,
        1850000000,
    );

    let leaf = X509Certificate::create_mock(
        "Alice Department Signer",
        "AetherOffice Intermediate CA",
        3,
        1710000000,
        1800000000,
    );

    let trusted_roots = vec![root.clone()];
    let intermediates = vec![intermediate.clone()];

    // Valid chain at timestamp 1750000000
    let res = CertificateChainValidator::validate_chain(
        &leaf,
        &intermediates,
        &trusted_roots,
        1750000000,
    );
    assert!(res.is_ok());

    // Expired leaf at timestamp 1810000000
    let res_expired = CertificateChainValidator::validate_chain(
        &leaf,
        &intermediates,
        &trusted_roots,
        1810000000,
    );
    assert!(res_expired.is_err());

    // Missing intermediate
    let empty_intermediates: Vec<X509Certificate> = Vec::new();
    let res_missing = CertificateChainValidator::validate_chain(
        &leaf,
        &empty_intermediates,
        &trusted_roots,
        1750000000,
    );
    assert!(res_missing.is_err());
}

#[test]
fn test_06_rfc3161_timestamp_token_verification() {
    let doc_data = b"Quarterly Financial Statement 2026-Q3";
    let digest = compute_digest(HashAlgorithm::Sha256, doc_data);

    let tsa_cert = X509Certificate::create_mock(
        "AetherOffice Qualified TSA",
        "AetherOffice Root CA",
        55,
        1700000000,
        1800000000,
    );

    let token =
        TimeStampToken::create_mock(&digest, HashAlgorithm::Sha256, 1750000000, Some(&tsa_cert));

    // Token parses and verifies
    let parsed_token =
        TimeStampToken::from_der(&token.raw_der).expect("TimeStampToken DER parse failed");
    assert_eq!(parsed_token.tst_info.gen_time, 1750000000);
    assert!(parsed_token.verify(&digest).is_ok());

    // Tampered digest fails verification
    let wrong_digest = compute_digest(HashAlgorithm::Sha256, b"Tampered Document Content");
    assert!(parsed_token.verify(&wrong_digest).is_err());
}

#[test]
fn test_07_pkcs7_detached_signature_roundtrip_and_tampering() {
    let doc_bytes = b"Contract Agreement between Parties A and B";

    let cert = X509Certificate::create_mock(
        "Bob Partner",
        "AetherOffice Root CA",
        102,
        1700000000,
        1800000000,
    );

    let digest = compute_digest(HashAlgorithm::Sha256, doc_bytes);
    let tst = TimeStampToken::create_mock(&digest, HashAlgorithm::Sha256, 1750000000, None);

    let pkcs7 =
        Pkcs7SignedData::create_detached_signature(doc_bytes, &cert, 1750000000, Some(&tst))
            .expect("Failed to build PKCS#7 signature");

    let parsed_pkcs7 = Pkcs7SignedData::from_der(&pkcs7.raw_der).expect("PKCS#7 DER parse failed");
    assert_eq!(parsed_pkcs7.certificates.len(), 1);
    assert_eq!(parsed_pkcs7.signer_infos.len(), 1);

    // Verify against unmodified document
    let trusted_roots: Vec<X509Certificate> = Vec::new();
    let result = parsed_pkcs7
        .verify_detached(doc_bytes, &trusted_roots, None)
        .expect("PKCS#7 verification failed");

    assert!(result.is_valid);
    assert!(result.digest_matches);
    assert!(result.has_timestamp);
    assert_eq!(result.signing_time, Some(1750000000));
    assert_eq!(
        result.signer_certificate.subject.common_name.as_deref(),
        Some("Bob Partner")
    );

    // Verify against modified document (tampering detected)
    let tampered_bytes = b"Contract Agreement between Parties A and HACKER";
    let tamper_res = parsed_pkcs7.verify_detached(tampered_bytes, &trusted_roots, None);
    assert!(tamper_res.is_err());
}

#[test]
fn test_08_pades_pdf_byte_range_and_signature_verification() {
    let base_pdf = b"%PDF-1.7\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R >>\nendobj\n";

    let signer_cert = X509Certificate::create_mock(
        "Counsel Jane Doe",
        "Legal Trust Root",
        999,
        1700000000,
        1800000000,
    );

    let signed_pdf = PadesSigner::sign_simple_pdf(base_pdf, &signer_cert, 1750000000, None)
        .expect("PDF signing failed");

    assert!(signed_pdf.len() > base_pdf.len());

    // Validate the signed PDF
    let trusted_roots: Vec<X509Certificate> = Vec::new();
    let report =
        PadesValidator::validate(&signed_pdf, &trusted_roots).expect("PAdES validation failed");

    assert!(report.is_valid);
    assert_eq!(report.signer_common_name, "Counsel Jane Doe");
    assert_eq!(report.signer_organization, "AetherOffice Trust Services");
    assert_eq!(report.signing_time, Some(1750000000));
    assert_eq!(report.byte_range[0], 0);
    assert!(report.byte_range[1] > 0);
    assert!(report.byte_range[2] > report.byte_range[1]);
}

#[test]
fn test_09_pades_tamper_detection_on_pdf_bytes() {
    let base_pdf = b"%PDF-1.7\n1 0 obj\n<< /Title (Confidential Acquisition) >>\nendobj\n";
    let signer_cert = X509Certificate::create_mock(
        "CEO John Smith",
        "Enterprise Root",
        888,
        1700000000,
        1800000000,
    );

    let mut signed_pdf = PadesSigner::sign_simple_pdf(base_pdf, &signer_cert, 1750000000, None)
        .expect("PDF signing failed");

    // Tamper with a single byte in the base PDF content (e.g. change 'C' in Confidential to 'X')
    let pos_opt = signed_pdf.windows(12).position(|w| w == b"Confidential");
    assert!(pos_opt.is_some(), "Target substring must exist");
    let pos = pos_opt.unwrap_or(0);
    signed_pdf[pos] = b'X';

    let trusted_roots: Vec<X509Certificate> = Vec::new();
    let report_res = PadesValidator::validate(&signed_pdf, &trusted_roots);
    assert!(
        report_res.is_err(),
        "PAdES validator must detect tampered document bytes"
    );
}

#[test]
fn test_10_ooxml_xmldsig_manifest_and_tamper_detection() {
    let doc_xml = b"<w:document><w:body><w:p><w:r><w:t>Confidential Merger Terms</w:t></w:r></w:p></w:body></w:document>";
    let content_types =
        b"<Types><Default Extension=\"xml\" ContentType=\"application/xml\"/></Types>";
    let rels_xml =
        b"<Relationships><Relationship Id=\"rId1\" Target=\"word/document.xml\"/></Relationships>";

    let signer_cert = X509Certificate::create_mock(
        "Auditor David Brown",
        "AetherOffice Root CA",
        777,
        1700000000,
        1800000000,
    );

    let references: &[(&str, &[u8])] = &[
        ("word/document.xml", doc_xml),
        ("[Content_Types].xml", content_types),
        ("_rels/.rels", rels_xml),
    ];

    let sig_xml = XmlDsigBuilder::build(references, &signer_cert).expect("XMLDSIG build failed");
    assert!(sig_xml.contains("<Signature"));
    assert!(sig_xml.contains("word/document.xml"));

    let parsed_sig = XmlDsigValidator::parse(&sig_xml).expect("XMLDSIG parse failed");
    assert_eq!(parsed_sig.references.len(), 3);
    assert!(parsed_sig.certificate.is_some());

    // Valid package verification
    let report = XmlDsigValidator::verify_parts(&parsed_sig, |part_path| match part_path {
        "word/document.xml" => Some(doc_xml.to_vec()),
        "[Content_Types].xml" => Some(content_types.to_vec()),
        "_rels/.rels" => Some(rels_xml.to_vec()),
        _ => None,
    })
    .expect("XMLDSIG verification failed");

    assert!(report.is_valid);
    assert_eq!(report.total_references, 3);
    assert_eq!(report.verified_parts.len(), 3);
    assert!(report.failed_parts.is_empty());
    assert_eq!(
        report.signer_common_name.as_deref(),
        Some("Auditor David Brown")
    );

    // Tampered package part verification
    let tampered_doc_xml = b"<w:document><w:body><w:p><w:r><w:t>Altered Merger Terms</w:t></w:r></w:p></w:body></w:document>";
    let tampered_report =
        XmlDsigValidator::verify_parts(&parsed_sig, |part_path| match part_path {
            "word/document.xml" => Some(tampered_doc_xml.to_vec()),
            "[Content_Types].xml" => Some(content_types.to_vec()),
            "_rels/.rels" => Some(rels_xml.to_vec()),
            _ => None,
        })
        .expect("XMLDSIG verification failed");

    assert!(!tampered_report.is_valid);
    assert_eq!(tampered_report.verified_parts.len(), 2);
    assert_eq!(tampered_report.failed_parts.len(), 1);
    assert!(tampered_report.failed_parts[0].contains("word/document.xml"));
}
