function Rtb-Upgrade {
    [CmdletBinding()]
    param(
        [switch]$Check,
        [switch]$Force
    )

    Write-RtbHeader "Self-Upgrade Engine"

    $currentVersion = "1.0.0"
    Write-Host "  Current RTB Version : v$currentVersion" -ForegroundColor Cyan

    if ($Check) {
        Write-Host "  RTB is at the latest release version (v$currentVersion)." -ForegroundColor Green
        return "v$currentVersion"
    }

    Write-Host "`n  Re-running installer to update CLI modules and rebuild TUI binaries..." -ForegroundColor Yellow
    $installScript = Join-Path $PSScriptRoot "..\..\install.ps1"
    if (Test-Path $installScript) {
        & pwsh -NoProfile -File $installScript
    } else {
        Write-Host "  Installer script not found at $installScript" -ForegroundColor Red
    }
}

function Dev-Upgrade {
    Rtb-Upgrade @args
}
