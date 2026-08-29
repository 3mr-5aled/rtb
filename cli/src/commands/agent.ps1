<#
.SYNOPSIS
    Rtb-Agent — AI Agent Discovery & Launcher for RTB projects.
.DESCRIPTION
    Discovers installed AI agent CLIs (agy, claude, gemini, codex) in system PATH,
    generates project context summary, and launches the target agent in the project directory.
#>

function Get-InstalledAgents {
    [CmdletBinding()]
    param()

    $agents = @(
        @{ Name = 'Google Antigravity'; Command = 'agy' },
        @{ Name = 'Claude Code';        Command = 'claude' },
        @{ Name = 'Gemini CLI';         Command = 'gemini' },
        @{ Name = 'Codex CLI';          Command = 'codex' }
    )

    $result = @()
    foreach ($a in $agents) {
        $cmd = Get-Command -Name $a.Command -ErrorAction SilentlyContinue
        $installed = [bool]($cmd)
        $result += [PSCustomObject]@{
            name      = $a.Name
            command   = $a.Command
            installed = $installed
        }
    }

    return $result
}

function Rtb-Agent {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$ProjectName,

        [Parameter(Position = 1)]
        [string]$Agent,

        [Switch]$List
    )

    $installedAgents = Get-InstalledAgents

    if ($List) {
        Write-RtbHeader -Title "Installed AI Agents"
        Write-Host ""
        foreach ($a in $installedAgents) {
            $statusStr = if ($a.installed) { "[Installed]" } else { "[Not Found]" }
            $color = if ($a.installed) { "Green" } else { "DarkGray" }
            Write-Host ("  {0,-20} ({1,-8}) {2}" -f $a.name, $a.command, $statusStr) -ForegroundColor $color
        }
        Write-Host ""
        return $installedAgents
    }

    # Resolve project path
    $targetPath = Get-Location
    $targetName = Split-Path $targetPath -Leaf

    if ($ProjectName) {
        $projMatch = Find-ProjectPath -Name $ProjectName
        if ($projMatch) {
            $targetPath = $projMatch.Path
            $targetName = $projMatch.Name
        } else {
            if (Test-Path $ProjectName) {
                $targetPath = (Resolve-Path $ProjectName).Path
                $targetName = Split-Path $targetPath -Leaf
            } else {
                Write-Host "Project or path '$ProjectName' not found." -ForegroundColor Red
                return
            }
        }
    }

    # Resolve target agent
    $selectedAgent = $null
    if ($Agent) {
        $selectedAgent = $installedAgents | Where-Object { $_.command -eq $Agent -or $_.name -like "*$Agent*" } | Select-Object -First 1
        if (-not $selectedAgent) {
            Write-Host "Specified agent '$Agent' is not recognized." -ForegroundColor Red
            return
        }
        if (-not $selectedAgent.installed) {
            Write-Host "Agent '$($selectedAgent.name)' ($($selectedAgent.command)) is not installed or not in PATH." -ForegroundColor Red
            return
        }
    } else {
        # Default logic: prefer 'agy' if installed, otherwise first available installed agent
        $selectedAgent = $installedAgents | Where-Object { $_.command -eq 'agy' -and $_.installed } | Select-Object -First 1
        if (-not $selectedAgent) {
            $selectedAgent = $installedAgents | Where-Object { $_.installed } | Select-Object -First 1
        }
    }

    if (-not $selectedAgent -or -not $selectedAgent.installed) {
        Write-Host "No installed AI agent found in PATH (agy, claude, gemini, codex)." -ForegroundColor Red
        Write-Host "Run 'rtb agent -List' to check agent status." -ForegroundColor Gray
        return
    }

    # Generate project context summary
    $details = Get-ProjectDetails -ProjectPath $targetPath -Status 'Active'

    Write-RtbHeader -Title "Launching AI Agent: $($selectedAgent.name) ($($selectedAgent.command))"
    Write-Host ""
    Write-Host "  Project Name:  $targetName" -ForegroundColor White
    Write-Host "  Project Path:  $targetPath" -ForegroundColor Gray
    if ($details.stack) {
        Write-Host "  Stack:         $($details.stack -join ', ')" -ForegroundColor Yellow
    }
    if ($details.git) {
        Write-Host "  Git Branch:    $($details.git.branch)" -ForegroundColor Cyan
    }
    Write-Host "  Status:        $($details.status)" -ForegroundColor White
    if ($details.readme_preview) {
        Write-Host "  README:        $($details.readme_preview.Split("`n")[0])" -ForegroundColor DarkGray
    }
    Write-Host ""
    Write-Host "Launching process '$($selectedAgent.command)' in $targetPath..." -ForegroundColor Green

    Push-Location $targetPath
    try {
        & $selectedAgent.command
    } finally {
        Pop-Location
    }
}

function Dev-Agent {
    Rtb-Agent @args
}


function Invoke-RtbAgent { Rtb-Agent @args }
