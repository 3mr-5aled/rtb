# RTB (ﺐﺘّﺭ) Automated Uninstaller Script

[CmdletBinding()]
param(
    [switch]$Force,
    [switch]$KeepConfig
)

$ErrorActionPreference = 'Stop'

Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Yellow
Write-Host "  Uninstalling RTB (ﺐﺘّﺭ) Developer Project Operations Suite" -ForegroundColor Yellow
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
$userConfigDir = Join-Path $userHomeDir '.config/rtb'
$moduleHome    = Join-Path $userConfigDir 'module'
$scriptsDir    = if ($env:RTB_BIN_DIR) {
    $env:RTB_BIN_DIR
} else {
    Join-Path $userConfigDir 'bin'
}

if (Test-Path $moduleHome) {
    Remove-Item -Recurse -Force $moduleHome -ErrorAction SilentlyContinue
    Write-Host "Removed RTB module directory: $moduleHome" -ForegroundColor Green
}

if (Test-Path $scriptsDir) {
    Remove-Item -Recurse -Force $scriptsDir -ErrorAction SilentlyContinue
    Write-Host "Removed RTB binaries directory: $scriptsDir" -ForegroundColor Green
}

# 4. Clean legacy AppData\Roaming\rtb if present
$legacyRoaming = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb' } else { $null }
if ($legacyRoaming -and (Test-Path $legacyRoaming)) {
    Remove-Item -Recurse -Force $legacyRoaming -ErrorAction SilentlyContinue
    Write-Host "Removed legacy RTB directory: $legacyRoaming" -ForegroundColor Green
}

# 4b. Clean standalone or PATH-discovered launchers (e.g. D:\bin, npm globals, etc.)
$foundCommands = @(Get-Command rtb, rtbtui -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)
foreach ($exe in $foundCommands) {
    if ($exe -and (Test-Path $exe) -and ($exe -notlike "*\rtb-command-tool\*")) {
        $parent = Split-Path $exe -Parent
        foreach ($comp in @('rtb', 'rtb.cmd', 'rtb.ps1', 'rtb.js', 'rtbtui', 'rtbtui.exe')) {
            $compPath = Join-Path $parent $comp
            if (Test-Path $compPath) {
                Remove-Item -Force $compPath -ErrorAction SilentlyContinue
                Write-Host "Removed RTB launcher: $compPath" -ForegroundColor Green
            }
        }
    }
}

if (Test-Path 'D:\bin') {
    foreach ($comp in @('rtb.cmd', 'rtb.ps1', 'rtb.js', 'rtbtui.exe')) {
        $p = Join-Path 'D:\bin' $comp
        if (Test-Path $p) {
            Remove-Item -Force $p -ErrorAction SilentlyContinue
            Write-Host "Removed RTB launcher: $p" -ForegroundColor Green
        }
    }
}

# 4c. Uninstall npm global packages if present
try {
    npm uninstall -g @3mr5aled/rtb @3mr-5aled/rtb 2>$null | Out-Null
} catch {}

# 5. Clean PATH environment variable
try {
    $curPath = [Environment]::GetEnvironmentVariable('PATH', 'User')
    if ($curPath) {
        $pathParts = @($curPath -split ';' | Where-Object {
            $_ -and
            $_ -ne $scriptsDir -and
            $_ -notmatch '(?i)[\\/]AppData[\\/]Roaming[\\/]rtb[\\/]bin' -and
            $_ -notmatch '(?i)\.config[\\/]rtb[\\/]bin'
        })
        [Environment]::SetEnvironmentVariable('PATH', ($pathParts -join ';'), 'User')
        Write-Host "Cleaned RTB from user PATH environment variable." -ForegroundColor Green
    }
} catch {}

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
