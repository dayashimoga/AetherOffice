//! Sub-Phase 53.1.1 Integration Tests:
//! Enterprise DLP Policy Engine, PII Sanitization Certificates & Tamper-Evident Audit Logs

use crypto_security::{
    AuditEventType, AuditLog, DlpAction, DlpEvaluator, DlpPolicy, DlpSeverity,
    SanitizationCertificate, SanitizationCertificateGenerator, SanitizationCertificateValidator,
    GENESIS_PREV_HASH,
};

#[test]
fn test_01_dlp_severity_and_action_ordering() {
    assert!(DlpSeverity::Low < DlpSeverity::Medium);
    assert!(DlpSeverity::Medium < DlpSeverity::High);
    assert!(DlpSeverity::High < DlpSeverity::Critical);

    assert!(DlpAction::Block.is_blocking());
    assert!(DlpAction::Quarantine.is_blocking());
    assert!(!DlpAction::Allow.is_blocking());
    assert!(!DlpAction::Warn.is_blocking());
    assert!(!DlpAction::RedactAndAllow.is_blocking());
    assert!(DlpAction::RedactAndAllow.requires_redaction());
}

#[test]
fn test_02_dlp_default_enterprise_policy_ssn_and_credit_card() {
    let policy = DlpPolicy::default_enterprise_policy();

    // 1. Text with valid credit card triggers Critical / Block
    let cc_text = "Billing card number: 4532 0150 0000 0007 for transaction.";
    let cc_res = DlpEvaluator::evaluate_text(&policy, cc_text);
    assert!(cc_res.is_violation);
    assert_eq!(cc_res.action, DlpAction::Block);
    assert_eq!(cc_res.highest_severity, Some(DlpSeverity::Critical));
    assert!(cc_res
        .triggered_rule_ids
        .contains(&"rule-pci-dss".to_string()));

    // 2. Text with SSN triggers High / RedactAndAllow
    let ssn_text = "Employee tax identifier: 123-45-6789.";
    let ssn_res = DlpEvaluator::evaluate_text(&policy, ssn_text);
    assert!(ssn_res.is_violation);
    assert_eq!(ssn_res.action, DlpAction::RedactAndAllow);
    assert_eq!(ssn_res.highest_severity, Some(DlpSeverity::High));
    assert!(ssn_res
        .triggered_rule_ids
        .contains(&"rule-ssn-redact".to_string()));

    // 3. Clean text triggers no violation
    let clean_text = "Quarterly financial summary looks exceptionally positive across all regions.";
    let clean_res = DlpEvaluator::evaluate_text(&policy, clean_text);
    assert!(!clean_res.is_violation);
    assert_eq!(clean_res.action, DlpAction::Allow);
    assert_eq!(clean_res.highest_severity, None);
}

#[test]
fn test_03_dlp_evaluate_and_redact_in_place() {
    let policy = DlpPolicy::default_enterprise_policy();
    let text = "Confidential memo: SSN 987-65-4320 must not be disclosed.";
    let (res, redacted) = DlpEvaluator::evaluate_and_redact(&policy, text);

    assert!(res.is_violation);
    assert_eq!(res.action, DlpAction::RedactAndAllow);
    assert!(!redacted.contains("987-65-4320"));
    assert!(redacted.contains("[REDACTED: SSN]"));
}

#[test]
fn test_04_dlp_strict_hipaa_policy_thresholds() {
    let policy = DlpPolicy::strict_hipaa_policy();

    // 1 contact (threshold is 2) does not trigger contact rule
    let single_contact = "Patient email is alice@hospital.org.";
    let res1 = DlpEvaluator::evaluate_text(&policy, single_contact);
    assert!(!res1
        .triggered_rule_ids
        .contains(&"hipaa-contact".to_string()));

    // 2 contacts trigger contact rule (RedactAndAllow)
    let two_contacts = "Patient email alice@hospital.org and mobile (555) 234-5678.";
    let res2 = DlpEvaluator::evaluate_text(&policy, two_contacts);
    assert!(res2
        .triggered_rule_ids
        .contains(&"hipaa-contact".to_string()));
    assert_eq!(res2.action, DlpAction::RedactAndAllow);

    // Patient SSN triggers Critical / Block immediately
    let ssn_patient = "Patient SSN is 111-22-3333.";
    let res3 = DlpEvaluator::evaluate_text(&policy, ssn_patient);
    assert!(res3.triggered_rule_ids.contains(&"hipaa-ssn".to_string()));
    assert_eq!(res3.action, DlpAction::Block);
}

#[test]
fn test_05_sanitization_certificate_generation_and_canonical_payload() {
    let original = b"Customer data with SSN 123-45-6789 and email john@acme.com.";
    let sanitized = b"Customer data with SSN [REDACTED: SSN] and email [REDACTED: EMAIL].";
    let key = b"enterprise-hmac-secret-key-2026";

    let cert = SanitizationCertificateGenerator::generate(
        "doc-fin-001",
        original,
        sanitized,
        "ent-dlp-std",
        1700000000000,
        vec![("SSN".to_string(), 1), ("EMAIL".to_string(), 1)],
        key,
    );

    assert_eq!(cert.doc_id, "doc-fin-001");
    assert_eq!(cert.policy_id, "ent-dlp-std");
    assert_eq!(cert.total_redactions(), 2);
    assert!(!cert.signature.is_empty());
    assert!(!cert.original_digest.is_empty());
    assert!(!cert.sanitized_digest.is_empty());
    assert_ne!(cert.original_digest, cert.sanitized_digest);

    let payload = cert.canonical_payload();
    assert!(payload.starts_with("AETHER-SAN-CERT:v1|doc=doc-fin-001"));
}

#[test]
fn test_06_sanitization_certificate_signature_verification() {
    let original = b"Medical record with PHI.";
    let sanitized = b"Medical record with [REDACTED: PHI].";
    let key = b"correct-audit-signing-key";
    let wrong_key = b"wrong-audit-signing-key";

    let cert = SanitizationCertificateGenerator::generate(
        "doc-med-99",
        original,
        sanitized,
        "hipaa-phi-strict",
        1700000000500,
        vec![("PHI".to_string(), 1)],
        key,
    );

    // 1. Valid signature passes
    assert!(SanitizationCertificateValidator::verify_signature(&cert, key).is_ok());

    // 2. Wrong key fails
    assert!(SanitizationCertificateValidator::verify_signature(&cert, wrong_key).is_err());

    // 3. Tampered certificate fields fail
    let mut tampered_cert = cert.clone();
    tampered_cert.doc_id = "doc-med-99-tampered".to_string();
    assert!(SanitizationCertificateValidator::verify_signature(&tampered_cert, key).is_err());
}

#[test]
fn test_07_sanitization_certificate_document_integrity_verification() {
    let original = b"Employee database export.";
    let sanitized = b"Employee database [REDACTED].";
    let key = b"integrity-key-xyz";

    let cert = SanitizationCertificateGenerator::generate(
        "doc-emp-10",
        original,
        sanitized,
        "ent-dlp-std",
        1700000001000,
        vec![("SSN".to_string(), 10)],
        key,
    );

    // 1. Matches post-sanitization document bytes
    assert!(
        SanitizationCertificateValidator::verify_document_integrity(&cert, sanitized, key).is_ok()
    );

    // 2. Mismatched document bytes rejected
    let modified_doc = b"Employee database [REDACTED] with extra leaked byte.";
    let err = SanitizationCertificateValidator::verify_document_integrity(&cert, modified_doc, key);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("Document digest mismatch"));
}

#[test]
fn test_08_tamper_evident_audit_log_genesis_and_chaining() {
    let mut log = AuditLog::new();
    assert_eq!(log.len(), 0);
    assert!(log.is_empty());

    // 1. Entry 1 links from GENESIS_PREV_HASH
    let e1 = log.append(
        1000,
        AuditEventType::DocumentOpened,
        "usr-cfo",
        "doc-q3-budget",
        "Opened budget spreadsheet",
    );
    assert_eq!(e1.sequence_number, 1);
    assert_eq!(e1.prev_hash, GENESIS_PREV_HASH);
    let hash1 = e1.entry_hash.clone();

    // 2. Entry 2 links from Entry 1
    let e2 = log.append(
        1050,
        AuditEventType::DlpViolationDetected,
        "usr-cfo",
        "doc-q3-budget",
        "Detected 1 unredacted credit card",
    );
    assert_eq!(e2.sequence_number, 2);
    assert_eq!(e2.prev_hash, hash1);
    let hash2 = e2.entry_hash.clone();

    // 3. Entry 3 links from Entry 2
    let e3 = log.append(
        1100,
        AuditEventType::DocumentSanitized,
        "system-dlp",
        "doc-q3-budget",
        "Sanitized credit card and applied certificate",
    );
    assert_eq!(e3.sequence_number, 3);
    assert_eq!(e3.prev_hash, hash2);

    assert_eq!(log.len(), 3);
    assert_eq!(log.verify_integrity().unwrap(), 3);
}

#[test]
fn test_09_tamper_evident_audit_log_detects_mutation_and_reordering() {
    let mut log = AuditLog::new();
    log.append(
        1000,
        AuditEventType::DocumentOpened,
        "alice",
        "doc-1",
        "open",
    );
    log.append(
        1010,
        AuditEventType::DocumentSaved,
        "alice",
        "doc-1",
        "save v1",
    );
    log.append(
        1020,
        AuditEventType::ExportExecuted,
        "alice",
        "doc-1",
        "export pdf",
    );

    assert_eq!(log.verify_integrity().unwrap(), 3);

    // 1. Mutating details in entry 2 breaks integrity
    let entries_mut = {
        let json = serde_json::to_string(&log).unwrap();
        let mut raw_val: serde_json::Value = serde_json::from_str(&json).unwrap();
        raw_val["entries"][1]["details"] = serde_json::Value::String("tampered save".to_string());
        serde_json::from_value::<AuditLog>(raw_val).unwrap()
    };
    let err = entries_mut.verify_integrity();
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("tamper detected at sequence 2"));

    // 2. Truncating / removing entry 1 breaks sequence & chain link
    let json = serde_json::to_string(&log).unwrap();
    let mut raw_val: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = raw_val["entries"].as_array_mut().unwrap();
    arr.remove(0); // Remove first entry
    let broken_log: AuditLog = serde_json::from_value(raw_val).unwrap();
    let err_seq = broken_log.verify_integrity();
    assert!(err_seq.is_err());
    assert!(err_seq.unwrap_err().contains("sequence break at index 0"));
}

#[test]
fn test_10_audit_log_and_dlp_serde_roundtrip() {
    let policy = DlpPolicy::default_enterprise_policy();
    let json_policy = serde_json::to_string_pretty(&policy).unwrap();
    let deserialized_policy: DlpPolicy = serde_json::from_str(&json_policy).unwrap();
    assert_eq!(policy, deserialized_policy);

    let cert = SanitizationCertificateGenerator::generate(
        "doc-json-test",
        b"raw-content",
        b"sanitized-content",
        "ent-dlp-std",
        123456789,
        vec![("SSN".to_string(), 1)],
        b"key-secret",
    );
    let json_cert = serde_json::to_string_pretty(&cert).unwrap();
    let deserialized_cert: SanitizationCertificate = serde_json::from_str(&json_cert).unwrap();
    assert_eq!(cert, deserialized_cert);

    let mut log = AuditLog::new();
    log.append(1000, AuditEventType::DocumentOpened, "usr", "doc", "init");
    let json_log = serde_json::to_string_pretty(&log).unwrap();
    let deserialized_log: AuditLog = serde_json::from_str(&json_log).unwrap();
    assert_eq!(log, deserialized_log);
    assert_eq!(deserialized_log.verify_integrity().unwrap(), 1);
}
