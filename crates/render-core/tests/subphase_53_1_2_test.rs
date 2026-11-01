use render_core::entitlement::{
    EditionTier, EntitlementEngine, EntitlementStatus, FeatureEntitlement, LicenseGenerator,
    LicenseToken, DEFAULT_GRACE_PERIOD_MS,
};

const SECRET_KEY: &[u8] = b"aetheroffice-entitlement-master-test-key-2026";
const ONE_DAY_MS: u64 = 24 * 60 * 60 * 1000;

#[test]
fn test_01_default_community_tier_and_basic_features() {
    let engine = EntitlementEngine::new();
    let now_ms = 1_700_000_000_000;

    assert_eq!(engine.current_tier(), EditionTier::Community);
    assert_eq!(engine.effective_tier(now_ms), EditionTier::Community);
    assert!(engine.active_license().is_none());

    // Open-core basic features must be granted
    assert_eq!(
        engine.check_feature(FeatureEntitlement::BasicEditing, now_ms),
        EntitlementStatus::Granted
    );
    assert_eq!(
        engine.check_feature(FeatureEntitlement::AdvancedTypography, now_ms),
        EntitlementStatus::Granted
    );
    assert_eq!(
        engine.check_feature(FeatureEntitlement::StandardFormulas, now_ms),
        EntitlementStatus::Granted
    );
    assert_eq!(
        engine.check_feature(FeatureEntitlement::FullDocumentIo, now_ms),
        EntitlementStatus::Granted
    );

    // Commercial features must be denied
    let copilot_status = engine.check_feature(FeatureEntitlement::LocalAiCopilot, now_ms);
    assert!(copilot_status.is_denied());
    if let EntitlementStatus::Denied { required_tier, .. } = copilot_status {
        assert_eq!(required_tier, EditionTier::Professional);
    }

    let dlp_status = engine.check_feature(FeatureEntitlement::EnterpriseDlp, now_ms);
    assert!(dlp_status.is_denied());
    if let EntitlementStatus::Denied { required_tier, .. } = dlp_status {
        assert_eq!(required_tier, EditionTier::Enterprise);
    }
}

#[test]
fn test_02_professional_tier_gating() {
    let engine = EntitlementEngine::with_tier(EditionTier::Professional);
    let now_ms = 1_700_000_000_000;

    assert_eq!(engine.effective_tier(now_ms), EditionTier::Professional);

    // Community features granted
    assert_eq!(
        engine.check_feature(FeatureEntitlement::BasicEditing, now_ms),
        EntitlementStatus::Granted
    );
    // Professional features granted
    assert_eq!(
        engine.check_feature(FeatureEntitlement::LocalAiCopilot, now_ms),
        EntitlementStatus::Granted
    );
    assert_eq!(
        engine.check_feature(FeatureEntitlement::MacroDebugger, now_ms),
        EntitlementStatus::Granted
    );
    assert_eq!(
        engine.check_feature(FeatureEntitlement::BatchConversion, now_ms),
        EntitlementStatus::Granted
    );

    // Enterprise features denied
    let cert_status = engine.check_feature(FeatureEntitlement::SanitizationCertificates, now_ms);
    assert!(cert_status.is_denied());
    let audit_status = engine.check_feature(FeatureEntitlement::TamperEvidentAuditLogs, now_ms);
    assert!(audit_status.is_denied());
    let collab_status = engine.check_feature(FeatureEntitlement::RealtimeCollabWebRtc, now_ms);
    assert!(collab_status.is_denied());
    let wasm_status = engine.check_feature(FeatureEntitlement::CustomWasmPlugins, now_ms);
    assert!(wasm_status.is_denied());
}

#[test]
fn test_03_enterprise_tier_full_grant() {
    let engine = EntitlementEngine::with_tier(EditionTier::Enterprise);
    let now_ms = 1_700_000_000_000;

    assert_eq!(engine.effective_tier(now_ms), EditionTier::Enterprise);

    let all_features = [
        FeatureEntitlement::BasicEditing,
        FeatureEntitlement::AdvancedTypography,
        FeatureEntitlement::StandardFormulas,
        FeatureEntitlement::FullDocumentIo,
        FeatureEntitlement::LocalAiCopilot,
        FeatureEntitlement::MacroDebugger,
        FeatureEntitlement::BatchConversion,
        FeatureEntitlement::EnterpriseDlp,
        FeatureEntitlement::SanitizationCertificates,
        FeatureEntitlement::TamperEvidentAuditLogs,
        FeatureEntitlement::RealtimeCollabWebRtc,
        FeatureEntitlement::CustomWasmPlugins,
    ];

    for feature in all_features {
        assert_eq!(
            engine.check_feature(feature, now_ms),
            EntitlementStatus::Granted,
            "Feature {:?} should be granted under Enterprise tier",
            feature
        );
    }
}

#[test]
fn test_04_perpetual_license_token_creation_and_activation() {
    let mut engine = EntitlementEngine::new();
    let now_ms = 1_700_000_000_000;

    let token = LicenseGenerator::create_signed_token(
        "LIC-ENT-2026-001",
        "Acme Global Holdings",
        Some("Acme Corp"),
        EditionTier::Enterprise,
        now_ms,
        None, // Perpetual
        500,
        vec![],
        SECRET_KEY,
    );

    assert!(token.expires_at_ms.is_none());
    assert!(!token.is_expired(now_ms));
    assert_eq!(token.days_remaining(now_ms), None);

    let result = engine.apply_license(token.clone(), SECRET_KEY, now_ms);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EntitlementStatus::Granted);

    assert_eq!(engine.effective_tier(now_ms), EditionTier::Enterprise);
    assert_eq!(
        engine.check_feature(FeatureEntitlement::EnterpriseDlp, now_ms),
        EntitlementStatus::Granted
    );
    assert_eq!(
        engine.check_feature(FeatureEntitlement::RealtimeCollabWebRtc, now_ms),
        EntitlementStatus::Granted
    );
}

#[test]
fn test_05_invalid_signature_rejection() {
    let mut engine = EntitlementEngine::new();
    let now_ms = 1_700_000_000_000;

    let wrong_key = b"wrong-secret-key-for-verification";
    let token = LicenseGenerator::create_signed_token(
        "LIC-PRO-2026-042",
        "Test Developer",
        None,
        EditionTier::Professional,
        now_ms,
        Some(now_ms + 365 * ONE_DAY_MS),
        1,
        vec![],
        wrong_key,
    );

    // Verification with SECRET_KEY must fail
    let result = engine.apply_license(token.clone(), SECRET_KEY, now_ms);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Invalid license signature: cryptographic verification failed"));

    // Verification with altered tier must fail even with correct key
    let mut tampered_token = LicenseGenerator::create_signed_token(
        "LIC-PRO-2026-042",
        "Test Developer",
        None,
        EditionTier::Professional,
        now_ms,
        Some(now_ms + 365 * ONE_DAY_MS),
        1,
        vec![],
        SECRET_KEY,
    );
    // Tamper tier from Professional to Enterprise
    tampered_token.tier = EditionTier::Enterprise;
    let tampered_result = engine.apply_license(tampered_token, SECRET_KEY, now_ms);
    assert!(tampered_result.is_err());
}

#[test]
fn test_06_expired_license_within_14_day_grace_period() {
    let mut engine = EntitlementEngine::new();
    let issued_ms = 1_700_000_000_000;
    let expires_ms = issued_ms + 30 * ONE_DAY_MS;
    // 5 days past expiration
    let current_ms = expires_ms + 5 * ONE_DAY_MS;

    let token = LicenseGenerator::create_signed_token(
        "LIC-ENT-2026-EXP",
        "Grace Period Test Org",
        Some("Grace Org"),
        EditionTier::Enterprise,
        issued_ms,
        Some(expires_ms),
        100,
        vec![],
        SECRET_KEY,
    );

    assert!(token.is_expired(current_ms));
    assert!(token.is_in_grace_period(current_ms, DEFAULT_GRACE_PERIOD_MS));
    assert_eq!(token.days_remaining(current_ms), Some(0));

    let result = engine.apply_license(token, SECRET_KEY, current_ms);
    assert!(result.is_ok());

    match result.unwrap() {
        EntitlementStatus::GracePeriod {
            days_remaining,
            message,
        } => {
            assert_eq!(days_remaining, 9); // 14 - 5 = 9 days
            assert!(message.contains("grace period"));
        }
        other => panic!("Expected GracePeriod status, got {:?}", other),
    }

    // Effective tier during grace period remains Enterprise
    assert_eq!(engine.effective_tier(current_ms), EditionTier::Enterprise);

    // Feature check also returns GracePeriod
    let feat_status = engine.check_feature(FeatureEntitlement::EnterpriseDlp, current_ms);
    assert!(feat_status.is_granted());
    match feat_status {
        EntitlementStatus::GracePeriod { days_remaining, .. } => {
            assert_eq!(days_remaining, 9);
        }
        other => panic!("Expected GracePeriod status, got {:?}", other),
    }
}

#[test]
fn test_07_expired_license_beyond_grace_period_rejection() {
    let mut engine = EntitlementEngine::new();
    let issued_ms = 1_700_000_000_000;
    let expires_ms = issued_ms + 30 * ONE_DAY_MS;
    // 20 days past expiration (exceeds 14-day grace period)
    let current_ms = expires_ms + 20 * ONE_DAY_MS;

    let token = LicenseGenerator::create_signed_token(
        "LIC-EXP-BEYOND",
        "Expired Org",
        None,
        EditionTier::Enterprise,
        issued_ms,
        Some(expires_ms),
        10,
        vec![],
        SECRET_KEY,
    );

    assert!(token.is_expired(current_ms));
    assert!(!token.is_in_grace_period(current_ms, DEFAULT_GRACE_PERIOD_MS));

    let result = engine.apply_license(token, SECRET_KEY, current_ms);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("License expired beyond allowable grace period"));

    // Engine remains Community tier
    assert_eq!(engine.effective_tier(current_ms), EditionTier::Community);
    assert!(engine
        .check_feature(FeatureEntitlement::EnterpriseDlp, current_ms)
        .is_denied());
}

#[test]
fn test_08_custom_feature_overrides() {
    let mut engine = EntitlementEngine::new();
    let now_ms = 1_700_000_000_000;

    // Community license with custom enterprise feature granted
    let token = LicenseGenerator::create_signed_token(
        "LIC-CUSTOM-FEAT",
        "Addon Subscriber",
        None,
        EditionTier::Community,
        now_ms,
        Some(now_ms + 60 * ONE_DAY_MS),
        1,
        vec![
            "local-ai-copilot".to_string(),
            "custom-wasm-plugins".to_string(),
        ],
        SECRET_KEY,
    );

    let res = engine.apply_license(token, SECRET_KEY, now_ms);
    assert!(res.is_ok());

    // Basic tier is Community
    assert_eq!(engine.effective_tier(now_ms), EditionTier::Community);

    // Overridden features should be granted
    assert_eq!(
        engine.check_feature(FeatureEntitlement::LocalAiCopilot, now_ms),
        EntitlementStatus::Granted
    );
    assert_eq!(
        engine.check_feature(FeatureEntitlement::CustomWasmPlugins, now_ms),
        EntitlementStatus::Granted
    );

    // Non-overridden Pro/Enterprise features must remain denied
    assert!(engine
        .check_feature(FeatureEntitlement::MacroDebugger, now_ms)
        .is_denied());
    assert!(engine
        .check_feature(FeatureEntitlement::EnterpriseDlp, now_ms)
        .is_denied());
}

#[test]
fn test_09_remove_license_reverts_to_community() {
    let mut engine = EntitlementEngine::new();
    let now_ms = 1_700_000_000_000;

    let token = LicenseGenerator::create_signed_token(
        "LIC-REMOVE-TEST",
        "Temporary Enterprise User",
        Some("Enterprise Co"),
        EditionTier::Enterprise,
        now_ms,
        None,
        50,
        vec![],
        SECRET_KEY,
    );

    let res = engine.apply_license(token, SECRET_KEY, now_ms);
    assert!(res.is_ok());
    assert_eq!(engine.effective_tier(now_ms), EditionTier::Enterprise);

    // Remove license
    engine.remove_license();
    assert_eq!(engine.current_tier(), EditionTier::Community);
    assert_eq!(engine.effective_tier(now_ms), EditionTier::Community);
    assert!(engine.active_license().is_none());

    // Commercial features are now denied
    assert!(engine
        .check_feature(FeatureEntitlement::EnterpriseDlp, now_ms)
        .is_denied());
    assert!(engine
        .check_feature(FeatureEntitlement::LocalAiCopilot, now_ms)
        .is_denied());
    // Open-core features remain granted
    assert_eq!(
        engine.check_feature(FeatureEntitlement::BasicEditing, now_ms),
        EntitlementStatus::Granted
    );
}

#[test]
fn test_10_serde_roundtrip_and_canonical_payload() {
    let now_ms = 1_700_000_000_000;
    let token = LicenseGenerator::create_signed_token(
        "LIC-SERDE-001",
        "Canonical Test Org",
        Some("Canonical Ltd"),
        EditionTier::Professional,
        now_ms,
        Some(now_ms + 180 * ONE_DAY_MS),
        25,
        vec!["macro-debugger".to_string()],
        SECRET_KEY,
    );

    let expected_prefix = "AETHER-LIC:v1|id=LIC-SERDE-001|user=Canonical Test Org|org=Canonical Ltd|tier=professional|issued=1700000000000|expires=";
    let canonical = token.canonical_payload();
    assert!(canonical.starts_with(expected_prefix));
    assert!(canonical.contains("|seats=25|custom=macro-debugger"));

    // Serde roundtrip for LicenseToken
    let json = serde_json::to_string(&token).expect("Serialization failed");
    let deserialized: LicenseToken = serde_json::from_str(&json).expect("Deserialization failed");
    assert_eq!(token, deserialized);

    // Serde roundtrip for EntitlementEngine
    let mut engine = EntitlementEngine::new();
    let _ = engine.apply_license(token, SECRET_KEY, now_ms);
    let engine_json = serde_json::to_string(&engine).expect("Engine serialization failed");
    let deserialized_engine: EntitlementEngine =
        serde_json::from_str(&engine_json).expect("Engine deserialization failed");
    assert_eq!(engine, deserialized_engine);
}
