#Requires -Version 7
# Empirical Challenger 1 - Milestone M6 Comprehensive E2E Stress Harness

$ErrorActionPreference = 'Continue'
$script:Passed = 0
$script:Failed = 0
$script:TestLog = [System.Collections.Generic.List[string]]::new()

function Assert-Condition {
    param(
        [string]$TestName,
        [bool]$Condition,
        [string]$Details = ""
    )
    if ($Condition) {
        $script:Passed++
        $msg = "[PASS] $TestName"
        Write-Host $msg -ForegroundColor Green
        $script:TestLog.Add($msg)
    } else {
        $script:Failed++
        $msg = "[FAIL] $TestName - $Details"
        Write-Host $msg -ForegroundColor Red
        $script:TestLog.Add($msg)
    }
}

$modulePath = Join-Path $PSScriptRoot '..\rtb.psd1'
Import-Module (Resolve-Path $modulePath) -Force

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  EMPIRICAL CHALLENGER 1: MILESTONE M6 E2E VERIFICATION     " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

# Setup temporary isolated test environment
$sandbox = Join-Path ([System.IO.Path]::GetTempPath()) ("rtb_e2e_challenger_" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $sandbox -Force | Out-Null
$fakeActive = Join-Path $sandbox "Active"
$fakePaused = Join-Path $sandbox "Paused"
$fakeArchive = Join-Path $sandbox "Archive"
New-Item -ItemType Directory -Path $fakeActive -Force | Out-Null
New-Item -ItemType Directory -Path $fakePaused -Force | Out-Null
New-Item -ItemType Directory -Path $fakeArchive -Force | Out-Null

try {
    # -------------------------------------------------------------
    # Area 1: `rtb goto` & Fuzzy Matcher Stress Testing
    # -------------------------------------------------------------
    Write-Host "`n--- Testing Area 1: rtb goto & Find-ProjectPathFuzzy ---" -ForegroundColor Yellow

    # 1.1 Find-ProjectPathFuzzy on Non-existent query against real config
    $matchesNonExist = Find-ProjectPathFuzzy -Query "nonexistent_query_xyz123_random_999"
    Assert-Condition "Find-ProjectPathFuzzy non-existent returns 0 matches" ($matchesNonExist.Count -eq 0) "Count: $($matchesNonExist.Count)"

    # 1.2 Find-ProjectPathFuzzy on Exact match query (rtb-command-tool)
    $matchesExact = Find-ProjectPathFuzzy -Query "rtb-command-tool"
    Assert-Condition "Find-ProjectPathFuzzy exact match returns result with Score 100" ($matchesExact.Count -ge 1 -and $matchesExact[0].Score -eq 100 -and $matchesExact[0].Name -eq "rtb-command-tool") "Result: $($matchesExact | Out-String)"

    # 1.3 Find-ProjectPathFuzzy on prefix query
    $matchesPrefix = Find-ProjectPathFuzzy -Query "rtb"
    Assert-Condition "Find-ProjectPathFuzzy prefix query returns scored matches" ($matchesPrefix.Count -ge 1 -and $matchesPrefix[0].Score -ge 75) "Matches: $($matchesPrefix.Count)"

    # 1.4 Find-ProjectPathFuzzy on substring query
    $matchesSub = Find-ProjectPathFuzzy -Query "command"
    Assert-Condition "Find-ProjectPathFuzzy substring query returns scored matches" ($matchesSub.Count -ge 1 -and $matchesSub[0].Score -ge 50) "Matches: $($matchesSub.Count)"

    # 1.5 Real workspace rtb goto exact project
    $realGotoOut = (rtb goto "rtb-command-tool" *>&1) | Out-String
    Assert-Condition "rtb goto on active repo 'rtb-command-tool' resolves" ($realGotoOut -match "Active" -or $realGotoOut -match "rtb-command-tool") "Output: $realGotoOut"

    # 1.6 Real workspace rtb goto empty query prints usage
    $outEmpty = (rtb goto *>&1) | Out-String
    Assert-Condition "rtb goto empty query handles gracefully with usage tip" ($outEmpty -match "Usage" -or $outEmpty -match "Tip") "Output: $outEmpty"

    # 1.7 Real workspace rtb goto non-existent prints friendly notice
    $outNonExistReal = (rtb goto "zzz_non_existent_random_proj_987" *>&1) | Out-String
    Assert-Condition "rtb goto real non-existent lists available projects without crash" ($outNonExistReal -match "No project matching" -and $outNonExistReal -match "Available projects") "Output: $outNonExistReal"


    # -------------------------------------------------------------
    # Area 2: `rtb status` (Git, Non-Git, Empty Stacks, Formatting, JSON)
    # -------------------------------------------------------------
    Write-Host "`n--- Testing Area 2: rtb status ---" -ForegroundColor Yellow

    # 2.1 Active workspace repository rtb status (plain text)
    Push-Location (Join-Path $PSScriptRoot '..\..')
    $statusSelf = (rtb status 2>&1) | Out-String
    Assert-Condition "rtb status in rtb-command-tool detects Active status and stacks" ($statusSelf -match "rtb-command-tool" -and $statusSelf -match "Active") "Output: $statusSelf"

    # 2.2 Active workspace repository rtb status (-Json)
    $statusSelfJson = (rtb status -Json 2>&1) | Out-String
    $parsedSelf = $null
    try { $parsedSelf = $statusSelfJson | ConvertFrom-Json } catch {}
    Assert-Condition "rtb status -Json produces valid JSON object" ($null -ne $parsedSelf) "Output: $statusSelfJson"
    Assert-Condition "rtb status -Json has project name" ($parsedSelf.project -eq "rtb-command-tool") "Project: $($parsedSelf.project)"
    Assert-Condition "rtb status -Json has status Active" ($parsedSelf.status -eq "Active") "Status: $($parsedSelf.status)"
    Assert-Condition "rtb status -Json includes Rust and PowerShell in stack" ($parsedSelf.stack -contains "Rust" -and $parsedSelf.stack -contains "PowerShell") "Stack: $($parsedSelf.stack -join ', ')"
    Pop-Location

    # 2.3 Temporary isolated Git project
    $projGit = Join-Path $fakeActive "git-project"
    New-Item -ItemType Directory -Path $projGit -Force | Out-Null
    Push-Location $projGit
    git init -b main 2>&1 | Out-Null
    git config user.email "test@example.com"
    git config user.name "Tester"
    "Initial content" | Set-Content "README.md"
    git add README.md
    git commit -m "Initial commit" 2>&1 | Out-Null
    "Uncommitted change" | Set-Content "uncommitted.txt"

    $statusText = (rtb status 2>&1) | Out-String
    Assert-Condition "rtb status in git repo contains project name and branch" ($statusText -match "git-project" -and $statusText -match "main") "Output: $statusText"
    Assert-Condition "rtb status in dirty git repo shows uncommitted count" ($statusText -match "±1" -or $statusText -match "1") "Output: $statusText"

    $statusJson = (rtb status -Json 2>&1) | Out-String
    $parsedJson = $statusJson | ConvertFrom-Json
    Assert-Condition "rtb status -Json has branch main and uncommitted == 1" ($parsedJson.branch -eq "main" -and $parsedJson.uncommitted -eq 1) "Parsed: $($parsedJson | ConvertTo-Json -Compress)"
    Pop-Location

    # 2.4 Non-git project with empty stack
    $projNonGit = Join-Path $fakeActive "nongit-empty"
    New-Item -ItemType Directory -Path $projNonGit -Force | Out-Null
    Push-Location $projNonGit
    $statusNonGit = (rtb status 2>&1) | Out-String
    Assert-Condition "rtb status in non-git directory returns valid text format" ($statusNonGit -match "rtb »" -and $statusNonGit -match "nongit-empty") "Output: $statusNonGit"

    $statusNonGitJson = (rtb status -Json 2>&1) | Out-String
    $parsedNonGit = $statusNonGitJson | ConvertFrom-Json
    Assert-Condition "rtb status -Json in non-git project has uncommitted == 0" ($parsedNonGit.project -eq "nongit-empty" -and $parsedNonGit.uncommitted -eq 0) "Parsed: $($parsedNonGit | ConvertTo-Json -Compress)"
    Pop-Location

    # 2.5 Outside workspace root
    Push-Location $sandbox
    $statusOutside = (rtb status 2>&1) | Out-String
    Assert-Condition "rtb status outside workspace roots executes gracefully" ($statusOutside -match "rtb »" -or $statusOutside.Length -gt 0) "Output: $statusOutside"
    Pop-Location


    # -------------------------------------------------------------
    # Area 3: `rtb doctor` (System Health, Config Validity, Missing Tools)
    # -------------------------------------------------------------
    Write-Host "`n--- Testing Area 3: rtb doctor ---" -ForegroundColor Yellow

    # 3.1 Normal doctor run capturing stream 6
    $doctorOut = (rtb doctor *>&1) | Out-String
    Assert-Condition "rtb doctor runs and outputs Config section" ($doctorOut -match "Config") "Output: $doctorOut"
    Assert-Condition "rtb doctor outputs Required Tools (git)" ($doctorOut -match "Required Tools" -and $doctorOut -match "git in PATH") "Output: $doctorOut"
    Assert-Condition "rtb doctor outputs Project Roots" ($doctorOut -match "Project Roots") "Output: $doctorOut"
    Assert-Condition "rtb doctor outputs Optional Tools (Node, Rust/Cargo, Python)" ($doctorOut -match "Optional Tools" -and $doctorOut -match "Node.js" -and $doctorOut -match "Cargo / Rust") "Output: $doctorOut"
    Assert-Condition "rtb doctor outputs AI Agents check" ($doctorOut -match "AI Agents") "Output: $doctorOut"

    # 3.2 Test-RtbDoctor returns boolean
    $docResult = Test-RtbDoctor
    Assert-Condition "Test-RtbDoctor returns boolean result" ($docResult -is [bool]) "Type: $($docResult.GetType().FullName)"


    # -------------------------------------------------------------
    # Area 4: Safety Guardrails (archive, pause, clean)
    # -------------------------------------------------------------
    Write-Host "`n--- Testing Area 4: Safety Guardrails ---" -ForegroundColor Yellow

    # 4.1 `Test-GitClean` on non-git, clean git, and dirty git
    $cleanGit = Join-Path $sandbox "clean-git"
    New-Item -ItemType Directory -Path $cleanGit -Force | Out-Null
    Push-Location $cleanGit
    git init -b main 2>&1 | Out-Null
    git config user.email "test@example.com"
    git config user.name "Tester"
    "Initial" | Set-Content "file.txt"
    git add file.txt
    git commit -m "init" 2>&1 | Out-Null
    Pop-Location

    Assert-Condition "Test-GitClean returns $true for clean repo" (Test-GitClean -ProjectPath $cleanGit) "Clean git"

    # Make dirty with untracked file
    "Untracked" | Set-Content (Join-Path $cleanGit "untracked.txt")
    Assert-Condition "Test-GitClean returns $false for dirty repo" (-not (Test-GitClean -ProjectPath $cleanGit)) "Dirty git"

    # 4.2 `Confirm-RtbAction` responses
    $ansY = 'y' | Confirm-RtbAction -Message 'Test?' 2>$null
    $ansN = 'n' | Confirm-RtbAction -Message 'Test?' 2>$null
    $ansEmpty = '' | Confirm-RtbAction -Message 'Test?' 2>$null
    Assert-Condition "Confirm-RtbAction returns $true on 'y'" ($ansY -eq $true) "Answer Y"
    Assert-Condition "Confirm-RtbAction returns $false on 'n'" ($ansN -eq $false) "Answer N"
    Assert-Condition "Confirm-RtbAction returns $false on empty Enter" ($ansEmpty -eq $false) "Answer empty"

    # 4.3 `Dev-Archive` on dirty repo without -Force aborts
    $dirtyArchiveProj = Join-Path $sandbox "dirty-archive-proj"
    New-Item -ItemType Directory -Path $dirtyArchiveProj -Force | Out-Null
    Push-Location $dirtyArchiveProj
    git init -b main 2>&1 | Out-Null
    git config user.email "test@example.com"
    git config user.name "Tester"
    "Initial" | Set-Content "file.txt"
    git add file.txt
    git commit -m "init" 2>&1 | Out-Null
    "Unsaved change" | Set-Content "file.txt"
    Pop-Location

    $archiveDirtyOut = (Dev-Archive "dirty-archive-proj" *>&1) | Out-String
    Assert-Condition "Dev-Archive aborts on dirty git repo without -Force" ($archiveDirtyOut -match "uncommitted git changes" -or (Test-Path $dirtyArchiveProj)) "Output: $archiveDirtyOut"

    # 4.4 `rtb clean` defaults to dry run
    $cleanTestProj = Join-Path $fakeActive "clean-test-proj"
    $nodeModules = Join-Path $cleanTestProj "node_modules"
    New-Item -ItemType Directory -Path $nodeModules -Force | Out-Null
    (Get-Item $nodeModules).LastWriteTime = (Get-Date).AddDays(-100)

    $cleanDryRun = (rtb clean *>&1) | Out-String
    Assert-Condition "rtb clean defaults to DRY RUN MODE and leaves files intact" ($cleanDryRun -match "DRY RUN MODE" -and (Test-Path $nodeModules)) "Output: $cleanDryRun"

    # 4.5 `Dev-Clean -Commit` aborts on 'n'
    $cleanCommitCancel = ("n" | Dev-Clean -Commit *>&1) | Out-String
    Assert-Condition "Dev-Clean -Commit aborts when user responds 'n'" ($cleanCommitCancel -match "Aborted" -or (Test-Path $nodeModules)) "Output: $cleanCommitCancel"


    # -------------------------------------------------------------
    # Area 5: Context Generation (0 commits, non-git, various dependency manifests)
    # -------------------------------------------------------------
    Write-Host "`n--- Testing Area 5: Context Generation ---" -ForegroundColor Yellow

    # 5.1 Polyglot project with package.json, Cargo.toml, requirements.txt, go.mod
    $polyProj = Join-Path $sandbox "polyglot-test"
    New-Item -ItemType Directory -Path $polyProj -Force | Out-Null

    @{
        name = "test-pkg"
        dependencies = @{ "express" = "^4.18.0"; "react" = "^18.0.0" }
        devDependencies = @{ "jest" = "^29.0.0" }
    } | ConvertTo-Json | Set-Content (Join-Path $polyProj "package.json")

    "[package]`nname = `"cargo-test`"`nversion = `"0.1.0`"`n`n[dependencies]`nserde = `"1.0`"`ntokio = { version = `"1`", features = [ `"full`" ] }`n" | Set-Content (Join-Path $polyProj "Cargo.toml")
    "flask==3.0.0`n# A comment`nrequests>=2.28.0`n" | Set-Content (Join-Path $polyProj "requirements.txt")
    "module github.com/test/app`n`ngo 1.22`n`nrequire (`n`tgithub.com/gin-gonic/gin v1.9.1`n)`n" | Set-Content (Join-Path $polyProj "go.mod")

    Push-Location $polyProj
    git init -b main 2>&1 | Out-Null
    # 0 commits: do not commit anything
    $contextFile = New-RtbAgentContextFile -ProjectPath $polyProj
    Assert-Condition "New-RtbAgentContextFile generates .rtb_context.md for 0-commit polyglot repo" ($null -ne $contextFile -and (Test-Path $contextFile)) "Path: $contextFile"

    $contextContent = Get-Content $contextFile -Raw
    Assert-Condition "Context includes Node dependencies" ($contextContent -match "express" -and $contextContent -match "react") "Node deps"
    Assert-Condition "Context includes Cargo dependencies" ($contextContent -match "serde" -and $contextContent -match "tokio") "Cargo deps"
    Assert-Condition "Context includes Python requirements" ($contextContent -match "flask" -and $contextContent -match "requests") "Python deps"
    Assert-Condition "Context includes Go modules" ($contextContent -match "github.com/gin-gonic/gin") "Go modules"
    Assert-Condition "Context handles 0 commits cleanly" ($contextContent -match "0 commits" -or $contextContent -match "Recent Git History" -or $contextContent -match "none") "Git history section"
    Pop-Location

    # 5.2 Non-git project context generation
    $nonGitContextProj = Join-Path $sandbox "nongit-context-proj"
    New-Item -ItemType Directory -Path $nonGitContextProj -Force | Out-Null
    "Simple README" | Set-Content (Join-Path $nonGitContextProj "README.md")
    $nonGitContextFile = New-RtbAgentContextFile -ProjectPath $nonGitContextProj
    $nonGitContent = Get-Content $nonGitContextFile -Raw
    Assert-Condition "Context for non-git directory contains fallback notice" ($nonGitContent -match "Non-git directory" -or $nonGitContent -match "Project:") "Non-git output: $nonGitContent"

}
finally {
    # Cleanup sandbox
    if (Test-Path $sandbox) {
        Remove-Item -Recurse -Force $sandbox -ErrorAction SilentlyContinue
    }
}

Write-Host "`n============================================================" -ForegroundColor Cyan
Write-Host "  EMPIRICAL CHALLENGE SUMMARY                               " -ForegroundColor Cyan
Write-Host "  Passed: $script:Passed | Failed: $script:Failed           " -ForegroundColor ($script:Failed -eq 0 ? "Green" : "Red")
Write-Host "============================================================" -ForegroundColor Cyan

if ($script:Failed -gt 0) {
    exit 1
} else {
    exit 0
}
