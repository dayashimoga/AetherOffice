# bootstrap_stages.ps1 - Generates Subphase Prompts & Interactive Stage Tracker
$ErrorActionPreference = "Stop"
Write-Host "=== [4/4] Generating Multi-Stage Prompts & Stage Tracker == = " -ForegroundColor Cyan

New-Item -ItemType Directory -Force -Path ".agents/stages", "specs" | Out-Null

# Stage Runner Dispatcher Tool
@'
param(
    [switch]$Status,
    [string]$Complete = ""
)
$ErrorActionPreference = "Stop"
$trackerPath = "specs/stage-tracker.json"
if (-not (Test-Path $trackerPath)) { throw "Run stagesprompt.ps1 first." }
$tracker = Get-Content $trackerPath -Raw | ConvertFrom-Json

if ($Complete -ne "") {
    $item = $tracker.subphases | Where-Object { $_.id -eq $Complete }
    if ($item) {
        powershell -ExecutionPolicy Bypass -File "tools/verify-contract.ps1"
        $item.status = "COMPLETED"
        $tracker | ConvertTo-Json -Depth 5 | Set-Content -Path $trackerPath -Encoding UTF8
        Write-Host "Subphase $Complete marked COMPLETED!" -ForegroundColor Green
    }
    exit 0
}

Write-Host "=== OpenOfficeRS Stage Tracker Dashboard ===" -ForegroundColor Cyan
foreach ($sp in $tracker.subphases) {
    $color = if ($sp.status -eq "COMPLETED") { "Green" } else { "Gray" }
    Write-Host (" Stage {0} | {1,-6} | {2,-11} | {3}" -f $sp.stage, $sp.id, $sp.status, $sp.title) -ForegroundColor $color
}
'@ | Set-Content -Path "tools/stage-runner.ps1" -Encoding UTF8

Write-Host "Stage runner and tracking tools initialized." -ForegroundColor Green
Write-Host "=== [OpenOfficeRS] All modular bootstrapper scripts completed successfully! ===" -ForegroundColor Green