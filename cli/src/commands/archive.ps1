function Dev-Archive {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$Name
    )

    if (-not $Name) {
        Write-Host 'Usage: dev archive <project-name>' -ForegroundColor Yellow
        return
    }

    $result = Find-ProjectPath -Name $Name
    if (-not $result) {
        Write-Host "  Project '$Name' not found!" -ForegroundColor Red
        return
    }

    $config = Get-DevConfig
    $snapshotDir = Join-Path $config.backupRoot 'project-snapshots'
    New-Item -Path $snapshotDir -ItemType Directory -Force | Out-Null

    $timestamp = Get-Date -Format 'yyyy-MM-dd'
    $projectName = Split-Path $result.Path -Leaf
    $archiveName = "$projectName-$timestamp.tar.gz"
    $archivePath = Join-Path $snapshotDir $archiveName

    Write-RtbHeader "Archiving: $projectName"

    # Prune heavy folders first
    Write-Host '  Pruning dependencies before archiving...' -ForegroundColor Gray
    $targets = $config.cleanDeps.targets
    foreach ($t in $targets) {
        $depPath = Join-Path $result.Path $t
        if (Test-Path $depPath) {
            Remove-Item -Path $depPath -Recurse -Force -EA SilentlyContinue
            Write-Host "    Removed $t" -ForegroundColor DarkGray
        }
    }

    # Create tar.gz archive
    $parentDir = Split-Path $result.Path -Parent
    Write-Host '  Compressing...' -ForegroundColor Gray
    Push-Location $parentDir
    tar -czf $archivePath $projectName 2>&1 | Out-Null
    Pop-Location

    if (Test-Path $archivePath) {
        $sizeMB = [math]::Round((Get-Item $archivePath).Length / 1MB, 2)
        Remove-Item -Path $result.Path -Recurse -Force
        Write-Host "  Archived: $archiveName ($sizeMB MB)" -ForegroundColor Green
        Write-Host "  Location: $archivePath" -ForegroundColor Gray
        Write-Host "  Original folder removed." -ForegroundColor Gray
        Write-Host "`n  To restore: dev unarchive $archiveName" -ForegroundColor Cyan
    } else {
        Write-Host '  Archive creation failed!' -ForegroundColor Red
    }
}
