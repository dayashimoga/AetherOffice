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
