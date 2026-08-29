function Dev-Goto {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$Name
    )

    if (-not $Name) {
        Write-Host 'Usage: dev goto <project-name>' -ForegroundColor Yellow
        Write-Host 'Tip: Press TAB after "dev goto " to see all projects.' -ForegroundColor Gray
        return
    }

    $result = Find-ProjectPath -Name $Name
    if ($result) {
        Set-Location $result.Path
        Write-Host "  $($result.Status) » $($result.Path)" -ForegroundColor Green
    } else {
        Write-Host "  Project '$Name' not found." -ForegroundColor Red
        Write-Host '  Available projects:' -ForegroundColor Gray
        Get-AllProjectNames | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
    }
}


function Set-RtbLocation { Dev-Goto @args }
