function Rtb-Build {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$ProjectName,

        [Parameter(Position = 1, ValueFromRemainingArguments)]
        [string[]]$ExtraArgs
    )

    $targetPath = Get-Location
    if ($ProjectName) {
        $proj = Find-ProjectPath -Name $ProjectName
        if ($proj) {
            $targetPath = $proj.Path
        } else {
            Write-Host "Project '$ProjectName' not found." -ForegroundColor Red
            return
        }
    }

    Write-RtbHeader -Title "Build Project ($(Split-Path $targetPath -Leaf))"
    Set-Location $targetPath

    if (Test-Path 'package.json') {
        $pkg = Get-Content 'package.json' -Raw | ConvertFrom-Json
        if ($pkg.scripts.build) {
            Write-Host "Running 'npm run build' in $targetPath..." -ForegroundColor Green
            npm run build @ExtraArgs
            return
        }
    }

    if (Test-Path 'Cargo.toml') {
        Write-Host "Running 'cargo build --release' in $targetPath..." -ForegroundColor Green
        cargo build --release @ExtraArgs
        return
    }

    if (Test-Path 'go.mod') {
        Write-Host "Running 'go build' in $targetPath..." -ForegroundColor Green
        go build @ExtraArgs
        return
    }

    Write-Host "No build configuration detected in $targetPath." -ForegroundColor Yellow
}
