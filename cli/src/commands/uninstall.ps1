function Rtb-Uninstall {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0, ValueFromRemainingArguments = $true)]
        [string[]]$ArgsList,

        [switch]$Force,
        [switch]$KeepConfig
    )

    $allArgs = @()
    if ($ArgsList) { $allArgs += $ArgsList }

    $isForce = $Force.IsPresent -or ($allArgs -contains '-Force') -or ($allArgs -contains '--force') -or ($allArgs -contains '-f')
    $isKeepConfig = $KeepConfig.IsPresent -or ($allArgs -contains '-KeepConfig') -or ($allArgs -contains '--keep-config')

    # Look for standalone uninstall.ps1 script in APPDATA or repo root
    $candidates = @(
        (Join-Path $env:APPDATA 'rtb\uninstall.ps1'),
        (Join-Path $env:APPDATA 'rtb\bin\uninstall.ps1')
    )

    # Traverse upwards to find repo root uninstall.ps1 if present
    $repoRoot = $PSScriptRoot
    while ($repoRoot -and (Test-Path $repoRoot)) {
        $foundScript = Join-Path $repoRoot "uninstall.ps1"
        if (Test-Path $foundScript) {
            $candidates += $foundScript
            break
        }
        $parent = Split-Path $repoRoot -Parent
        if ($parent -eq $repoRoot) { break }
        $repoRoot = $parent
    }

    $uninstallScript = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1

    if ($uninstallScript) {
        $params = @{}
        if ($isForce) { $params['Force'] = $true }
        if ($isKeepConfig) { $params['KeepConfig'] = $true }
        & $uninstallScript @params
    } else {
        # Self-contained fallback uninstallation
        Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Yellow
        Write-Host "  Uninstalling RTB (رتّب) Developer Project Operations Suite" -ForegroundColor Yellow
        Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Yellow

        if (-not $isForce) {
            $confirm = Read-Host "Are you sure you want to uninstall RTB from your system? (y/N)"
            if ($confirm -notmatch '^(y|yes)$') {
                Write-Host "Uninstallation canceled." -ForegroundColor Gray
                return
            }
        }

        # 1. Unload module
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
        if (-not $isForce) {
            Write-Host ""
            $profileAns = Read-Host "Remove RTB autoload from your PowerShell profile(s)? (y/N)"
            $shouldCleanProfile = ($profileAns -match '^(y|yes)$')
        }

        $oldPattern = "(?m)^.*(Import-Module.*?(dev-tools|dev-cli|rtb-command-tool|rtb[\\/]module[\\/]rtb\.psd1|rtb\.psd1)|# RTB CLI Module).*`r?`n?"

        if ($shouldCleanProfile) {
            foreach ($pPath in $profilePaths) {
                if ($pPath -and (Test-Path $pPath)) {
                    $pContent = Get-Content $pPath -Raw -ErrorAction SilentlyContinue
                    if ($pContent -and ($pContent -match "(dev-tools|dev-cli|rtb-command-tool|rtb[\\/]module[\\/]rtb\.psd1|rtb\.psd1)")) {
                        $cleanedContent = [regex]::Replace($pContent, $oldPattern, "")
                        Set-Content -Path $pPath -Value $cleanedContent.TrimEnd() -Encoding UTF8
                        Write-Host "Removed RTB autoload entry from profile: $pPath" -ForegroundColor Green
                    }
                }
            }
        } else {
            Write-Host "`n  ⚠️  The Import-Module line was kept in your PowerShell profile(s)." -ForegroundColor Yellow
            Write-Host "     This will cause an error on every new shell until you remove it manually." -ForegroundColor Yellow
            Write-Host "     To remove it manually, edit your profile: notepad `$PROFILE" -ForegroundColor Gray
        }

        # 3. Clean up installed binaries, module, and config
        $userHomeDir   = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
        $userConfigDir = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb' } else { Join-Path $userHomeDir '.config/rtb' }
        $moduleHome    = Join-Path $userConfigDir 'module'
        $scriptsDir    = if ($env:RTB_BIN_DIR) { $env:RTB_BIN_DIR } elseif ($env:APPDATA) { Join-Path $env:APPDATA 'rtb\bin' } else { Join-Path $userHomeDir '.config\rtb\bin' }

        if (Test-Path $moduleHome) {
            Remove-Item -Recurse -Force $moduleHome -ErrorAction SilentlyContinue
            Write-Host "Removed RTB module directory: $moduleHome" -ForegroundColor Green
        }

        if (Test-Path $scriptsDir) {
            Remove-Item -Recurse -Force $scriptsDir -ErrorAction SilentlyContinue
            Write-Host "Removed RTB binaries directory: $scriptsDir" -ForegroundColor Green
        }

        if (-not $isKeepConfig) {
            if (Test-Path $userConfigDir) {
                Remove-Item -Recurse -Force $userConfigDir -ErrorAction SilentlyContinue
                Write-Host "Removed user configuration directory: $userConfigDir" -ForegroundColor Green
            }
        } else {
            Write-Host "Preserved user configuration directory at: $userConfigDir" -ForegroundColor Cyan
        }

        Write-Host "`nUninstallation Complete! RTB has been removed from your system." -ForegroundColor Green
    }
}

function Dev-Uninstall {
    Rtb-Uninstall @args
}
