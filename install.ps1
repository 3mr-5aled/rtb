# RTB (رتّب) Dual-Mode Installer & Integrator Script

$ErrorActionPreference = 'Stop'

Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  Installing & Configuring RTB (رتّب) Project Suite" -ForegroundColor Cyan
Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Cyan

$scriptRoot = $PSScriptRoot
$isStandalone = (-not $scriptRoot) -or (-not (Test-Path (Join-Path $scriptRoot "cli\rtb.psd1")))

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

# 1. Ensure target directories exist
if (-not (Test-Path $scriptsDir)) {
    New-Item -ItemType Directory -Path $scriptsDir -Force | Out-Null
}
if (-not (Test-Path $userConfigDir)) {
    New-Item -ItemType Directory -Path $userConfigDir -Force | Out-Null
}
if (-not (Test-Path $moduleHome)) {
    New-Item -ItemType Directory -Path $moduleHome -Force | Out-Null
}

# 2. Deploy module files and binaries
if ($isStandalone) {
    Write-Host "Running in standalone mode: downloading release bundle from GitHub..." -ForegroundColor Yellow
    $zipUrl = 'https://github.com/3mr-5aled/rtb/releases/latest/download/rtb-cli.zip'
    $tempZip = Join-Path ([System.IO.Path]::GetTempPath()) "rtb-install-$(Get-Random).zip"
    $tempExtract = Join-Path ([System.IO.Path]::GetTempPath()) "rtb-install-$(Get-Random)"

    try {
        Invoke-WebRequest -Uri $zipUrl -OutFile $tempZip -UseBasicParsing -TimeoutSec 60 -ErrorAction Stop
        Expand-Archive -Path $tempZip -DestinationPath $tempExtract -Force

        $extractedCli = Join-Path $tempExtract 'cli'
        if (Test-Path $extractedCli) {
            Copy-Item -Path "$extractedCli\*" -Destination $moduleHome -Recurse -Force
        }

        $extractedTui = Join-Path $tempExtract 'rtbtui.exe'
        if (Test-Path $extractedTui) {
            Copy-Item -Path $extractedTui -Destination "$scriptsDir\rtbtui.exe" -Force
            Copy-Item -Path $extractedTui -Destination "$scriptsDir\devtui.exe" -Force -ErrorAction SilentlyContinue
        }

        $extractedLogo = Join-Path $tempExtract 'logo.txt'
        if (Test-Path $extractedLogo) {
            Copy-Item -Path $extractedLogo -Destination "$scriptsDir\logo.txt" -Force
        }

        $extractedUninstall = Join-Path $tempExtract 'uninstall.ps1'
        if (Test-Path $extractedUninstall) {
            Copy-Item -Path $extractedUninstall -Destination "$scriptsDir\uninstall.ps1" -Force
            Copy-Item -Path $extractedUninstall -Destination "$userConfigDir\uninstall.ps1" -Force
        }
        Write-Host "Extracted RTB components to $userConfigDir" -ForegroundColor Green
    } catch {
        Write-Host "Warning: Could not download release bundle: $($_.Exception.Message)" -ForegroundColor Yellow
        Write-Host "If running from source, please run from the repository root." -ForegroundColor Gray
    } finally {
        if (Test-Path $tempZip) { Remove-Item -Force $tempZip -ErrorAction SilentlyContinue }
        if (Test-Path $tempExtract) { Remove-Item -Recurse -Force $tempExtract -ErrorAction SilentlyContinue }
    }
} else {
    Write-Host "Running in repository mode: deploying local components..." -ForegroundColor Gray
    $sourceCli = Join-Path $scriptRoot "cli"
    if (Test-Path $sourceCli) {
        Copy-Item -Path "$sourceCli\*" -Destination $moduleHome -Recurse -Force
        Write-Host "Deployed CLI module to $moduleHome" -ForegroundColor Green
    }

    $tuiDir = Join-Path $scriptRoot "tui"
    $cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
    if ($cargoCmd -and (Test-Path (Join-Path $tuiDir 'Cargo.toml'))) {
        Write-Host "Building rtbtui binary with Cargo..." -ForegroundColor Yellow
        Push-Location $tuiDir
        try {
            cargo build --release
            $builtBin = Join-Path $tuiDir "target\release\rtbtui.exe"
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
        if (Test-Path $existingTui) {
            Copy-Item $existingTui "$scriptsDir\rtbtui.exe" -Force
            Write-Host "Copied prebuilt rtbtui.exe to $scriptsDir" -ForegroundColor Green
        }
    }

    $logoSrc = Join-Path $scriptRoot "logo.txt"
    if (Test-Path $logoSrc) {
        Copy-Item $logoSrc "$scriptsDir\logo.txt" -Force
    }

    $uninstallSrc = Join-Path $scriptRoot "uninstall.ps1"
    if (Test-Path $uninstallSrc) {
        Copy-Item $uninstallSrc "$scriptsDir\uninstall.ps1" -Force
        Copy-Item $uninstallSrc "$userConfigDir\uninstall.ps1" -Force
    }
}

# 3. Permanently configure User PATH (Registry) & current session PATH
$userPath = [Environment]::GetEnvironmentVariable('PATH', 'User')
if ($userPath) {
    $pathParts = $userPath -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    if ($pathParts -notcontains $scriptsDir) {
        $newPath = "$userPath;$scriptsDir"
        [Environment]::SetEnvironmentVariable('PATH', $newPath, 'User')
        Write-Host "Added RTB binary directory to User PATH." -ForegroundColor Green
    }
} else {
    [Environment]::SetEnvironmentVariable('PATH', $scriptsDir, 'User')
    Write-Host "Added RTB binary directory to User PATH." -ForegroundColor Green
}

if (($env:PATH -split ';') -notcontains $scriptsDir) {
    $env:PATH = "$scriptsDir;$env:PATH"
}

# 4. Configure PowerShell Profiles
$cliPsdPath = Join-Path $moduleHome "rtb.psd1"
$docsDir = [Environment]::GetFolderPath('MyDocuments')
$profilePaths = @(
    $PROFILE,
    (Join-Path $docsDir "WindowsPowerShell\Microsoft.PowerShell_profile.ps1"),
    (Join-Path $docsDir "PowerShell\Microsoft.PowerShell_profile.ps1")
) | Select-Object -Unique

$moduleImportLine = "Import-Module '$cliPsdPath' -DisableNameChecking -Force"
$oldPattern = "(?m)^.*(Import-Module.*?(dev-tools|dev-cli|rtb-command-tool|rtb[\\/]module[\\/]rtb\.psd1|rtb\.psd1)|# RTB CLI Module).*`r?`n?"

foreach ($pPath in $profilePaths) {
    if (-not $pPath) { continue }
    if (-not (Test-Path $pPath)) {
        $parentDir = Split-Path $pPath -Parent
        if (-not (Test-Path $parentDir)) { New-Item -ItemType Directory -Path $parentDir -Force | Out-Null }
        New-Item -ItemType File -Path $pPath -Force | Out-Null
    }

    $pContent = Get-Content $pPath -Raw -ErrorAction SilentlyContinue
    if ($null -eq $pContent) { $pContent = "" }

    if ($pContent -match "(dev-tools|dev-cli|rtb-command-tool|rtb[\\/]module[\\/]rtb\.psd1|rtb\.psd1)") {
        $pContent = [regex]::Replace($pContent, $oldPattern, "")
    }

    if (-not ($pContent.Contains($moduleImportLine))) {
        Add-Content -Path $pPath -Value "`n# RTB CLI Module`n$moduleImportLine" -Encoding UTF8
        Write-Host "Configured RTB module autoload in profile: $pPath" -ForegroundColor Green
    } else {
        Write-Host "Profile ($pPath) already configured." -ForegroundColor Gray
    }
}

# 5. Import module in current session
if (Test-Path $cliPsdPath) {
    Import-Module $cliPsdPath -DisableNameChecking -Force
    Write-Host "Successfully loaded 'rtb' CLI module into current session!" -ForegroundColor Cyan
}

Write-Host "`nInstallation Complete! Next steps:" -ForegroundColor Green
Write-Host "  1. Run 'rtb init' to configure your projects workspace." -ForegroundColor White
Write-Host "  2. Run 'rtb help' to explore available commands." -ForegroundColor White
Write-Host "  3. Run 'rtbtui' or 'rtb ui' to open the interactive terminal dashboard." -ForegroundColor White
