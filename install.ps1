# RTB (ﺐﺘّﺭ) Installer & Integrator Script

$ErrorActionPreference = 'Stop'

Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  Installing & Configuring RTB (ﺐﺘّﺭ) Project Suite" -ForegroundColor Cyan
Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Cyan

$scriptRoot = $PSScriptRoot
if (-not $scriptRoot) { $scriptRoot = (Get-Location).Path }

$cliPsdPath = Join-Path $scriptRoot "cli\rtb.psd1"
$tuiDir     = Join-Path $scriptRoot "tui"
$scriptsDir = if ($env:RTB_BIN_DIR) {
    $env:RTB_BIN_DIR
} elseif ($env:APPDATA) {
    Join-Path $env:APPDATA 'rtb\bin'
} else {
    Join-Path ([Environment]::GetFolderPath('UserProfile')) '.config\rtb\bin'
}

# 1. Ensure scripts directory & user config paths exist
if (-not (Test-Path $scriptsDir)) {
    New-Item -ItemType Directory -Path $scriptsDir -Force | Out-Null
    Write-Host "Created tools scripts folder: $scriptsDir" -ForegroundColor Gray
}

$userConfigDir = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb' } else { Join-Path $env:HOME '.config/rtb' }
if (-not (Test-Path $userConfigDir)) {
    New-Item -ItemType Directory -Path $userConfigDir -Force | Out-Null
}

$legacyConfigDir = Join-Path (Split-Path $scriptRoot -Parent) 'dev-cli\config'
if (-not (Test-Path $legacyConfigDir)) {
    New-Item -ItemType Directory -Path $legacyConfigDir -Force | Out-Null
}
$sourceConfig = Join-Path $scriptRoot 'config\rtb.config.json'
if (Test-Path $sourceConfig) {
    Copy-Item $sourceConfig (Join-Path $userConfigDir 'rtb.config.json') -Force -ErrorAction SilentlyContinue
    Copy-Item $sourceConfig (Join-Path $userConfigDir 'dev.config.json') -Force -ErrorAction SilentlyContinue
    Copy-Item $sourceConfig (Join-Path $legacyConfigDir 'dev.config.json') -Force -ErrorAction SilentlyContinue
    Copy-Item $sourceConfig (Join-Path $legacyConfigDir 'rtb.config.json') -Force -ErrorAction SilentlyContinue
    Copy-Item $sourceConfig (Join-Path $scriptRoot 'config\dev.config.json') -Force -ErrorAction SilentlyContinue
}

# 2. Build or verify rtbtui binary
$cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargoCmd) {
    Write-Host "Building rtbtui binary with Cargo..." -ForegroundColor Yellow
    Push-Location $tuiDir
    try {
        cargo build --release
        $builtBin = Join-Path $tuiDir "target\release\rtbtui.exe"
        if (-not (Test-Path $builtBin)) {
            $builtBin = Join-Path $tuiDir "target\release\devtui.exe"
        }
        if (Test-Path $builtBin) {
            Copy-Item $builtBin "$scriptsDir\rtbtui.exe" -Force
            Copy-Item $builtBin "$scriptsDir\devtui.exe" -Force -ErrorAction SilentlyContinue
            Write-Host "Updated rtbtui.exe binary in $scriptsDir" -ForegroundColor Green
        }
    } catch {
        Write-Host "Warning: Cargo build failed, retaining existing binary if present." -ForegroundColor Yellow
    } finally {
        Pop-Location
    }
} else {
    $existingTui = Join-Path $tuiDir "target\release\rtbtui.exe"
    if (-not (Test-Path $existingTui)) {
        $existingTui = Join-Path $tuiDir "target\release\devtui.exe"
    }
    if (Test-Path $existingTui) {
        Copy-Item $existingTui "$scriptsDir\rtbtui.exe" -Force
        Copy-Item $existingTui "$scriptsDir\devtui.exe" -Force -ErrorAction SilentlyContinue
        Write-Host "Copied prebuilt rtbtui.exe to $scriptsDir" -ForegroundColor Green
    } elseif (Test-Path "$scriptsDir\rtbtui.exe") {
        Write-Host "Found existing rtbtui.exe in $scriptsDir" -ForegroundColor Green
    } else {
        Write-Host "Warning: TUI binary not built yet. Run 'cargo build --release' inside tui/ when Rust is available." -ForegroundColor Yellow
    }
}

# Always deploy logo.txt next to the binary so rtbtui picks it up at runtime
$logoSrc = Join-Path $scriptRoot "logo.txt"
if (Test-Path $logoSrc) {
    Copy-Item $logoSrc "$scriptsDir\logo.txt" -Force
    Write-Host "Deployed logo.txt to $scriptsDir" -ForegroundColor Green
}

# 3. Configure PowerShell Profiles (both Windows PowerShell 5.1 & PowerShell 7)
$docsDir = [Environment]::GetFolderPath('MyDocuments')
$profilePaths = @(
    $PROFILE,
    (Join-Path $docsDir "WindowsPowerShell\Microsoft.PowerShell_profile.ps1"),
    (Join-Path $docsDir "PowerShell\Microsoft.PowerShell_profile.ps1")
) | Select-Object -Unique

$moduleImportLine = "Import-Module '$cliPsdPath' -DisableNameChecking -Force"
$oldPattern = "(?m)^Import-Module\s+['\`"].*?(dev-tools|dev-cli|rtb-command-tool)[\\/].*?['\`"].*$"

foreach ($pPath in $profilePaths) {
    if (-not $pPath) { continue }
    if (-not (Test-Path $pPath)) {
        $parentDir = Split-Path $pPath -Parent
        if (-not (Test-Path $parentDir)) { New-Item -ItemType Directory -Path $parentDir -Force | Out-Null }
        New-Item -ItemType File -Path $pPath -Force | Out-Null
    }

    $pContent = Get-Content $pPath -Raw -ErrorAction SilentlyContinue
    if ($null -eq $pContent) { $pContent = "" }

    if ($pContent -match $oldPattern) {
        $pContent = [regex]::Replace($pContent, $oldPattern, $moduleImportLine)
        Set-Content -Path $pPath -Value $pContent -Encoding UTF8
        Write-Host "Updated profile ($pPath) entry." -ForegroundColor Green
    } elseif (-not ($pContent.Contains($moduleImportLine))) {
        Add-Content -Path $pPath -Value "`n# RTB CLI Module`n$moduleImportLine" -Encoding UTF8
        Write-Host "Added RTB module import to PowerShell profile ($pPath)." -ForegroundColor Green
    } else {
        Write-Host "PowerShell profile ($pPath) already up to date." -ForegroundColor Gray
    }
}

# 4. Import module in current session
Import-Module $cliPsdPath -DisableNameChecking -Force
Write-Host "Successfully loaded 'rtb' CLI module into current session!" -ForegroundColor Cyan

Write-Host "`nInstallation Complete! Available commands:" -ForegroundColor Green
Write-Host "  rtb help           : View all CLI commands" -ForegroundColor White
Write-Host "  rtb init           : Generate user configuration" -ForegroundColor White
Write-Host "  rtb list           : List active & paused projects" -ForegroundColor White
Write-Host "  rtb goto <project> : Switch to project folder" -ForegroundColor White
Write-Host "  rtb ui             : Launch interactive TUI command center" -ForegroundColor White
Write-Host "  rtbtui             : Direct TUI command" -ForegroundColor White

