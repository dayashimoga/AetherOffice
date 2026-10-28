# ==============================================================================
# tools/stage-runner.ps1 - Interactive Stage & Subphase Dispatcher
# Usage:
#   .\tools\stage-runner.ps1 -Status          # Show progress table across all 5 stages
#   .\tools\stage-runner.ps1 -Next            # Print & copy next PENDING subphase prompt to clipboard
#   .\tools\stage-runner.ps1 -Stage 2         # Print all prompts for Stage 2
#   .\tools\stage-runner.ps1 -Complete 1.1.1  # Run GEC-v1.0 audit & mark subphase 1.1.1 COMPLETED
# ==============================================================================
param(
    [switch]$Status,
    [switch]$Next,
    [int]$Stage = 0,
    [string]$Complete = ""
)
$ErrorActionPreference = "Stop"
$trackerPath = "specs/stage-tracker.json"

if (-not (Test-Path $trackerPath)) {
    throw "specs/stage-tracker.json not found. Run .\stagesprompt.ps1 first."
}

$tracker = Get-Content $trackerPath -Raw | ConvertFrom-Json

if ($Complete -ne "") {
    $item = $tracker.subphases | Where-Object { $_.id -eq $Complete }
    if (-not $item) { throw "Subphase ID '$Complete' not found in tracker." }

    Write-Host "=== Verifying Global Engineering Contract (GEC-v1.0) before completing Subphase $Complete ===" -ForegroundColor Cyan
    powershell -ExecutionPolicy Bypass -File "tools/verify-contract.ps1"
    if ($LASTEXITCODE -ne 0) {
        throw "Contract verification failed! Cannot mark Subphase $Complete as COMPLETED."
    }

    $item.status = "COMPLETED"
    $item.completed_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    $tracker | ConvertTo-Json -Depth 5 | Set-Content -Path $trackerPath -Encoding UTF8

    $doneCount = @($tracker.subphases | Where-Object { $_.status -eq "COMPLETED" }).Count
    $pct = [math]::Round(($doneCount / $tracker.total_subphases) * 100, 1)
    Write-Host "=== Subphase $Complete ($($item.title)) marked COMPLETED! Overall Progress: $doneCount/$($tracker.total_subphases) ($pct%) ===" -ForegroundColor Green
    exit 0
}

if ($Stage -gt 0) {
    $items = $tracker.subphases | Where-Object { $_.stage -eq $Stage }
    Write-Host "=== Stage $Stage Subphases ===" -ForegroundColor Cyan
    foreach ($i in $items) {
        Write-Host ("[{0}] {1} - {2} ({3})" -f $i.status, $i.id, $i.title, $i.prompt_file)
    }
    Write-Host "`nTip: In Antigravity IDE chat, type /stage-$Stage to execute this entire stage." -ForegroundColor Yellow
    exit 0
}

if ($Next) {
    $nextItem = $tracker.subphases | Where-Object { $_.status -ne "COMPLETED" } | Select-Object -First 1
    if (-not $nextItem) {
        Write-Host "ALL 22 SUBPHASES ACROSS ALL 5 STAGES ARE COMPLETED!" -ForegroundColor Green
        exit 0
    }

    Write-Host "=== NEXT SUBPHASE TO EXECUTE: $($nextItem.id) - $($nextItem.title) ===" -ForegroundColor Cyan
    if ($nextItem.rea_required) {
        Write-Host "[!] Notice: This subphase uses REA binary analysis. Run .\rea_bootstrap.ps1 if not active." -ForegroundColor Yellow
    }

    $promptContent = Get-Content $nextItem.prompt_file -Raw
    Write-Host "`n$promptContent" -ForegroundColor Gray

    if (Get-Command "Set-Clipboard" -ErrorAction SilentlyContinue) {
        try {
            Set-Clipboard -Value $promptContent
            Write-Host ">>> Prompt automatically copied to your clipboard! Paste it into Antigravity Agent Manager or type `/run-next-subphase`." -ForegroundColor Green
        } catch {
            Write-Host ">>> (Note: Clipboard access unavailable in this session. Copy the prompt above manually.)" -ForegroundColor Yellow
        }
    }
    exit 0
}

# Default or -Status: Display Full Progress Dashboard
$doneCount = @($tracker.subphases | Where-Object { $_.status -eq "COMPLETED" }).Count
$pct = [math]::Round(($doneCount / $tracker.total_subphases) * 100, 1)
Write-Host "=====================================================================================" -ForegroundColor Cyan
Write-Host " OpenOfficeRS Multi-Stage Execution Tracker | Progress: $doneCount / $($tracker.total_subphases) ($pct%)" -ForegroundColor Cyan
Write-Host "=====================================================================================" -ForegroundColor Cyan

foreach ($sp in $tracker.subphases) {
    $color = switch ($sp.status) {
        "COMPLETED" { "Green" }
        "IN_PROGRESS" { "Yellow" }
        default { "Gray" }
    }
    $reaTag = if ($sp.rea_required) { "[REA]" } else { "[OOXML]" }
    Write-Host (" Stage {0} | {1,-6} | {2,-11} | {3,-7} | {4}" -f $sp.stage, $sp.id, $sp.status, $reaTag, $sp.title) -ForegroundColor $color
}
Write-Host "-------------------------------------------------------------------------------------"
Write-Host "Commands:" -ForegroundColor Yellow
Write-Host "  .\tools\stage-runner.ps1 -Next             # Copy next subphase prompt to clipboard"
Write-Host "  .\tools\stage-runner.ps1 -Complete <ID>    # Verify GEC-v1.0 & mark subphase done"
Write-Host "  Antigravity Slash Commands: /run-next-subphase or /stage-1 .. /stage-5"
