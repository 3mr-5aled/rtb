<#
.SYNOPSIS
    rtb — RTB (رتّب) Developer Project Operations CLI
.DESCRIPTION
    Consolidates project lifecycle management, health monitoring, and maintenance
    into a single 'rtb' command with full tab completion.
#>

# Load utilities
. (Join-Path $PSScriptRoot 'src\utils\helpers.ps1')

# Load all command implementations
Get-ChildItem -Path (Join-Path $PSScriptRoot 'src\commands') -Filter '*.ps1' -ErrorAction SilentlyContinue | ForEach-Object {
    . $_.FullName
}

function Invoke-RtbNativeRedirect {
    param(
        [Parameter(Mandatory = $true)][string]$Subcommand,
        [string[]]$SubArgs
    )
    $exe = if ($env:_RTB_BIN -and (Test-Path $env:_RTB_BIN)) {
        $env:_RTB_BIN
    } else {
        (Get-Command rtb -CommandType Application -ErrorAction SilentlyContinue).Source
    }
    if ($exe) {
        if ($SubArgs) {
            & $exe $Subcommand @SubArgs
        } else {
            & $exe $Subcommand
        }
        return $true
    }
    return $false
}

# Main entry point
function rtb {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$Command,

        [Parameter(Position = 1, ValueFromRemainingArguments)]
        [string[]]$Arguments
    )

    if (-not $Command -or $Command -eq '--help' -or $Command -eq '-h') { $Command = 'help' }
    if ($Command -eq '--version' -or $Command -eq '-v') {
        $ver = '0.4.0'
        $psdPath = Join-Path $PSScriptRoot 'rtb.psd1'
        if (Test-Path $psdPath) {
            try {
                $manifest = Import-PowerShellDataFile -Path $psdPath -ErrorAction SilentlyContinue
                if ($manifest -and $manifest.ModuleVersion) { $ver = $manifest.ModuleVersion }
            } catch {}
        }
        Write-Host "RTB (ﺐﺘّﺭ) CLI v$ver" -ForegroundColor Green
        return
    }

    # Config Gate for data-dependent commands
    $freeCommands = @('help', 'init', 'config', 'doctor', 'uninstall', '--version', '-v', '--help', '-h')
    if ($Command.ToLower() -notin $freeCommands -and -not (Test-RtbConfigured)) {
        $userHomeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
        $userConfigDir = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb' } else { Join-Path $userHomeDir '.config/rtb' }
        $userConfigFile = Join-Path $userConfigDir 'rtb.config.json'
        Write-Host ""
        Write-Host "  ⚠  RTB is not configured yet." -ForegroundColor Yellow
        Write-Host "     Run 'rtb init' to set up your workspace (or edit $userConfigFile directly)." -ForegroundColor Gray
        Write-Host ""

        $isNonInteractive = [bool]($env:RTB_NON_INTERACTIVE -or $env:CI -or $env:GITHUB_ACTIONS -or [Console]::IsInputRedirected)
        if (-not $isNonInteractive) {
            try {
                $answer = Read-Host "  Would you like to configure now? (Y/n)"
                if ([string]::IsNullOrWhiteSpace($answer) -or $answer.Trim() -match '^(y|yes)$') {
                    Rtb-Init
                    return
                }
            } catch {
                return
            }
        }
        return
    }

    switch ($Command.ToLower()) {
        'init'        { if ($Arguments) { Rtb-Init @Arguments } else { Rtb-Init } }
        'config'      { if ($Arguments) { Rtb-Config @Arguments } else { Rtb-Config } }
        'run'         { if ($Arguments) { Rtb-Run @Arguments } else { Rtb-Run } }
        'build'       { if ($Arguments) { Rtb-Build @Arguments } else { Rtb-Build } }
        'test'        { if ($Arguments) { Rtb-Test @Arguments } else { Rtb-Test } }
        'goto'        { if ($Arguments) { Dev-Goto @Arguments } else { Dev-Goto } }
        'new'         { if ($Arguments) { Dev-New @Arguments } else { Dev-New } }
        'pause'       { if ($Arguments) { Dev-Pause @Arguments } else { Dev-Pause } }
        'resume'      { if ($Arguments) { Dev-Resume @Arguments } else { Dev-Resume } }
        'deploy'      { if ($Arguments) { Dev-Deploy @Arguments } else { Dev-Deploy } }
        'archive'     { if ($Arguments) { Dev-Archive @Arguments } else { Dev-Archive } }
        'unarchive'   { if ($Arguments) { Dev-Unarchive @Arguments } else { Dev-Unarchive } }
        'list'        { if (-not (Invoke-RtbNativeRedirect 'list' $Arguments)) { if ($Arguments) { Dev-List @Arguments } else { Dev-List } } }
        'info'        { if ($Arguments) { Rtb-Info @Arguments } else { Rtb-Info } }
        'health'      { if ($Arguments) { Dev-Health @Arguments } else { Dev-Health } }
        'commit'      { if ($Arguments) { Rtb-Commit @Arguments } else { Rtb-Commit } }
        'clean'       { if ($Arguments) { Dev-Clean @Arguments } else { Dev-Clean } }
        'index'       { if ($Arguments) { Dev-Index @Arguments } else { Dev-Index } }
        'backup'      { if ($Arguments) { Dev-Backup @Arguments } else { Dev-Backup } }
        'guard'       { if ($Arguments) { Dev-Guard @Arguments } else { Dev-Guard } }
        'env'         { if ($Arguments) { Dev-Env @Arguments } else { Dev-Env } }
        'maintenance' { if ($Arguments) { Dev-Maintenance @Arguments } else { Dev-Maintenance } }
        'agent'       { if ($Arguments) { Rtb-Agent @Arguments } else { Rtb-Agent } }
        'agy'         { Rtb-Agent -Agent 'agy' @Arguments }
        'claude'      { Rtb-Agent -Agent 'claude' @Arguments }
        'gemini'      { Rtb-Agent -Agent 'gemini' @Arguments }
        'codex'       { Rtb-Agent -Agent 'codex' @Arguments }
        'cursor'      { Rtb-Agent -Agent 'cursor' @Arguments }
        'windsurf'    { Rtb-Agent -Agent 'windsurf' @Arguments }
        'aider'       { Rtb-Agent -Agent 'aider' @Arguments }
        'openhands'   { Rtb-Agent -Agent 'openhands' @Arguments }
        'open'        { if ($Arguments) { Dev-Open @Arguments } else { Dev-Open } }
        'deps'        { if ($Arguments) { Rtb-Deps @Arguments } else { Rtb-Deps } }
        'workspace'   { if ($Arguments) { Rtb-Workspace @Arguments } else { Rtb-Workspace } }
        'upgrade'     { if ($Arguments) { Rtb-Upgrade @Arguments } else { Rtb-Upgrade } }
        'uninstall'   { if ($Arguments) { Rtb-Uninstall @Arguments } else { Rtb-Uninstall } }
        'doctor'      { if ($Arguments) { Rtb-Doctor @Arguments } else { Rtb-Doctor } }
        'status'      { if (-not (Invoke-RtbNativeRedirect 'status' $Arguments)) { if ($Arguments) { Rtb-Status @Arguments } else { Rtb-Status } } }
        'ui'          { Dev-Ui }
        'help'        { Dev-Help }
        default {
            Write-Host "Unknown command: $Command" -ForegroundColor Red
            Write-Host "Run 'rtb help' for available commands." -ForegroundColor Gray
        }
    }
}

function dev {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$Command,

        [Parameter(Position = 1, ValueFromRemainingArguments)]
        [string[]]$Arguments
    )

    if ($Command) {
        if ($Arguments) { rtb $Command @Arguments } else { rtb $Command }
    } else {
        rtb
    }
}

# Load tab completion
if (Test-Path (Join-Path $PSScriptRoot 'src\completions\rtb.completion.ps1')) {
    . (Join-Path $PSScriptRoot 'src\completions\rtb.completion.ps1')
} elseif (Test-Path (Join-Path $PSScriptRoot 'src\completions\dev.completion.ps1')) {
    . (Join-Path $PSScriptRoot 'src\completions\dev.completion.ps1')
}

# Export functions
Export-ModuleMember -Function *


