function Dev-Pause {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$Name
    )

    $prune = $args -contains '--prune'

    if (-not $Name -or $Name -eq '--prune') {
        Write-Host 'Usage: dev pause <project-name> [--prune]' -ForegroundColor Yellow
        return
    }

    $config = Get-DevConfig
    $kebabName = $Name.ToLower() -replace '[^a-z0-9\-]', '-'
    $activePath = Join-Path $config.projectRoots.active $kebabName
    $pausedPath = Join-Path $config.projectRoots.paused $kebabName

    if (-not (Test-Path $activePath)) {
        Write-Host "  Project '$kebabName' not found in Active!" -ForegroundColor Red
        return
    }

    Write-RtbHeader "Pausing: $kebabName"

    if ($prune) {
        Write-Host '  Pruning dependencies...' -ForegroundColor Yellow
        $targets = $config.cleanDeps.targets
        foreach ($t in $targets) {
            $depPath = Join-Path $activePath $t
            if (Test-Path $depPath) {
                $size = [math]::Round(((Get-ChildItem $depPath -Recurse -File -EA SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1MB), 1)
                Remove-Item -Path $depPath -Recurse -Force -EA SilentlyContinue
                Write-Host "    Removed $t ($size MB)" -ForegroundColor Gray
            }
        }
    }

    Move-Item -Path $activePath -Destination $pausedPath -Force
    Write-Host "  '$kebabName' moved to 04-Paused" -ForegroundColor Green
}
