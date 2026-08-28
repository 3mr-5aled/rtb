function Rtb-Test {
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

    Write-RtbHeader -Title "Test Project ($(Split-Path $targetPath -Leaf))"
    Set-Location $targetPath

    if (Test-Path 'package.json') {
        $pkg = Get-Content 'package.json' -Raw | ConvertFrom-Json
        if ($pkg.scripts.test) {
            Write-Host "Running 'npm test' in $targetPath..." -ForegroundColor Green
            npm test @ExtraArgs
            return
        }
    }

    if (Test-Path 'Cargo.toml') {
        Write-Host "Running 'cargo test' in $targetPath..." -ForegroundColor Green
        cargo test @ExtraArgs
        return
    }

    if (Test-Path 'pytest.ini') {
        Write-Host "Running 'pytest' in $targetPath..." -ForegroundColor Green
        pytest @ExtraArgs
        return
    }

    if (Test-Path 'cli/tests') {
        Write-Host "Running 'Invoke-Pester' in $targetPath/cli/tests..." -ForegroundColor Green
        Invoke-Pester cli/tests/ @ExtraArgs
        return
    }

    if (Test-Path 'tests') {
        Write-Host "Running 'Invoke-Pester' in $targetPath..." -ForegroundColor Green
        Invoke-Pester tests/ @ExtraArgs
        return
    }

    Write-Host "No test configuration detected in $targetPath." -ForegroundColor Yellow
}
