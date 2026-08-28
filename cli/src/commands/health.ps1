function Dev-Health {
    $config = Get-DevConfig
    Write-RtbHeader 'Git Repository Health'

    $issues = 0
    $scanned = 0

    foreach ($root in $config.gitHealth.scanRoots) {
        if (-not (Test-Path $root)) { continue }
        Get-ChildItem -Path $root -Recurse -Directory -Filter '.git' -Force -EA SilentlyContinue | ForEach-Object {
            $repo = $_.Parent.FullName
            $scanned++
            Push-Location $repo
            try {
                $repoIssues = @()
                $status = git status --porcelain 2>$null
                if ($status) { $repoIssues += "UNCOMMITTED ($(@($status).Count) files)" }
                $unpushed = git log --branches --not --remotes --oneline 2>$null
                if ($unpushed) { $repoIssues += "UNPUSHED ($(@($unpushed).Count))" }
                $lastDate = git log -1 --format='%ai' 2>$null
                $lastRel = git log -1 --format='%cr' 2>$null
                if ($lastDate) {
                    $days = ((Get-Date) - [datetime]$lastDate).Days
                    if ($days -gt $config.staleThresholdDays) { $repoIssues += "STALE ($days days)" }
                }
                if (-not (git remote 2>$null)) { $repoIssues += 'NO REMOTE' }
                if ($repoIssues.Count -gt 0) {
                    $issues++
                    Write-Host "`n  $($repo.Replace('D:\',''))" -ForegroundColor Yellow
                    Write-Host "    Last commit: $lastRel" -ForegroundColor DarkGray
                    foreach ($i in $repoIssues) {
                        $c = if ($i -match 'UNCOMMITTED|UNPUSHED') {'Red'} else {'Yellow'}
                        Write-Host "    ⚠ $i" -ForegroundColor $c
                    }
                }
            } finally { Pop-Location }
        }
    }
    Write-Host "`n  Scanned: $scanned repos | Issues: $issues" -ForegroundColor $(if ($issues) {'Yellow'} else {'Green'})
}
