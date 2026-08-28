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
        Write-Host "RTB (ﺐﺘّﺭ) CLI v1.0.0" -ForegroundColor Green
        return
    }

    switch ($Command.ToLower()) {
        'init'        { if ($Arguments) { Rtb-Init @Arguments } else { Rtb-Init } }
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
        'list'        { if ($Arguments) { Dev-List @Arguments } else { Dev-List } }
        'info'        { if ($Arguments) { Rtb-Info @Arguments } else { Rtb-Info } }
        'health'      { if ($Arguments) { Dev-Health @Arguments } else { Dev-Health } }
        'clean'       { if ($Arguments) { Dev-Clean @Arguments } else { Dev-Clean } }
        'index'       { if ($Arguments) { Dev-Index @Arguments } else { Dev-Index } }
        'backup'      { if ($Arguments) { Dev-Backup @Arguments } else { Dev-Backup } }
        'guard'       { if ($Arguments) { Dev-Guard @Arguments } else { Dev-Guard } }
        'env'         { if ($Arguments) { Dev-Env @Arguments } else { Dev-Env } }
        'maintenance' { if ($Arguments) { Dev-Maintenance @Arguments } else { Dev-Maintenance } }
        'ui'          { Dev-Ui }
        'help'        { Dev-Help }
        default {
            Write-Host "Unknown command: $Command" -ForegroundColor Red
            Write-Host "Run 'rtb help' for available commands." -ForegroundColor Gray
        }
    }
}

function dev {
    rtb @args
}

# Load tab completion
if (Test-Path (Join-Path $PSScriptRoot 'src\completions\rtb.completion.ps1')) {
    . (Join-Path $PSScriptRoot 'src\completions\rtb.completion.ps1')
} elseif (Test-Path (Join-Path $PSScriptRoot 'src\completions\dev.completion.ps1')) {
    . (Join-Path $PSScriptRoot 'src\completions\dev.completion.ps1')
}

# Export functions
Export-ModuleMember -Function 'rtb', 'dev', 'Rtb-Info', 'Dev-Info', 'Rtb-List', 'Dev-List', 'Get-ProjectDetails', 'Get-AllProjectsDetails', 'Get-AllProjectNames', 'Get-ProjectsByStatus', 'Find-ProjectPath', 'Get-RtbConfig', 'Get-DevConfig'

