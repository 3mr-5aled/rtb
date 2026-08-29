function Rtb-Deps {
    [CmdletBinding()]
    param(
        [string]$SubCommand = "outdated",
        [string]$ProjectName = "",
        [switch]$Json
    )

    $config = Get-RtbConfig
    $projectPath = (Get-Location).Path

    $targetName = if ($ProjectName) { $ProjectName } elseif ($SubCommand -and $SubCommand -ne "outdated") { $SubCommand } else { "" }
    if ($targetName) {
        if (Test-Path $targetName) {
            $projectPath = (Resolve-Path $targetName).Path
        } else {
            $found = Find-ProjectPath $targetName
            if ($found -and $found.Path) { $projectPath = $found.Path }
        }
    }

    Write-RtbHeader "Dependency Inspector ($projectPath)"

    $depsList = @()

    # 1. Inspect package.json (Node.js)
    $pkgJsonPath = Join-Path $projectPath 'package.json'
    if (Test-Path $pkgJsonPath) {
        try {
            $pkgJson = Get-Content $pkgJsonPath -Raw | ConvertFrom-Json
            if ($pkgJson.dependencies) {
                $pkgJson.dependencies.psobject.properties | ForEach-Object {
                    $depsList += [PSCustomObject]@{
                        Package = $_.Name
                        Spec    = $_.Value
                        Type    = "npm/pnpm/yarn"
                        Status  = "Declared"
                    }
                }
            }
            if ($pkgJson.devDependencies) {
                $pkgJson.devDependencies.psobject.properties | ForEach-Object {
                    $depsList += [PSCustomObject]@{
                        Package = $_.Name
                        Spec    = $_.Value
                        Type    = "npm/pnpm (dev)"
                        Status  = "Declared"
                    }
                }
            }
        } catch {}
    }

    # 2. Inspect Cargo.toml (Rust)
    $cargoPath = Join-Path $projectPath 'Cargo.toml'
    if (Test-Path $cargoPath) {
        Get-Content $cargoPath | ForEach-Object {
            if ($_ -match '^\s*([a-zA-Z0-9_-]+)\s*=\s*"([^"]+)"') {
                $depsList += [PSCustomObject]@{
                    Package = $matches[1]
                    Spec    = $matches[2]
                    Type    = "Cargo (Rust)"
                    Status  = "Declared"
                }
            }
        }
    }

    # 3. Inspect pyproject.toml / requirements.txt (Python)
    $pyprojectPath = Join-Path $projectPath 'pyproject.toml'
    if (Test-Path $pyprojectPath) {
        Get-Content $pyprojectPath | ForEach-Object {
            if ($_ -match '^\s*"([a-zA-Z0-9_-]+)([<>=!~]+[^"]+)"') {
                $depsList += [PSCustomObject]@{
                    Package = $matches[1]
                    Spec    = $matches[2]
                    Type    = "Python (pyproject)"
                    Status  = "Declared"
                }
            }
        }
    }

    if ($Json) {
        return ($depsList | ConvertTo-Json -Depth 5)
    }

    if ($depsList.Count -eq 0) {
        Write-Host "  No dependencies found in $projectPath" -ForegroundColor Yellow
        return
    }

    Write-Host ("  Found {0} declared dependencies:`n" -f $depsList.Count) -ForegroundColor Green
    $depsList | Format-Table -AutoSize
}

function Dev-Deps {
    Rtb-Deps @args
}


function Get-RtbDependencies { Rtb-Deps @args }
