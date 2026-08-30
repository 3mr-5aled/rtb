<#
.SYNOPSIS
    Rtb-Agent — AI Agent Discovery & Launcher for RTB projects.
.DESCRIPTION
    Discovers installed AI agent CLIs (agy, claude, gemini, codex, cursor, windsurf, aider, openhands) in system PATH,
    generates project context summary, and launches the target agent in the project directory.
#>

function Get-InstalledAgents {
    [CmdletBinding()]
    param()

    $agents = @(
        @{ Name = 'Google Antigravity'; Command = 'agy' },
        @{ Name = 'Claude Code';        Command = 'claude' },
        @{ Name = 'Gemini CLI';         Command = 'gemini' },
        @{ Name = 'Codex CLI';          Command = 'codex' },
        @{ Name = 'Cursor';             Command = 'cursor' },
        @{ Name = 'Windsurf';           Command = 'windsurf' },
        @{ Name = 'Aider';              Command = 'aider' },
        @{ Name = 'OpenHands';          Command = 'openhands' }
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

function New-RtbAgentContextFile {
    param(
        [Parameter(Mandatory = $true)][string]$ProjectPath,
        [string]$ProjectName = "",
        [string[]]$Stack = @(),
        [string]$Status = "Active",
        [string]$GitBranch = "",
        [string]$ReadmePreview = ""
    )

    if (-not (Test-Path $ProjectPath)) { return $null }
    $name = if ($ProjectName) { $ProjectName } else { Split-Path $ProjectPath -Leaf }
    $contextPath = Join-Path $ProjectPath ".rtb_context.md"
    $stackStr = if ($Stack -and $Stack.Count -gt 0 -and $Stack[0] -ne '-') { $Stack -join ', ' } else { 'Unknown' }
    $branchStr = if ($GitBranch -and $GitBranch -ne '-') { $GitBranch } else { 'unknown' }

    # Git Context
    $gitLogLines = '  (not a git repository)'
    $gitDiffStat = '  (not a git repository)'
    if (Test-Path (Join-Path $ProjectPath '.git')) {
        $logRaw = git -C $ProjectPath log --oneline -10 2>$null
        $gitLogLines = if ($logRaw) { ($logRaw | ForEach-Object { "  $_" }) -join "`n" } else { '  (no commits)' }
        $diffRaw = git -C $ProjectPath diff --stat HEAD 2>$null
        $gitDiffStat = if ($diffRaw -and $diffRaw.Trim()) { ($diffRaw | ForEach-Object { "  $_" }) -join "`n" } else { '  (working tree clean)' }
    }

    # Dependencies
    $depsSection = ''
    $pkgPath = Join-Path $ProjectPath 'package.json'
    if (Test-Path $pkgPath) {
        try {
            $pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json
            if ($pkg.dependencies) {
                $deps = ($pkg.dependencies.PSObject.Properties.Name | Select-Object -First 20) -join ', '
                if ($deps) { $depsSection += "**package.json deps:** $deps`n" }
            }
            if ($pkg.devDependencies) {
                $devDeps = ($pkg.devDependencies.PSObject.Properties.Name | Select-Object -First 10) -join ', '
                if ($devDeps) { $depsSection += "**devDependencies:** $devDeps`n" }
            }
        } catch { $depsSection += "(could not parse package.json)`n" }
    }
    if (Test-Path (Join-Path $ProjectPath 'Cargo.toml')) {
        $cargoContent = Get-Content (Join-Path $ProjectPath 'Cargo.toml') -Raw
        $crates = ([regex]::Matches($cargoContent, '^\s*(\w[\w-]*)\s*=', 'Multiline') |
            Select-Object -First 20 | ForEach-Object { $_.Groups[1].Value }) -join ', '
        if ($crates) {
            $depsSection += "**Cargo.toml crates:** $crates`n"
        }
    }
    if (Test-Path (Join-Path $ProjectPath 'requirements.txt')) {
        $reqs = (Get-Content (Join-Path $ProjectPath 'requirements.txt') |
            Where-Object { $_.Trim() -and -not $_.Trim().StartsWith('#') } |
            Select-Object -First 20) -join ', '
        if ($reqs) {
            $depsSection += "**requirements.txt:** $reqs`n"
        }
    }
    if (Test-Path (Join-Path $ProjectPath 'go.mod')) {
        $goMods = (Get-Content (Join-Path $ProjectPath 'go.mod') |
            Where-Object { $_ -match '^\s+\S+\s+v' } | ForEach-Object { $_.Trim() } | Select-Object -First 20) -join ', '
        if ($goMods) {
            $depsSection += "**go.mod requires:** $goMods`n"
        }
    }
    if (-not $depsSection -or -not $depsSection.Trim()) { $depsSection = "(no recognised dependency manifest found)`n" }

    $readmeStr = if ($ReadmePreview -and $ReadmePreview.Trim()) { $ReadmePreview } else { '(no README)' }

    $content = @"
# RTB Agent Workspace Context: $name

## Project Info
- **Project Path**: $ProjectPath
- **Status**: $Status
- **Detected Stack**: $stackStr
- **Git Branch**: $branchStr
- **Generated At**: $((Get-Date).ToString('o'))

## README Preview
$readmeStr

## Git Context

### Last 10 Commits
$gitLogLines

### Current Diff (--stat HEAD)
$gitDiffStat

## Dependencies
$depsSection
"@

    Set-Content -Path $contextPath -Value $content -Force
    return $contextPath
}

function Rtb-Agent {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$ProjectName,

        [Parameter(Position = 1)]
        [string]$Agent,

        [Switch]$List,

        [Switch]$Agy,
        [Switch]$Claude,
        [Switch]$Gemini,
        [Switch]$Codex,
        [Switch]$Cursor,
        [Switch]$Windsurf,
        [Switch]$Aider,
        [Switch]$OpenHands
    )

    # Flag parsing precedence and agent name normalization
    if ($Agent) {
        $Agent = $Agent.TrimStart('-').ToLower()
    } else {
        if ($Agy)           { $Agent = 'agy' }
        elseif ($Claude)    { $Agent = 'claude' }
        elseif ($Gemini)    { $Agent = 'gemini' }
        elseif ($Codex)     { $Agent = 'codex' }
        elseif ($Cursor)    { $Agent = 'cursor' }
        elseif ($Windsurf)  { $Agent = 'windsurf' }
        elseif ($Aider)     { $Agent = 'aider' }
        elseif ($OpenHands) { $Agent = 'openhands' }
    }

    $installedAgents = Get-InstalledAgents

    if ($List) {
        Write-RtbHeader -Title "Installed AI Agents"
        Write-Host ""
        foreach ($a in $installedAgents) {
            $statusStr = if ($a.installed) { "[Installed]" } else { "[Not Found]" }
            $color = if ($a.installed) { "Green" } else { "DarkGray" }
            Write-Host ("  {0,-20} ({1,-10}) {2}" -f $a.name, $a.command, $statusStr) -ForegroundColor $color
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
        Write-Host "No installed AI agent found in PATH (agy, claude, gemini, codex, cursor, windsurf, aider, openhands)." -ForegroundColor Red
        Write-Host "Run 'rtb agent -List' to check agent status." -ForegroundColor Gray
        return
    }

    # Generate project context summary
    $details = Get-ProjectDetails -ProjectPath $targetPath -Status 'Active'
    $gitBranch = if ($details.git) { $details.git.branch } else { "" }
    New-RtbAgentContextFile -ProjectPath $targetPath -ProjectName $targetName -Stack $details.stack -Status $details.status -GitBranch $gitBranch -ReadmePreview $details.readme_preview | Out-Null

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
    Write-Host "  Context File:  .rtb_context.md" -ForegroundColor DarkCyan
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

# Shorthand Agent Launchers
function Rtb-Agy { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'agy' }
function Dev-Agy { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'agy' }

function Rtb-Claude { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'claude' }
function Dev-Claude { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'claude' }

function Rtb-Gemini { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'gemini' }
function Dev-Gemini { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'gemini' }

function Rtb-Codex { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'codex' }
function Dev-Codex { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'codex' }

function Rtb-Cursor { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'cursor' }
function Dev-Cursor { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'cursor' }

function Rtb-Windsurf { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'windsurf' }
function Dev-Windsurf { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'windsurf' }

function Rtb-Aider { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'aider' }
function Dev-Aider { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'aider' }

function Rtb-OpenHands { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'openhands' }
function Dev-OpenHands { param([string]$ProjectName) Rtb-Agent -ProjectName $ProjectName -Agent 'openhands' }

