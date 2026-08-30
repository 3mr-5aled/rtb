function Dev-Archive {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$Name,

        [Parameter(Position = 1, ValueFromRemainingArguments = $true)]
        [string[]]$RemainingArgs,

        [switch]$Force
    )

    $allArgs = @()
    if ($RemainingArgs) { $allArgs += $RemainingArgs }

    $isForce = $Force.IsPresent -or ($allArgs -contains '-Force') -or ($allArgs -contains '--force') -or ($allArgs -contains '-f') -or ($allArgs -contains '-F')

    # If $Name was supplied as a flag first (e.g. `rtb archive -Force my-proj` or `rtb archive --force my-proj`)
    if ($Name -in @('-Force', '--force', '-f', '-F')) {
        $isForce = $true
        $Name = $null
        foreach ($item in $RemainingArgs) {
            if ($item -notmatch '^-') {
                $Name = $item
                break
            }
        }
    }

    if (-not $Name) {
        Write-Host 'Usage: rtb archive <project-name> [-Force]' -ForegroundColor Yellow
        return
    }

    $result = Find-ProjectPath -Name $Name
    if (-not $result) {
        Write-Host "  Project '$Name' not found!" -ForegroundColor Red
        return
    }

    # ── Git safety check ───────────────────────────────────────────────────
    if (-not (Test-GitClean -ProjectPath $result.Path)) {
        Write-Host '  ⚠ WARNING: This project has uncommitted git changes!' -ForegroundColor Red
        Write-Host '  Commit or stash your changes first, or pass -Force to override.' -ForegroundColor Yellow
        if (-not $isForce) {
            Write-Host '  Aborting.' -ForegroundColor Red
            return
        }
    }

    # ── Confirmation prompt ────────────────────────────────────────────────
    $projectName = Split-Path $result.Path -Leaf
    $config = Get-RtbConfig
    Write-Host ''
    Write-Host '  This will:' -ForegroundColor Cyan
    Write-Host '    1. Prune dep folders (node_modules, target, .venv, etc.)' -ForegroundColor Gray
    Write-Host "    2. Create a .tar.gz in $($config.backupRoot)" -ForegroundColor Gray
    Write-Host "    3. PERMANENTLY DELETE: $($result.Path)" -ForegroundColor Red
    Write-Host ''
    if (-not $isForce) {
        if (-not (Confirm-RtbAction -Message "Archive and delete '$projectName'?")) {
            Write-Host '  Aborted.' -ForegroundColor Gray
            return
        }
    }

    $snapshotDir = Join-Path $config.backupRoot 'project-snapshots'
    New-Item -Path $snapshotDir -ItemType Directory -Force | Out-Null

    $timestamp = Get-Date -Format 'yyyy-MM-dd'
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
    $tarOutput = tar -czf $archivePath $projectName 2>&1
    $tarExitCode = $LASTEXITCODE
    Pop-Location

    # ── Post-tar safety verification ─────────────────────────────────────────
    if ($tarExitCode -eq 0 -and (Test-Path $archivePath) -and ((Get-Item $archivePath).Length -gt 0)) {
        $sizeMB = [math]::Round((Get-Item $archivePath).Length / 1MB, 2)
        Remove-Item -Path $result.Path -Recurse -Force
        Write-Host "  Archived: $archiveName ($sizeMB MB)" -ForegroundColor Green
        Write-Host "  Location: $archivePath" -ForegroundColor Gray
        Write-Host "  Original folder removed." -ForegroundColor Gray
        Write-Host "`n  To restore: rtb unarchive $archiveName" -ForegroundColor Cyan
    } else {
        Write-Host '  Archive creation FAILED — source folder was NOT deleted.' -ForegroundColor Red
        if (Test-Path $archivePath) {
            Remove-Item $archivePath -Force -EA SilentlyContinue
        }
    }
}

function Rtb-Archive { Dev-Archive @args }
function Compress-RtbProject { Dev-Archive @args }


