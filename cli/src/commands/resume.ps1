function Dev-Resume {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$Name
    )

    $install = $args -contains '--install'

    if (-not $Name -or $Name -eq '--install') {
        Write-Host 'Usage: dev resume <project-name> [--install]' -ForegroundColor Yellow
        return
    }

    $config = Get-DevConfig
    $kebabName = $Name.ToLower() -replace '[^a-z0-9\-]', '-'
    $pausedPath = Join-Path $config.projectRoots.paused $kebabName
    $activePath = Join-Path $config.projectRoots.active $kebabName

    if (-not (Test-Path $pausedPath)) {
        Write-Host "  Project '$kebabName' not found in Paused!" -ForegroundColor Red
        return
    }

    Write-RtbHeader "Resuming: $kebabName"
    Move-Item -Path $pausedPath -Destination $activePath -Force
    Write-Host "  '$kebabName' moved to 01-Active" -ForegroundColor Green

    if ($install) {
        Push-Location $activePath
        if (Test-Path 'package.json') {
            Write-Host '  Running npm install...' -ForegroundColor Gray
            npm install 2>&1 | Out-Null
            Write-Host '  npm install complete!' -ForegroundColor Green
        } elseif (Test-Path 'requirements.txt') {
            Write-Host '  Running pip install...' -ForegroundColor Gray
            pip install -r requirements.txt 2>&1 | Out-Null
            Write-Host '  pip install complete!' -ForegroundColor Green
        }
        Pop-Location
    }

    Write-Host "  Run: dev goto $kebabName" -ForegroundColor Cyan
}


function Resume-RtbProject { Dev-Resume @args }
