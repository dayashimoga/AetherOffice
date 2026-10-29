param(
    [switch]$KeepGlobalNpmPackage = $false
)

Write-Host "=== [Clean-Room Isolation Audit] Purging REA & Staged Binaries ===" -ForegroundColor Cyan

# 1. Check for any staged binaries or proprietary DLLs
$staged = Get-ChildItem -Path . -Recurse -Include "*.dll","*.so","*.dylib" -ErrorAction SilentlyContinue | Where-Object { $_.FullName -notmatch "\\target\\" -and $_.FullName -notmatch "\\\.git\\" }
if ($staged) {
    Write-Host "Found staged binaries to purge:" -ForegroundColor Yellow
    $staged | ForEach-Object {
        Write-Host "  Removing: $($_.FullName)"
        Remove-Item $_.FullName -Force
    }
} else {
    Write-Host "  PASS: Zero staged binaries or proprietary DLLs found." -ForegroundColor Green
}

# 2. Check for temporary files
$temps = Get-ChildItem -Path . -Recurse -Include "*.tmp","*.bak","*~" -ErrorAction SilentlyContinue | Where-Object { $_.FullName -notmatch "\\target\\" -and $_.FullName -notmatch "\\\.git\\" }
if ($temps) {
    $temps | ForEach-Object { Remove-Item $_.FullName -Force }
}

Write-Host "=== [Clean-Room Isolation Audit] REA Clean Room Locked & Pure! ===" -ForegroundColor Green
exit 0
