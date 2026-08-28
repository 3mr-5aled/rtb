function Dev-New {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$Name,
        [string]$Stack = 'generic'
    )

    # Parse --stack from remaining args
    $args_list = @($Name) + $args
    for ($i = 0; $i -lt $args_list.Count; $i++) {
        if ($args_list[$i] -eq '--stack' -and ($i + 1) -lt $args_list.Count) {
            $Stack = $args_list[$i + 1]
            $i++
        } elseif (-not $Name -or $Name -eq '--stack') {
            $Name = $args_list[$i]
        }
    }

    if (-not $Name) {
        Write-Host 'Usage: dev new <project-name> [--stack react|nextjs|node|python|generic]' -ForegroundColor Yellow
        return
    }

    $config = Get-DevConfig
    $kebabName = $Name.ToLower() -replace '[^a-z0-9\-]', '-' -replace '-+', '-'
    $targetDir = Join-Path $config.projectRoots.active $kebabName

    if (Test-Path $targetDir) {
        Write-Host "  Project '$kebabName' already exists!" -ForegroundColor Red
        return
    }

    New-Item -Path $targetDir -ItemType Directory -Force | Out-Null
    Write-RtbHeader "Creating project: $kebabName"

    # Copy PROJECT.md template
    $templatePath = Join-Path $config.templateDir 'PROJECT.md'
    if (Test-Path $templatePath) {
        $meta = Get-Content $templatePath -Raw
        $meta = $meta -replace '\[Project Name\]', $Name -replace 'YYYY-MM-DD', (Get-Date -Format 'yyyy-MM-dd') -replace '\[e\.g\..*\]', $Stack
        Set-Content -Path (Join-Path $targetDir 'PROJECT.md') -Value $meta
        Write-Host '  Created PROJECT.md' -ForegroundColor Gray
    }

    # .gitignore
    @('node_modules/', '.next/', '.venv/', '__pycache__/', 'dist/', 'build/', '.env', '.env.local', '*.log') |
        Set-Content (Join-Path $targetDir '.gitignore')
    Write-Host '  Created .gitignore' -ForegroundColor Gray

    # README
    "# $Name`n`nNew development project ($Stack stack).`n`nCreated: $((Get-Date).ToString('MMMM yyyy'))" |
        Set-Content (Join-Path $targetDir 'README.md')
    Write-Host '  Created README.md' -ForegroundColor Gray

    Write-Host "`n  Project '$kebabName' created in 01-Active!" -ForegroundColor Green
    Write-Host "  Run: dev goto $kebabName" -ForegroundColor Cyan
}
