//! Sub-Phase 53.2.2 Integration Test Suite:
//! Universal 5-Platform Installer Matrix & Software Bill of Materials (SBOM) Publication.
//!
//! Complies with GEC-v7.0 Article I, III, IV & V:
//! - Complete 5-platform installer coverage (Windows, Linux, macOS, Android, iOS)
//! - Cryptographic SHA-256 checksum verification & tampering resistance
//! - CycloneDX v1.5 JSON & SPDX 2.3 JSON / Tag-Value generation
//! - Clean-Room provenance & GPL contamination detection

use render_core::{
    InstallerMatrixRegistry, LicenseAuditReport, PackagingBundle, SbomComponent, SbomComponentType,
    SbomGenerator, TargetArch, TargetOs,
};

#[test]
fn test_01_canonical_installer_matrix_coverage_5_platforms() {
    let registry = InstallerMatrixRegistry::canonical_registry("AetherOffice", "0.1.0");
    assert_eq!(registry.app_name, "AetherOffice");
    assert_eq!(registry.version, "0.1.0");

    let coverage_res = registry.verify_coverage();
    assert!(coverage_res.is_ok());
    assert!(coverage_res.unwrap());

    // Verify all 5 operating systems are covered
    let win_artifacts = registry.artifacts_for_platform(TargetOs::Windows);
    assert!(!win_artifacts.is_empty());
    assert!(win_artifacts.len() >= 3); // MSI, NSIS, Portable EXE

    let linux_artifacts = registry.artifacts_for_platform(TargetOs::Linux);
    assert!(!linux_artifacts.is_empty());
    assert!(linux_artifacts.len() >= 3); // AppImage, DEB, RPM

    let macos_artifacts = registry.artifacts_for_platform(TargetOs::MacOS);
    assert!(!macos_artifacts.is_empty());
    assert!(macos_artifacts
        .iter()
        .any(|a| a.bundle_format == PackagingBundle::Dmg));

    let android_artifacts = registry.artifacts_for_platform(TargetOs::Android);
    assert!(!android_artifacts.is_empty());
    assert!(android_artifacts
        .iter()
        .any(|a| a.bundle_format == PackagingBundle::Apk));
    assert!(android_artifacts
        .iter()
        .any(|a| a.bundle_format == PackagingBundle::Aab));

    let ios_artifacts = registry.artifacts_for_platform(TargetOs::Ios);
    assert!(!ios_artifacts.is_empty());
    assert!(ios_artifacts
        .iter()
        .any(|a| a.bundle_format == PackagingBundle::Ipa));
}

#[test]
fn test_02_installer_artifact_metadata_and_naming() {
    let registry = InstallerMatrixRegistry::canonical_registry("AetherOffice", "0.1.0");

    let msi = registry
        .find_artifact("win-x64-msi")
        .expect("Windows MSI must exist");
    assert_eq!(msi.platform, TargetOs::Windows);
    assert_eq!(msi.arch, TargetArch::X86_64);
    assert_eq!(msi.bundle_format, PackagingBundle::Msi);
    assert!(msi.file_name.ends_with(".msi"));
    assert!(msi.is_signed);
    assert!(msi.cert_fingerprint.is_some());
    assert!(msi.file_size_bytes > 0);
    assert!(msi.download_url.starts_with("https://"));

    let appimage = registry
        .find_artifact("linux-x64-appimage")
        .expect("AppImage must exist");
    assert_eq!(appimage.platform, TargetOs::Linux);
    assert_eq!(appimage.bundle_format, PackagingBundle::AppImage);
    assert!(appimage.file_name.ends_with(".AppImage"));

    let deb = registry
        .find_artifact("linux-x64-deb")
        .expect("DEB must exist");
    assert_eq!(deb.bundle_format, PackagingBundle::Deb);
    assert!(deb.file_name.ends_with(".deb"));

    let rpm = registry
        .find_artifact("linux-x64-rpm")
        .expect("RPM must exist");
    assert_eq!(rpm.bundle_format, PackagingBundle::Rpm);
    assert!(rpm.file_name.ends_with(".rpm"));

    let dmg = registry
        .find_artifact("macos-universal-dmg")
        .expect("macOS DMG must exist");
    assert_eq!(dmg.arch, TargetArch::Universal);
    assert_eq!(dmg.bundle_format, PackagingBundle::Dmg);
    assert!(dmg.file_name.ends_with(".dmg"));
}

#[test]
fn test_03_installer_sha256_checksum_verification() {
    // Empty bytes payload: SHA-256 is e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    let empty_payload = b"";
    let registry = InstallerMatrixRegistry::canonical_registry("AetherOffice", "0.1.0");

    let verify_res = registry.verify_artifact_payload("win-x64-msi", empty_payload);
    assert!(verify_res.is_ok());
    assert!(
        verify_res.unwrap(),
        "Empty payload should match canonical win-x64-msi reference SHA256"
    );

    let verify_hex = registry.verify_artifact_checksum(
        "win-x64-msi",
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );
    assert!(verify_hex.is_ok());
    assert!(verify_hex.unwrap());
}

#[test]
fn test_04_installer_checksum_mismatch_detection() {
    let registry = InstallerMatrixRegistry::canonical_registry("AetherOffice", "0.1.0");

    // Altered payload must fail verification
    let altered_payload = b"corrupted bytes";
    let verify_res = registry.verify_artifact_payload("win-x64-msi", altered_payload);
    assert!(verify_res.is_ok());
    assert!(
        !verify_res.unwrap(),
        "Corrupted payload must not match SHA-256"
    );

    // Invalid hex string must fail verification
    let verify_hex = registry.verify_artifact_checksum(
        "win-x64-msi",
        "0000000000000000000000000000000000000000000000000000000000000000",
    );
    assert!(verify_hex.is_ok());
    assert!(
        !verify_hex.unwrap(),
        "Mismatched hex checksum must return false"
    );

    // Unknown artifact ID returns Err
    let err_res = registry.verify_artifact_payload("non-existent-id", b"");
    assert!(err_res.is_err());
}

#[test]
fn test_05_sbom_generator_canonical_components_12_crates() {
    let sbom = SbomGenerator::canonical_sbom("AetherOffice", "0.1.0");
    assert_eq!(sbom.app_name, "AetherOffice");
    assert_eq!(sbom.version, "0.1.0");
    assert!(sbom.components.len() >= 18); // 12 domain crates + 6 third-party crates

    let crate_names: Vec<&str> = sbom.components.iter().map(|c| c.name.as_str()).collect();

    // Verify all 12 AetherOffice domain crates are explicitly declared
    assert!(crate_names.contains(&"ooxml-opc"));
    assert!(crate_names.contains(&"word-engine"));
    assert!(crate_names.contains(&"excel-engine"));
    assert!(crate_names.contains(&"ppt-engine"));
    assert!(crate_names.contains(&"pdf-engine"));
    assert!(crate_names.contains(&"crypto-security"));
    assert!(crate_names.contains(&"render-core"));
    assert!(crate_names.contains(&"crdt-collab"));
    assert!(crate_names.contains(&"macro-engine"));
    assert!(crate_names.contains(&"plugin-runtime"));
    assert!(crate_names.contains(&"data-connectors"));
    assert!(crate_names.contains(&"copilot-core"));

    for component in &sbom.components {
        assert!(!component.purl.is_empty());
        assert!(!component.sha256_hash.is_empty());
        assert_eq!(component.sha256_hash.len(), 64);
        assert!(!component.license_spdx.is_empty());
        assert!(!component.description.is_empty());
    }
}

#[test]
fn test_06_cyclonedx_1_5_json_schema_and_serialization() {
    let sbom = SbomGenerator::canonical_sbom("AetherOffice", "0.1.0");
    let json_str = sbom
        .generate_cyclonedx_1_5_json()
        .expect("CycloneDX JSON generation must succeed");

    assert!(json_str.contains("\"bomFormat\": \"CycloneDX\""));
    assert!(json_str.contains("\"specVersion\": \"1.5\""));
    assert!(json_str.contains("\"serialNumber\": \"urn:uuid:"));
    assert!(json_str.contains("\"components\": ["));
    assert!(json_str.contains("\"dependencies\": ["));

    // Parse back to generic JSON to verify schema validity
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Valid JSON");
    assert_eq!(parsed["bomFormat"], "CycloneDX");
    assert_eq!(parsed["specVersion"], "1.5");
    assert_eq!(parsed["metadata"]["component"]["name"], "AetherOffice");

    let comps = parsed["components"].as_array().expect("components array");
    assert!(comps.len() >= 18);
    let ooxml_comp = comps
        .iter()
        .find(|c| c["name"] == "ooxml-opc")
        .expect("ooxml-opc component");
    assert_eq!(ooxml_comp["purl"], "pkg:cargo/ooxml-opc@0.1.0");
    assert_eq!(ooxml_comp["hashes"][0]["alg"], "SHA-256");
}

#[test]
fn test_07_spdx_2_3_json_schema_and_serialization() {
    let sbom = SbomGenerator::canonical_sbom("AetherOffice", "0.1.0");
    let json_str = sbom
        .generate_spdx_2_3_json()
        .expect("SPDX 2.3 JSON generation must succeed");

    assert!(json_str.contains("\"spdxVersion\": \"SPDX-2.3\""));
    assert!(json_str.contains("\"dataLicense\": \"CC0-1.0\""));
    assert!(json_str.contains("\"SPDXID\": \"SPDXRef-DOCUMENT\""));
    assert!(json_str.contains("\"packages\": ["));

    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Valid JSON");
    assert_eq!(parsed["spdxVersion"], "SPDX-2.3");
    assert_eq!(parsed["dataLicense"], "CC0-1.0");

    let packages = parsed["packages"].as_array().expect("packages array");
    assert!(packages.len() >= 18);
    let render_pkg = packages
        .iter()
        .find(|p| p["name"] == "render-core")
        .expect("render-core package");
    assert_eq!(render_pkg["licenseConcluded"], "Apache-2.0 OR MIT");
    assert_eq!(render_pkg["checksums"][0]["algorithm"], "SHA256");
}

#[test]
fn test_08_spdx_2_3_tag_value_serialization() {
    let sbom = SbomGenerator::canonical_sbom("AetherOffice", "0.1.0");
    let tag_value = sbom
        .generate_spdx_2_3_tag_value()
        .expect("SPDX Tag-Value generation must succeed");

    assert!(tag_value.starts_with("SPDXVersion: SPDX-2.3\n"));
    assert!(tag_value.contains("DataLicense: CC0-1.0\n"));
    assert!(tag_value.contains("DocumentName: AetherOffice-v0.1.0\n"));
    assert!(tag_value.contains("PackageName: word-engine\n"));
    assert!(tag_value.contains("PackageChecksum: SHA256: "));
    assert!(tag_value.contains("PackageLicenseConcluded: Apache-2.0 OR MIT\n"));
}

#[test]
fn test_09_license_compliance_audit_clean_room() {
    let sbom = SbomGenerator::canonical_sbom("AetherOffice", "0.1.0");
    let audit: LicenseAuditReport = sbom.audit_license_compliance();

    assert!(
        audit.is_clean_room_compliant,
        "Canonical SBOM must be 100% clean-room compliant"
    );
    assert!(
        !audit.has_gpl_contamination,
        "Must not have any GPL contamination"
    );
    assert_eq!(audit.non_compliant_count, 0);
    assert_eq!(audit.unlicensed_count, 0);
    assert!(audit.violation_details.is_empty());
    assert_eq!(audit.compliant_count, audit.total_components);

    // Verify found licenses are permissive
    assert!(audit
        .licenses_found
        .contains(&"Apache-2.0 OR MIT".to_string()));
    assert!(audit.licenses_found.contains(&"MIT".to_string()));
}

#[test]
fn test_10_license_compliance_detects_gpl_contamination() {
    let mut sbom = SbomGenerator::canonical_sbom("AetherOffice", "0.1.0");

    // Inject a rogue GPL copyleft component
    sbom.components.push(SbomComponent::new(
        "rogue-gpl-tool",
        "1.0.0",
        "pkg:cargo/rogue-gpl-tool@1.0.0",
        "GPL-3.0-only",
        "A copyleft component violating clean-room provenance",
        "0000111122223333444455556666777788889999aaaabbbbccccddddeeeeffff",
        SbomComponentType::Library,
        vec![],
    ));

    let audit = sbom.audit_license_compliance();
    assert!(
        !audit.is_clean_room_compliant,
        "Must detect GPL contamination"
    );
    assert!(
        audit.has_gpl_contamination,
        "has_gpl_contamination flag must be true"
    );
    assert_eq!(audit.non_compliant_count, 1);
    assert!(!audit.violation_details.is_empty());
    assert!(audit.violation_details[0].contains("contains copyleft/GPL license: 'GPL-3.0-only'"));
}
