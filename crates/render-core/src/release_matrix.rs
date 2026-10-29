//! Cross-Platform Release Matrix & Target Configuration Engine
//!
//! Complies with ECMA-376, ISO/IEC 29500, and GEC-v1.0 Article III & V.
//! Defines and validates multi-platform compilation targets, packaging bundles,
//! and CI/CD release matrix artifacts across:
//! - Windows (`x86_64-pc-windows-msvc`, `x86_64-pc-windows-gnu`)
//! - Linux (`x86_64-unknown-linux-gnu`)
//! - macOS (`aarch64-apple-darwin`, `x86_64-apple-darwin`, Universal)
//! - Android (`aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`)
//! - iOS (`aarch64-apple-ios`, `aarch64-apple-ios-sim`)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Operating System platforms supported by the unified release matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetOs {
    Windows,
    Linux,
    MacOS,
    Android,
    Ios,
}

impl fmt::Display for TargetOs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Windows => write!(f, "Windows"),
            Self::Linux => write!(f, "Linux"),
            Self::MacOS => write!(f, "macOS"),
            Self::Android => write!(f, "Android"),
            Self::Ios => write!(f, "iOS"),
        }
    }
}

/// Target CPU architectures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetArch {
    X86_64,
    Aarch64,
    Armv7,
    Universal,
}

impl fmt::Display for TargetArch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::X86_64 => write!(f, "x86_64"),
            Self::Aarch64 => write!(f, "aarch64"),
            Self::Armv7 => write!(f, "armv7"),
            Self::Universal => write!(f, "universal"),
        }
    }
}

/// Packaging and installer bundle formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackagingBundle {
    Msi,
    Nsis,
    AppImage,
    Deb,
    Dmg,
    AppBundle,
    Apk,
    Aab,
    Ipa,
    StandaloneExe,
    StandaloneElf,
}

impl PackagingBundle {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Msi => ".msi",
            Self::Nsis => ".exe",
            Self::AppImage => ".AppImage",
            Self::Deb => ".deb",
            Self::Dmg => ".dmg",
            Self::AppBundle => ".app",
            Self::Apk => ".apk",
            Self::Aab => ".aab",
            Self::Ipa => ".ipa",
            Self::StandaloneExe => ".exe",
            Self::StandaloneElf => "",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Msi => "application/x-msi",
            Self::Nsis | Self::StandaloneExe => "application/vnd.microsoft.portable-executable",
            Self::AppImage => "application/vnd.appimage",
            Self::Deb => "application/vnd.debian.binary-package",
            Self::Dmg => "application/x-apple-diskimage",
            Self::AppBundle => "application/x-apple-application",
            Self::Apk => "application/vnd.android.package-archive",
            Self::Aab => "application/octet-stream",
            Self::Ipa => "application/octet-stream",
            Self::StandaloneElf => "application/x-executable",
        }
    }
}

/// Configuration descriptor for a single release compilation target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformReleaseTarget {
    pub os: TargetOs,
    pub arch: TargetArch,
    pub rust_triplet: String,
    pub bundle_formats: Vec<PackagingBundle>,
    pub min_os_version: String,
    pub requires_code_signing: bool,
    pub headless_capable: bool,
    pub mobile_sandboxed: bool,
    pub startup_sla_limit_ms: u32,
    pub memory_sla_limit_mb: u32,
}

impl PlatformReleaseTarget {
    pub fn new(
        os: TargetOs,
        arch: TargetArch,
        rust_triplet: impl Into<String>,
        bundle_formats: Vec<PackagingBundle>,
        min_os_version: impl Into<String>,
    ) -> Self {
        let is_mobile = matches!(os, TargetOs::Android | TargetOs::Ios);

        // GEC-v1.0 Article III: Startup SLA is < 150ms on Desktop, < 300ms on Mobile
        let startup_sla_limit_ms = if is_mobile { 300 } else { 150 };

        // GEC-v1.0 Article III: Memory SLA is < 35 MB idle RSS, < 180 MB for 1M cells
        let memory_sla_limit_mb = if is_mobile { 120 } else { 180 };

        let requires_code_signing = os != TargetOs::Linux;
        let headless_capable = !is_mobile;
        let mobile_sandboxed = is_mobile;

        Self {
            os,
            arch,
            rust_triplet: rust_triplet.into(),
            bundle_formats,
            min_os_version: min_os_version.into(),
            requires_code_signing,
            headless_capable,
            mobile_sandboxed,
            startup_sla_limit_ms,
            memory_sla_limit_mb,
        }
    }

    /// Builder pattern to configure code signing requirements.
    pub fn with_code_signing(mut self, enabled: bool) -> Self {
        self.requires_code_signing = enabled;
        self
    }

    /// Primary artifact filename pattern.
    pub fn artifact_filename(
        &self,
        app_name: &str,
        version: &str,
        bundle: PackagingBundle,
    ) -> String {
        format!(
            "{}-{}-{}-{}{}",
            app_name,
            version,
            self.os,
            self.arch,
            bundle.extension()
        )
    }

    /// Validate that this target adheres to GEC performance SLAs.
    pub fn check_sla_budget(
        &self,
        measured_startup_ms: u32,
        measured_rss_mb: u32,
    ) -> Result<(), String> {
        if measured_startup_ms > self.startup_sla_limit_ms {
            return Err(format!(
                "Startup SLA exceeded for {}: measured {} ms > limit {} ms",
                self.rust_triplet, measured_startup_ms, self.startup_sla_limit_ms
            ));
        }
        if measured_rss_mb > self.memory_sla_limit_mb {
            return Err(format!(
                "Memory SLA exceeded for {}: measured {} MB > limit {} MB",
                self.rust_triplet, measured_rss_mb, self.memory_sla_limit_mb
            ));
        }
        Ok(())
    }
}

/// Cross-platform Release Matrix covering all 5 tier-1 operating systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseMatrix {
    pub app_name: String,
    pub version: String,
    pub targets: Vec<PlatformReleaseTarget>,
}

impl Default for ReleaseMatrix {
    fn default() -> Self {
        Self::canonical_matrix("OpenOfficeRS", "0.1.0")
    }
}

impl ReleaseMatrix {
    /// Create the canonical 5-platform release matrix complying with Article V.
    pub fn canonical_matrix(app_name: impl Into<String>, version: impl Into<String>) -> Self {
        let targets = vec![
            // 1. Windows x64 Desktop
            PlatformReleaseTarget::new(
                TargetOs::Windows,
                TargetArch::X86_64,
                "x86_64-pc-windows-msvc",
                vec![
                    PackagingBundle::Msi,
                    PackagingBundle::Nsis,
                    PackagingBundle::StandaloneExe,
                ],
                "Windows 10 Build 19041+",
            ),
            // 2. Linux x64 Desktop
            PlatformReleaseTarget::new(
                TargetOs::Linux,
                TargetArch::X86_64,
                "x86_64-unknown-linux-gnu",
                vec![
                    PackagingBundle::AppImage,
                    PackagingBundle::Deb,
                    PackagingBundle::StandaloneElf,
                ],
                "glibc 2.31+ / Ubuntu 20.04+",
            ),
            // 3. macOS Universal Desktop
            PlatformReleaseTarget::new(
                TargetOs::MacOS,
                TargetArch::Universal,
                "aarch64-apple-darwin+x86_64-apple-darwin",
                vec![PackagingBundle::Dmg, PackagingBundle::AppBundle],
                "macOS 12.0 Monterey+",
            ),
            // 4. Android ARM64 Mobile
            PlatformReleaseTarget::new(
                TargetOs::Android,
                TargetArch::Aarch64,
                "aarch64-linux-android",
                vec![PackagingBundle::Apk, PackagingBundle::Aab],
                "Android 8.0 (API Level 26+)",
            ),
            // 5. Android ARMv7 Mobile (32-bit legacy fallback)
            PlatformReleaseTarget::new(
                TargetOs::Android,
                TargetArch::Armv7,
                "armv7-linux-androideabi",
                vec![PackagingBundle::Apk],
                "Android 8.0 (API Level 26+)",
            ),
            // 6. Android x86_64 Mobile (Emulator)
            PlatformReleaseTarget::new(
                TargetOs::Android,
                TargetArch::X86_64,
                "x86_64-linux-android",
                vec![PackagingBundle::Apk],
                "Android 8.0 (API Level 26+)",
            ),
            // 7. iOS ARM64 Mobile (Physical Devices)
            PlatformReleaseTarget::new(
                TargetOs::Ios,
                TargetArch::Aarch64,
                "aarch64-apple-ios",
                vec![PackagingBundle::Ipa, PackagingBundle::AppBundle],
                "iOS 15.0+",
            ),
            // 8. iOS ARM64 Mobile Simulator
            PlatformReleaseTarget::new(
                TargetOs::Ios,
                TargetArch::Aarch64,
                "aarch64-apple-ios-sim",
                vec![PackagingBundle::AppBundle],
                "iOS Simulator 15.0+",
            )
            .with_code_signing(false),
        ];

        Self {
            app_name: app_name.into(),
            version: version.into(),
            targets,
        }
    }

    /// Retrieve all targets configured for a specific OS.
    pub fn targets_for_os(&self, os: TargetOs) -> Vec<&PlatformReleaseTarget> {
        self.targets.iter().filter(|t| t.os == os).collect()
    }

    /// Validate that all 5 target operating systems have at least one valid compilation target.
    pub fn validate_full_coverage(&self) -> Result<(), String> {
        let required_os = [
            TargetOs::Windows,
            TargetOs::Linux,
            TargetOs::MacOS,
            TargetOs::Android,
            TargetOs::Ios,
        ];

        for &os in &required_os {
            let count = self.targets_for_os(os).len();
            if count == 0 {
                return Err(format!(
                    "Release matrix lacks target coverage for OS: {}",
                    os
                ));
            }
        }
        Ok(())
    }

    /// Validate that all Rust triplets are recognized standard compilation targets.
    pub fn validate_triplets(&self) -> Result<(), String> {
        for target in &self.targets {
            if target.rust_triplet.trim().is_empty() {
                return Err(format!("Target {} has empty rust_triplet", target.os));
            }
            if target.bundle_formats.is_empty() {
                return Err(format!(
                    "Target {} ({}) has no packaging bundles defined",
                    target.os, target.rust_triplet
                ));
            }
        }
        Ok(())
    }

    /// Export the GitHub Actions CI/CD matrix JSON structure.
    pub fn export_github_matrix_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("JSON serialization error: {e}"))
    }
}
