function Rtb-Upgrade {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0, ValueFromRemainingArguments = $true)]
        [string[]]$ArgsList,

        [switch]$Check,
        [switch]$Force
    )

    $allArgs = @()
    if ($ArgsList) { $allArgs += $ArgsList }
    $isCheck = $Check.IsPresent -or ($allArgs -contains '-Check') -or ($allArgs -contains '--check') -or ($allArgs -contains '-c')
    $isForce = $Force.IsPresent -or ($allArgs -contains '-Force') -or ($allArgs -contains '--force') -or ($allArgs -contains '-f')

    Write-RtbHeader "Self-Upgrade Engine"

    # 1. Resolve current installed version from rtb.psd1
    $currentVersion = '0.4.0'
    $psdCandidates = @(
        (Join-Path $PSScriptRoot '..\..\rtb.psd1'),
        (Join-Path $env:APPDATA 'rtb\module\rtb.psd1')
    )
    foreach ($psd in $psdCandidates) {
        if ($psd -and (Test-Path $psd)) {
            try {
                $manifest = Import-PowerShellDataFile -Path $psd -ErrorAction SilentlyContinue
                if ($manifest -and $manifest.ModuleVersion) {
                    $currentVersion = $manifest.ModuleVersion
                    break
                }
            } catch {}
        }
    }

    Write-Host "  Current RTB Version : v$currentVersion" -ForegroundColor Cyan

    # 2. Fetch latest release from GitHub API
    $apiUrl = 'https://api.github.com/repos/3mr-5aled/rtb/releases/latest'
    $latestTag = $null
    $hasUpdate = $false

    try {
        $headers = @{ 'User-Agent' = 'RTB-CLI-Upgrade-Check' }
        $release = Invoke-RestMethod -Uri $apiUrl -Headers $headers -TimeoutSec 10 -ErrorAction Stop
        if ($release -and $release.tag_name) {
            $latestTag = $release.tag_name.Trim()
            $cleanLatest = $latestTag -replace '^v', ''
            $cleanCurrent = $currentVersion -replace '^v', ''

            if ([System.Version]::TryParse($cleanLatest, [ref]$null) -and [System.Version]::TryParse($cleanCurrent, [ref]$null)) {
                $vLatest = [System.Version]$cleanLatest
                $vCurrent = [System.Version]$cleanCurrent
                if ($vLatest -gt $vCurrent) { $hasUpdate = $true }
            } elseif ($cleanLatest -ne $cleanCurrent) {
                $hasUpdate = $true
            }
        }
    } catch {
        Write-Host "  ⚠ Could not reach GitHub API to check for updates: $($_.Exception.Message)" -ForegroundColor DarkGray
    }

    if ($isCheck) {
        if ($latestTag) {
            if ($hasUpdate) {
                Write-Host "  ⚠ A newer version is available: v$currentVersion → $latestTag" -ForegroundColor Yellow
                Write-Host "    Run 'rtb upgrade' to update to the latest version." -ForegroundColor Gray
            } else {
                Write-Host "  ✓ RTB is up to date (v$currentVersion)." -ForegroundColor Green
            }
        } else {
            Write-Host "  RTB version: v$currentVersion" -ForegroundColor Gray
        }
        return "v$currentVersion"
    }

    # 3. Perform upgrade
    if (-not $hasUpdate -and -not $isForce -and $latestTag) {
        Write-Host "  ✓ RTB is already at the latest release version (v$currentVersion)." -ForegroundColor Green
        Write-Host "    Use 'rtb upgrade -Force' to reinstall the current version." -ForegroundColor Gray
        return
    }

    Write-Host "`n  Downloading latest release bundle from GitHub..." -ForegroundColor Yellow
    $zipUrl = 'https://github.com/3mr-5aled/rtb/releases/latest/download/rtb-cli.zip'
    $tempZip = Join-Path ([System.IO.Path]::GetTempPath()) "rtb-cli-$(Get-Random).zip"
    $tempExtract = Join-Path ([System.IO.Path]::GetTempPath()) "rtb-extract-$(Get-Random)"

    $moduleDir = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb\module' } else { Join-Path $env:HOME '.config/rtb/module' }
    $binDir    = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb\bin' } else { Join-Path $env:HOME '.config/rtb/bin' }

    # If repo installer exists locally, use local installer first
    $localInstaller = Join-Path $PSScriptRoot '..\..\..\install.ps1'
    if (Test-Path $localInstaller) {
        Write-Host "  Running local repository installer..." -ForegroundColor Gray
        & pwsh -NoProfile -File $localInstaller
        return
    }

    try {
        Invoke-WebRequest -Uri $zipUrl -OutFile $tempZip -UseBasicParsing -TimeoutSec 60 -ErrorAction Stop
        Expand-Archive -Path $tempZip -DestinationPath $tempExtract -Force

        if (-not (Test-Path $moduleDir)) { New-Item -ItemType Directory -Path $moduleDir -Force | Out-Null }
        if (-not (Test-Path $binDir)) { New-Item -ItemType Directory -Path $binDir -Force | Out-Null }

        $extractedCli = Join-Path $tempExtract 'cli'
        if (Test-Path $extractedCli) {
            Copy-Item -Path "$extractedCli\*" -Destination $moduleDir -Recurse -Force
        }

        $extractedTui = Join-Path $tempExtract 'rtb.exe'
        if (Test-Path $extractedTui) {
            Copy-Item -Path $extractedTui -Destination "$binDir\rtb.exe" -Force
        }

        $extractedLogo = Join-Path $tempExtract 'logo.txt'
        if (Test-Path $extractedLogo) {
            Copy-Item -Path $extractedLogo -Destination "$binDir\logo.txt" -Force
        }

        $extractedUninstall = Join-Path $tempExtract 'uninstall.ps1'
        if (Test-Path $extractedUninstall) {
            Copy-Item -Path $extractedUninstall -Destination "$binDir\uninstall.ps1" -Force
            Copy-Item -Path $extractedUninstall -Destination (Join-Path (Split-Path $binDir -Parent) 'uninstall.ps1') -Force
        }

        # Reload module
        $psdPath = Join-Path $moduleDir 'rtb.psd1'
        if (Test-Path $psdPath) {
            Import-Module $psdPath -DisableNameChecking -Force
        }

        Write-Host "  ✓ RTB successfully upgraded to $latestTag!" -ForegroundColor Green
    } catch {
        Write-Host "  Error during upgrade: $($_.Exception.Message)" -ForegroundColor Red
    } finally {
        if (Test-Path $tempZip) { Remove-Item -Force $tempZip -ErrorAction SilentlyContinue }
        if (Test-Path $tempExtract) { Remove-Item -Recurse -Force $tempExtract -ErrorAction SilentlyContinue }
    }
}

function Dev-Upgrade {
    Rtb-Upgrade @args
}

function Update-RtbCli { Rtb-Upgrade @args }
