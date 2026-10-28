# ==============================================================================
# bootstrap.ps1 - Unified Master Bootstrap for OpenOfficeRS in Antigravity IDE
# Sets up: Cargo Workspace, Adaptive Tauri Shell, Global Engineering Contract,
#          Antigravity Rules/Skills/Workflows, Git Hooks, & 5-Platform CI/CD.
# ==============================================================================
$ErrorActionPreference = "Stop"

Write-Host "=== [OpenOfficeRS] Starting Unified Workspace Bootstrap ===" -ForegroundColor Cyan

# 1. Create Complete Directory Hierarchy
$dirs = @(
    ".agents/rules",
    ".agents/skills",
    ".agents/workflows",
    ".github/workflows",
    ".githooks",
    "crates/ooxml-opc/src",
    "crates/word-engine/src",
    "crates/excel-engine/src",
    "crates/ppt-engine/src",
    "crates/render-core/src",
    "apps/unified-shell/src",
    "apps/unified-shell/src-tauri/src",
    "specs/evidence/word",
    "specs/evidence/excel",
    "specs/evidence/ppt",
    "tools/com-ground-truth"
)
foreach ($d in $dirs) {
    New-Item -ItemType Directory -Force -Path $d | Out-Null
}

# 2. Initialize Git & .gitignore
if (-not (Test-Path ".git")) {
    git init | Out-Null
}

@'
/target
/node_modules
/dist
/.rea-stage
*.pdb
*.idb
*.i64
*.gpr
.DS_Store
Thumbs.db
'@ | Set-Content -Path ".gitignore" -Encoding UTF8

# 3. Root Cargo.toml Workspace
@'
[workspace]
resolver = "2"
members = [
    "crates/ooxml-opc",
    "crates/word-engine",
    "crates/excel-engine",
    "crates/ppt-engine",
    "crates/render-core"
]

[workspace.dependencies]
quick-xml = { version = "0.37", features = ["serialize", "async-tokio"] }
zip = { version = "2.2", default-features = false, features = ["deflate"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rayon = "1.10"
thiserror = "2.0"
'@ | Set-Content -Path "Cargo.toml" -Encoding UTF8

# 4. Initialize Rust Engine Crates with Strict Safety Headers
$crates = @("ooxml-opc", "word-engine", "excel-engine", "ppt-engine", "render-core")
foreach ($c in $crates) {
@"
[package]
name = "$c"
version = "0.1.0"
edition = "2021"

[dependencies]
quick-xml.workspace = true
zip.workspace = true
serde.workspace = true
serde_json.workspace = true
rayon.workspace = true
thiserror.workspace = true
"@ | Set-Content -Path "crates/$c/Cargo.toml" -Encoding UTF8

    $forbidUnsafe = if ($c -ne "render-core") { "#![forbid(unsafe_code)]`n" } else { "" }
@"
${forbidUnsafe}//! $c - ECMA-376 / Clean-Room Engine Module

pub fn health_check() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check() {
        assert!(health_check());
    }
}
"@ | Set-Content -Path "crates/$c/src/lib.rs" -Encoding UTF8
}

# 5. Global Engineering Contract (.agents/rules/00-global-engineering-contract.md)
@'
---
trigger: always_on
description: Global Engineering Contract (GEC-v1.0) governing clean-room reverse engineering, ECMA-376 fidelity, performance SLAs, cross-platform adaptive UI, and zero-defect verification.
---

# GLOBAL ENGINEERING CONTRACT (GEC-v1.0)
**Project:** OpenOfficeRS (Word, Excel, PowerPoint Unified Rust Engine & Shell)
**Status:** MANDATORY / NON-NEGOTIABLE FOR ALL ANTIGRAVITY AGENTS

## ARTICLE I: LEGAL CLEAN-ROOM & BINARY ISOLATION WALL
1. **Open Format First:** All `.docx`, `.xlsx`, and `.pptx` structural parsing and serialization MUST follow the public ECMA-376 / ISO/IEC 29500 standard.
2. **Strict Two-Phase Separation (When REA is Active):**
   - **Analysis Phase (`@rea-binary-analyst`):** May inspect staged binaries ONLY to document mathematical equations, state transitions, and input/output test vectors inside `specs/evidence/<app>/`.
   - **Implementation Phase (`@ecma376-architect`, `@adaptive-ui-builder`):** Writes clean, idiomatic Rust/TypeScript in `crates/` and `apps/` reading ONLY from ECMA-376 specs and `specs/evidence/`.
3. **Zero Contamination:** Never paste decompiled C/C++, x86_64 assembly, or Microsoft internal symbols into `crates/` or `apps/`.

## ARTICLE II: ZERO-LOSS ECMA-376 ROUND-TRIP FIDELITY
1. Opening and saving any `.docx`, `.xlsx`, or `.pptx` file MUST NEVER strip or corrupt unmodeled XML elements, attributes (`mc:Ignorable`), macros (`vbaProject.bin`), or OPC `.rels` relationships.
2. Every parser must retain unrecognized nodes/parts in an `UnparsedPartStore` and write them back deterministically on save.

## ARTICLE III: HARD PERFORMANCE & RESOURCE SLAs
1. **Startup:** `< 150 ms` on Desktop; `< 300 ms` on Mobile.
2. **Memory:** Idle `< 35 MB` RSS; 1M populated Excel cells `< 180 MB` RSS via Compressed Sparse Page Grid (CSPG).
3. **Latency:** Viewport scrolling/zoom locked at `120 fps` (`< 8.33 ms` frame budget); Word typing `< 1.0 ms` per keystroke via Piece-Tree Rope.
4. **Threading:** Never block the UI or GPU thread with file I/O, XML parsing, or formula evaluation.

## ARTICLE IV: RUST & TYPESCRIPT CODE QUALITY INVARIANTS
1. `#[forbid(unsafe_code)]` enforced in `ooxml-opc`, `word-engine`, `excel-engine`, and `ppt-engine`.
2. `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, and `unimplemented!()` are **BANNED** in non-test library code (`crates/*/src/**`). Return typed `Result<T, EngineError>` using `thiserror`.
3. Zero warnings allowed under `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check`.

## ARTICLE V: UNIVERSAL CROSS-PLATFORM & ADAPTIVE UI PARITY
1. All engine crates must compile cleanly for Windows, Linux, macOS, Android, and iOS.
2. `apps/unified-shell` must automatically adapt across:
   - **Desktop/Laptop (`>= 1024px`, fine pointer):** Multi-tab workspace, full Ribbon, formula bar, dockable panels, keyboard shortcuts.
   - **Tablet (`768px - 1023px`, touch/stylus):** Compact touch ribbon, stylus ink overlay, split-view support.
   - **Mobile (`< 768px`, touch):** Bottom command sheets, virtual-keyboard inset avoidance, Word Reflow Reader mode, Excel card editor.

## ARTICLE VI: DEFINITION OF DONE (DoD) GATEKEEPING
No subphase is complete until `tools/verify-contract.ps1` exits with code `0` and changes are committed with `[GEC-Verified]`.
'@ | Set-Content -Path ".agents/rules/00-global-engineering-contract.md" -Encoding UTF8

# 6. Core Antigravity Skills (ECMA-376 Architect & Adaptive UI Builder)
@'
---
name: ecma376-architect
description: Designs and implements zero-copy, streaming ECMA-376 (OOXML) parsers and serializers in Rust.
---
1. Follow ISO/IEC 29500-1 (Markup) and Part 2 (Open Packaging Conventions).
2. Use `quick-xml` streaming pull-parsers and `rayon` parallel iterators.
3. Store any unrecognized XML elements or OPC parts inside `UnparsedPartStore` for 100% lossless round-trip saving.
4. Write unit tests proving lossless round-trip serialization before marking any task complete.
'@ | Set-Content -Path ".agents/skills/ecma376-architect.md" -Encoding UTF8

@'
---
name: adaptive-ui-builder
description: Builds cross-platform responsive UI components in Tauri v2 that adapt across Desktop, Tablet, and Mobile.
---
1. Detect viewport width and pointer accuracy (`pointer: fine` vs `pointer: coarse`):
   - Desktop (`>= 1024px`): Full Ribbon toolbar, multi-window tabs, status bar, keyboard shortcuts.
   - Tablet (`768px - 1023px`): Collapsible touch ribbon, stylus ink canvas overlay.
   - Mobile (`< 768px`): Thumb-friendly bottom drawers, virtual-keyboard viewport avoidance, Word Reflow Reader, Excel Card Inspector.
2. Route all canvas rendering through `crates/render-core` at 120fps.
'@ | Set-Content -Path ".agents/skills/adaptive-ui-builder.md" -Encoding UTF8

# 7. Automated Subphase Execution Workflow (.agents/workflows/execute-subphase.md)
@'
---
description: Executes any Stage.Phase.Subphase prompt under strict Global Engineering Contract (GEC-v1.0) enforcement.
---
# Workflow: Execute Subphase (`/execute-subphase`)

Execute this deterministic 6-step loop for the requested subphase:
1. **Spec & Evidence:**
   - If binary behavior analysis is needed and `.agents/skills/rea-binary-analyst.md` is installed, invoke `@rea-binary-analyst` to write mathematical specs to `specs/evidence/<app>/<subphase_id>.md`.
   - Otherwise, document ECMA-376 schema rules in `specs/evidence/<app>/<subphase_id>.md` and generate JSON test vectors (`*_vectors.json`).
2. **Red Test Phase:** Write integration tests in `crates/<crate>/tests/` loading `specs/evidence/<app>/<subphase_id>_vectors.json`.
3. **Green Code Phase:** Implement clean-room Rust/UI code obeying GEC-v1.0 (zero `.unwrap()`/`.expect()`/`todo!()`, full `UnparsedPartStore` preservation).
4. **Contract Audit:** Run `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1` and fix any violations until it exits `0`.
5. **Cross-Target Check:** Run `cargo check --workspace --all-targets`.
6. **Atomic Commit:** Commit with `git commit -m "feat(subphase-<id>): <summary> [GEC-Verified]"`.
'@ | Set-Content -Path ".agents/workflows/execute-subphase.md" -Encoding UTF8

# 8. Automated Contract Enforcer (tools/verify-contract.ps1)
@'
$ErrorActionPreference = "Stop"
Write-Host "=== [GEC-v1.0] Running Automated Global Engineering Contract Audit ===" -ForegroundColor Cyan

Write-Host "[1/4] Scanning crates/*/src for banned .unwrap(), .expect(), todo!(), unimplemented!()..."
$libFiles = Get-ChildItem -Path "crates" -Recurse -Filter "*.rs" | Where-Object { $_.FullName -notmatch "\\tests\\" }
$violations = @()

foreach ($file in $libFiles) {
    $lines = Get-Content $file.FullName
    $inTestModule = $false
    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        if ($line -match "#\[cfg\(test\)\]") { $inTestModule = $true }
        if (-not $inTestModule) {
            if ($line -match "\.unwrap\(\)|\.expect\(|todo!\(|unimplemented!\(") {
                $violations += "$($file.FullName):$($i+1): $line"
            }
        }
    }
}

if ($violations.Count -gt 0) {
    Write-Host "CONTRACT VIOLATION (Article IV.2): Banned panic macros found in production code:" -ForegroundColor Red
    $violations | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    exit 1
}
Write-Host "  PASS: Zero panic macros in production paths." -ForegroundColor Green

Write-Host "[2/4] Verifying Rust formatting (cargo fmt)..."
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "[3/4] Running strict Clippy static analysis..."
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "[4/4] Running full workspace test suite..."
cargo test --workspace --all-targets
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "=== [GEC-v1.0] ALL CONTRACT GATES PASSED SUCCESSFULLY ===" -ForegroundColor Green
exit 0
'@ | Set-Content -Path "tools/verify-contract.ps1" -Encoding UTF8

# 9. Git Pre-Commit Hook (.githooks/pre-commit)
@'
#!/bin/sh
pwsh -NoProfile -ExecutionPolicy Bypass -File tools/verify-contract.ps1 || powershell -NoProfile -ExecutionPolicy Bypass -File tools/verify-contract.ps1
'@ | Set-Content -Path ".githooks/pre-commit" -Encoding UTF8
git config core.hooksPath .githooks

# 10. Python COM Ground-Truth Harness (tools/com-ground-truth/generate_truth.py)
@'
"""
generate_truth.py - Drives locally installed Microsoft Office via COM to generate
clean-room JSON input/output test vectors for OpenOfficeRS.
Requires: pip install pywin32
"""
import argparse
import json
import math
from pathlib import Path

def generate_excel_vectors(output_path: Path):
    import win32com.client as win32
    excel = win32.DispatchEx("Excel.Application")
    excel.Visible = False
    excel.DisplayAlerts = False
    wb = excel.Workbooks.Add()
    ws = wb.ActiveSheet

    test_formulas = [
        {"id": "ieee_01", "setup": {"A1": 1.333, "B1": 1.225}, "formula": "=A1-B1-0.108"},
        {"id": "ieee_02", "setup": {"A1": 1.333, "B1": 1.225}, "formula": "=(A1-B1-0.108)"},
        {"id": "fin_pmt_01", "setup": {}, "formula": "=PMT(0.08/12, 10, 10000)"},
        {"id": "stat_round_01", "setup": {}, "formula": "=ROUND(2.15, 1)"}
    ]

    vectors = []
    try:
        for case in test_formulas:
            ws.Cells.Clear()
            for cell_ref, val in case["setup"].items():
                ws.Range(cell_ref).Value = val
            ws.Range("Z1").Formula = case["formula"]
            excel.CalculateFullRebuild()
            res = ws.Range("Z1").Value
            vectors.append({
                "id": case["id"],
                "setup": case["setup"],
                "formula": case["formula"],
                "expected_value": res if not (isinstance(res, float) and math.isnan(res)) else "NaN"
            })
    finally:
        wb.Close(SaveChanges=False)
        excel.Quit()

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps({"app": "excel", "vectors": vectors}, indent=2), encoding="utf-8")
    print(f"[COM Ground Truth] Wrote {len(vectors)} vectors to {output_path}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", default="specs/evidence/excel/baseline_vectors.json")
    args = parser.parse_args()
    generate_excel_vectors(Path(args.out))
'@ | Set-Content -Path "tools/com-ground-truth/generate_truth.py" -Encoding UTF8

# 11. 5-Platform GitHub Actions CI/CD (.github/workflows/build-all-platforms.yml)
@'
name: Build & Upload All Platforms (Windows, Linux, macOS, Android, iOS)

on:
  push:
    branches: [ main ]
    tags: [ 'v*' ]
  pull_request:
    branches: [ main ]
  workflow_dispatch:

jobs:
  verify-engine:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Verify Formatting, Clippy & Workspace Parity Tests
        run: |
          cargo fmt --all -- --check
          cargo clippy --workspace --all-targets -- -D warnings
          cargo test --workspace --all-targets

  build-desktop:
    needs: verify-engine
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: windows-latest
            artifact_name: OpenOfficeRS-Windows-x64
            bundle_path: apps/unified-shell/src-tauri/target/release/bundle/msi/*.msi
          - platform: ubuntu-22.04
            artifact_name: OpenOfficeRS-Linux-x64
            bundle_path: apps/unified-shell/src-tauri/target/release/bundle/appimage/*.AppImage
          - platform: macos-14
            artifact_name: OpenOfficeRS-macOS-Universal
            bundle_path: apps/unified-shell/src-tauri/target/release/bundle/dmg/*.dmg
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '22'
      - uses: dtolnay/rust-toolchain@stable
      - name: Install Linux Dependencies
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
      - name: Build Tauri Desktop App
        if: hashFiles('apps/unified-shell/package.json') != ''
        working-directory: apps/unified-shell
        run: |
          npm install
          npx tauri build
      - name: Upload Desktop Artifact
        if: hashFiles('apps/unified-shell/package.json') != ''
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact_name }}
          path: ${{ matrix.bundle_path }}

  build-android:
    needs: verify-engine
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '22'
      - uses: actions/setup-java@v4
        with:
          distribution: 'temurin'
          java-version: '17'
      - uses: android-actions/setup-android@v3
      - name: Install Android NDK & Rust Targets
        run: |
          sdkmanager --install "ndk;27.0.12077973"
          rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
      - name: Build Tauri Android APK
        if: hashFiles('apps/unified-shell/package.json') != ''
        working-directory: apps/unified-shell
        env:
          NDK_HOME: ${{ env.ANDROID_HOME }}/ndk/27.0.12077973
        run: |
          npm install
          npx tauri android init
          npx tauri android build --apk
      - name: Upload Android APK Artifact
        if: hashFiles('apps/unified-shell/package.json') != ''
        uses: actions/upload-artifact@v4
        with:
          name: OpenOfficeRS-Android-Universal-APK
          path: apps/unified-shell/src-tauri/gen/android/app/build/outputs/apk/universal/release/*.apk

  build-ios:
    needs: verify-engine
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '22'
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-ios,aarch64-apple-ios-sim
      - name: Build Tauri iOS App & Package Unsigned IPA
        if: hashFiles('apps/unified-shell/package.json') != ''
        working-directory: apps/unified-shell
        run: |
          npm install
          npx tauri ios init
          npx tauri ios build --export-method debugging || true
          mkdir -p Payload
          cp -r src-tauri/gen/apple/build/arm64/OpenOfficeRS.app Payload/ || true
          zip -r OpenOfficeRS-iOS.ipa Payload
      - name: Upload iOS IPA Artifact
        if: hashFiles('apps/unified-shell/package.json') != ''
        uses: actions/upload-artifact@v4
        with:
          name: OpenOfficeRS-iOS-IPA
          path: apps/unified-shell/OpenOfficeRS-iOS.ipa
'@ | Set-Content -Path ".github/workflows/build-all-platforms.yml" -Encoding UTF8

Write-Host "=== [OpenOfficeRS] Master Bootstrap Complete! ===" -ForegroundColor Green
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Run .\tools\verify-contract.ps1 to verify initial workspace health."
Write-Host "  2. Run .\rea_bootstrap.ps1 whenever you need REA binary analysis in Antigravity IDE."
Write-Host "  3. Run .\removeRea_bootstrap.ps1 after extracting specs to purge REA & staged binaries."