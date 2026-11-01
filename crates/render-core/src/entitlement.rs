//! Open-Core & Commercial Tier Entitlement Engine.
//!
//! Governs feature gating, cryptographic license token verification (HMAC-SHA256),
//! offline validity verification, and grace-period management across
//! Community (Open-Core), Professional, and Enterprise tiers.

use crypto_security::digest::HashAlgorithm;
use crypto_security::kdf::hmac;
use serde::{Deserialize, Serialize};

/// Standard 14-day grace period in milliseconds for expired commercial licenses.
pub const DEFAULT_GRACE_PERIOD_MS: u64 = 14 * 24 * 60 * 60 * 1000;

/// Commercial edition tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditionTier {
    Community,
    Professional,
    Enterprise,
}

impl EditionTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Community => "community",
            Self::Professional => "professional",
            Self::Enterprise => "enterprise",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "community" | "open-core" | "free" => Some(Self::Community),
            "professional" | "pro" => Some(Self::Professional),
            "enterprise" | "ent" | "commercial" => Some(Self::Enterprise),
            _ => None,
        }
    }
}

/// Managed product capability subject to tier entitlement gating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeatureEntitlement {
    // Community (Open-Core) Tier
    BasicEditing,
    AdvancedTypography,
    StandardFormulas,
    FullDocumentIo,

    // Professional Tier
    LocalAiCopilot,
    MacroDebugger,
    BatchConversion,

    // Enterprise Tier
    EnterpriseDlp,
    SanitizationCertificates,
    TamperEvidentAuditLogs,
    RealtimeCollabWebRtc,
    CustomWasmPlugins,
}

impl FeatureEntitlement {
    /// Minimum required license tier to access this capability.
    pub fn required_tier(&self) -> EditionTier {
        match self {
            Self::BasicEditing
            | Self::AdvancedTypography
            | Self::StandardFormulas
            | Self::FullDocumentIo => EditionTier::Community,

            Self::LocalAiCopilot | Self::MacroDebugger | Self::BatchConversion => {
                EditionTier::Professional
            }

            Self::EnterpriseDlp
            | Self::SanitizationCertificates
            | Self::TamperEvidentAuditLogs
            | Self::RealtimeCollabWebRtc
            | Self::CustomWasmPlugins => EditionTier::Enterprise,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BasicEditing => "basic-editing",
            Self::AdvancedTypography => "advanced-typography",
            Self::StandardFormulas => "standard-formulas",
            Self::FullDocumentIo => "full-document-io",
            Self::LocalAiCopilot => "local-ai-copilot",
            Self::MacroDebugger => "macro-debugger",
            Self::BatchConversion => "batch-conversion",
            Self::EnterpriseDlp => "enterprise-dlp",
            Self::SanitizationCertificates => "sanitization-certificates",
            Self::TamperEvidentAuditLogs => "tamper-evident-audit-logs",
            Self::RealtimeCollabWebRtc => "realtime-collab-webrtc",
            Self::CustomWasmPlugins => "custom-wasm-plugins",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().replace('_', "-").as_str() {
            "basic-editing" => Some(Self::BasicEditing),
            "advanced-typography" => Some(Self::AdvancedTypography),
            "standard-formulas" => Some(Self::StandardFormulas),
            "full-document-io" => Some(Self::FullDocumentIo),
            "local-ai-copilot" | "copilot" => Some(Self::LocalAiCopilot),
            "macro-debugger" | "macro" => Some(Self::MacroDebugger),
            "batch-conversion" | "conversion" => Some(Self::BatchConversion),
            "enterprise-dlp" | "dlp" => Some(Self::EnterpriseDlp),
            "sanitization-certificates" | "sanitization" => Some(Self::SanitizationCertificates),
            "tamper-evident-audit-logs" | "audit-logs" => Some(Self::TamperEvidentAuditLogs),
            "realtime-collab-webrtc" | "collab" => Some(Self::RealtimeCollabWebRtc),
            "custom-wasm-plugins" | "plugins" => Some(Self::CustomWasmPlugins),
            _ => None,
        }
    }
}

/// Cryptographically verifiable license token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseToken {
    pub license_id: String,
    pub licensee_name: String,
    pub organization: Option<String>,
    pub tier: EditionTier,
    pub issued_at_ms: u64,
    pub expires_at_ms: Option<u64>,
    pub max_seats: u32,
    pub custom_features: Vec<String>,
    pub signature: String,
}

impl LicenseToken {
    /// Generates canonical string representation for cryptographic signing and verification.
    pub fn canonical_payload(&self) -> String {
        let org_str = self.organization.as_deref().unwrap_or("none");
        let exp_str = match self.expires_at_ms {
            Some(exp) => exp.to_string(),
            None => "perpetual".to_string(),
        };

        format!(
            "AETHER-LIC:v1|id={}|user={}|org={}|tier={}|issued={}|expires={}|seats={}|custom={}",
            self.license_id,
            self.licensee_name,
            org_str,
            self.tier.as_str(),
            self.issued_at_ms,
            exp_str,
            self.max_seats,
            self.custom_features.join(",")
        )
    }

    /// Checks if license has exceeded expiration timestamp.
    pub fn is_expired(&self, now_ms: u64) -> bool {
        match self.expires_at_ms {
            Some(exp) => now_ms > exp,
            None => false,
        }
    }

    /// Checks if license is within grace period after expiration.
    pub fn is_in_grace_period(&self, now_ms: u64, grace_period_ms: u64) -> bool {
        match self.expires_at_ms {
            Some(exp) => now_ms > exp && now_ms <= exp.saturating_add(grace_period_ms),
            None => false,
        }
    }

    /// Computes days remaining before expiration (or 0 if already expired).
    pub fn days_remaining(&self, now_ms: u64) -> Option<u32> {
        match self.expires_at_ms {
            Some(exp) => {
                if exp > now_ms {
                    let diff_ms = exp - now_ms;
                    Some((diff_ms / (24 * 60 * 60 * 1000)) as u32)
                } else {
                    Some(0)
                }
            }
            None => None, // Perpetual
        }
    }
}

/// Entitlement verification evaluation status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum EntitlementStatus {
    Granted,
    GracePeriod {
        days_remaining: u32,
        message: String,
    },
    Denied {
        required_tier: EditionTier,
        reason: String,
    },
}

impl EntitlementStatus {
    pub fn is_granted(&self) -> bool {
        matches!(self, Self::Granted | Self::GracePeriod { .. })
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, Self::Denied { .. })
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Helper generating signed license tokens using HMAC-SHA256.
pub struct LicenseGenerator;

impl LicenseGenerator {
    #[allow(clippy::too_many_arguments)]
    pub fn create_signed_token(
        license_id: &str,
        licensee_name: &str,
        organization: Option<&str>,
        tier: EditionTier,
        issued_at_ms: u64,
        expires_at_ms: Option<u64>,
        max_seats: u32,
        custom_features: Vec<String>,
        secret_key: &[u8],
    ) -> LicenseToken {
        let mut token = LicenseToken {
            license_id: license_id.to_string(),
            licensee_name: licensee_name.to_string(),
            organization: organization.map(|s| s.to_string()),
            tier,
            issued_at_ms,
            expires_at_ms,
            max_seats,
            custom_features,
            signature: String::new(),
        };

        let payload = token.canonical_payload();
        let sig = hmac(HashAlgorithm::Sha256, secret_key, payload.as_bytes());
        token.signature = bytes_to_hex(&sig);
        token
    }
}

/// Pure-Rust offline entitlement and license governance engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitlementEngine {
    current_tier: EditionTier,
    active_license: Option<LicenseToken>,
    grace_period_ms: u64,
}

impl Default for EntitlementEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EntitlementEngine {
    pub fn new() -> Self {
        Self {
            current_tier: EditionTier::Community,
            active_license: None,
            grace_period_ms: DEFAULT_GRACE_PERIOD_MS,
        }
    }

    pub fn with_tier(tier: EditionTier) -> Self {
        Self {
            current_tier: tier,
            active_license: None,
            grace_period_ms: DEFAULT_GRACE_PERIOD_MS,
        }
    }

    /// Returns currently effective edition tier accounting for license and expiration.
    pub fn effective_tier(&self, now_ms: u64) -> EditionTier {
        if let Some(ref lic) = self.active_license {
            if !lic.is_expired(now_ms) || lic.is_in_grace_period(now_ms, self.grace_period_ms) {
                return lic.tier;
            }
        }
        self.current_tier
    }

    /// Verifies and activates a commercial license token offline.
    pub fn apply_license(
        &mut self,
        token: LicenseToken,
        verification_key: &[u8],
        now_ms: u64,
    ) -> Result<EntitlementStatus, String> {
        let payload = token.canonical_payload();
        let expected_sig = hmac(HashAlgorithm::Sha256, verification_key, payload.as_bytes());
        let expected_hex = bytes_to_hex(&expected_sig);

        if !token.signature.eq_ignore_ascii_case(&expected_hex) {
            return Err("Invalid license signature: cryptographic verification failed".to_string());
        }

        if token.is_expired(now_ms) {
            if token.is_in_grace_period(now_ms, self.grace_period_ms) {
                let exp = token.expires_at_ms.unwrap_or(now_ms);
                let grace_end = exp.saturating_add(self.grace_period_ms);
                let days_left = ((grace_end.saturating_sub(now_ms)) / (24 * 60 * 60 * 1000)) as u32;
                self.current_tier = token.tier;
                self.active_license = Some(token);
                return Ok(EntitlementStatus::GracePeriod {
                    days_remaining: days_left,
                    message: format!(
                        "License expired. Operating in grace period with {days_left} day(s) remaining."
                    ),
                });
            } else {
                return Err("License expired beyond allowable grace period".to_string());
            }
        }

        self.current_tier = token.tier;
        self.active_license = Some(token);
        Ok(EntitlementStatus::Granted)
    }

    /// Deactivates commercial license and reverts system to Community tier.
    pub fn remove_license(&mut self) {
        self.active_license = None;
        self.current_tier = EditionTier::Community;
    }

    /// Evaluates if a specific feature entitlement is granted under active tier and license.
    pub fn check_feature(&self, feature: FeatureEntitlement, now_ms: u64) -> EntitlementStatus {
        let effective = self.effective_tier(now_ms);
        let required = feature.required_tier();

        // 1. Custom feature override in active license
        if let Some(ref lic) = self.active_license {
            if lic
                .custom_features
                .iter()
                .any(|f| f.eq_ignore_ascii_case(feature.as_str()))
            {
                if lic.is_in_grace_period(now_ms, self.grace_period_ms) {
                    let exp = lic.expires_at_ms.unwrap_or(now_ms);
                    let grace_end = exp.saturating_add(self.grace_period_ms);
                    let days_left =
                        ((grace_end.saturating_sub(now_ms)) / (24 * 60 * 60 * 1000)) as u32;
                    return EntitlementStatus::GracePeriod {
                        days_remaining: days_left,
                        message: format!(
                            "Grace period active for custom feature '{}'",
                            feature.as_str()
                        ),
                    };
                } else if !lic.is_expired(now_ms) {
                    return EntitlementStatus::Granted;
                }
            }
        }

        // 2. Tier comparison
        if effective >= required {
            if let Some(ref lic) = self.active_license {
                if lic.is_in_grace_period(now_ms, self.grace_period_ms) {
                    let exp = lic.expires_at_ms.unwrap_or(now_ms);
                    let grace_end = exp.saturating_add(self.grace_period_ms);
                    let days_left =
                        ((grace_end.saturating_sub(now_ms)) / (24 * 60 * 60 * 1000)) as u32;
                    return EntitlementStatus::GracePeriod {
                        days_remaining: days_left,
                        message: format!("License in grace period ({days_left} day(s) remaining)"),
                    };
                }
            }
            EntitlementStatus::Granted
        } else {
            EntitlementStatus::Denied {
                required_tier: required,
                reason: format!(
                    "Feature '{}' requires {} tier (Current: {})",
                    feature.as_str(),
                    required.as_str(),
                    effective.as_str()
                ),
            }
        }
    }

    pub fn current_tier(&self) -> EditionTier {
        self.current_tier
    }

    pub fn active_license(&self) -> Option<&LicenseToken> {
        self.active_license.as_ref()
    }

    /// Admin / development tier override.
    pub fn set_tier_override(&mut self, tier: EditionTier) {
        self.current_tier = tier;
    }
}
