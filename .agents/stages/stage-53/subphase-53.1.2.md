# Sub-Phase 53.1.2: Open-Core & Commercial Tier Entitlement Configuration

## Overview & Scope
Sub-Phase 53.1.2 establishes the Open-Core and Commercial Tier Entitlement Engine in `render-core` and the unified presentation shell. It allows clear segregation and offline cryptographic verification of features across Community (Open-Core), Professional, and Enterprise tiers without compromising file format integrity, offline privacy, or GEC-v7.0 invariants.

## Architectural Deliverables

### 1. Entitlement Engine (`crates/render-core/src/entitlement.rs`)
- `EditionTier`: `Community`, `Professional`, `Enterprise`.
- `FeatureEntitlement`:
  - `BasicEditing`, `AdvancedTypography`, `StandardFormulas`, `FullDocumentIo` (Community+).
  - `LocalAiCopilot`, `MacroDebugger`, `BatchConversion` (Professional+).
  - `EnterpriseDlp`, `SanitizationCertificates`, `TamperEvidentAuditLogs`, `RealtimeCollabWebRtc`, `CustomWasmPlugins` (Enterprise).
- `LicenseToken`:
  - `license_id`, `licensee_name`, `organization`, `tier`, `issued_at_ms`, `expires_at_ms`, `max_seats`, `signature`.
  - Canonical serialization for signature verification.
- `EntitlementStatus`:
  - `Granted`
  - `Denied { required_tier: EditionTier, reason: String }`
  - `GracePeriod { days_remaining: u32, message: String }`
- `EntitlementEngine`:
  - Offline cryptographic token validation (HMAC-SHA256 signature).
  - Expiration and 14-day grace period computation.
  - Query method `check_feature(&self, feature: FeatureEntitlement, now_ms: u64) -> EntitlementStatus`.
  - Tier switching for development/testing or enterprise seat distribution.

### 2. Tauri Desktop IPC (`apps/unified-shell/src-tauri`)
- Extended `ShellState` with `entitlement_engine: Mutex<EntitlementEngine>`.
- IPC command endpoints:
  - `entitlement_get_tier(state: &ShellState) -> Result<EditionTier, String>`
  - `entitlement_check_feature(state: &ShellState, feature_str: String, now_ms: Option<u64>) -> Result<EntitlementStatusDto, String>`
  - `entitlement_apply_license(state: &ShellState, token: LicenseToken, verification_key: Vec<u8>) -> Result<EntitlementStatusDto, String>`
  - `entitlement_get_license_info(state: &ShellState) -> Result<Option<LicenseToken>, String>`
  - `entitlement_set_tier(state: &ShellState, tier: EditionTier) -> Result<(), String>`

### 3. Presentation Layer Coordinator (`apps/unified-shell/src/state/EntitlementCoordinator.ts`)
- TypeScript reactive coordinator managing active tier state, license application, feature gates, and commercial upgrade modals.
- IPC integration in `ipc.ts` with browser mock fallbacks.

## Invariants & Contract Adherence
- `#![forbid(unsafe_code)]` strictly enforced.
- Zero `.unwrap()`, `.expect()`, or `panic!()` in non-test library code.
- Clean-room Rust implementation, 100% offline verification with zero external telemetric tracking.
- Zero compiler or clippy warnings under `cargo clippy --workspace --all-targets -- -D warnings`.
- Master gatekeeper `tools/verify-contract.ps1` exit code 0.

