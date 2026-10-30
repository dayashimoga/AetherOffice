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
            Write-Host "VIOLATION in $($file.FullName):$($i+1): $($lines[$i])" -ForegroundColor Red
            exit 1
        }
    }
}
Write-Host "  PASS: Rust safety verified." -ForegroundColor Green

# Gate 2: Auditing Frontend for hardcoded '22 words' bug & fake text
if (Test-Path "apps/unified-shell/src") {
    Write-Host "[2/5] Auditing Frontend for hardcoded '22 words' bug & fake text..."
    $uiFiles = Get-ChildItem -Path "apps/unified-shell/src" -Recurse -Include "*.ts","*.tsx" | Where-Object { $_.FullName -notmatch "__tests__" -and $_.FullName -notmatch "\.test\." }
    $bannedPatterns = @(
        "Executive Technical Summary",
        "Performance SLAs & Benchmark Compliance",
        "Live Demo & Architecture Validation",
        "commands for PowerPoint",
        "commands for Excel",
        "22 words"
    )
    foreach ($uf in $uiFiles) {
        $raw = Get-Content $uf.FullName -Raw
        foreach ($pat in $bannedPatterns) {
            if ($raw -match [regex]::Escape($pat)) {
                Write-Host "CONTRACT VIOLATION (Article II.4): Banned/fake pattern '$pat' found in $($uf.FullName)!" -ForegroundColor Red
                exit 1
            }
        }
    }
    Write-Host "  PASS: Zero hardcoded status bar or fake strings found." -ForegroundColor Green
}

# Gate 3: Rust Format & Clippy
Write-Host "[3/5] Running cargo fmt & clippy..."
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit 1 }
cargo clippy --workspace --all-targets --jobs 2 -- -D warnings
if ($LASTEXITCODE -ne 0) { exit 1 }

# Gate 4: Rust Workspace Tests
Write-Host "[4/5] Running Rust workspace unit tests..."
cargo test --workspace --all-targets --jobs 2
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
