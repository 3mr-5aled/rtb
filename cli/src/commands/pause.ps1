function Dev-Pause {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$Name,

        [Parameter(Position = 1, ValueFromRemainingArguments = $true)]
        [string[]]$RemainingArgs,

        [switch]$Prune,
        [switch]$Force
    )

    $allArgs = @()
    if ($RemainingArgs) { $allArgs += $RemainingArgs }

    $isPrune = $Prune.IsPresent -or ($allArgs -contains '-Prune') -or ($allArgs -contains '--prune') -or ($allArgs -contains '-p') -or ($allArgs -contains '-P')
    $isForce = $Force.IsPresent -or ($allArgs -contains '-Force') -or ($allArgs -contains '--force') -or ($allArgs -contains '-f') -or ($allArgs -contains '-F')

    # If $Name was passed as a flag first (e.g., `rtb pause --prune my-proj` or `rtb pause -Force my-proj`)
    if ($Name -in @('-Prune', '--prune', '-p', '-P', '-Force', '--force', '-f', '-F')) {
        if ($Name -in @('-Prune', '--prune', '-p', '-P')) { $isPrune = $true }
        if ($Name -in @('-Force', '--force', '-f', '-F')) { $isForce = $true }
        $Name = $null
        foreach ($item in $RemainingArgs) {
            if ($item -notmatch '^-') {
                $Name = $item
                break
            }
        }
    }

    if (-not $Name) {
        Write-Host 'Usage: rtb pause <project-name> [--prune] [-Force]' -ForegroundColor Yellow
        return
    }

    $config = Get-RtbConfig
    $kebabName = $Name.ToLower() -replace '[^a-z0-9\-]', '-'
    $activePath = Join-Path $config.projectRoots.active $kebabName
    $pausedPath = Join-Path $config.projectRoots.paused $kebabName

    if (-not (Test-Path $activePath)) {
        Write-Host "  Project '$kebabName' not found in Active!" -ForegroundColor Red
        return
    }

    # ── Git safety check ───────────────────────────────────────────────────
    if (-not (Test-GitClean -ProjectPath $activePath)) {
        Write-Host '  ⚠ WARNING: This project has uncommitted git changes!' -ForegroundColor Red
        Write-Host '  Commit or stash first, or pass -Force to override.' -ForegroundColor Yellow
        if (-not $isForce) {
            Write-Host '  Aborting.' -ForegroundColor Red
            return
        }
    }

    Write-RtbHeader "Pausing: $kebabName"

    if ($isPrune) {
        $shouldPrune = $isForce
        if (-not $shouldPrune) {
            $shouldPrune = Confirm-RtbAction -Message 'Prune dependency folders (node_modules, target, .venv)?'
        }
        if ($shouldPrune) {
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
        } else {
            Write-Host '  Prune skipped.' -ForegroundColor Gray
        }
    }

    $pausedParent = Split-Path $pausedPath -Parent
    if (-not (Test-Path $pausedParent)) {
        New-Item -Path $pausedParent -ItemType Directory -Force | Out-Null
    }

    Move-Item -Path $activePath -Destination $pausedPath -Force
    Write-Host "  '$kebabName' moved to Paused ($pausedPath)" -ForegroundColor Green
}

function Rtb-Pause { Dev-Pause @args }
function Suspend-RtbProject { Dev-Pause @args }


