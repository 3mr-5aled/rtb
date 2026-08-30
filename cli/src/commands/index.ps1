function Dev-Index {
    $config = Get-DevConfig
    Write-RtbHeader 'Project Index Generator'
    $output = "# Project Index`n`n> Generated $(Get-Date -Format 'yyyy-MM-dd HH:mm')`n`n| Project | Status | Stack | Last Modified |`n|:---|:---|:---|:---|`n"
    $total = 0
    $categories = @(
        @{Path=$config.projectRoots.active;Status='🟢 Active'},
        @{Path=$config.projectRoots.paused;Status='⏸️ Paused'},
        @{Path=$config.projectRoots.production;Status='🚀 Production'},
        @{Path=$config.projectRoots.vibe;Status='⚡ Vibe'}
    )
    foreach ($cat in $categories) {
        if (-not (Test-Path $cat.Path)) { continue }
        Get-ChildItem $cat.Path -Directory | ForEach-Object {
            $total++
            $stack = @()
            $pkgPath = Join-Path $_.FullName 'package.json'
            if (Test-Path $pkgPath) {
                try {
                    $pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json -EA SilentlyContinue
                    $all = @()
                    if ($pkg.dependencies) { $all += $pkg.dependencies.PSObject.Properties.Name }
                    if ($pkg.devDependencies) { $all += $pkg.devDependencies.PSObject.Properties.Name }
                    if ('next' -in $all) { $stack += 'Next.js' }
                    elseif ('react' -in $all) { $stack += 'React' }
                    if ('tailwindcss' -in $all) { $stack += 'Tailwind' }
                    if ('prisma' -in $all) { $stack += 'Prisma' }
                    if ('typescript' -in $all) { $stack += 'TypeScript' }
                } catch {}
            }
            if (Test-Path (Join-Path $_.FullName 'requirements.txt')) { $stack += 'Python' }
            if (-not $stack) { $stack = @('-') }
            $lf = Get-ChildItem $_.FullName -Recurse -File -EA SilentlyContinue | Where-Object { $_.FullName -notmatch 'node_modules|\.git' } | Sort-Object LastWriteTime -Descending | Select-Object -First 1
            $lm = if ($lf) { $lf.LastWriteTime.ToString('yyyy-MM-dd') } else { '-' }
            $output += "| $($_.Name) | $($cat.Status) | $($stack -join ', ') | $lm |`n"
        }
    }
    $output += "`n---`n*Total: $total projects*`n"
    $outPath = if ($config.projectRoots.active -and (Test-Path $config.projectRoots.active)) { Join-Path (Split-Path $config.projectRoots.active -Parent) 'PROJECT-INDEX.md' } else { 'PROJECT-INDEX.md' }
    $output | Set-Content $outPath -Encoding UTF8
    Write-Host "  Generated index: $total projects → $outPath" -ForegroundColor Green
}


function Update-RtbIndex { Dev-Index @args }
