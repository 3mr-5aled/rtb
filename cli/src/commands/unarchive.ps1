function Dev-Unarchive {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$ArchiveName
    )

    if (-not $ArchiveName) {
        Write-Host 'Usage: dev unarchive <archive-name.tar.gz>' -ForegroundColor Yellow
        return
    }

    $config = Get-DevConfig
    $snapshotDir = Join-Path $config.backupRoot 'project-snapshots'
    $archivePath = Join-Path $snapshotDir $ArchiveName

    # Try finding by partial name
    if (-not (Test-Path $archivePath)) {
        $match = Get-ChildItem $snapshotDir -Filter "*$ArchiveName*" -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($match) { $archivePath = $match.FullName; $ArchiveName = $match.Name }
    }

    if (-not (Test-Path $archivePath)) {
        Write-Host "  Archive '$ArchiveName' not found in $snapshotDir" -ForegroundColor Red
        return
    }

    Write-RtbHeader "Unarchiving: $ArchiveName"
    $activeDir = $config.projectRoots.active

    Push-Location $activeDir
    tar -xzf $archivePath 2>&1 | Out-Null
    Pop-Location

    Write-Host "  Extracted to: $activeDir" -ForegroundColor Green
    Write-Host '  Run: dev list --active' -ForegroundColor Cyan
}
