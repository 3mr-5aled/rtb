# RTB (رتّب) Automated Uninstaller Script

[CmdletBinding()]
param(
    [switch]$Force,
    [switch]$KeepConfig
)

$ErrorActionPreference = 'Stop'

Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Yellow
Write-Host "  Uninstalling RTB (رتّب) Developer Project Operations Suite" -ForegroundColor Yellow
Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Yellow

if (-not $Force) {
    $confirm = Read-Host "Are you sure you want to uninstall RTB from your system? (y/N)"
    if ($confirm -notmatch '^(y|yes)$') {
        Write-Host "Uninstallation canceled." -ForegroundColor Gray
        return
    }
}

# 1. Unload module from current PowerShell session
if (Get-Module -Name rtb -ErrorAction SilentlyContinue) {
    Remove-Module rtb -ErrorAction SilentlyContinue
    Write-Host "Unloaded 'rtb' module from active PowerShell session." -ForegroundColor Green
}

# 2. Profile cleanup prompt
$docsDir = [Environment]::GetFolderPath('MyDocuments')
$profilePaths = @(
    $PROFILE,
    (Join-Path $docsDir "WindowsPowerShell\Microsoft.PowerShell_profile.ps1"),
    (Join-Path $docsDir "PowerShell\Microsoft.PowerShell_profile.ps1")
) | Select-Object -Unique

$shouldCleanProfile = $true
if (-not $Force) {
    Write-Host ""
    $profileAns = Read-Host "Remove RTB autoload from your PowerShell profile(s)? (y/N)"
    $shouldCleanProfile = ($profileAns -match '^(y|yes)$')
}

if ($shouldCleanProfile) {
    foreach ($pPath in $profilePaths) {
        if ($pPath -and (Test-Path $pPath)) {
            $pLines = Get-Content $pPath -ErrorAction SilentlyContinue
            if ($pLines) {
                $cleanedLines = @($pLines | Where-Object {
                    $_ -notmatch 'Import-Module\s+.*?[''"].*?(rtb|dev-tools|dev-cli|rtb-command-tool).*?\.psd1[''"]' -and
                    $_ -notmatch '#\s*RTB.*?Module'
                })
                $cleanedLines -join "`r`n" | Set-Content -Path $pPath -Encoding UTF8
                Write-Host "Removed RTB autoload entry from profile: $pPath" -ForegroundColor Green
            }
        }
    }
} else {
    Write-Host "`n  ⚠️  The Import-Module line was kept in your PowerShell profile(s)." -ForegroundColor Yellow
    Write-Host "     This will cause an error on every new shell until you remove it manually." -ForegroundColor Yellow
    Write-Host "     To remove it manually, edit your profile:" -ForegroundColor Gray
    Write-Host "       notepad `$PROFILE" -ForegroundColor White
    Write-Host "     And delete the line containing 'rtb.psd1'." -ForegroundColor Gray
}

# 3. Clean up installed binaries, module files, and config
$userHomeDir   = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
$userConfigDir = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb' } else { Join-Path $userHomeDir '.config/rtb' }
$moduleHome    = Join-Path $userConfigDir 'module'
$scriptsDir    = if ($env:RTB_BIN_DIR) {
    $env:RTB_BIN_DIR
} elseif ($env:APPDATA) {
    Join-Path $env:APPDATA 'rtb\bin'
} else {
    Join-Path $userHomeDir '.config\rtb\bin'
}

if (Test-Path $moduleHome) {
    Remove-Item -Recurse -Force $moduleHome -ErrorAction SilentlyContinue
    Write-Host "Removed RTB module directory: $moduleHome" -ForegroundColor Green
}

if (Test-Path $scriptsDir) {
    Remove-Item -Recurse -Force $scriptsDir -ErrorAction SilentlyContinue
    Write-Host "Removed RTB binaries directory: $scriptsDir" -ForegroundColor Green
}

if (-not $KeepConfig) {
    if (Test-Path $userConfigDir) {
        Remove-Item -Recurse -Force $userConfigDir -ErrorAction SilentlyContinue
        Write-Host "Removed user configuration directory: $userConfigDir" -ForegroundColor Green
    }
} else {
    Write-Host "Preserved user configuration directory at: $userConfigDir" -ForegroundColor Cyan
}

Write-Host "`nUninstallation Complete! RTB has been removed from your system." -ForegroundColor Green
if ($KeepConfig) {
    Write-Host "Note: Your user settings in $userConfigDir were preserved." -ForegroundColor Gray
}
