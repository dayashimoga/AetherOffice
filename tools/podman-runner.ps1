# ==============================================================================
# tools/podman-runner.ps1 - Automatic Podman Container Fallback Runner
# If any required CLI tool is not installed locally on the host, runs it inside
# an appropriate Podman container with the workspace mounted at /workspace.
# ==============================================================================
[CmdletBinding(PositionalBinding=$false)]
param(
    [Parameter(Mandatory=$true, Position=0)]
    [string]$Tool,

    [Parameter(Position=1, ValueFromRemainingArguments=$true)]
    [string[]]$ToolArgs = @(),

    [Parameter(Mandatory=$false)]
    [string]$Image = "",

    [Parameter(Mandatory=$false)]
    [switch]$ForceContainer
)

$ErrorActionPreference = "Stop"

# Check if tool is available locally and container is not forced
if (-not $ForceContainer -and (Get-Command $Tool -ErrorAction SilentlyContinue)) {
    & $Tool @ToolArgs
    exit $LASTEXITCODE
}

# Ensure Podman is available
if (-not (Get-Command "podman" -ErrorAction SilentlyContinue)) {
    throw "Neither '$Tool' is installed locally nor is 'podman' available."
}

Write-Host "[Podman Fallback] '$Tool' not found locally (or forced). Running inside Podman container..." -ForegroundColor Yellow

$pwdWindows = (Get-Location).Path
$pwdMount = "${pwdWindows}:/workspace"

# Map tool to appropriate container and execution command
switch ($Tool.ToLower()) {
    "gh" {
        $img = if ($Image -ne "") { $Image } else { "alpine:latest" }
        $envArgs = @()
        if ($env:GITHUB_TOKEN) { $envArgs += @("-e", "GITHUB_TOKEN=$($env:GITHUB_TOKEN)") }
        if ($env:GH_TOKEN) { $envArgs += @("-e", "GH_TOKEN=$($env:GH_TOKEN)") }

        $argsString = ($ToolArgs | ForEach-Object { "'$_'" }) -join " "
        podman run --rm -v $pwdMount -w /workspace @envArgs $img sh -c "apk add --no-cache github-cli git >/dev/null 2>&1 && gh $argsString"
    }
    { $_ -in "node", "npm", "npx" } {
        $img = if ($Image -ne "") { $Image } else { "node:22-alpine" }
        podman run --rm -v $pwdMount -w /workspace $img $Tool @ToolArgs
    }
    { $_ -in "python", "python3", "pip" } {
        $img = if ($Image -ne "") { $Image } else { "python:3.11-slim" }
        podman run --rm -v $pwdMount -w /workspace $img $Tool @ToolArgs
    }
    { $_ -in "cargo", "rustc" } {
        $img = if ($Image -ne "") { $Image } else { "rust:latest" }
        podman run --rm -v $pwdMount -w /workspace $img $Tool @ToolArgs
    }
    default {
        if ($Image -eq "") {
            throw "No default Podman image configured for tool '$Tool'. Pass -Image <image_name>."
        }
        podman run --rm -v $pwdMount -w /workspace $Image $Tool @ToolArgs
    }
}

exit $LASTEXITCODE
