# ==============================================================================
# bootstrap_ui_contract_v2.ps1 - Upgrades Contract to GEC-v2.0 (Zero-Mockup UI Gate)
# Blocks agents from completing stages if static demo data or dead buttons exist.
# ==============================================================================
$ErrorActionPreference = "Stop"
Write-Host "=== [Remediation 1/3] Upgrading Global Engineering Contract to GEC-v2.0 ===" -ForegroundColor Cyan

@'
---
trigger: always_on
description: Global Engineering Contract (GEC-v2.0) enforcing zero-mockup UI, real-time Rust IPC wiring, 5 standalone binaries, and E2E Playwright verification.
---

# GLOBAL ENGINEERING CONTRACT (GEC-v2.0)

## ARTICLES I - VI (INHERITED FROM GEC-v1.0)
All Clean-Room, ECMA-376 / ISO 32000 Round-Trip, Performance SLAs (<150ms startup, 120fps), Zero-Panic Rust (`#[forbid(unsafe_code)]`, no `.unwrap()`/`.expect()`), and 5-OS Portability rules remain strictly in force.

## ARTICLE VII: ZERO-MOCKUP UI & LIVE RUST IPC MANDATE (NEW - CRITICAL)
1. **Ban on Hardcoded Demo Documents:**
   - The frontend (`apps/unified-shell/src/`) MUST NEVER boot into hardcoded fake tabs (`Financials.xlsx`, `Pitch.pptx`, or `"OpenOfficeRS — Executive Technical Summary"`).
   - Every standalone app (`Writer`, `Calc`, `Slides`, `PDF`) must boot into a clean, editable blank document (`Untitled.docx`, `Book1.xlsx`, `Presentation1.pptx`, or PDF Open/Create Screen) or open a file passed via CLI/OS file association.
2. **Ban on Dead / Placeholder Ribbon Tabs:**
   - Every top-level menu tab (`File`, `Home`, `Insert`, `Layout`, `Formulas`, `Data`, `Transitions`, `Animations`, `Slide Show`, `PDF Tools`, `Review`, `View`) MUST switch the active ribbon panel cleanly (`activeRibbonTab` state) and render real, clickable controls specific ONLY to the active application mode.
   - Word styles (`Normal`, `Heading 1`) must NEVER appear inside PowerPoint or Excel ribbons.
3. **5-Binary Standalone + Unified Hub Architecture:**
   - The project must produce **5 distinct executable entrypoints**:
     1. `aether-hub` (Main Unified Launcher & Multi-Window Suite Manager)
     2. `aether-writer` (Standalone Word Processor `.docx` App)
     3. `aether-calc` (Standalone Spreadsheet `.xlsx` App)
     4. `aether-slides` (Standalone Presentation `.pptx` App)
     5. `aether-pdf` (Standalone PDF Read/Edit/Write `.pdf` App)
   - Passing `--app writer|calc|slides|pdf` or launching the respective binary must mount ONLY that application's dedicated chrome, menu bar, shortcuts, and file dialogs—zero cross-format clutter.
4. **End-to-End Interactive Verification Gate:**
   - No UI subphase (Stages 8–13) may be marked `COMPLETED` based solely on Rust unit tests.
   - Every subphase MUST include an automated frontend integration/E2E test (`apps/unified-shell/src/__tests__/stage_<id>.test.ts` via Vitest/Playwright DOM) that programmatically clicks the Ribbon buttons, simulates keyboard typing/cell editing/shape dragging, and asserts that the underlying document state and DOM canvas actually mutated.
'@ | Set-Content -Path ".agents/rules/00-global-engineering-contract.md" -Encoding UTF8

# Update tools/verify-contract.ps1 to enforce Frontend TypeScript + Vitest DOM tests & Anti-Mockup scan
@'
$ErrorActionPreference = "Stop"
Write-Host "=== [GEC-v2.0] Running Full Rust + Frontend UI Interactivity Contract Audit ===" -ForegroundColor Cyan

# Gate 1: Forbidden Panic Macros in Rust
Write-Host "[1/6] Scanning crates/*/src for banned .unwrap(), .expect(), todo!(), unimplemented!()..."
$libFiles = Get-ChildItem -Path "crates" -Recurse -Filter "*.rs" | Where-Object { $_.FullName -notmatch "\\tests\\" }
$violations = @()
foreach ($file in $libFiles) {
    $lines = Get-Content $file.FullName
    $inTest = $false
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match "#\[cfg\(test\)\]") { $inTest = $true }
        if (-not $inTest -and $lines[$i] -match "\.unwrap\(\)|\.expect\(|todo!\(|unimplemented!\(") {
            $violations += "$($file.FullName):$($i+1): $($lines[$i])"
        }
    }
}
if ($violations.Count -gt 0) {
    Write-Host "CONTRACT VIOLATION (Article IV.2): Banned panic macros found:" -ForegroundColor Red
    $violations | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    exit 1
}
Write-Host "  PASS: Zero panic macros." -ForegroundColor Green

# Gate 2: Rust Formatting & Clippy
Write-Host "[2/6] Verifying Rust fmt & strict Clippy..."
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit 1 }
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit 1 }

# Gate 3: Rust Workspace Unit Tests
Write-Host "[3/6] Running Rust workspace unit tests..."
cargo test --workspace --all-targets
if ($LASTEXITCODE -ne 0) { exit 1 }

# Gate 4: Frontend Anti-Placeholder Scan
if (Test-Path "apps/unified-shell/src") {
    Write-Host "[4/6] Scanning Frontend for dead placeholder strings..."
    $uiFiles = Get-ChildItem -Path "apps/unified-shell/src" -Recurse -Include "*.ts","*.tsx" | Where-Object { $_.FullName -notmatch "__tests__" }
    foreach ($uf in $uiFiles) {
        $content = Get-Content $uf.FullName -Raw
        if ($content -match "commands for PowerPoint|commands for Excel") {
            Write-Host "CONTRACT VIOLATION (Article VII.2): Dead placeholder text found in $($uf.FullName)" -ForegroundColor Red
            exit 1
        }
    }
    Write-Host "  PASS: Zero dead ribbon placeholder strings." -ForegroundColor Green
}

# Gate 5: Frontend Build & TypeScript Strict Check
if (Test-Path "apps/unified-shell/package.json") {
    Write-Host "[5/6] Building Frontend & verifying TypeScript types..."
    Push-Location "apps/unified-shell"
    npm run build
    if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
    Pop-Location
}

# Gate 6: Frontend Interactive DOM / E2E Test Suite
if (Test-Path "apps/unified-shell/package.json") {
    $pkgJson = Get-Content "apps/unified-shell/package.json" -Raw
    if ($pkgJson -match '"test"') {
        Write-Host "[6/6] Running Frontend Interactive DOM & Command Wiring Tests..."
        Push-Location "apps/unified-shell"
        npm test -- --run
        if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
        Pop-Location
    }
}

Write-Host "=== [GEC-v2.0] ALL RUST + INTERACTIVE UI CONTRACT GATES PASSED! ===" -ForegroundColor Green
exit 0
'@ | Set-Content -Path "tools/verify-contract.ps1" -Encoding UTF8

Write-Host "GEC-v2.0 Contract and UI Verification Gate installed!" -ForegroundColor Green