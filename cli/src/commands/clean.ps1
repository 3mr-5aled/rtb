function Rtb-Clean {
    [CmdletBinding()]
    param(
        [switch]$Force,
        [switch]$DryRun,
        [int]$Days = 60
    )

    $isDryRun = $DryRun -or (-not $Force) -or ($args -contains '--dry-run')
    $config = Get-RtbConfig
    $cutoff = (Get-Date).AddDays(-$Days)
    $targets = $config.cleanDeps.targets
    $searchPaths = @($config.projectRoots.active, $config.projectRoots.paused, $config.projectRoots.vibe, $config.projectRoots.sandbox)

    Write-RtbHeader "Dependency Pruning (${Days}d threshold)"
    if ($isDryRun) {
        Write-Host " [DRY RUN MODE] No files will be deleted. Use '-Force' to perform deletion." -ForegroundColor Cyan
    }

    $flagged = 0; $totalBytes = 0

    foreach ($base in $searchPaths) {
        if (-not $base -or -not (Test-Path $base)) { continue }
        Get-ChildItem -Path $base -Recurse -Directory -EA SilentlyContinue |
            Where-Object { 
                $targets -contains $_.Name -and 
                $_.Name -ne '.git' -and 
                $_.FullName -notmatch '\\\.git($|\\)' -and
                $_.LastWriteTime -lt $cutoff 
            } |
            ForEach-Object {
                $flagged++
                $size = (Get-ChildItem $_.FullName -Recurse -File -EA SilentlyContinue | Measure-Object -Property Length -Sum).Sum
                $totalBytes += $size
                $mb = [math]::Round($size / 1MB, 1)
                Write-Host "  $($_.FullName)" -NoNewline -ForegroundColor Yellow
                Write-Host " ($mb MB)" -ForegroundColor DarkGray
                if (-not $isDryRun) {
                    Remove-Item $_.FullName -Recurse -Force -EA SilentlyContinue
                    Write-Host '    → DELETED' -ForegroundColor Green
                }
            }
    }
    $gb = [math]::Round($totalBytes / 1GB, 2)
    Write-Host "`n  Flagged: $flagged folders | Space: $gb GB $(if($isDryRun){'(dry run)'}else{'recovered'})" -ForegroundColor $(if($isDryRun){'Yellow'}else{'Green'})
}

function Dev-Clean {
    Rtb-Clean @args
}
