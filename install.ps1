#Requires -Version 5.1
# RTB Setup Wizard - Windows / PowerShell
# Interactive and CI-capable Setup Wizard for the RTB Project Operations Suite.
param(
    [string]$InstallPath = '',
    [switch]$Quiet,
    [switch]$NoExec
)

$ErrorActionPreference = 'Stop'

# Ensure console output handles UTF-8
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    $OutputEncoding = [System.Text.Encoding]::UTF8
} catch {}

$script:InstallPath = $InstallPath
$script:scriptRoot  = $PSScriptRoot

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

function global:Show-Header {
    if ($script:QUIET) {
        Write-Host 'RTB Setup Wizard'
        return
    }
    $c = Esc '36m'
    $b = Esc '1m'
    $r = Esc '0m'
    $d = Esc '90m'
    Write-Host ""
    Write-Host "  ${b}${c}██████╗ ████████╗██████╗ ${r}"
    Write-Host "  ${b}${c}██╔══██╗╚══██╔══╝██╔══██╗${r}"
    Write-Host "  ${b}${c}██████╔╝   ██║   ██████╔╝${r}"
    Write-Host "  ${b}${c}██╔══██╗   ██║   ██╔══██╗${r}"
    Write-Host "  ${b}${c}██║  ██║   ██║   ██████╔╝${r}"
    Write-Host "  ${b}${c}╚═╝  ╚═╝   ╚═╝   ╚═════╝ ${r}  Setup Wizard"
    Write-Host ""
    Write-Host "  ${c}RTB - Repository & Tooling Base${r}"
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

    Write-Host ""
    Write-Host "  ${b}${g}$check RTB installed successfully!${r}"
    Write-Host ""
    Write-Host "  ${c}Install path:${r}  $installPath"
    if ($profiles -and $profiles.Count -gt 0) {
        foreach ($p in $profiles) {
            Write-Host "  ${c}Profile:${r}       $p"
        }
    } else {
        Write-Host "  ${c}Profile:${r}       (none configured)"
    }
    Write-Host ""
    Write-Host "  ${b}Next steps:${r}"
    Write-Host "    ${g}rtb init${r}  ${d}- configure your project workspace${r}"
    Write-Host "    ${g}rtb help${r}  ${d}- explore available commands${r}"
    Write-Host "    ${g}rtb ui${r}    ${d}- open the interactive terminal dashboard${r}"
    Write-Host ""
}

function global:Find-RepoRoot {
    $dir = if ($script:scriptRoot) { $script:scriptRoot } elseif ($PSScriptRoot) { $PSScriptRoot } else { (Get-Location).Path }
    while ($dir -and (Test-Path $dir)) {
        if (Test-Path (Join-Path $dir 'cli\rtb.psd1')) {
            return $dir
        }
        $parent = Split-Path $dir -Parent
        if (-not $parent -or $parent -eq $dir) {
            return $null
        }
        $dir = $parent
    }
    return $null
}

function global:Install-Steps {
    $TOTAL = 4
    $repoRoot = Find-RepoRoot
    $isStandalone = if ($script:isStandaloneOverride -ne $null) {
        $script:isStandaloneOverride
    } else {
        (-not $repoRoot)
    }

    # Step 1: Directories & Phase 2 Cleanup (Critical)
    Write-Step 1 $TOTAL 'Creating directories & cleaning legacy artefacts'
    $ctx = Start-Spinner 'Setting up install directories'
    try {
        foreach ($d in @($script:scriptsDir, $script:userConfigDir)) {
            if (-not (Test-Path $d)) {
                New-Item -ItemType Directory -Path $d -Force | Out-Null
            }
        }
        # Phase 2 Cleanup: remove old PowerShell module directory if present
        if (Test-Path $script:moduleHome) {
            Remove-Item -Recurse -Force $script:moduleHome -ErrorAction SilentlyContinue
        }
        Stop-Spinner $ctx $true
    } catch {
        Stop-Spinner $ctx $false
        Write-Fail "Cannot create directories: $_"
    }

    # Step 2: Binary Deployment (Critical)
    Write-Step 2 $TOTAL 'Installing rtb binary'
    if ($isStandalone) {
        $binUrl = 'https://github.com/3mr-5aled/rtb/releases/latest/download/rtb-windows-amd64.exe'
        $tmpBin = Join-Path ([System.IO.Path]::GetTempPath()) "rtb-$(Get-Random).exe"
        $ctx = Start-Spinner 'Downloading rtb.exe'
        try {
            Invoke-WebRequest -Uri $binUrl -OutFile $tmpBin -UseBasicParsing -TimeoutSec 180 -EA Stop
            Copy-Item $tmpBin "$script:scriptsDir\rtb.exe" -Force
            Copy-Item $tmpBin "$script:scriptsDir\dev.exe" -Force -ErrorAction SilentlyContinue
            Stop-Spinner $ctx $true
        } catch {
            Stop-Spinner $ctx $false
            Write-Fail "Download failed: $_`nCheck https://github.com/3mr-5aled/rtb/releases"
        } finally {
            Remove-Item $tmpBin -Force -ErrorAction SilentlyContinue
        }
    } else {
        $tuiDir = Join-Path $repoRoot 'tui'
        $cargo = Get-Command cargo -ErrorAction SilentlyContinue
        if ($cargo -and (Test-Path (Join-Path $tuiDir 'Cargo.toml'))) {
            $ctx = Start-Spinner 'Building rtb with Cargo'
            Push-Location $tuiDir
            try {
                cargo build --release 2>&1 | Out-Null
                $candidates = @(
                    (Join-Path $tuiDir 'target\release\rtb.exe'),
                    (Join-Path $tuiDir 'target\release\rtb'),
                    (Join-Path $repoRoot 'target\release\rtb.exe'),
                    (Join-Path $repoRoot 'target\release\rtb')
                )
                $bin = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
                if ($bin) {
                    $ext = if ($bin -like '*.exe') { '.exe' } else { '' }
                    Copy-Item $bin "$script:scriptsDir\rtb$ext" -Force
                    Copy-Item $bin "$script:scriptsDir\dev$ext" -Force -ErrorAction SilentlyContinue
                    Stop-Spinner $ctx $true
                } else {
                    Stop-Spinner $ctx $false
                    Write-Fail 'Cargo build succeeded but binary not found in target\release.'
                }
            } catch {
                Stop-Spinner $ctx $false
                Write-Fail "Cargo build failed: $_"
            } finally {
                Pop-Location
            }
        } else {
            $candidates = @(
                (Join-Path $tuiDir 'target\release\rtb.exe'),
                (Join-Path $tuiDir 'target\release\rtb'),
                (Join-Path $repoRoot 'target\release\rtb.exe'),
                (Join-Path $repoRoot 'target\release\rtb')
            )
            $pre = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
            if ($pre) {
                $ext = if ($pre -like '*.exe') { '.exe' } else { '' }
                Copy-Item $pre "$script:scriptsDir\rtb$ext" -Force
                Copy-Item $pre "$script:scriptsDir\dev$ext" -Force -ErrorAction SilentlyContinue
                Write-Warn 'cargo not found - copied prebuilt binary.'
            } else {
                Write-Fail "cargo not found and no prebuilt binary at target\release."
            }
        }
    }

    # Step 3: PATH Configuration
    Write-Step 3 $TOTAL 'Configuring PATH'
    $ctx = Start-Spinner 'Updating User PATH'
    try {
        $cur = [Environment]::GetEnvironmentVariable('PATH', 'User')
        $pathParts = if ($cur) { $cur -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } } else { @() }
        if ($pathParts -notcontains $script:scriptsDir) {
            $newPath = if ($cur) { "$cur;$($script:scriptsDir)" } else { $script:scriptsDir }
            [Environment]::SetEnvironmentVariable('PATH', $newPath, 'User')
        }
        if (($env:PATH -split ';') -notcontains $script:scriptsDir) {
            $env:PATH = "$($script:scriptsDir);$($env:PATH)"
        }
        Stop-Spinner $ctx $true
    } catch {
        Stop-Spinner $ctx $false
        Write-Warn "PATH update failed - add '$($script:scriptsDir)' manually."
    }

    # Step 4: Profile Injection & Phase 2 Cleanup
    Write-Step 4 $TOTAL 'Configuring PowerShell profile(s)'
    $line = 'Invoke-Expression (& rtb shell-init pwsh)'
    foreach ($p in $script:resolvedProfiles) {
        if ($p) {
            $ctx = Start-Spinner "Updating $([System.IO.Path]::GetFileName($p))"
            try {
                $dir = Split-Path $p -Parent
                if ($dir -and -not (Test-Path -LiteralPath $dir)) {
                    New-Item -ItemType Directory -Path $dir -Force | Out-Null
                }
                if (-not (Test-Path -LiteralPath $p)) {
                    New-Item -ItemType File -Path $p -Force | Out-Null
                }
                $pLines = Get-Content -LiteralPath $p -ErrorAction SilentlyContinue
                [array]$clean = if ($pLines) {
                    @($pLines | Where-Object {
                        -not [string]::IsNullOrWhiteSpace($_) -and
                        $_ -notmatch 'Import-Module\s+.*?(rtb|dev-tools|dev-cli|rtb-command-tool).*?\.psd1' -and
                        $_ -notmatch 'Invoke-Expression\s+.*?rtb\s+shell-init' -and
                        $_ -notmatch '#\s*RTB.*?(Module|CLI|Integration)'
                    })
                } else {
                    @()
                }
                $newContent = ($clean + @('', '# RTB Shell Integration', $line)) -join "`r`n"
                [System.IO.File]::WriteAllText($p, ($newContent.TrimEnd() + "`r`n"), [System.Text.Encoding]::UTF8)
                Stop-Spinner $ctx $true
            } catch {
                Stop-Spinner $ctx $false
                Write-Warn "Could not update $p - $_"
            }
        }
    }

    # Active session shell integration
    $targetBin = Join-Path $script:scriptsDir 'rtb.exe'
    if (Test-Path $targetBin) {
        try {
            Invoke-Expression (& $targetBin shell-init pwsh)
        } catch {}
    }
}

function global:Main {
    Show-Header

    $default = if ($env:APPDATA) {
        Join-Path $env:APPDATA 'rtb'
    } else {
        $homeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
        Join-Path $homeDir '.config\rtb'
    }

    $script:userConfigDir = Prompt-InstallPath $default
    $script:moduleHome    = Join-Path $script:userConfigDir 'module'
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

    Install-Steps
    Show-Summary $script:userConfigDir $script:resolvedProfiles

    if (Prompt-RunInit) {
        $binPath = Join-Path $script:scriptsDir 'rtb.exe'
        if (Test-Path $binPath) {
            & $binPath init
        } elseif (Get-Command rtb -ErrorAction SilentlyContinue) {
            rtb init
        }
    }
}

if (-not $NoExec) {
    Main
}
