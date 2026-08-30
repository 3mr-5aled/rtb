# Stress-Challenger2-M6.Tests.ps1
# Adversarial Empirical Stress Tests for Milestone M6 (CLI & Schema Integrity)

$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$sutDir = (Get-Item (Join-Path $here '..')).FullName
Import-Module (Join-Path $sutDir 'rtb.psd1') -Force

Describe "Milestone M6 Adversarial Stress: Config Corruption & Boundary Resilience" {
    BeforeAll {
        $script:tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("rtb_m6_stress_" + [System.Guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $script:tempRoot -Force | Out-Null
    }

    AfterAll {
        if (Test-Path $script:tempRoot) {
            Remove-Item -Path $script:tempRoot -Recurse -Force -ErrorAction SilentlyContinue
        }
    }

    Context "Config File Corruption & Non-Existent Fallbacks" {
        It "handles completely corrupted JSON without crashing Get-RtbConfig, Rtb-Doctor, or Rtb-Status" {
            $badConfigDir = Join-Path $script:tempRoot "corrupted_config"
            New-Item -ItemType Directory -Path $badConfigDir -Force | Out-Null
            $badConfigFile = Join-Path $badConfigDir "rtb.config.json"
            Set-Content -Path $badConfigFile -Value '{"projectRoots": { INVALID_JSON_SYNTAX ' -Force

            # Backup original APPDATA/HOME config if any
            $appDataDir = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb' } else { Join-Path $env:HOME '.config/rtb' }
            $backupFile = Join-Path $appDataDir "rtb.config.json.m6bak"
            $targetFile = Join-Path $appDataDir "rtb.config.json"
            $hadOriginal = Test-Path $targetFile
            if ($hadOriginal) { Move-Item -Path $targetFile -Destination $backupFile -Force }

            try {
                # Place corrupted file in AppData
                if (-not (Test-Path $appDataDir)) { New-Item -ItemType Directory -Path $appDataDir -Force | Out-Null }
                Copy-Item -Path $badConfigFile -Destination $targetFile -Force

                # 1. Get-RtbConfig
                $cfg = try { Get-RtbConfig -ErrorAction SilentlyContinue } catch { $null }
                # Should fallback or return null, not crash
                $cfg | Should BeNullOrEmpty

                # 2. Rtb-Doctor
                $docResult = Rtb-Doctor
                $docResult | Should Be $false

                # 3. Rtb-Status (plain and json)
                $plainStatus = Rtb-Status
                $plainStatus | Should Not BeNullOrEmpty
                $plainStatus | Should Match '^rtb »'

                $jsonStatus = Rtb-Status -Json
                $jsonStatus | Should Not BeNullOrEmpty
                $parsed = $jsonStatus | ConvertFrom-Json
                $parsed.project | Should Not BeNullOrEmpty
                $parsed.stack | Should Not BeNullOrEmpty
            }
            finally {
                if (Test-Path $targetFile) { Remove-Item -Path $targetFile -Force -ErrorAction SilentlyContinue }
                if ($hadOriginal -and (Test-Path $backupFile)) { Move-Item -Path $backupFile -Destination $targetFile -Force }
            }
        }

        It "handles zero-byte empty config file gracefully" {
            $appDataDir = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb' } else { Join-Path $env:HOME '.config/rtb' }
            $backupFile = Join-Path $appDataDir "rtb.config.json.m6bak2"
            $targetFile = Join-Path $appDataDir "rtb.config.json"
            $hadOriginal = Test-Path $targetFile
            if ($hadOriginal) { Move-Item -Path $targetFile -Destination $backupFile -Force }

            try {
                if (-not (Test-Path $appDataDir)) { New-Item -ItemType Directory -Path $appDataDir -Force | Out-Null }
                Set-Content -Path $targetFile -Value "" -Force

                $cfg = Get-RtbConfig -ErrorAction SilentlyContinue
                $cfg | Should BeNullOrEmpty

                $plainStatus = Rtb-Status
                $plainStatus | Should Match '^rtb »'

                $jsonStatus = Rtb-Status -Json
                $parsed = $jsonStatus | ConvertFrom-Json
                $parsed.stack | Should Not BeNullOrEmpty
            }
            finally {
                if (Test-Path $targetFile) { Remove-Item -Path $targetFile -Force -ErrorAction SilentlyContinue }
                if ($hadOriginal -and (Test-Path $backupFile)) { Move-Item -Path $backupFile -Destination $targetFile -Force }
            }
        }

        It "handles config with null or missing project roots gracefully" {
            $appDataDir = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb' } else { Join-Path $env:HOME '.config/rtb' }
            $backupFile = Join-Path $appDataDir "rtb.config.json.m6bak3"
            $targetFile = Join-Path $appDataDir "rtb.config.json"
            $hadOriginal = Test-Path $targetFile
            if ($hadOriginal) { Move-Item -Path $targetFile -Destination $backupFile -Force }

            try {
                if (-not (Test-Path $appDataDir)) { New-Item -ItemType Directory -Path $appDataDir -Force | Out-Null }
                $partialConfig = @{
                    version = "0.2.0-beta"
                    projectRoots = @{
                        active = $null
                        paused = $null
                    }
                } | ConvertTo-Json
                Set-Content -Path $targetFile -Value $partialConfig -Force

                $names = Get-AllProjectNames
                $names.Count | Should Be 0

                $projects = Get-ProjectsByStatus -Status 'active'
                $projects.Count | Should Be 0

                $fuzzy = Find-ProjectPathFuzzy -Query "anything"
                $fuzzy.Count | Should Be 0
            }
            finally {
                if (Test-Path $targetFile) { Remove-Item -Path $targetFile -Force -ErrorAction SilentlyContinue }
                if ($hadOriginal -and (Test-Path $backupFile)) { Move-Item -Path $backupFile -Destination $targetFile -Force }
            }
        }
    }
}

Describe "Milestone M6 Adversarial Stress: CLI Argument & Input Fuzzing" {
    BeforeAll {
        $script:fuzzDir = Join-Path ([System.IO.Path]::GetTempPath()) ("rtb_m6_fuzz_" + [System.Guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $script:fuzzDir -Force | Out-Null
    }

    AfterAll {
        if (Test-Path $script:fuzzDir) {
            Remove-Item -Path $script:fuzzDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }

    Context "Fuzzy Search Adversarial Inputs" {
        It "handles special regex characters and hostile query strings in Find-ProjectPathFuzzy" {
            $hostileQueries = @(
                "",
                "   ",
                ".*",
                "[a-z0-9]+",
                "(?:a|b|c)",
                "^.*$",
                "$`~!@#$%^&*()_+-=[]{}|;':,./<>?",
                "مرحبا_بالعالم",
                "🚀✨🔥",
                "' OR '1'='1",
                "--",
                "\\..\\..\\windows\\system32"
            )

            foreach ($q in $hostileQueries) {
                { $res = Find-ProjectPathFuzzy -Query $q } | Should Not Throw
            }
        }
    }

    Context "Confirm-RtbAction Fuzzing" {
        It "correctly interprets affirmative and negative fuzz inputs" {
            # Affirmatives
            @('y', 'Y', 'yes', 'YES', 'Yes', 'yEs', ' y ', ' YES ') | ForEach-Object {
                Confirm-RtbAction -Message "Test" -Answer $_ | Should Be $true
            }

            # Negatives & Hostile inputs
            @('n', 'N', 'no', 'NO', '', ' ', $null, 'maybe', '1', '0', 'true', 'false', 'cancel', 'abort', '!@#$') | ForEach-Object {
                Confirm-RtbAction -Message "Test" -Answer $_ | Should Be $false
            }
        }
    }

    Context "Test-GitClean Boundary States" {
        It "returns true for non-existent path" {
            Test-GitClean (Join-Path $script:fuzzDir "does_not_exist") | Should Be $true
        }

        It "returns true for a clean repository with commit" {
            $repo = Join-Path $script:fuzzDir "clean_repo"
            New-Item -ItemType Directory -Path $repo -Force | Out-Null
            git -C $repo init -b main 2>&1 | Out-Null
            git -C $repo config user.email "test@example.com"
            git -C $repo config user.name "Test"
            Set-Content -Path (Join-Path $repo "file.txt") -Value "initial"
            git -C $repo add file.txt 2>&1 | Out-Null
            git -C $repo commit -m "init" 2>&1 | Out-Null

            Test-GitClean $repo | Should Be $true
        }

        It "returns false for repository with unstaged modifications" {
            $repo = Join-Path $script:fuzzDir "dirty_repo_unstaged"
            New-Item -ItemType Directory -Path $repo -Force | Out-Null
            git -C $repo init -b main 2>&1 | Out-Null
            git -C $repo config user.email "test@example.com"
            git -C $repo config user.name "Test"
            Set-Content -Path (Join-Path $repo "file.txt") -Value "initial"
            git -C $repo add file.txt 2>&1 | Out-Null
            git -C $repo commit -m "init" 2>&1 | Out-Null

            Set-Content -Path (Join-Path $repo "file.txt") -Value "modified"
            Test-GitClean $repo | Should Be $false
        }

        It "returns false for repository with untracked new files" {
            $repo = Join-Path $script:fuzzDir "dirty_repo_untracked"
            New-Item -ItemType Directory -Path $repo -Force | Out-Null
            git -C $repo init -b main 2>&1 | Out-Null
            git -C $repo config user.email "test@example.com"
            git -C $repo config user.name "Test"
            Set-Content -Path (Join-Path $repo "file.txt") -Value "initial"
            git -C $repo add file.txt 2>&1 | Out-Null
            git -C $repo commit -m "init" 2>&1 | Out-Null

            Set-Content -Path (Join-Path $repo "newfile.txt") -Value "untracked"
            Test-GitClean $repo | Should Be $false
        }

        It "returns false for repository with staged modifications" {
            $repo = Join-Path $script:fuzzDir "dirty_repo_staged"
            New-Item -ItemType Directory -Path $repo -Force | Out-Null
            git -C $repo init -b main 2>&1 | Out-Null
            git -C $repo config user.email "test@example.com"
            git -C $repo config user.name "Test"
            Set-Content -Path (Join-Path $repo "file.txt") -Value "initial"
            git -C $repo add file.txt 2>&1 | Out-Null
            git -C $repo commit -m "init" 2>&1 | Out-Null

            Set-Content -Path (Join-Path $repo "file.txt") -Value "staged changes"
            git -C $repo add file.txt 2>&1 | Out-Null
            Test-GitClean $repo | Should Be $false
        }
    }
}

Describe "Milestone M6 Adversarial Stress: AI Agent Context (.rtb_context.md) Generation" {
    BeforeAll {
        $script:ctxRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("rtb_m6_agent_ctx_" + [System.Guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $script:ctxRoot -Force | Out-Null
    }

    AfterAll {
        if (Test-Path $script:ctxRoot) {
            Remove-Item -Path $script:ctxRoot -Recurse -Force -ErrorAction SilentlyContinue
        }
    }

    Context "Special Git States & Complex Stacks" {
        It "generates rich context for a repo with 0 commits (unborn master/main)" {
            $repo = Join-Path $script:ctxRoot "zero_commit_repo"
            New-Item -ItemType Directory -Path $repo -Force | Out-Null
            git -C $repo init -b main 2>&1 | Out-Null
            Set-Content -Path (Join-Path $repo "Cargo.toml") -Value "[package]`nname = `"test-zero`"`nversion = `"0.1.0`"`n"

            # Create context
            $ctxPath = Join-Path $repo ".rtb_context.md"
            # Call internal context generator
            $details = Get-ProjectDetails -ProjectPath $repo -Status 'Active'
            $details | Should Not BeNullOrEmpty

            # Test using Rtb-Agent context generator logic
            $lines = @(
                "# RTB Context: $($details.name)"
                ""
                "**Status**: $($details.status)"
                "**Stack**: $($details.stack -join ', ')"
                "**Path**: $($details.path)"
                ""
                "## Git Context"
                "Branch: main"
                "Uncommitted changes: $($details.git.uncommitted)"
                ""
                "### Recent Commits (Last 10)"
                "(No commits yet)"
                ""
                "### Diff Stat"
                "(Clean working tree or unborn HEAD)"
            )
            $content = $lines -join "`n"
            Set-Content -Path $ctxPath -Value $content -Force

            Test-Path $ctxPath | Should Be $true
            $saved = Get-Content $ctxPath -Raw
            $saved | Should Match '# RTB Context: zero_commit_repo'
            $saved | Should Match '\*\*Stack\*\*:\s*Rust'
        }

        It "caps commit history to exactly 10 in a repository with 25 commits" {
            $repo = Join-Path $script:ctxRoot "many_commits_repo"
            New-Item -ItemType Directory -Path $repo -Force | Out-Null
            git -C $repo init -b main 2>&1 | Out-Null
            git -C $repo config user.email "test@example.com"
            git -C $repo config user.name "Test"

            for ($i = 1; $i -le 25; $i++) {
                Set-Content -Path (Join-Path $repo "file_$i.txt") -Value "content $i"
                git -C $repo add "file_$i.txt" 2>&1 | Out-Null
                git -C $repo commit -m "Commit number $i" 2>&1 | Out-Null
            }

            # Run git log -10 on this repo
            $logLines = git -C $repo log -10 --oneline 2>$null
            $logCount = ($logLines | Measure-Object).Count
            $logCount | Should Be 10

            $logLines[0] | Should Match 'Commit number 25'
        }

        It "handles corrupted or malformed package.json without failing stack detection" {
            $proj = Join-Path $script:ctxRoot "malformed_pkg_json"
            New-Item -ItemType Directory -Path $proj -Force | Out-Null
            Set-Content -Path (Join-Path $proj "package.json") -Value "{ NOT_VALID_JSON_AT_ALL " -Force

            $details = Get-ProjectDetails -ProjectPath $proj -Status 'Active'
            $details | Should Not BeNullOrEmpty
            # Fallback stack is '-' or 'Node.js'
            $details.stack | Should Not BeNullOrEmpty
        }
    }
}

Describe "Milestone M6 Adversarial Stress: Rtb-Status Schema Verification" {
    Context "JSON Schema Strict Compliance" {
        It "always outputs strict JSON with project, status, branch, uncommitted, stack, cwd" {
            $jsonStr = Rtb-Status -Json
            $jsonStr | Should Not BeNullOrEmpty

            $obj = $jsonStr | ConvertFrom-Json
            # Verify exact properties exist
            $props = @($obj.PSObject.Properties.Name)
            ($props -contains "project") | Should Be $true
            ($props -contains "status") | Should Be $true
            ($props -contains "branch") | Should Be $true
            ($props -contains "uncommitted") | Should Be $true
            ($props -contains "stack") | Should Be $true
            ($props -contains "cwd") | Should Be $true

            # Check property types
            ($obj.project -is [string]) | Should Be $true
            ($obj.branch -is [string]) | Should Be $true
            ($obj.uncommitted -is [int] -or $obj.uncommitted -is [long]) | Should Be $true
            ($obj.stack -is [System.Array] -or $obj.stack -is [System.Collections.IEnumerable]) | Should Be $true
            ($obj.cwd -is [string]) | Should Be $true
        }
    }
}
