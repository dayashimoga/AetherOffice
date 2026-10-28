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
