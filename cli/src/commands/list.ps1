function Rtb-List {
    param([Parameter(ValueFromRemainingArguments)][string[]]$Flags)

    $filter = 'all'
    $asJson = $false
    foreach ($a in $Flags) {
        switch ($a) {
            '--active'   { $filter = 'active' }
            '--paused'   { $filter = 'paused' }
            '--deployed' { $filter = 'deployed' }
            '--vibe'     { $filter = 'vibe' }
            '--all'      { $filter = 'all' }
            '--json'     { $asJson = $true }
            '-Json'      { $asJson = $true }
        }
    }

    if ($asJson) {
        $projects = Get-AllProjectsDetails -Filter $filter
        $projects | ConvertTo-Json -Depth 5
        return
    }


    $config = Get-DevConfig
    Write-RtbHeader 'Project List'
    Write-Host ''

    $categories = @(
        @{ Name = if ($config.projectRoots.active.label) { $config.projectRoots.active.label } else { 'Active' };     Icon = if ($config.projectRoots.active.emoji) { $config.projectRoots.active.emoji } else { '📁' }; Path = (Get-RtbRootPath $config.projectRoots.active);     Show = $filter -in 'all','active' },
        @{ Name = if ($config.projectRoots.paused.label) { $config.projectRoots.paused.label } else { 'Paused' };     Icon = if ($config.projectRoots.paused.emoji) { $config.projectRoots.paused.emoji } else { '⏸️' };  Path = (Get-RtbRootPath $config.projectRoots.paused);     Show = $filter -in 'all','paused' },
        @{ Name = if ($config.projectRoots.production.label) { $config.projectRoots.production.label } else { 'Production' }; Icon = if ($config.projectRoots.production.emoji) { $config.projectRoots.production.emoji } else { '🚀' }; Path = (Get-RtbRootPath $config.projectRoots.production); Show = $filter -in 'all','deployed' },
        @{ Name = if ($config.projectRoots.staging.label) { $config.projectRoots.staging.label } else { 'Staging' };    Icon = if ($config.projectRoots.staging.emoji) { $config.projectRoots.staging.emoji } else { '🚀' }; Path = (Get-RtbRootPath $config.projectRoots.staging);    Show = $filter -in 'all','deployed' },
        @{ Name = if ($config.projectRoots.vibe.label) { $config.projectRoots.vibe.label } else { 'Vibe' };       Icon = if ($config.projectRoots.vibe.emoji) { $config.projectRoots.vibe.emoji } else { '✨' }; Path = (Get-RtbRootPath $config.projectRoots.vibe);       Show = $filter -in 'all','vibe' }
    )

    $total = 0
    foreach ($cat in $categories) {
        if (-not $cat.Show -or -not $cat.Path -or -not (Test-Path $cat.Path)) { continue }
        $projects = Get-ChildItem $cat.Path -Directory -ErrorAction SilentlyContinue
        if ($projects.Count -eq 0) { continue }

        Write-Host "  $($cat.Icon) $($cat.Name) ($($projects.Count))" -ForegroundColor Yellow
        foreach ($p in $projects) {
            $total++
            $lastFile = Get-ChildItem $p.FullName -Recurse -File -ErrorAction SilentlyContinue |
                Where-Object { $_.FullName -notmatch 'node_modules|\.git|dist|build|\.next' } |
                Sort-Object LastWriteTime -Descending | Select-Object -First 1
            $lastMod = if ($lastFile) { $lastFile.LastWriteTime.ToString('yyyy-MM-dd') } else { '-' }
            Write-Host "    $($p.Name)" -NoNewline -ForegroundColor White
            Write-Host "  ($lastMod)" -ForegroundColor DarkGray
        }
        Write-Host ''
    }
    Write-Host "  Total: $total projects" -ForegroundColor Gray
}

function Dev-List {
    Rtb-List @args
}


function Get-RtbProjectList { Dev-List @args }
