# Sub-Phase 53.2.2: Universal 5-Platform Installer Matrix & SBOM Publication

## Objective
Establish the Universal 5-Platform Installer Matrix, Software Bill of Materials (SBOM) generation engine (CycloneDX v1.5 JSON and SPDX 2.3 JSON/Tag-Value), cryptographic artifact checksum verifier, and clean-room license compliance auditor in `render-core` and the unified presentation shell under GEC-v7.0.

## Scope & Deliverables
1. **Rust Core (`crates/render-core/src/release_matrix.rs` & `crates/render-core/src/sbom.rs`)**:
   - `PackagingBundle`: Full packaging formats across 5 platforms (`Msi`, `Nsis`, `AppImage`, `Deb`, `Rpm`, `Dmg`, `AppBundle`, `Apk`, `Aab`, `Ipa`, `StandaloneExe`, `StandaloneElf`).
   - `InstallerArtifact`: Metadata for distribution artifacts including OS, CPU architecture, bundle format, filename, size, SHA-256 digest, minimum OS requirement, code signing status, and certificate fingerprint.
   - `InstallerMatrixRegistry`: Canonical distribution matrix for all 5 tier-1 operating systems (Windows, Linux, macOS, Android, iOS), manifest exporter, and constant-time SHA-256 checksum verification.
   - `SbomComponent`: Component metadata including package URL (`purl`), version, declared/concluded license, cryptographic SHA-256 hash, and component type.
   - `SbomGenerator`:
     - Populates all 12 AetherOffice workspace domain crates (`ooxml-opc`, `word-engine`, `excel-engine`, `ppt-engine`, `pdf-engine`, `crypto-security`, `render-core`, `crdt-collab`, `macro-engine`, `plugin-runtime`, `data-connectors`, `copilot-core`) and core third-party dependencies.
     - `generate_cyclonedx_1_5_json`: CycloneDX v1.5 JSON schema compliant BOM.
     - `generate_spdx_2_3_json`: SPDX 2.3 JSON specification compliant document with CC0-1.0 data license and SHA-256 package checksums.
     - `generate_spdx_2_3_tag_value`: SPDX 2.3 standard Tag-Value text format.
     - `audit_license_compliance`: Rigorous provenance auditor ensuring 100% clean-room permissive licenses (MIT, Apache-2.0, BSD-3-Clause, CC0-1.0) and zero GPL/AGPL copyleft contamination.
2. **Rust Integration Suite (`crates/render-core/tests/subphase_53_2_2_test.rs`)**:
   - 10 rigorous tests covering 5-platform installer coverage, artifact filename conventions, SHA-256 checksum verification, tampering detection, CycloneDX 1.5 JSON generation, SPDX 2.3 JSON and Tag-Value generation, clean-room license compliance, and GPL contamination rejection.
3. **Tauri IPC Command Endpoints (`apps/unified-shell/src-tauri`)**:
   - `release_matrix_get_full`: Returns full 5-platform installer matrix and manifest.
   - `sbom_generate_cyclonedx`: Generates CycloneDX 1.5 JSON BOM.
   - `sbom_generate_spdx`: Generates SPDX 2.3 JSON or Tag-Value SBOM.
   - `sbom_audit_licenses`: Returns comprehensive license provenance and clean-room audit report.
   - `installer_verify_checksum`: Verifies SHA-256 digest of installer payload or artifact.
4. **Presentation Shell (`apps/unified-shell/src/state/ReleaseAndSbomCoordinator.ts`)**:
   - Reactive coordinator managing release matrix inspection, SBOM downloads, license compliance status badges, and installer artifact integrity checks.
5. **Vitest Test Suite (`apps/unified-shell/src/__tests__/release_and_sbom_coordinator.test.ts`)**:
   - 10 comprehensive tests validating 5-platform targets, SBOM formatting, license audit results, and reactive state transitions.
6. **Master Gatekeeper Pass**:
   - `tools/verify-contract.ps1` exit code 0.

