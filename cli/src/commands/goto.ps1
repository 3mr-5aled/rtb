function Dev-Goto {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$Name,

        [Parameter(Position = 1)]
        [string]$Agent,

        [Parameter()]
        [string]$Choice,

        [Switch]$Agy,
        [Switch]$Claude,
        [Switch]$Gemini,
        [Switch]$Codex,
        [Switch]$Cursor,
        [Switch]$Windsurf,
        [Switch]$Aider,
        [Switch]$OpenHands
    )

    # Handle agent flag passed before project name (e.g. 'rtb goto -Claude <project>')
    $knownAgents = @('agy', 'claude', 'gemini', 'codex', 'cursor', 'windsurf', 'aider', 'openhands')
    if ($Name -and $Name.StartsWith('-')) {
        $candidateAgent = $Name.TrimStart('-').ToLower()
        if ($candidateAgent -in $knownAgents) {
            $realProject = if ($Agent -and -not $Agent.StartsWith('-')) { $Agent } else { $null }
            $Agent = $candidateAgent
            $Name = $realProject
        }
    }

    # Resolve and normalize agent parameter
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

    if (-not $Name) {
        Write-Host 'Usage: rtb goto <project-name> [--agy|--claude|...]' -ForegroundColor Yellow
        Write-Host 'Tip: Tab after "rtb goto " to see all projects.' -ForegroundColor Gray
        return
    }

    $matches = @(Find-ProjectPathFuzzy -Query $Name)

    if ($matches.Count -eq 0) {
        Write-Host "  No project matching '$Name' found." -ForegroundColor Red
        Write-Host '  Available projects:' -ForegroundColor Gray
        Get-AllProjectNames | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
        return
    }

    $target = $null
    if ($matches.Count -eq 1 -or ($matches[0].Score -eq 100 -and $matches[1].Score -lt 100)) {
        $target = $matches[0]
    } else {
        Write-Host ''
        Write-Host "  Multiple projects match '$Name':" -ForegroundColor Yellow
        $limit = [Math]::Min($matches.Count, 9)
        for ($i = 0; $i -lt $limit; $i++) {
            $m = $matches[$i]
            Write-Host ("  [{0}] {1,-35} ({2})" -f ($i + 1), $m.Name, $m.Status) -ForegroundColor Cyan
        }
        Write-Host ''
        if (-not $Choice) {
            Write-Host -NoNewline '  Select [1-9] or Enter to cancel: ' -ForegroundColor Yellow
            $Choice = Read-Host
        }
        $sel = 0
        if ([int]::TryParse($Choice, [ref]$sel)) {
            $idx = $sel - 1
            if (($idx -ge 0) -and ($idx -lt $limit)) {
                $target = $matches[$idx]
            }
        }
    }

    if (-not $target) {
        Write-Host '  Cancelled.' -ForegroundColor Gray
        return
    }

    Set-Location $target.Path
    Write-Host "  $($target.Status) » $($target.Path)" -ForegroundColor Green

    if ($Agent) {
        Rtb-Agent -ProjectName $target.Path -Agent $Agent
    }
}

function Rtb-Goto { Dev-Goto @args }
function Set-RtbLocation { Dev-Goto @args }
