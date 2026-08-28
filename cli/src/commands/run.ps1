function Rtb-Run {
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

    Write-RtbHeader -Title "Run Project ($(Split-Path $targetPath -Leaf))"
    Set-Location $targetPath

    if (Test-Path 'package.json') {
        $pkg = Get-Content 'package.json' -Raw | ConvertFrom-Json
        if ($pkg.scripts.dev) {
            Write-Host "Running 'npm run dev' in $targetPath..." -ForegroundColor Green
            npm run dev @ExtraArgs
            return
        } elseif ($pkg.scripts.start) {
            Write-Host "Running 'npm start' in $targetPath..." -ForegroundColor Green
            npm start @ExtraArgs
            return
        }
    }

    if (Test-Path 'Cargo.toml') {
        Write-Host "Running 'cargo run' in $targetPath..." -ForegroundColor Green
        cargo run @ExtraArgs
        return
    }

    if (Test-Path 'go.mod') {
        Write-Host "Running 'go run .' in $targetPath..." -ForegroundColor Green
        go run . @ExtraArgs
        return
    }

    if (Test-Path 'main.py') {
        Write-Host "Running 'python main.py' in $targetPath..." -ForegroundColor Green
        python main.py @ExtraArgs
        return
    }

    Write-Host "No runnable script or main entrypoint detected in $targetPath." -ForegroundColor Yellow
}
