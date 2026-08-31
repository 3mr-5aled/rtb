function Rtb-Clean {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [object]$FirstArg,

        [Parameter(Position = 1, ValueFromRemainingArguments = $true)]
        [string[]]$RemainingArgs,

        [switch]$Commit,   # Actually delete — must be explicit
        [switch]$DryRun,   # Alias kept for backwards compat
        [int]$Days = 60
    )

    $allArgs = @()
    if ($null -ne $FirstArg) { $allArgs += "$FirstArg" }
    if ($RemainingArgs) { $allArgs += $RemainingArgs }

    $isCommit = $Commit.IsPresent -or ($allArgs -contains '-Commit') -or ($allArgs -contains '--commit') -or ($allArgs -contains '-c') -or ($allArgs -contains '-C')
    $isDryRunExplicit = $DryRun.IsPresent -or ($allArgs -contains '-DryRun') -or ($allArgs -contains '--dry-run') -or ($allArgs -contains '-n')

    # Resolve days threshold: check $Days parameter or parse numeric token / -Days / --days in arguments
    $resolvedDays = if ($PSBoundParameters.ContainsKey('Days')) { $Days } else { 60 }

    for ($i = 0; $i -lt $allArgs.Count; $i++) {
        $token = $allArgs[$i]
        if ($token -in @('-Days', '--days', '-d') -and ($i + 1) -lt $allArgs.Count) {
            $parsedDays = 0
            if ([int]::TryParse($allArgs[$i + 1], [ref]$parsedDays)) {
                $resolvedDays = $parsedDays
                $i++
            }
        } elseif ($token -match '^\d+$') {
            $resolvedDays = [int]$token
        }
    }

    $isDryRun = (-not $isCommit) -or $isDryRunExplicit
    $config = Get-RtbConfig
    $cutoff = (Get-Date).AddDays(-$resolvedDays)
    $targets = $config.cleanDeps.targets
    $searchPaths = @(
        (Get-RtbRootPath $config.projectRoots.active),
        (Get-RtbRootPath $config.projectRoots.paused),
        (Get-RtbRootPath $config.projectRoots.vibe),
        (Get-RtbRootPath $config.projectRoots.sandbox)
    )

    Write-RtbHeader "Dependency Pruning (${resolvedDays}d threshold)"
    if ($isDryRun) {
        Write-Host " [DRY RUN MODE] No files will be deleted. Use '-Commit' to perform deletion." -ForegroundColor Cyan
    }

    $flaggedItems = [System.Collections.Generic.List[PSCustomObject]]::new()
    $totalBytes = 0

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
                $size = (Get-ChildItem $_.FullName -Recurse -File -EA SilentlyContinue | Measure-Object -Property Length -Sum).Sum
                $totalBytes += $size
                $mb = [math]::Round($size / 1MB, 1)
                $flaggedItems.Add([PSCustomObject]@{
                    Path   = $_.FullName
                    Size   = $size
                    SizeMB = $mb
                })
                Write-Host "  $($_.FullName)" -NoNewline -ForegroundColor Yellow
                Write-Host " ($mb MB)" -ForegroundColor DarkGray
            }
    }

    $gb = [math]::Round($totalBytes / 1GB, 2)
    $flaggedCount = $flaggedItems.Count
    $suffix = if ($isDryRun) { '(dry run)' } else { 'flagged' }
    $fgColor = if ($isDryRun) { 'Yellow' } else { 'Green' }
    Write-Host "`n  Flagged: $flaggedCount folders | Space: $gb GB $suffix" -ForegroundColor $fgColor

    if (-not $isDryRun -and $flaggedCount -gt 0) {
        if (-not (Confirm-RtbAction -Message "Delete $flaggedCount dep folders ($gb GB)?")) {
            Write-Host '  Aborted.' -ForegroundColor Gray
            return
        }
        foreach ($item in $flaggedItems) {
            Remove-Item $item.Path -Recurse -Force -EA SilentlyContinue
            Write-Host "    -> DELETED: $($item.Path)" -ForegroundColor Green
        }
        Write-Host "`n  Clean complete. Space recovered: $gb GB" -ForegroundColor Green
    }
}

function Dev-Clean {
    Rtb-Clean @args
}

function Clear-RtbDependencies { Rtb-Clean @args }


