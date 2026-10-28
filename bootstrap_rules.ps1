# bootstrap_rules.ps1 - Installs GEC-v1.0, Agent Skills & Verification Enforcer
$ErrorActionPreference = "Stop"
Write-Host "=== [2/4] Installing Global Engineering Contract & Rules == = " -ForegroundColor Cyan

$dirs = @(".agents/rules", ".agents/skills", ".agents/workflows", ".githooks", "tools")
foreach ($d in $dirs) { New-Item -ItemType Directory -Force -Path $d | Out-Null }

# Global Engineering Contract
@'
---
trigger: always_on
description: Global Engineering Contract (GEC-v1.0) governing clean-room reverse engineering, ECMA-376 fidelity, performance SLAs, and verification.
---
# GLOBAL ENGINEERING CONTRACT (GEC-v1.0)
1. **Clean-Room Isolation:** All file parsing must adhere strictly to ECMA-376. REA is used exclusively for math/behavior extraction into `specs/evidence/` with zero proprietary code contamination.
2. **Zero-Loss Round-Trip:** Unrecognized XML nodes and OPC parts must be preserved via `UnparsedPartStore`.
3. **Performance SLAs:** `<150ms` desktop startup, `120fps` rendering, zero-blocking UI thread.
4. **Code Quality:** `#[forbid(unsafe_code)]` in engine crates; zero `.unwrap()` or `.expect()` in non-test library paths.
5. **Universal Portability:** Clean compilation across Windows, Linux, macOS, Android, and iOS.
'@ | Set-Content -Path ".agents/rules/00-global-engineering-contract.md" -Encoding UTF8

# Skills & Workflows
@'
---
name: ecma376-architect
description: Implements streaming ECMA-376 parsers with lossless round-tripping.
---
Use quick-xml pull-parsing,rayon, and UnparsedPartStore preservation.
'@ | Set-Content -Path ".agents/skills/ecma376-architect.md" -Encoding UTF8

# Contract Enforcer Tool
@'
$ErrorActionPreference = "Stop"
$libFiles = Get-ChildItem -Path "crates" -Recurse -Filter "*.rs" | Where-Object { $_.FullName -notmatch "\\tests\\" }
foreach ($file in $libFiles) {
    $lines = Get-Content $file.FullName
    $inTest = $false
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match "#\[cfg\(test\)\]") { $inTest = $true }
        if (-not $inTest -and $lines[$i] -match "\.unwrap\(\)|\.expect\(|todo!\(|unimplemented!\(") {
            Write-Host "Violation in $($file.FullName):$($i+1)" -ForegroundColor Red
            exit 1
        }
    }
}
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
Write-Host "All GEC-v1.0 contract gates passed!" -ForegroundColor Green
'@ | Set-Content -Path "tools/verify-contract.ps1" -Encoding UTF8

@'
#!/bin/sh
pwsh -NoProfile -ExecutionPolicy Bypass -File tools/verify-contract.ps1
'@ | Set-Content -Path ".githooks/pre-commit" -Encoding UTF8
git config core.hooksPath .githooks

Write-Host "Rules, contracts, and verification enforcer installed." -ForegroundColor Green