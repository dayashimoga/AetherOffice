# bootstrap_core.ps1 - Initializes Cargo Workspace & Engine Crates
$ErrorActionPreference = "Stop"
Write-Host "=== [1/4] Bootstrapping Core Workspace & Engine Crates == = " -ForegroundColor Cyan

$dirs = @(
    "crates/ooxml-opc/src",
    "crates/word-engine/src",
    "crates/excel-engine/src",
    "crates/ppt-engine/src",
    "crates/render-core/src",
    "specs/evidence/word",
    "specs/evidence/excel",
    "specs/evidence/ppt",
    "tools/com-ground-truth"
)
foreach ($d in $dirs) { New-Item -ItemType Directory -Force -Path $d | Out-Null }

if (-not (Test-Path ".git")) { git init | Out-Null }

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

pub fn health_check() -> bool { true }
"@ | Set-Content -Path "crates/$c/src/lib.rs" -Encoding UTF8
}
Write-Host "Core workspace scaffolded successfully." -ForegroundColor Green