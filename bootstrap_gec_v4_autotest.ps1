# ==============================================================================
# bootstrap_gec_v4_autotest.ps1 - Installs GEC-v4.0 Contract & Automated
# Feature-by-Feature Verification Gate so ZERO broken buttons can pass.
# ==============================================================================
$ErrorActionPreference = "Stop"
Write-Host "=== [GEC-v4.0 1/2] Installing Exhaustive Automated UI Feature Gate ===" -ForegroundColor Cyan

@'
---
trigger: always_on
description: Global Engineering Contract (GEC-v4.0) enforcing 100% Automated Feature Testing across Word, Excel, PPT, PDF, and TXT, Full-App Theme Cascading, Multi-Window Per Format Management, and Zero Visual Bleed.
---

# GLOBAL ENGINEERING CONTRACT (GEC-v4.0)

## ARTICLE I: SEPARATE WINDOWS PER DOCUMENT TYPE + UNIFIED SESSION HUB (LOW RAM)
1. **Dedicated App Windows Inside One Shared Process:**
   - Users must be able to open **Word (`.docx`)**, **Excel (`.xlsx`)**, **PowerPoint (`.pptx`)**, **PDF Studio (`.pdf`)**, and **Notepad (`.txt`/`.md`)** in their own dedicated, separate windows (via Tauri `WebviewWindowBuilder` sharing the single Rust process for `< 25 MB` overhead) OR dock/undock them at will from the **Window / App Hub Switcher**.
   - Every window is scoped cleanly to its document type (e.g., the Word window holds Word tabs and Word chrome; the Excel window holds Excel workbooks; the PDF window holds PDF documents) while a **Global Session & Auto-Save Registry** tracks all open documents across windows so nothing is ever lost and any open document is reachable in 1 click.

## ARTICLE II: ZERO LAYOUT BLEEDING & FULL-APP DARK/LIGHT THEME
1. **No Ghost Sidebars:** Word's `Navigation` sidebar MUST ONLY render when the active view is Word and `showNavPane == true`. It must NEVER appear on the left side of Excel, PowerPoint, PDF, or Notepad!
2. **No Permanent Alt-Keytip Badges:** The black `[F]`, `[H]`, `[N]`, `[P]` badges on Ribbon tabs (`File`, `Home`, `Insert`) MUST stay hidden by default and only appear when the user presses the `Alt` key.
3. **Full-App Theme Cascade:** Toggling `Dark / Light` mode MUST apply CSS theme variables (`--bg-app`, `--bg-toolbar`, `--bg-sidebar`, `--bg-workspace`, `--bg-surface`, `--text-primary`, `--text-muted`, `--border-subtle`) across 100% of the window (Titlebar, Tab bar, Ribbon, Left Sidebar, Workspace background, Dialogs, and Bottom Status Bar).
4. **Dynamic Per-App Status Bar:** The bottom status bar must show accurate metrics for the active app (Word: `Page X of Y | N words | N chars`; Excel: `Sheet X of Y | Selected: A1 | Sum / Avg / Count`; PPT: `Slide X of Y | Theme Name`; PDF: `Page X of Y | Dimensions | Security Status`; TXT: `Ln X, Col Y | N lines | N words | UTF-8`). Never hardcode `22 words`!

## ARTICLE III: 100% WORKING FILE I/O & RIBBON FEATURES (AUTOMATED TEST MANDATE)
1. **Working File Menu (`New`, `Open`, `Save`, `Save As`, `Export`, `Print`):**
   - Must work in ALL 5 apps (`docx`, `xlsx`, `pptx`, `pdf`, `txt`) using real file pickers (`<input type="file">` + Tauri FS API) and real file downloads/writes + clean print frames.
2. **Automated Feature Matrix Gate:**
   - Every single button in every ribbon tab across Word, Excel, PowerPoint, PDF, and Notepad must be tested by an automated Vitest DOM test in `apps/unified-shell/src/__tests__/exhaustive_feature_matrix.test.tsx`.
'@ | Set-Content -Path ".agents/rules/00-global-engineering-contract.md" -Encoding UTF8

# Update tools/verify-contract.ps1 to require the exhaustive automated test matrix
@'
$ErrorActionPreference = "Stop"
Write-Host "=== [GEC-v4.0] Running Automated Feature-Matrix & Contract Audit ===" -ForegroundColor Cyan

# Gate 1: Rust Safety Check
Write-Host "[1/5] Checking Rust crates for banned .unwrap() / .expect()..."
$libFiles = Get-ChildItem -Path "crates" -Recurse -Filter "*.rs" | Where-Object { $_.FullName -notmatch "\\tests\\" }
foreach ($file in $libFiles) {
    $lines = Get-Content $file.FullName
    $inTest = $false
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match "#\[cfg\(test\)\]") { $inTest = $true }
        if (-not $inTest -and $lines[$i] -match "\.unwrap\(\)|\.expect\(|todo!\(|unimplemented!\(") {
            Write-Host "VIOLATION in $($file.FullName):$($i+1)" -ForegroundColor Red
            exit 1
        }
    }
}

# Gate 2: Scan for hardcoded "22 words" status bar bug or fake demo text
if (Test-Path "apps/unified-shell/src") {
    Write-Host "[2/5] Auditing Frontend for hardcoded '22 words' bug & fake text..."
    $uiFiles = Get-ChildItem -Path "apps/unified-shell/src" -Recurse -Include "*.ts","*.tsx" | Where-Object { $_.FullName -notmatch "__tests__" -and $_.FullName -notmatch "\.test\." }
    foreach ($uf in $uiFiles) {
        $raw = Get-Content $uf.FullName -Raw
        if ($raw -match "22 words") {
            Write-Host "CONTRACT VIOLATION (Article II.4): Hardcoded '22 words' status bar found in $($uf.FullName)!" -ForegroundColor Red
            exit 1
        }
    }
    Write-Host "  PASS: Zero hardcoded status bar strings." -ForegroundColor Green
}

# Gate 3: Rust Format & Clippy
Write-Host "[3/5] Running cargo fmt & clippy..."
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit 1 }
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit 1 }

# Gate 4: Rust Workspace Tests
Write-Host "[4/5] Running Rust workspace unit tests..."
cargo test --workspace --all-targets
if ($LASTEXITCODE -ne 0) { exit 1 }

# Gate 5: Frontend Build & Exhaustive Automated Feature Matrix Tests
if (Test-Path "apps/unified-shell/package.json") {
    Write-Host "[5/5] Running Frontend Build & Exhaustive Automated Feature Matrix Tests..."
    Push-Location "apps/unified-shell"
    npm run build
    if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
    npm test -- --run
    if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
    Pop-Location
}

Write-Host "=== [GEC-v4.0] ALL AUTOMATED FEATURE TESTS & CONTRACT GATES PASSED! ===" -ForegroundColor Green
exit 0
'@ | Set-Content -Path "tools/verify-contract.ps1" -Encoding UTF8

Write-Host "GEC-v4.0 Contract and Automated Test Gate installed." -ForegroundColor Green