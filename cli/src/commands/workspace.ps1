function Rtb-Workspace {
    [CmdletBinding()]
    param(
        [string]$ProjectName = "",
        [switch]$Json
    )

    $projectPath = (Get-Location).Path
    if ($ProjectName) {
        if (Test-Path $ProjectName) {
            $projectPath = (Resolve-Path $ProjectName).Path
        } else {
            $found = Find-ProjectPath $ProjectName
            if ($found -and $found.Path) { $projectPath = $found.Path }
        }
    }

    Write-RtbHeader "Monorepo Workspace Inspector ($projectPath)"

    $workspacePackages = @()
    $workspaceType = "Single Package / Standard Repository"

    # 1. Check pnpm-workspace.yaml
    $pnpmWsPath = Join-Path $projectPath 'pnpm-workspace.yaml'
    if (Test-Path $pnpmWsPath) {
        $workspaceType = "pnpm Workspaces"
        Get-Content $pnpmWsPath | ForEach-Object {
            if ($_ -match "^\s*-\s*['`"]?([^'`"]+)['`"]?") {
                $workspacePackages += [PSCustomObject]@{
                    PackagePattern = $matches[1]
                    Type           = "pnpm"
                }
            }
        }
    }

    # 2. Check package.json workspaces
    $pkgJsonPath = Join-Path $projectPath 'package.json'
    if (Test-Path $pkgJsonPath) {
        try {
            $pkgJson = Get-Content $pkgJsonPath -Raw | ConvertFrom-Json
            if ($pkgJson.workspaces) {
                $workspaceType = "npm/yarn Workspaces"
                foreach ($ws in $pkgJson.workspaces) {
                    $workspacePackages += [PSCustomObject]@{
                        PackagePattern = $ws
                        Type           = "npm/yarn"
                    }
                }
            }
        } catch {}
    }

    # 3. Check Cargo.toml workspace members
    $cargoPath = Join-Path $projectPath 'Cargo.toml'
    if (Test-Path $cargoPath) {
        $inWorkspace = $false
        Get-Content $cargoPath | ForEach-Object {
            if ($_ -match '^\s*\[workspace\]') { $inWorkspace = $true }
            elseif ($_ -match '^\s*\[') { $inWorkspace = $false }
            elseif ($inWorkspace -and $_ -match '^\s*"([^"]+)"') {
                $workspaceType = "Cargo Workspace (Rust)"
                $workspacePackages += [PSCustomObject]@{
                    PackagePattern = $matches[1]
                    Type           = "Cargo"
                }
            }
        }
    }

    $result = [PSCustomObject]@{
        ProjectPath       = $projectPath
        WorkspaceType     = $workspaceType
        IsMonorepo        = ($workspacePackages.Count -gt 0)
        Packages          = $workspacePackages
    }

    if ($Json) {
        return ($result | ConvertTo-Json -Depth 5)
    }

    Write-Host "  Monorepo Type: $workspaceType" -ForegroundColor Cyan
    if ($workspacePackages.Count -gt 0) {
        Write-Host "  Declared Workspace Patterns:" -ForegroundColor Green
        $workspacePackages | Format-Table -AutoSize
    } else {
        Write-Host "  No active monorepo workspace configurations detected." -ForegroundColor Yellow
    }
}

function Dev-Workspace {
    Rtb-Workspace @args
}
