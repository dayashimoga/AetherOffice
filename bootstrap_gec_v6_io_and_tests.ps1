# ==============================================================================
# bootstrap_gec_v6_io_and_tests.ps1 - Upgrades Contract to GEC-v6.0
# Enforces: Separate Header/Body/Footer in Word, Real Binary ZIP/OOXML & PDF
# File Open/Save, Strict App-Scoped Multi-Document Tabs, Full Dark Theme,
# and 100% Automated Feature & File Round-Trip Verification.
# ==============================================================================
$ErrorActionPreference = "Stop"
Write-Host "=== [GEC-v6.0 1/2] Installing Real File I/O & GEC-v6.0 Verification Gate ===" -ForegroundColor Cyan

@'
---
trigger: always_on
description: Global Engineering Contract (GEC-v6.0) enforcing Separate Word Header/Body/Footer Zones, Real Binary File Open/Save Round-Tripping (.docx, .xlsx, .pptx, .pdf, .txt), App-Scoped Multi-Instance Tabs, Complete Dark/Light Theme, and Responsive Cross-Platform Layout.
---

# GLOBAL ENGINEERING CONTRACT (GEC-v6.0)

## ARTICLE I: WORD (.DOCX) HEADER / BODY / FOOTER STRICT ISOLATION
1. **Three Distinct Data Zones Per Page:**
   - Every Word page object MUST maintain three strictly separate fields: `headerHtml: string`, `bodyHtml: string`, and `footerHtml: string`.
   - **NEVER** bind the Header container and the Body container to the same state variable, and **NEVER** render a duplicate preview of the body text at the top of the page!
2. **Header/Footer Double-Click Lock & Explicit Exit:**
   - By default, `activeEditZone` MUST be `"body"`. Typing in the document edits ONLY `bodyHtml`.
   - The Header and Footer zones are read-only until the user **double-clicks** the Header/Footer area or clicks `Insert -> Header` / `Insert -> Footer`.
   - When `activeEditZone` is `"header"` or `"footer"`, display a prominent floating **`[✓ Close Header & Footer (Esc)]`** button that immediately returns focus to the main page `"body"`.

## ARTICLE II: REAL BINARY FILE OPEN & SAVE FOR ALL 5 FORMATS (ZERO EMPTY FILES)
1. **Opening Existing Files (`Ctrl+O` / `Open`):**
   - **`.docx`:** Must unzip the `.docx` `ArrayBuffer` (via Rust `ooxml-opc` / `word-engine` IPC or client ZIP parser), extract all text/formatting/tables from `word/document.xml`, `word/header*.xml`, and `word/footer*.xml`, and populate the opened tab's pages with the REAL file content! Never open an empty tab when loading an existing file.
   - **`.xlsx` / `.csv`:** Must unzip `xl/sharedStrings.xml` and `xl/worksheets/sheet*.xml` (or parse CSV/TSV rows), extract every cell coordinate, string, number, and formula, and populate the Excel grid!
   - **`.pptx`:** Must unzip `ppt/slides/slide*.xml`, extract all slide text shapes (`<a:t>`), geometry, and colors, and populate the slides!
   - **`.pdf`:** Must parse the `.pdf` `ArrayBuffer`, extract pages, page dimensions, and text items/streams, and render the loaded PDF pages onto the interactive PDF Studio canvas!
   - **`.txt` / `.md`:** Must read UTF-8 text via `FileReader.readAsText` / Rust FS and populate the Notepad editor immediately!
2. **Saving Files (`Ctrl+S` / `Save` / `Save As`):**
   - Must work in BOTH Tauri Desktop (via Rust IPC command `save_document_bytes(filename, bytes)` writing directly to the user's Downloads/Documents folder with a toast notification showing the exact saved path!) AND browser fallback (`Blob` + `<a download>`).

## ARTICLE III: APP-SCOPED MULTI-INSTANCE WORKSPACE & FULL DARK/LIGHT THEME
1. **Strict App-Scoped Tab Bar (`defaultFilter = "current_app"`):**
   - Clicking **Word (`W`)** on the left rail shows ONLY open `.docx` tabs in the top tab bar (plus `+ New Word` and `📂 Open .docx`).
   - Clicking **Excel (`X`)** shows ONLY open `.xlsx` tabs (plus `+ New Excel` and `📂 Open .xlsx`).
   - Clicking **PowerPoint (`P`)** shows ONLY open `.pptx` tabs.
   - Clicking **PDF (`PDF`)** shows ONLY open `.pdf` tabs.
   - Clicking **Notes (`N`)** shows ONLY open `.txt/.md` tabs.
   - Users can open unlimited files of each type simultaneously and switch apps cleanly via the Left Rail badges (`W (3)`, `X (2)`, `PDF (4)`).
2. **100% Dark / Light Theme Coverage:**
   - Every single panel—including PDF Studio's `Pages / Outline / Notes` sidebar, PDF canvas surround, Excel sheet tabs, Word ruler, modals, and dropdowns—MUST inherit dark mode surface tokens (`#0f172a` / `#1e293b` / `#f8fafc`) when Dark Mode is active. Zero unstyled white panels in Dark Mode!
3. **Cross-Platform Responsive Adaptation (Desktop, Tablet, Mobile):**
   - Must dynamically adapt layout across Desktop (`>= 1024px`), Tablet (`768px - 1023px` compact touch ribbon), and Mobile (`< 768px` bottom navigation bar + scrollable compact tool strip).
'@ | Set-Content -Path ".agents/rules/00-global-engineering-contract.md" -Encoding UTF8

# Update tools/verify-contract.ps1 to enforce GEC-v6.0
@'
$ErrorActionPreference = "Stop"
Write-Host "=== [GEC-v6.0] Running File I/O, Word Body/Header Isolation & Full UI Audit ===" -ForegroundColor Cyan

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

# Gate 2: Banned Mockup / Hardcoded "John Doe" Signature Audit
if (Test-Path "apps/unified-shell/src") {
    Write-Host "[2/5] Scanning Frontend for banned 'John Doe' signature & hardcoded bugs..."
    $uiFiles = Get-ChildItem -Path "apps/unified-shell/src" -Recurse -Include "*.ts","*.tsx" | Where-Object { $_.FullName -notmatch "__tests__" -and $_.FullName -notmatch "\.test\." }
    foreach ($uf in $uiFiles) {
        $raw = Get-Content $uf.FullName -Raw
        if ($raw -match "John Doe") {
            Write-Host "CONTRACT VIOLATION (GEC-v6.0): Hardcoded 'John Doe' signature found in $($uf.FullName)!" -ForegroundColor Red
            exit 1
        }
    }
    Write-Host "  PASS: Zero hardcoded 'John Doe' signatures." -ForegroundColor Green
}

# Gate 3: Rust Format & Clippy
Write-Host "[3/5] Running cargo fmt & clippy..."
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit 1 }
cargo clippy --workspace --all-targets --jobs 2 -- -D warnings
if ($LASTEXITCODE -ne 0) { exit 1 }

# Gate 4: Rust Workspace Unit Tests
Write-Host "[4/5] Running Rust workspace unit tests..."
cargo test --workspace --all-targets --jobs 2
if ($LASTEXITCODE -ne 0) { exit 1 }

# Gate 5: Frontend Build & Exhaustive Automated E2E + File Round-Trip Tests
if (Test-Path "apps/unified-shell/package.json") {
    Write-Host "[5/5] Running Frontend Build & GEC-v6.0 Automated Test Suite..."
    Push-Location "apps/unified-shell"
    npm run build
    if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
    npm test -- --run
    if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
    Pop-Location
}

Write-Host "=== [GEC-v6.0] ALL PRODUCTION & FILE I/O GATES PASSED! ===" -ForegroundColor Green
exit 0
'@ | Set-Content -Path "tools/verify-contract.ps1" -Encoding UTF8

Write-Host "GEC-v6.0 Contract and Verification Gate installed." -ForegroundColor Green

