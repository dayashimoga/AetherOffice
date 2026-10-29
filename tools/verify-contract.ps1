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
cargo clippy --workspace --all-targets --jobs 2 -- -D warnings
if ($LASTEXITCODE -ne 0) { exit 1 }

# Gate 3: Rust Workspace Unit Tests
Write-Host "[3/6] Running Rust workspace unit tests..."
cargo test --workspace --all-targets --jobs 2
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
