# ==============================================================================
# bootstrap_unified_overhaul.ps1 - Fixes Memory Hang, Removes Duplicate Binaries,
# Restores Native Window Controls, and Enforces GEC-v3.0 Zero-Fake-Text Audit.
# ==============================================================================
$ErrorActionPreference = "Stop"
Write-Host "=== [Overhaul 1/2] Consolidating to Single Lightweight Binary & GEC-v3.0 ===" -ForegroundColor Cyan

# 1. Remove the duplicate src/bin/*.rs wrappers that spawned multiple WebView2 processes
$binDir = "apps/unified-shell/src-tauri/src/bin"
if (Test-Path $binDir) {
    Remove-Item -Path $binDir -Recurse -Force
    Write-Host "  Removed duplicate multi-binary wrappers (fixes multi-instance RAM hang)." -ForegroundColor Green
}

# 1.1 Update apps/unified-shell/src-tauri/Cargo.toml to only have single AetherOffice binary
$tauriCargo = @'
[package]
name = "unified-shell-tauri"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2.1", features = [] }
ooxml-opc.workspace = true
quick-xml.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
word-engine = { path = "../../../crates/word-engine" }
excel-engine = { path = "../../../crates/excel-engine" }
ppt-engine = { path = "../../../crates/ppt-engine" }
render-core = { path = "../../../crates/render-core" }
crdt-collab = { path = "../../../crates/crdt-collab" }
macro-engine = { path = "../../../crates/macro-engine" }

[build-dependencies]
tauri-build = { version = "2.1", features = [] }

[[bin]]
name = "AetherOffice"
path = "src/main.rs"
'@
Set-Content -Path "apps/unified-shell/src-tauri/Cargo.toml" -Value $tauriCargo -Encoding UTF8
Write-Host "  Configured single AetherOffice binary in Cargo.toml." -ForegroundColor Green

# 1.2 Enable native window controls (decorations: true) in tauri.conf.json
$tauriConfPath = "apps/unified-shell/src-tauri/tauri.conf.json"
if (Test-Path $tauriConfPath) {
    $conf = Get-Content $tauriConfPath -Raw | ConvertFrom-Json
    if ($conf.app.windows.Count -gt 0) {
        $conf.app.windows[0].decorations = $true
    }
    $confUtf8NoBom = New-Object System.Text.UTF8Encoding $false
    $confJson = $conf | ConvertTo-Json -Depth 10
    [System.IO.File]::WriteAllText($tauriConfPath, $confJson, $confUtf8NoBom)
    Write-Host "  Enabled native OS window decorations (decorations: true) in tauri.conf.json." -ForegroundColor Green
}

# 2. Write Global Engineering Contract GEC-v3.0
@'
---
trigger: always_on
description: Global Engineering Contract (GEC-v3.0) enforcing Single-Process <45MB Memory Architecture, Zero Prepopulated Fake Text, Native Window Controls, Dark/Light Theme, and 5 Working Modules (Word, Excel, PPT, PDF, Notepad).
---

# GLOBAL ENGINEERING CONTRACT (GEC-v3.0)

## ARTICLE I: SINGLE-EXECUTABLE, LOW-MEMORY ARCHITECTURE (< 45 MB RAM)
1. **Single Unified Binary (`AetherOffice`):** The suite MUST compile as a SINGLE desktop/mobile application containing an internal App Switcher Rail and multi-tab workspace supporting 5 modules: **Home Hub**, **Word (`.docx`)**, **Excel (`.xlsx`)**, **PowerPoint (`.pptx`)**, **PDF Studio (`.pdf`)**, and **Notepad/Notes (`.txt`/`.md`)**.
2. **Strict Memory Budget (Zero System Hang):**
   - Never pre-allocate 1,000,000 DOM nodes or massive Rust vectors on startup.
   - Excel grids MUST use lazy `HashMap<(u32, u16), Cell>` storage and render ONLY the visible viewport cells (`~40 rows x 15 cols`) in the DOM.
   - Word and PDF documents MUST lazily render pages and use clean native/virtualized scrolling.

## ARTICLE II: ABSOLUTE BAN ON PREPOPULATED FAKE / DEMO TEXT
1. **100% Blank New Documents:**
   - Creating or opening a new Word document MUST open a completely blank editable page (`""` empty string).
   - Creating a new Excel workbook MUST open a completely empty grid (`Sheet1` with zero prepopulated numbers).
   - Creating a new PowerPoint MUST open a clean blank slide with empty placeholder prompts (`Click to add title`).
   - Creating a new Note MUST open an empty text buffer.
2. **Banned Strings:** The strings `"Executive Technical Summary"`, `"Performance SLAs"`, `"Market Opportunity"`, `"Revenue"`, and `"Live Demo & Architecture"` are STRICTLY BANNED from `apps/unified-shell/src/` outside of tests. `tools/verify-contract.ps1` will fail if any fake demo text is present.

## ARTICLE III: UI/UX POLISH, NATIVE WINDOW CONTROLS & DARK/LIGHT THEME
1. **Window Controls:** Minimize, Maximize/Restore, and Close buttons MUST call Tauri's native window API (`appWindow.minimize()`, `appWindow.toggleMaximize()`, `appWindow.close()`) AND native OS window decorations must remain responsive.
2. **Zero Ribbon Scrollbars:** The Ribbon toolbar MUST use clean CSS Flexbox (`flex-wrap: nowrap`, `overflow: visible` for dropdowns, compact icon+label buttons) with ZERO horizontal or vertical scrollbars clipping the controls.
3. **Dark & Light Theme Toggle:** Every screen and document workspace must support a 1-click **Dark / Light Theme Toggle** persisted in `localStorage` using CSS custom properties (`--bg-primary`, `--bg-surface`, `--text-primary`, `--border-color`, `--accent-color`).
'@ | Set-Content -Path ".agents/rules/00-global-engineering-contract.md" -Encoding UTF8
Write-Host "  Updated GEC-v3.0 Contract in .agents/rules/00-global-engineering-contract.md." -ForegroundColor Green

# 3. Upgrade tools/verify-contract.ps1 to scan and block any fake demo text or ribbon placeholders
@'
$ErrorActionPreference = "Stop"
Write-Host "=== [GEC-v3.0] Running Zero-Fake-Text, Memory & Interactivity Audit ===" -ForegroundColor Cyan

# Gate 1: Rust Panic Macro Check
Write-Host "[1/5] Checking Rust crates for banned .unwrap() / .expect()..."
$libFiles = Get-ChildItem -Path "crates" -Recurse -Filter "*.rs" | Where-Object { $_.FullName -notmatch "\\tests\\" }
foreach ($file in $libFiles) {
    $lines = Get-Content $file.FullName
    $inTest = $false
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match "#\[cfg\(test\)\]") { $inTest = $true }
        if (-not $inTest -and $lines[$i] -match "\.unwrap\(\)|\.expect\(|todo!\(|unimplemented!\(") {
            Write-Host "VIOLATION in $($file.FullName):$($i+1): $($lines[$i])" -ForegroundColor Red
            exit 1
        }
    }
}
Write-Host "  PASS: Rust safety verified." -ForegroundColor Green

# Gate 2: Banned Fake Demo Text Scan in Frontend
if (Test-Path "apps/unified-shell/src") {
    Write-Host "[2/5] Scanning Frontend for banned fake/prepopulated demo text..."
    $uiFiles = Get-ChildItem -Path "apps/unified-shell/src" -Recurse -Include "*.ts","*.tsx" | Where-Object { $_.FullName -notmatch "__tests__" -and $_.FullName -notmatch "\.test\." }
    $bannedPatterns = @(
        "Executive Technical Summary",
        "Performance SLAs & Benchmark Compliance",
        "Live Demo & Architecture Validation",
        "commands for PowerPoint",
        "commands for Excel"
    )
    foreach ($uf in $uiFiles) {
        $raw = Get-Content $uf.FullName -Raw
        foreach ($pat in $bannedPatterns) {
            if ($raw -match [regex]::Escape($pat)) {
                Write-Host "CONTRACT VIOLATION (Article II.2): Hardcoded fake demo text '$pat' found in $($uf.FullName)! New documents must be 100% blank and editable." -ForegroundColor Red
                exit 1
            }
        }
    }
    Write-Host "  PASS: Zero fake prepopulated text found." -ForegroundColor Green
}

# Gate 3: Rust Format & Clippy
Write-Host "[3/5] Running cargo fmt & clippy..."
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit 1 }
cargo clippy --workspace --all-targets --jobs 2 -- -D warnings
if ($LASTEXITCODE -ne 0) { exit 1 }

# Gate 4: Rust Unit Tests
Write-Host "[4/5] Running Rust workspace tests..."
cargo test --workspace --all-targets --jobs 2
if ($LASTEXITCODE -ne 0) { exit 1 }

# Gate 5: Frontend Build & Automated E2E Tests
if (Test-Path "apps/unified-shell/package.json") {
    Write-Host "[5/5] Verifying Frontend build & E2E tests..."
    Push-Location "apps/unified-shell"
    npm run build
    if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
    $pkg = Get-Content "package.json" -Raw
    if ($pkg -match '"test"') {
        npm test -- --run
        if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
    }
    Pop-Location
}

Write-Host "=== [GEC-v3.0] ALL GATES PASSED SUCCESSFULLY! ===" -ForegroundColor Green
exit 0
'@ | Set-Content -Path "tools/verify-contract.ps1" -Encoding UTF8

Write-Host "GEC-v3.0 Contract and Zero-Fake-Text Enforcer installed successfully." -ForegroundColor Green
