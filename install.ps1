#Requires -Version 5.1
# RTB Setup Wizard - Windows / PowerShell
# Interactive and CI-capable Setup Wizard for the RTB Project Operations Suite.
param(
    [string]$InstallPath = '',
    [switch]$Quiet,
    [switch]$NoExec,
    [switch]$SkipUI,
    [switch]$InstallUI
)

$ErrorActionPreference = 'Stop'

# Ensure console output handles UTF-8
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    $OutputEncoding = [System.Text.Encoding]::UTF8
} catch {}

$script:InstallPath = $InstallPath
$script:scriptRoot  = $PSScriptRoot
$script:SkipUI      = $SkipUI.IsPresent
$script:InstallUI   = $InstallUI.IsPresent
$script:shouldInstallUI = $true

# Non-interactive / CI / Quiet Detection
$script:QUIET = $Quiet.IsPresent -or ($env:RTB_QUIET -eq '1') -or ($env:RTB_NON_INTERACTIVE -in @('1', 'true', 'True')) -or ($env:CI -eq 'true') -or ($env:GITHUB_ACTIONS -eq 'true')

# ANSI Capability Detection
$script:ANSI = (-not $script:QUIET) -and (
    $PSVersionTable.PSVersion.Major -ge 7 -or
    $env:TERM -match 'xterm|screen|256color|alacritty|kitty' -or
    $env:WT_SESSION -or
    ((Test-Path variable:Host) -and ($Host.UI.RawUI.ForegroundColor -ne -1))
)

# Braille Spinner Animation Frames
$script:SPINNER_FRAMES = @(
    [char]0x280B, [char]0x2819, [char]0x2839, [char]0x2838, [char]0x283C,
    [char]0x2834, [char]0x2826, [char]0x2827, [char]0x2807, [char]0x280F
)

function global:Esc([string]$code) {
    if ($script:ANSI) {
        "$([char]27)[$code"
    } else {
        ''
    }
}

function global:Write-Step([int]$n, [int]$total, [string]$label) {
    if ($script:QUIET) {
        Write-Host "[$n/$total] $label"
        return
    }
    $c = Esc '36m'
    $b = Esc '1m'
    $r = Esc '0m'
    $diamond = [char]0x25C6
    Write-Host "  ${b}${c}[$n/$total]${r} $diamond $label"
}

function global:Write-Warn([string]$msg) {
    $y = Esc '33m'
    $r = Esc '0m'
    $warnIcon = [char]0x26A0
    Write-Host "  ${y}$warnIcon  $msg${r}"
}

function global:Write-Fail([string]$msg) {
    $red = Esc '31m'
    $r = Esc '0m'
    $failIcon = [char]0x2717
    Write-Host "  ${red}$failIcon  $msg${r}"
    if ($script:NoExitOnFail) {
        throw $msg
    } else {
        exit 1
    }
}

function global:Start-Spinner([string]$Label) {
    $isRedirected = $false
    try {
        if (-not $script:ForceInteractive) {
            $isRedirected = [Console]::IsInputRedirected -or [Console]::IsOutputRedirected
        }
    } catch {}

    if ($script:QUIET -or $isRedirected -or (-not $script:ANSI)) {
        Write-Host "  ... $Label"
        return @{ Type = 'Quiet'; Label = $Label; Job = $null }
    }

    try {
        $frames = $script:SPINNER_FRAMES
        $rs = [System.Management.Automation.Runspaces.RunspaceFactory]::CreateRunspace()
        $rs.Open()
        $ps = [System.Management.Automation.PowerShell]::Create()
        $ps.Runspace = $rs

        $scriptBlock = {
            param($frames, $label)
            $i = 0
            while ($true) {
                $frame = $frames[$i % $frames.Count]
                [Console]::Write("`r  $frame  $label")
                [System.Threading.Thread]::Sleep(80)
                $i++
            }
        }

        $ps.AddScript($scriptBlock).AddArgument($frames).AddArgument($Label) | Out-Null
        $handle = $ps.BeginInvoke()

        return @{
            Type       = 'Runspace'
            Runspace   = $rs
            PowerShell = $ps
            Handle     = $handle
            Label      = $Label
        }
    } catch {
        Write-Host "  ... $Label"
        return @{ Type = 'Quiet'; Label = $Label; Job = $null }
    }
}

function global:Stop-Spinner([hashtable]$ctx, [bool]$success) {
    if ($ctx -and $ctx.Type -eq 'Runspace') {
        try {
            if ($ctx.PowerShell) {
                $ctx.PowerShell.Stop()
                $ctx.PowerShell.Dispose()
            }
            if ($ctx.Runspace) {
                $ctx.Runspace.Close()
                $ctx.Runspace.Dispose()
            }
        } catch {}
        try {
            [Console]::Write("`r" + (" " * 80) + "`r")
        } catch {}
    } elseif ($ctx -and $ctx.Job) {
        try {
            Stop-Job $ctx.Job -ErrorAction SilentlyContinue
            Remove-Job $ctx.Job -Force -ErrorAction SilentlyContinue
            [Console]::Write("`r" + (" " * 80) + "`r")
        } catch {}
    }

    $lbl = if ($ctx -and $ctx.Label) { $ctx.Label } else { '' }
    $icon = if ($success) { [char]0x2705 } else { [char]0x274C }
    $color = if ($success) { Esc '32m' } else { Esc '31m' }
    $reset = Esc '0m'
    Write-Host "  $icon  ${color}$lbl${reset}"
}

function global:Find-RepoRoot {
    $candidates = @()
    if ($script:scriptRoot) { $candidates += $script:scriptRoot }
    if ($PSScriptRoot) { $candidates += $PSScriptRoot }
    $candidates += (Get-Location).Path

    foreach ($startDir in $candidates) {
        if (-not $startDir) { continue }
        $dir = $startDir
        while ($dir -and (Test-Path $dir)) {
            $pkgJson = Join-Path $dir 'core\package.json'
            if (Test-Path -LiteralPath $pkgJson) {
                try {
                    $pkg = Get-Content -LiteralPath $pkgJson -Raw | ConvertFrom-Json
                    if ($pkg.name -eq '@3mr5aled/rtb' -or $pkg.name -eq '@3mr-5aled/rtb') {
                        return $dir
                    }
                } catch {}
            }
            $parent = Split-Path $dir -Parent
            if (-not $parent -or $parent -eq $dir) { break }
            $dir = $parent
        }
    }
    return $null
}

function global:Get-RtbInstallerVersion {
    # 1. Local RTB repository checkout
    $repoRoot = Find-RepoRoot
    if ($repoRoot) {
        $vFile = Join-Path $repoRoot 'VERSION'
        if (Test-Path -LiteralPath $vFile) {
            try {
                $raw = (Get-Content -LiteralPath $vFile -Raw).Trim()
                if ($raw -and ($raw -replace '^v','') -match '^\d+\.\d+\.\d+') {
                    return [string]($raw -replace '^v','')
                }
            } catch {}
        }
        $pkgFile = Join-Path $repoRoot 'core\package.json'
        if (Test-Path -LiteralPath $pkgFile) {
            try {
                $pkg = (Get-Content -LiteralPath $pkgFile -Raw) | ConvertFrom-Json
                if ($pkg.version -and [string]$pkg.version -match '^\d+\.\d+\.\d+') {
                    return [string]$pkg.version
                }
            } catch {}
        }
    }

    # 2. Standalone / Web One-Liner (irm ... | iex): Query the latest remote version from GitHub
    try {
        $remoteRaw = (Invoke-RestMethod -Uri 'https://raw.githubusercontent.com/3mr-5aled/rtb/main/VERSION' -TimeoutSec 5 -ErrorAction Stop)
        if ($remoteRaw -and ($remoteRaw -is [string])) {
            $cleaned = $remoteRaw.Trim() -replace '^v',''
            if ($cleaned -match '^\d+\.\d+\.\d+') {
                return $cleaned
            }
        }
    } catch {}

    try {
        $rel = Invoke-RestMethod -Uri 'https://api.github.com/repos/3mr-5aled/rtb/releases/latest' -TimeoutSec 5 -Headers @{ 'User-Agent' = 'rtb-installer' } -ErrorAction Stop
        if ($rel -and $rel.tag_name) {
            $cleaned = ([string]$rel.tag_name).Trim() -replace '^v',''
            if ($cleaned -match '^\d+\.\d+\.\d+') {
                return $cleaned
            }
        }
    } catch {}

    return '0.13.2'
}

$script:VERSION = Get-RtbInstallerVersion

function global:Show-Header {
    $ver = if ($script:VERSION) { $script:VERSION } else { Get-RtbInstallerVersion }
    if ($script:QUIET) {
        Write-Host "RTB Setup Wizard v$ver"
        return
    }
    $c = Esc '36m'
    $b = Esc '1m'
    $r = Esc '0m'
    $d = Esc '90m'
    $g = Esc '32m'
    Write-Host ""
    Write-Host "  ${b}${c}██████╗ ████████╗██████╗ ${r}"
    Write-Host "  ${b}${c}██╔══██╗╚══██╔══╝██╔══██╗${r}"
    Write-Host "  ${b}${c}██████╔╝   ██║   ██████╔╝${r}"
    Write-Host "  ${b}${c}██╔══██╗   ██║   ██╔══██╗${r}"
    Write-Host "  ${b}${c}██║  ██║   ██║   ██████╔╝${r}"
    Write-Host "  ${b}${c}╚═╝  ╚═╝   ╚═╝   ╚═════╝ ${r}  Setup Wizard ${g}v$ver${r}"
    Write-Host ""
    Write-Host "  ${c}RTB - Repository & Tooling Base${r} ${d}(v$ver)${r}"
    Write-Host "  ${d}Windows / PowerShell installer${r}"
    Write-Host ""
}

function global:Prompt-InstallPath([string]$default) {
    $isRedirected = $false
    try {
        if (-not $script:ForceInteractive) {
            $isRedirected = [Console]::IsInputRedirected
        }
    } catch {}

    if ($script:QUIET -or $script:InstallPath -or $isRedirected) {
        if ($script:InstallPath) { return $script:InstallPath } else { return $default }
    }

    Write-Host "  $(Esc '32m')?$(Esc '0m') Install location $(Esc '90m')(Enter to accept)$(Esc '0m')"
    Write-Host "    $(Esc '90m')$default$(Esc '0m')"
    Write-Host -NoNewline "  $([char]0x203A) "
    $in = Read-Host
    if ($in -and $in.Trim()) {
        return $in.Trim()
    } else {
        return $default
    }
}

function global:Prompt-Profiles([string[]]$candidates) {
    $isRedirected = $false
    try {
        if (-not $script:ForceInteractive) {
            $isRedirected = [Console]::IsInputRedirected
        }
    } catch {}

    if ($script:QUIET -or $isRedirected) {
        return @($candidates | Where-Object { [bool]$_ })
    }

    $selected = @()
    Write-Host ""
    Write-Host "  $(Esc '32m')?$(Esc '0m') Which PowerShell profiles should RTB auto-load into?"
    foreach ($p in $candidates) {
        if ($p) {
            Write-Host -NoNewline "    Include $(Esc '90m')$p$(Esc '0m')? [Y/n] "
            $ans = Read-Host
            if ($ans -notmatch '^[Nn]') {
                $selected += $p
            }
        }
    }
    return $selected
}

function global:Prompt-InstallUI {
    $isRedirected = $false
    try {
        if (-not $script:ForceInteractive) {
            $isRedirected = [Console]::IsInputRedirected
        }
    } catch {}

    if ($script:SkipUI -or ($env:RTB_SKIP_UI -in @('1', 'true', 'True'))) {
        return $false
    }
    if ($script:InstallUI -or ($env:RTB_INSTALL_UI -in @('1', 'true', 'True'))) {
        return $true
    }
    if ($script:QUIET -or $isRedirected) {
        return $true
    }

    Write-Host ""
    Write-Host "  $(Esc '32m')?$(Esc '0m') Download RTB Terminal UI (rtbtui) now?"
    Write-Host "    $(Esc '90m')[1] Download now (Recommended)$(Esc '0m')"
    Write-Host "    $(Esc '90m')[2] Download later (on first 'rtb ui' run)$(Esc '0m')"
    Write-Host -NoNewline "  $([char]0x203A) Choose [1/2] or [y/n] (Default: 1): "
    $ans = Read-Host
    if ($ans -match '^[2Nn]' -or $ans -eq 'later') {
        return $false
    }
    return $true
}

function global:Prompt-RunInit {
    $isRedirected = $false
    try {
        if (-not $script:ForceInteractive) {
            $isRedirected = [Console]::IsInputRedirected
        }
    } catch {}

    if ($script:QUIET -or $isRedirected) {
        return $false
    }
    Write-Host ""
    Write-Host -NoNewline "  $(Esc '32m')?$(Esc '0m') Run 'rtb init' now? [Y/n] "
    $ans = Read-Host
    return ($ans -notmatch '^[Nn]')
}

function global:Show-Summary([string]$installPath, [string[]]$profiles) {
    $g = Esc '32m'
    $b = Esc '1m'
    $c = Esc '36m'
    $d = Esc '90m'
    $r = Esc '0m'
    $check = [char]0x2714

    $ver = if ($script:VERSION) { $script:VERSION } else { Get-RtbInstallerVersion }
    Write-Host ""
    Write-Host "  ${b}${g}$check RTB v$ver installed successfully!${r}"
    Write-Host ""
    Write-Host "  ${c}RTB Version:${r}   v$ver"
    Write-Host "  ${c}Install path:${r}  $installPath"
    if ($profiles -and $profiles.Count -gt 0) {
        foreach ($p in $profiles) {
            Write-Host "  ${c}Profile:${r}       $p"
        }
    } else {
        Write-Host "  ${c}Profile:${r}       (none configured)"
    }
    if ($script:shouldInstallUI) {
        Write-Host "  ${c}TUI binary:${r}    Installed"
    } else {
        Write-Host "  ${c}TUI binary:${r}    Skipped (run 'rtb ui' to download anytime)"
    }
    Write-Host ""
    Write-Host "  ${b}Next steps:${r}"
    Write-Host "    ${g}rtb init${r}  ${d}- configure your project workspace${r}"
    Write-Host "    ${g}rtb help${r}  ${d}- explore available commands${r}"
    Write-Host "    ${g}rtb ui${r}    ${d}- open the interactive terminal dashboard${r}"
    Write-Host ""
}

function global:Ensure-Node {
    $nodeCmd = Get-Command node -ErrorAction SilentlyContinue
    if ($nodeCmd) {
        $vStr = (& node -v) -replace '^v',''
        $major = [int]($vStr.Split('.')[0])
        if ($major -ge 18) {
            return
        }
        Write-Warn "Node.js is installed but version ($major) is less than required (>= 18)."
    } else {
        Write-Warn "Node.js (>= 18) was not found on your system."
    }

    if ($script:QUIET) {
        Write-Fail "Node.js >= 18 is required. Install from https://nodejs.org"
    }

    Write-Host "  Please install Node.js >= 18 from https://nodejs.org or via winget: winget install OpenJS.NodeJS.LTS" -ForegroundColor Yellow
    Write-Fail "Node.js >= 18 is required to run RTB."
}



function global:Install-Steps {
    $TOTAL = 5
    $repoRoot = Find-RepoRoot
    $isStandalone = if ($script:isStandaloneOverride -ne $null) {
        $script:isStandaloneOverride
    } else {
        (-not $repoRoot)
    }

    # Step 1: Directories (Critical)
    Write-Step 1 $TOTAL 'Creating directories'
    $ctx = Start-Spinner 'Setting up install directories'
    try {
        foreach ($d in @($script:scriptsDir, $script:userConfigDir)) {
            if (-not (Test-Path $d)) {
                New-Item -ItemType Directory -Path $d -Force | Out-Null
            }
        }
        # Clean up stale legacy module directory if present in .config/rtb
        $staleModule = Join-Path $script:userConfigDir 'module'
        if (Test-Path $staleModule) {
            Remove-Item $staleModule -Recurse -Force -ErrorAction SilentlyContinue
        }
        # Clean up stale legacy module in AppData\Roaming\rtb if present
        if ($env:APPDATA) {
            $roamingModule = Join-Path $env:APPDATA 'rtb\module'
            if (Test-Path $roamingModule) {
                Remove-Item $roamingModule -Recurse -Force -ErrorAction SilentlyContinue
            }
        }
        Stop-Spinner $ctx $true
    } catch {
        Stop-Spinner $ctx $false
        Write-Fail "Cannot create directories: $_"
    }

    # Step 2: Deploy RTB Engine (Critical)
    Write-Step 2 $TOTAL 'Deploying RTB CLI engine'
    if ($isStandalone) {
        $bundleUrl = 'https://github.com/3mr-5aled/rtb/releases/latest/download/rtb-cli.js'
        $versionUrl = 'https://raw.githubusercontent.com/3mr-5aled/rtb/main/VERSION'
        $uninstUrl = 'https://raw.githubusercontent.com/3mr-5aled/rtb/main/uninstall.ps1'
        $destJs = Join-Path $script:scriptsDir 'rtb.js'
        $ctx = Start-Spinner 'Downloading rtb-cli.js'
        try {
            Invoke-WebRequest -Uri $bundleUrl -OutFile $destJs -UseBasicParsing -TimeoutSec 120 -ErrorAction Stop
            try {
                Invoke-WebRequest -Uri $versionUrl -OutFile (Join-Path $script:userConfigDir 'VERSION') -UseBasicParsing -TimeoutSec 15 -ErrorAction SilentlyContinue
                Copy-Item (Join-Path $script:userConfigDir 'VERSION') (Join-Path $script:scriptsDir 'VERSION') -Force -ErrorAction SilentlyContinue
            } catch {}
            if (-not (Test-Path (Join-Path $script:scriptsDir 'VERSION')) -and $script:VERSION) {
                Set-Content -Path (Join-Path $script:scriptsDir 'VERSION') -Value $script:VERSION -Encoding UTF8 -ErrorAction SilentlyContinue
                Copy-Item (Join-Path $script:scriptsDir 'VERSION') (Join-Path $script:userConfigDir 'VERSION') -Force -ErrorAction SilentlyContinue
            }
            try {
                Invoke-WebRequest -Uri $uninstUrl -OutFile (Join-Path $script:userConfigDir 'uninstall.ps1') -UseBasicParsing -TimeoutSec 15 -ErrorAction SilentlyContinue
            } catch {}
            Stop-Spinner $ctx $true
        } catch {
            Stop-Spinner $ctx $false
            # Fallback to rtb-cli.zip if standalone zip is published
            $ctxZip = Start-Spinner 'Downloading fallback rtb-cli.zip'
            $zipUrl = 'https://github.com/3mr-5aled/rtb/releases/latest/download/rtb-cli.zip'
            $tmpZip = Join-Path ([System.IO.Path]::GetTempPath()) "rtb-install-$(Get-Random).zip"
            $tmpExt = Join-Path ([System.IO.Path]::GetTempPath()) "rtb-install-$(Get-Random)"
            try {
                Invoke-WebRequest -Uri $zipUrl -OutFile $tmpZip -UseBasicParsing -TimeoutSec 60 -ErrorAction Stop
                Expand-Archive -Path $tmpZip -DestinationPath $tmpExt -Force
                if (Test-Path (Join-Path $tmpExt 'rtb.js')) {
                    Copy-Item (Join-Path $tmpExt 'rtb.js') "$script:scriptsDir\rtb.js" -Force
                } elseif (Test-Path (Join-Path $tmpExt 'core\dist\index.js')) {
                    Copy-Item (Join-Path $tmpExt 'core\dist\index.js') "$script:scriptsDir\rtb.js" -Force
                }
                foreach ($f in @('logo.txt', 'uninstall.ps1', 'VERSION')) {
                    $src = Join-Path $tmpExt $f
                    if (Test-Path $src) {
                        Copy-Item $src "$script:scriptsDir\$f" -Force
                        Copy-Item $src "$script:userConfigDir\$f" -Force -ErrorAction SilentlyContinue
                    }
                }
                Stop-Spinner $ctxZip $true
            } catch {
                Stop-Spinner $ctxZip $false
                Write-Fail "Download failed: $_`nCheck https://github.com/3mr-5aled/rtb/releases"
            } finally {
                Remove-Item $tmpZip, $tmpExt -Recurse -Force -ErrorAction SilentlyContinue
            }
        }
    } else {
        $ctx = Start-Spinner 'Deploying local CLI bundle'
        $coreDir = Join-Path $repoRoot 'core'
        $builtBundle = Join-Path $coreDir 'dist\index.js'
        if (-not (Test-Path $builtBundle)) {
            Push-Location $coreDir
            try {
                npm install --silent 2>&1 | Out-Null
                npm run build --silent 2>&1 | Out-Null
            } finally {
                Pop-Location
            }
        }

        if (Test-Path $builtBundle) {
            Copy-Item $builtBundle "$script:scriptsDir\rtb.js" -Force
        } else {
            Stop-Spinner $ctx $false
            Write-Fail "Core bundle not found at $builtBundle after build."
        }

        Stop-Spinner $ctx $true

        foreach ($f in @('logo.txt', 'uninstall.ps1', 'VERSION')) {
            $s = Join-Path $repoRoot $f
            if (Test-Path $s) {
                Copy-Item $s "$script:scriptsDir\$f" -Force
                Copy-Item $s "$script:userConfigDir\$f" -Force -ErrorAction SilentlyContinue
            }
        }
    }

    # Generate rtb.cmd and rtb.ps1 wrappers in bin/
    $cmdContent = "@echo off`r`nnode `"%~dp0rtb.js`" %*"
    Set-Content -Path (Join-Path $script:scriptsDir 'rtb.cmd') -Value $cmdContent -Encoding ASCII
    $ps1Content = "& node (Join-Path `$PSScriptRoot 'rtb.js') @args"
    Set-Content -Path (Join-Path $script:scriptsDir 'rtb.ps1') -Value $ps1Content -Encoding UTF8

    # Step 3: TUI Binary (Non-critical)
    Write-Step 3 $TOTAL 'Installing rtbtui binary'
    if (-not $script:shouldInstallUI) {
        $ctx = Start-Spinner 'Skipping rtbtui download (download later via "rtb ui")'
        Stop-Spinner $ctx $true
    } elseif ($isStandalone) {
        $binUrl = 'https://github.com/3mr-5aled/rtb/releases/latest/download/rtbtui-windows-amd64.exe'
        $tmpBin = Join-Path ([System.IO.Path]::GetTempPath()) "rtbtui-$(Get-Random).exe"
        $ctx = Start-Spinner 'Downloading rtbtui.exe'
        try {
            Invoke-WebRequest -Uri $binUrl -OutFile $tmpBin -UseBasicParsing -TimeoutSec 180 -EA Stop
            Copy-Item $tmpBin "$script:scriptsDir\rtbtui.exe" -Force
            Copy-Item $tmpBin "$script:scriptsDir\devtui.exe" -Force -ErrorAction SilentlyContinue
            Stop-Spinner $ctx $true
        } catch {
            Stop-Spinner $ctx $false
            Write-Warn "TUI binary download failed - 'rtb ui' unavailable, CLI is fine."
        } finally {
            Remove-Item $tmpBin -Force -ErrorAction SilentlyContinue
        }
    } else {
        $tuiDir = Join-Path $repoRoot 'tui'
        $cargo = Get-Command cargo -ErrorAction SilentlyContinue
        if ($cargo -and (Test-Path (Join-Path $tuiDir 'Cargo.toml'))) {
            $ctx = Start-Spinner 'Building rtbtui with Cargo'
            Push-Location $tuiDir
            try {
                cargo build --release 2>&1 | Out-Null
                $bin = Join-Path $tuiDir 'target\release\rtbtui.exe'
                if (Test-Path $bin) {
                    Copy-Item $bin "$script:scriptsDir\rtbtui.exe" -Force
                    Copy-Item $bin "$script:scriptsDir\devtui.exe" -Force -ErrorAction SilentlyContinue
                    Stop-Spinner $ctx $true
                } else {
                    Stop-Spinner $ctx $false
                    Write-Warn 'Cargo build succeeded but binary not found in target\release.'
                }
            } catch {
                Stop-Spinner $ctx $false
                Write-Warn 'Cargo build failed - retaining existing binary if present.'
            } finally {
                Pop-Location
            }
        } else {
            $pre = Join-Path $tuiDir 'target\release\rtbtui.exe'
            if (Test-Path $pre) {
                Copy-Item $pre "$script:scriptsDir\rtbtui.exe" -Force
                Copy-Item $pre "$script:scriptsDir\devtui.exe" -Force -ErrorAction SilentlyContinue
                Write-Warn 'cargo not found - copied prebuilt binary.'
            } else {
                Write-Warn "cargo not found and no prebuilt binary - 'rtb ui' will not work."
            }
        }
    }

    # Step 4: PATH Configuration (Non-critical)
    Write-Step 4 $TOTAL 'Configuring PATH'
    $ctx = Start-Spinner 'Updating User PATH'
    try {
        $cur = [Environment]::GetEnvironmentVariable('PATH', 'User')
        $pathParts = if ($cur) { $cur -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } } else { @() }
        
        # Clean up legacy Roaming\rtb\bin and old temp test entries
        $cleanedParts = @($pathParts | Where-Object {
            $_ -notmatch '(?i)[\\/]AppData[\\/]Roaming[\\/]rtb[\\/]bin' -and
            $_ -notmatch '(?i)[\\/]AppData[\\/]Local[\\/]Temp[\\/]rtb'
        })
        
        if ($cleanedParts -notcontains $script:scriptsDir) {
            $cleanedParts = @($script:scriptsDir) + $cleanedParts
        }
        $newPath = $cleanedParts -join ';'
        [Environment]::SetEnvironmentVariable('PATH', $newPath, 'User')

        # Clean current process PATH as well
        $procParts = @($env:PATH -split ';' | Where-Object {
            $_ -notmatch '(?i)[\\/]AppData[\\/]Roaming[\\/]rtb[\\/]bin' -and
            $_ -notmatch '(?i)[\\/]AppData[\\/]Local[\\/]Temp[\\/]rtb'
        })
        if ($procParts -notcontains $script:scriptsDir) {
            $procParts = @($script:scriptsDir) + $procParts
        }
        $env:PATH = $procParts -join ';'

        # Also clean legacy AppData\Roaming\rtb folder if installed to .config\rtb
        $legacyRoaming = Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'rtb'
        if ($legacyRoaming -and (Test-Path $legacyRoaming) -and ($legacyRoaming -ne $script:userConfigDir)) {
            $legacyCfg = Join-Path $legacyRoaming 'rtb.config.json'
            $targetCfg = Join-Path $script:userConfigDir 'rtb.config.json'
            if ((Test-Path $legacyCfg) -and (-not (Test-Path $targetCfg))) {
                Copy-Item $legacyCfg $targetCfg -Force -ErrorAction SilentlyContinue
            }
            Remove-Item (Join-Path $legacyRoaming 'bin') -Recurse -Force -ErrorAction SilentlyContinue
            Remove-Item (Join-Path $legacyRoaming 'module') -Recurse -Force -ErrorAction SilentlyContinue
            Remove-Item $legacyCfg -Force -ErrorAction SilentlyContinue
        }

        Stop-Spinner $ctx $true
    } catch {
        Stop-Spinner $ctx $false
        Write-Warn "PATH update failed - add '$($script:scriptsDir)' manually."
    }

    # Step 5: Profile Injection (Non-critical)
    Write-Step 5 $TOTAL 'Configuring PowerShell profile(s)'
    $line = '(& rtb shell-init pwsh | Out-String) | Invoke-Expression'

    foreach ($p in $script:resolvedProfiles) {
        if ($p) {
            $ctx = Start-Spinner "Updating $([System.IO.Path]::GetFileName($p))"
            try {
                $dir = Split-Path $p -Parent
                if ($dir -and -not (Test-Path $dir)) {
                    New-Item -ItemType Directory -Path $dir -Force | Out-Null
                }
                if (-not (Test-Path $p)) {
                    New-Item -ItemType File -Path $p -Force | Out-Null
                }
                $pLines = Get-Content -LiteralPath $p -ErrorAction SilentlyContinue
                $clean = if ($pLines) {
                    @($pLines | Where-Object {
                        $_ -notmatch 'Import-Module\s+.*?(rtb|dev-tools|dev-cli|rtb-command-tool).*?\.psd1' -and
                        $_ -notmatch 'rtb\s+shell-init' -and
                        $_ -notmatch '#\s*RTB.*?Module' -and
                        $_ -notmatch '#\s*RTB.*?Shell Integration'
                    })
                } else {
                    @()
                }
                $outputLines = [System.Collections.Generic.List[string]]::new()
                foreach ($l in $clean) { $outputLines.Add($l) }
                $outputLines.Add('')
                $outputLines.Add('# RTB Shell Integration')
                $outputLines.Add($line)
                $newContent = ($outputLines -join "`r`n").TrimEnd() + "`r`n"
                $newContent | Set-Content -LiteralPath $p -Encoding UTF8
                Stop-Spinner $ctx $true
            } catch {
                Stop-Spinner $ctx $false
                Write-Warn "Could not update $p - $_"
            }
        }
    }

    # Step 6: Verify Installation (Smoke check)
    $smokeRtb = Join-Path $script:scriptsDir 'rtb.cmd'
    if (Test-Path $smokeRtb) {
        $checkCtx = Start-Spinner 'Verifying RTB installation'
        try {
            $verOutput = (& $smokeRtb --version 2>&1)
            if ($LASTEXITCODE -eq 0 -or $verOutput -match '\d+\.\d+\.\d+') {
                Stop-Spinner $checkCtx $true
            } else {
                Stop-Spinner $checkCtx $false
                Write-Warn "Smoke check returned unexpected output: $verOutput"
            }
        } catch {
            Stop-Spinner $checkCtx $false
            Write-Warn "Smoke check failed to execute: $_"
        }
    }
}

function global:Main {
    Show-Header
    Ensure-Node
    $homeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
    $default = Join-Path $homeDir '.config\rtb'

    $script:userConfigDir = Prompt-InstallPath $default
    $script:scriptsDir    = if ($env:RTB_BIN_DIR) { $env:RTB_BIN_DIR } else { Join-Path $script:userConfigDir 'bin' }

    $docs = [Environment]::GetFolderPath('MyDocuments')
    $candidateProfiles = @()
    if ($PROFILE) { $candidateProfiles += $PROFILE }
    if ($docs) {
        $candidateProfiles += (Join-Path $docs 'WindowsPowerShell\Microsoft.PowerShell_profile.ps1')
        $candidateProfiles += (Join-Path $docs 'PowerShell\Microsoft.PowerShell_profile.ps1')
    }
    $allProfiles = @($candidateProfiles | Where-Object { [bool]$_ } | Select-Object -Unique)
    $script:resolvedProfiles = Prompt-Profiles $allProfiles
    $script:shouldInstallUI  = Prompt-InstallUI

    Install-Steps
    Show-Summary $script:userConfigDir $script:resolvedProfiles

    if (Prompt-RunInit) {
        # Unload any previously cached rtb module or function from active session
        Remove-Module rtb -Force -ErrorAction SilentlyContinue
        Remove-Item Function:\rtb -Force -ErrorAction SilentlyContinue
        Remove-Item Function:\Rtb-Init -Force -ErrorAction SilentlyContinue

        $rtbCmd = Join-Path $script:scriptsDir 'rtb.cmd'
        if (Test-Path $rtbCmd) {
            & $rtbCmd init
        } elseif (Get-Command rtb -ErrorAction SilentlyContinue) {
            rtb init
        }
    }
}

if (-not $NoExec) {
    Main
}
