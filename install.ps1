# RTB (ﺐﺘّﺭ) Installer & Integrator Script

$ErrorActionPreference = 'Stop'

Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  Installing & Configuring RTB (ﺐﺘّﺭ) Project Suite" -ForegroundColor Cyan
Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Cyan

$scriptRoot = $PSScriptRoot
if (-not $scriptRoot) { $scriptRoot = 'D:\02-Projects\01-Development\01-Active\dev-tools' }

$cliPsdPath = Join-Path $scriptRoot "cli\rtb.psd1"
$tuiDir     = Join-Path $scriptRoot "tui"
$scriptsDir = "D:\06-Tools\scripts"

# 1. Ensure scripts directory & legacy config compatibility paths exist
if (-not (Test-Path $scriptsDir)) {
    New-Item -ItemType Directory -Path $scriptsDir -Force | Out-Null
    Write-Host "Created tools scripts folder: $scriptsDir" -ForegroundColor Gray
}

$legacyConfigDir = 'D:\02-Projects\01-Development\01-Active\dev-cli\config'
if (-not (Test-Path $legacyConfigDir)) {
    New-Item -ItemType Directory -Path $legacyConfigDir -Force | Out-Null
}
$sourceConfig = Join-Path $scriptRoot 'config\rtb.config.json'
if (Test-Path $sourceConfig) {
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

# 3. Configure PowerShell Profile
if (-not (Test-Path $PROFILE)) {
    New-Item -ItemType File -Path $PROFILE -Force | Out-Null
    Write-Host "Created PowerShell profile at $PROFILE" -ForegroundColor Gray
}

$profileContent = Get-Content $PROFILE -Raw -ErrorAction SilentlyContinue
if ($null -eq $profileContent) { $profileContent = "" }

$moduleImportLine = "Import-Module '$cliPsdPath' -DisableNameChecking -Force"
$oldModuleLine1   = "Import-Module 'D:\02-Projects\01-Development\01-Active\dev-tools\cli\dev.psd1' -Force"
$oldModuleLine2   = "Import-Module 'D:\02-Projects\01-Development\01-Active\dev-cli\dev.psd1' -Force"
$oldModuleLine3   = "Import-Module 'D:\02-Projects\01-Development\01-Active\dev-tools\cli\rtb.psd1' -Force"

if ($profileContent -match [regex]::Escape($oldModuleLine3)) {
    $profileContent = $profileContent.Replace($oldModuleLine3, $moduleImportLine)
    Set-Content -Path $PROFILE -Value $profileContent -Encoding UTF8
    Write-Host "Updated profile entry to include -DisableNameChecking." -ForegroundColor Green
} elseif ($profileContent -match [regex]::Escape($oldModuleLine1)) {
    $profileContent = $profileContent.Replace($oldModuleLine1, $moduleImportLine)
    Set-Content -Path $PROFILE -Value $profileContent -Encoding UTF8
    Write-Host "Updated profile entry to rtb.psd1." -ForegroundColor Green
} elseif ($profileContent -match [regex]::Escape($oldModuleLine2)) {
    $profileContent = $profileContent.Replace($oldModuleLine2, $moduleImportLine)
    Set-Content -Path $PROFILE -Value $profileContent -Encoding UTF8
    Write-Host "Updated profile entry to rtb.psd1." -ForegroundColor Green
} elseif (-not ($profileContent -match [regex]::Escape($moduleImportLine))) {
    Add-Content -Path $PROFILE -Value "`n# RTB (رتّب) CLI Module`n$moduleImportLine" -Encoding UTF8
    Write-Host "Added RTB module import to PowerShell profile ($PROFILE)." -ForegroundColor Green
} else {
    Write-Host "PowerShell profile already contains RTB module import." -ForegroundColor Gray
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

