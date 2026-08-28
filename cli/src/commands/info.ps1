function Rtb-Info {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [string]$ProjectName,

        [Parameter(Position = 1, ValueFromRemainingArguments)]
        [string[]]$Flags
    )

    $asJson = $false
    foreach ($f in $Flags) {
        if ($f -eq '--json' -or $f -eq '-Json') {
            $asJson = $true
        }
    }

    if (-not $ProjectName) {
        Write-Host "Usage: rtb info <project-name> [--json]" -ForegroundColor Yellow
        return
    }

    $projMatch = Find-ProjectPath -Name $ProjectName
    if (-not $projMatch) {
        Write-Host "Project '$ProjectName' not found." -ForegroundColor Red
        return
    }

    $details = Get-ProjectDetails -ProjectPath $projMatch.Path -Status $projMatch.Status

    if ($asJson) {
        $details | ConvertTo-Json -Depth 5
        return
    }

    Write-RtbHeader -Title "Project Info: $($details.name)"
    Write-Host ''
    Write-Host "  Name:            $($details.name)" -ForegroundColor White
    Write-Host "  Status:          $($details.status)" -ForegroundColor Cyan
    Write-Host "  Path:            $($details.path)" -ForegroundColor Gray
    Write-Host "  Stack:           $($details.stack -join ', ')" -ForegroundColor Yellow
    Write-Host "  Monorepo:        $(if ($details.is_monorepo) { 'Yes' } else { 'No' })" -ForegroundColor White
    Write-Host "  CI/CD:           $(if ($details.ci_cd) { $details.ci_cd } else { 'None' })" -ForegroundColor White
    Write-Host "  Runtime Version: $(if ($details.runtime_version) { $details.runtime_version } else { 'N/A' })" -ForegroundColor White

    if ($details.git) {
        Write-Host ''
        Write-Host "  Git Info:" -ForegroundColor Yellow
        Write-Host "    Branch:        $($details.git.branch)" -ForegroundColor White
        Write-Host "    Uncommitted:   $($details.git.uncommitted)" -ForegroundColor White
        Write-Host "    Unpushed:      $($details.git.unpushed)" -ForegroundColor White
        Write-Host "    Has Remote:    $($details.git.has_remote)" -ForegroundColor White
        if ($details.git.last_commit_msg) {
            Write-Host "    Last Commit:   $($details.git.last_commit_msg) ($($details.git.last_commit_relative))" -ForegroundColor DarkGray
        }
    }

    if ($details.readme_preview) {
        Write-Host ''
        Write-Host "  README Preview:" -ForegroundColor Yellow
        $details.readme_preview.Split("`n") | ForEach-Object {
            Write-Host "    $_" -ForegroundColor DarkGray
        }
    }
    Write-Host ''
}

function Dev-Info {
    Rtb-Info @args
}
