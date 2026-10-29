# ==============================================================================
# bootstrap_5_binaries.ps1 - Configures 5 Native Executables in Tauri
# Creates: aether-hub, aether-writer, aether-calc, aether-slides, aether-pdf
# ==============================================================================
$ErrorActionPreference = "Stop"
Write-Host "=== [Remediation 2/3] Configuring 5 Standalone Binaries + Main Hub ===" -ForegroundColor Cyan

$tauriBinDir = "apps/unified-shell/src-tauri/src/bin"
New-Item -ItemType Directory -Force -Path $tauriBinDir | Out-Null

# Create dedicated entrypoint wrappers that set AETHER_APP_MODE before launching Tauri
$modes = @(
    @{ bin = "aether-hub";    mode = "hub";    title = "AetherOffice Hub" },
    @{ bin = "aether-writer"; mode = "writer"; title = "Aether Writer (Word)" },
    @{ bin = "aether-calc";   mode = "calc";   title = "Aether Calc (Excel)" },
    @{ bin = "aether-slides"; mode = "slides"; title = "Aether Slides (PowerPoint)" },
    @{ bin = "aether-pdf";    mode = "pdf";    title = "Aether PDF Studio" }
)

foreach ($m in $modes) {
@"
// Standalone Binary Entrypoint: $($m.title)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    std::env::set_var("AETHER_APP_MODE", "$($m.mode)");
    std::env::set_var("AETHER_APP_TITLE", "$($m.title)");
    unified_shell_lib::run();
}
"@ | Set-Content -Path "$tauriBinDir/$($m.bin).rs" -Encoding UTF8
    Write-Host "  Created binary entrypoint: $tauriBinDir/$($m.bin).rs" -ForegroundColor Gray
}

Write-Host "5 Standalone binary entrypoints generated in $tauriBinDir." -ForegroundColor Green