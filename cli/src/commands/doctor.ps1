function Rtb-Doctor {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0, ValueFromRemainingArguments = $true)]
        [string[]]$RemainingArgs
    )

    Write-RtbHeader 'System Doctor'
    Write-Host ''

    $allGood = $true

    $WriteCheck = {
        param([bool]$Pass, [string]$Label, [string]$Detail = '')
        if ($Pass) {
            Write-Host "  ✅ $Label" -ForegroundColor Green
        } else {
            Write-Host "  ❌ $Label" -ForegroundColor Red
            if ($Detail) { Write-Host "     → $Detail" -ForegroundColor Yellow }
        }
    }

    # 1. Config Check
    Write-Host '  Config' -ForegroundColor Cyan
    $config = try { Get-RtbConfig -ErrorAction SilentlyContinue } catch { $null }
    $userConfigDir = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb' } else { Join-Path $env:HOME '.config/rtb' }
    $userConfigFile = Join-Path $userConfigDir 'rtb.config.json'
    $configPassed = ($null -ne $config)
    $configLabel = if ($configPassed) { "rtb.config.json ($userConfigFile)" } else { "rtb.config.json found and parseable" }
    & $WriteCheck $configPassed $configLabel "Run 'rtb init' to create your config at $userConfigFile"
    if (-not $configPassed) { $allGood = $false }

    # 2. Project Roots Check (9 roots)
    Write-Host ''
    Write-Host '  Project Roots' -ForegroundColor Cyan
    if ($config -and $config.projectRoots) {
        $rootMap = [ordered]@{
            active     = $config.projectRoots.active
            paused     = $config.projectRoots.paused
            planning   = $config.projectRoots.planning
            testing    = $config.projectRoots.testing
            production = $config.projectRoots.production
            staging    = $config.projectRoots.staging
            vibe       = $config.projectRoots.vibe
            sandbox    = $config.projectRoots.sandbox
            abandoned  = $config.projectRoots.abandoned
        }
        foreach ($key in $rootMap.Keys) {
            $entry = $rootMap[$key]
            $pathVal = if ($entry -is [string]) { $entry } elseif ($entry -and $entry.PSObject.Properties['path']) { $entry.path } else { $null }
            $emojiVal = if ($entry -and $entry.PSObject.Properties['emoji']) { $entry.emoji } else { '📁' }
            $labelName = if ($entry -and $entry.PSObject.Properties['label']) { $entry.label } else { $key }

            $exists = [bool]($pathVal -and (Test-Path -LiteralPath $pathVal))
            $label = if ($pathVal) { "$emojiVal $labelName ($key) → $pathVal" } else { "$key → (not configured)" }
            & $WriteCheck $exists $label "Directory does not exist. Create it or update projectRoots.$key in your config."
            if (-not $exists) { $allGood = $false }
        }
    } else {
        & $WriteCheck $false 'Cannot check project roots (invalid or missing config)' "Fix rtb.config.json or run 'rtb init -Force'"
        $allGood = $false
    }

    # 3. Required Tools
    Write-Host ''
    Write-Host '  Required Tools' -ForegroundColor Cyan
    foreach ($tool in @('git')) {
        $found = [bool](Get-Command -Name $tool -ErrorAction SilentlyContinue)
        & $WriteCheck $found "$tool in PATH" "Install $tool and ensure it is on your PATH"
        if (-not $found) { $allGood = $false }
    }

    # 4. Optional Tools
    Write-Host ''
    Write-Host '  Optional Tools' -ForegroundColor Cyan
    $optionals = @(
        @{ Name = 'node';   Label = 'Node.js (for JavaScript/TypeScript projects)' },
        @{ Name = 'cargo';  Label = 'Cargo / Rust (for Rust projects and rtb build)' },
        @{ Name = 'python'; Label = 'Python (for Python projects)' },
        @{ Name = 'tar';    Label = 'tar (for rtb archive/unarchive)' }
    )
    foreach ($tool in $optionals) {
        $found = [bool](Get-Command -Name $tool.Name -ErrorAction SilentlyContinue)
        $icon = if ($found) { '✅' } else { '⚠ ' }
        $color = if ($found) { 'Green' } else { 'DarkYellow' }
        Write-Host "  $icon $($tool.Label)" -ForegroundColor $color
    }

    # 5. AI Agents
    Write-Host ''
    Write-Host '  AI Agents' -ForegroundColor Cyan
    $agents = @('agy','claude','gemini','codex','cursor','windsurf','aider','openhands')
    $foundAgents = @($agents | Where-Object { Get-Command -Name $_ -ErrorAction SilentlyContinue })
    if ($foundAgents.Count -gt 0) {
        Write-Host "  ✅ Installed: $($foundAgents -join ', ')" -ForegroundColor Green
    } else {
        Write-Host '  ⚠  No AI agents found in PATH' -ForegroundColor DarkYellow
    }

    # 6. TUI Binary
    Write-Host ''
    Write-Host '  TUI Binary' -ForegroundColor Cyan
    $tuiCmd = Get-Command -Name 'rtb' -CommandType Application -ErrorAction SilentlyContinue
    $appDataBinary = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb\bin\rtb.exe' } else { $null }
    $userHomeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
    $userConfigBinary = Join-Path $userHomeDir '.config\rtb\bin\rtb.exe'

    $localTarget = Join-Path $PSScriptRoot '..\..\..\tui\target\release\rtb.exe'
    $localDebugTarget = Join-Path $PSScriptRoot '..\..\..\tui\target\debug\rtb.exe'
    $localBuilt = (Test-Path $localTarget) -or (Test-Path $localDebugTarget)

    $installedBinaryPath = if ($tuiCmd -and ($tuiCmd.Source -or $tuiCmd.Name)) {
        if ($tuiCmd.Source) { $tuiCmd.Source } else { $tuiCmd.Name }
    } elseif ($appDataBinary -and (Test-Path $appDataBinary)) {
        $appDataBinary
    } elseif ($userConfigBinary -and (Test-Path $userConfigBinary)) {
        $userConfigBinary
    } else {
        $null
    }

    if ($installedBinaryPath) {
        & $WriteCheck $true "rtb binary installed ($installedBinaryPath)"
        $binDir = Split-Path $installedBinaryPath -Parent
        if ($env:PATH -notlike "*$binDir*") {
            $env:PATH = "$binDir;$env:PATH"
        }
    } elseif ($localBuilt) {
        $builtPath = if (Test-Path $localTarget) { $localTarget } else { $localDebugTarget }
        & $WriteCheck $false "rtb binary built locally ($builtPath) but not installed" "Run '.\install.ps1' to install rtb"
        $allGood = $false
    } else {
        & $WriteCheck $false 'rtb binary installed' "Build with: cargo build --release -p rtb, then run '.\install.ps1'"
        $allGood = $false
    }

    # 7. Summary
    Write-Host ''
    Write-Host '══════════════════════════════════════════' -ForegroundColor Cyan
    if ($allGood) {
        Write-Host '  ✅ All checks passed — RTB is healthy!' -ForegroundColor Green
    } else {
        Write-Host '  ❌ Some checks failed — see above for details.' -ForegroundColor Red
    }
    Write-Host '══════════════════════════════════════════' -ForegroundColor Cyan

    return $allGood
}

function Dev-Doctor { Rtb-Doctor @args }
function Test-RtbDoctor { Rtb-Doctor @args }
function Test-RtbEnvironment { Rtb-Doctor @args }
