function Dev-Deploy {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$Name
    )

    $target = 'production'
    if ($args -contains '--staging') { $target = 'staging' }
    if ($args -contains '--prod') { $target = 'production' }

    if (-not $Name -or $Name -like '--*') {
        Write-Host 'Usage: dev deploy <project-name> [--prod|--staging]' -ForegroundColor Yellow
        return
    }

    $config = Get-DevConfig
    $kebabName = $Name.ToLower() -replace '[^a-z0-9\-]', '-'
    $activePath = Join-Path $config.projectRoots.active $kebabName
    $deployRoot = if ($target -eq 'production') { $config.projectRoots.production } else { $config.projectRoots.staging }
    $deployPath = Join-Path $deployRoot $kebabName

    if (-not (Test-Path $activePath)) {
        Write-Host "  Project '$kebabName' not found in Active!" -ForegroundColor Red
        return
    }

    Write-RtbHeader "Deploying: $kebabName → $target"
    Move-Item -Path $activePath -Destination $deployPath -Force
    Write-Host "  '$kebabName' deployed to $target!" -ForegroundColor Green
}
