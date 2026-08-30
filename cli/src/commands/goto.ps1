function Dev-Goto {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$Name,

        [Parameter(Position = 1)]
        [string]$Agent,

        [Switch]$Agy,
        [Switch]$Claude,
        [Switch]$Gemini,
        [Switch]$Codex,
        [Switch]$Cursor,
        [Switch]$Windsurf,
        [Switch]$Aider,
        [Switch]$OpenHands
    )

    if (-not $Name) {
        Write-Host 'Usage: dev goto <project-name> [--agy|--claude|--gemini|...]' -ForegroundColor Yellow
        Write-Host 'Tip: Press TAB after "dev goto " to see all projects and agent options.' -ForegroundColor Gray
        return
    }

    # Resolve agent parameter
    if (-not $Agent) {
        if ($Agy)       { $Agent = 'agy' }
        elseif ($Claude)    { $Agent = 'claude' }
        elseif ($Gemini)    { $Agent = 'gemini' }
        elseif ($Codex)     { $Agent = 'codex' }
        elseif ($Cursor)    { $Agent = 'cursor' }
        elseif ($Windsurf)  { $Agent = 'windsurf' }
        elseif ($Aider)     { $Agent = 'aider' }
        elseif ($OpenHands) { $Agent = 'openhands' }
    }

    $result = Find-ProjectPath -Name $Name
    if ($result) {
        Set-Location $result.Path
        Write-Host "  $($result.Status) » $($result.Path)" -ForegroundColor Green

        if ($Agent) {
            Rtb-Agent -ProjectName $result.Path -Agent $Agent
        }
    } else {
        Write-Host "  Project '$Name' not found." -ForegroundColor Red
        Write-Host '  Available projects:' -ForegroundColor Gray
        Get-AllProjectNames | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
    }
}


function Set-RtbLocation { Dev-Goto @args }
