<#
.SYNOPSIS
    Rtb-Open — Open project directory in File Explorer.
.DESCRIPTION
    Resolves project path and opens it in Windows File Explorer (or native OS file manager).
#>

function Rtb-Open {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$ProjectName
    )

    $targetPath = Get-Location
    $targetName = Split-Path $targetPath -Leaf

    if ($ProjectName) {
        $projMatch = Find-ProjectPath -Name $ProjectName
        if ($projMatch) {
            $targetPath = $projMatch.Path
            $targetName = $projMatch.Name
        } else {
            if (Test-Path $ProjectName) {
                $targetPath = (Resolve-Path $ProjectName).Path
                $targetName = Split-Path $targetPath -Leaf
            } else {
                Write-Host "Project or path '$ProjectName' not found." -ForegroundColor Red
                Write-Host 'Available projects:' -ForegroundColor Gray
                Get-AllProjectNames | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
                return
            }
        }
    }

    Write-Host "Opening project '$targetName' in file explorer..." -ForegroundColor Green
    Write-Host "  Path: $targetPath" -ForegroundColor Gray

    try {
        if ($IsWindows -or $env:OS -like '*Windows*') {
            Start-Process 'explorer.exe' -ArgumentList "`"$targetPath`""
        } elseif ($IsMacOS) {
            Start-Process 'open' -ArgumentList "`"$targetPath`""
        } else {
            Start-Process 'xdg-open' -ArgumentList "`"$targetPath`""
        }
    } catch {
        Invoke-Item $targetPath
    }
}

function Dev-Open {
    Rtb-Open @args
}

function Invoke-RtbOpen { Rtb-Open @args }
