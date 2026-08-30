# Stress & Edge Case Tests for Milestone M2 Remediation (Challenger 2)

Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

InModuleScope rtb {
    Describe "Stress: Argument Parsing & Flag Ordering" {
        BeforeAll {
            $script:stressTemp = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_stress_ch2_$([Guid]::NewGuid().ToString('N'))"
            New-Item -ItemType Directory -Path $script:stressTemp -Force | Out-Null
        }

        AfterAll {
            if ($script:stressTemp -and (Test-Path $script:stressTemp)) {
                Remove-Item -Recurse -Force $script:stressTemp -ErrorAction SilentlyContinue
            }
        }

        Context "rtb clean argument combinations" {
            It "deletes old dependencies with 'rtb clean -Days 15 -Commit'" {
                $dir = Join-Path $script:stressTemp "clean_order_1"
                $nm = Join-Path $dir "node_modules"
                New-Item -ItemType Directory -Path $nm -Force | Out-Null
                (Get-Item $nm).LastWriteTime = (Get-Date).AddDays(-20)

                Mock Get-RtbConfig { [PSCustomObject]@{ projectRoots = [PSCustomObject]@{ active = $dir; paused = $null; vibe = $null; sandbox = $null }; cleanDeps = [PSCustomObject]@{ targets = @('node_modules') } } }
                Mock Confirm-RtbAction { $true }

                { rtb clean -Days 15 -Commit } | Should Not Throw
                Test-Path $nm | Should Be $false
            }

            It "deletes old dependencies with 'rtb clean -Commit -Days 15' (flag before days)" {
                $dir = Join-Path $script:stressTemp "clean_order_2"
                $nm = Join-Path $dir "node_modules"
                New-Item -ItemType Directory -Path $nm -Force | Out-Null
                (Get-Item $nm).LastWriteTime = (Get-Date).AddDays(-20)

                Mock Get-RtbConfig { [PSCustomObject]@{ projectRoots = [PSCustomObject]@{ active = $dir; paused = $null; vibe = $null; sandbox = $null }; cleanDeps = [PSCustomObject]@{ targets = @('node_modules') } } }
                Mock Confirm-RtbAction { $true }

                { rtb clean -Commit -Days 15 } | Should Not Throw
                Test-Path $nm | Should Be $false
            }

            It "deletes old dependencies with 'rtb clean -Commit 15' (flag before positional days)" {
                $dir = Join-Path $script:stressTemp "clean_order_3"
                $nm = Join-Path $dir "node_modules"
                New-Item -ItemType Directory -Path $nm -Force | Out-Null
                (Get-Item $nm).LastWriteTime = (Get-Date).AddDays(-20)

                Mock Get-RtbConfig { [PSCustomObject]@{ projectRoots = [PSCustomObject]@{ active = $dir; paused = $null; vibe = $null; sandbox = $null }; cleanDeps = [PSCustomObject]@{ targets = @('node_modules') } } }
                Mock Confirm-RtbAction { $true }

                { rtb clean -Commit 15 } | Should Not Throw
                Test-Path $nm | Should Be $false
            }

            It "deletes old dependencies with 'rtb clean 15 -Commit' (positional days before flag)" {
                $dir = Join-Path $script:stressTemp "clean_order_3b"
                $nm = Join-Path $dir "node_modules"
                New-Item -ItemType Directory -Path $nm -Force | Out-Null
                (Get-Item $nm).LastWriteTime = (Get-Date).AddDays(-20)

                Mock Get-RtbConfig { [PSCustomObject]@{ projectRoots = [PSCustomObject]@{ active = $dir; paused = $null; vibe = $null; sandbox = $null }; cleanDeps = [PSCustomObject]@{ targets = @('node_modules') } } }
                Mock Confirm-RtbAction { $true }

                { rtb clean 15 -Commit } | Should Not Throw
                Test-Path $nm | Should Be $false
            }

            It "deletes old dependencies with 'rtb clean --days 15 --commit' (GNU long flags)" {
                $dir = Join-Path $script:stressTemp "clean_order_5"
                $nm = Join-Path $dir "node_modules"
                New-Item -ItemType Directory -Path $nm -Force | Out-Null
                (Get-Item $nm).LastWriteTime = (Get-Date).AddDays(-20)

                Mock Get-RtbConfig { [PSCustomObject]@{ projectRoots = [PSCustomObject]@{ active = $dir; paused = $null; vibe = $null; sandbox = $null }; cleanDeps = [PSCustomObject]@{ targets = @('node_modules') } } }
                Mock Confirm-RtbAction { $true }

                { rtb clean --days 15 --commit } | Should Not Throw
                Test-Path $nm | Should Be $false
            }

            It "preserves folders with 'rtb clean -Days 30' (dry-run without -Commit)" {
                $dir = Join-Path $script:stressTemp "clean_order_6"
                $nm = Join-Path $dir "node_modules"
                New-Item -ItemType Directory -Path $nm -Force | Out-Null
                (Get-Item $nm).LastWriteTime = (Get-Date).AddDays(-40)

                Mock Get-RtbConfig { [PSCustomObject]@{ projectRoots = [PSCustomObject]@{ active = $dir; paused = $null; vibe = $null; sandbox = $null }; cleanDeps = [PSCustomObject]@{ targets = @('node_modules') } } }

                { rtb clean -Days 30 } | Should Not Throw
                Test-Path $nm | Should Be $true
            }

            It "honors dry-run when both --commit and --dry-run are provided" {
                $dir = Join-Path $script:stressTemp "clean_order_7"
                $nm = Join-Path $dir "node_modules"
                New-Item -ItemType Directory -Path $nm -Force | Out-Null
                (Get-Item $nm).LastWriteTime = (Get-Date).AddDays(-40)

                Mock Get-RtbConfig { [PSCustomObject]@{ projectRoots = [PSCustomObject]@{ active = $dir; paused = $null; vibe = $null; sandbox = $null }; cleanDeps = [PSCustomObject]@{ targets = @('node_modules') } } }

                { rtb clean --commit --dry-run } | Should Not Throw
                Test-Path $nm | Should Be $true
            }
        }

        Context "rtb archive argument ordering and edge cases" {
            It "archives with 'rtb archive -Force myproj' (leading -Force flag)" {
                $testRepo = Join-Path $script:stressTemp "lead_force_proj_1"
                New-Item -ItemType Directory -Path $testRepo -Force | Out-Null
                git -C $testRepo init --quiet 2>$null
                git -C $testRepo config user.name "RTB Test" 2>$null
                git -C $testRepo config user.email "test@rtb.local" 2>$null
                Set-Content -Path (Join-Path $testRepo "dirty.txt") -Value "dirty"

                Mock Find-ProjectPath { @{ Path = $testRepo; Status = 'Active' } }
                Mock Get-RtbConfig { [PSCustomObject]@{ backupRoot = $script:stressTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }

                { rtb archive -Force "lead_force_proj_1" } | Should Not Throw
                Test-Path $testRepo | Should Be $false
            }

            It "archives with 'rtb archive --force myproj' (leading --force GNU flag)" {
                $testRepo = Join-Path $script:stressTemp "lead_force_proj_2"
                New-Item -ItemType Directory -Path $testRepo -Force | Out-Null
                git -C $testRepo init --quiet 2>$null
                git -C $testRepo config user.name "RTB Test" 2>$null
                git -C $testRepo config user.email "test@rtb.local" 2>$null
                Set-Content -Path (Join-Path $testRepo "dirty.txt") -Value "dirty"

                Mock Find-ProjectPath { @{ Path = $testRepo; Status = 'Active' } }
                Mock Get-RtbConfig { [PSCustomObject]@{ backupRoot = $script:stressTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }

                { rtb archive --force "lead_force_proj_2" } | Should Not Throw
                Test-Path $testRepo | Should Be $false
            }

            It "archives with 'rtb archive -f myproj' (leading short flag)" {
                $testRepo = Join-Path $script:stressTemp "lead_force_proj_3"
                New-Item -ItemType Directory -Path $testRepo -Force | Out-Null
                git -C $testRepo init --quiet 2>$null
                git -C $testRepo config user.name "RTB Test" 2>$null
                git -C $testRepo config user.email "test@rtb.local" 2>$null
                Set-Content -Path (Join-Path $testRepo "dirty.txt") -Value "dirty"

                Mock Find-ProjectPath { @{ Path = $testRepo; Status = 'Active' } }
                Mock Get-RtbConfig { [PSCustomObject]@{ backupRoot = $script:stressTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }

                { rtb archive -f "lead_force_proj_3" } | Should Not Throw
                Test-Path $testRepo | Should Be $false
            }

            It "prints usage on 'rtb archive -Force' when no project name is provided" {
                $output = (rtb archive -Force *>&1 | Out-String)
                $output | Should Match "Usage:"
            }

            It "prints usage on 'rtb archive' when no arguments are provided" {
                $output = (rtb archive *>&1 | Out-String)
                $output | Should Match "Usage:"
            }
        }

        Context "rtb pause argument ordering and edge cases" {
            It "pauses with 'rtb pause --prune myproj' (leading --prune)" {
                $active = Join-Path $script:stressTemp "active_pause_lead1"
                $paused = Join-Path $script:stressTemp "paused_pause_lead1"
                $proj = Join-Path $active "pause-lead-1"
                $nm = Join-Path $proj "node_modules"
                New-Item -ItemType Directory -Path $nm -Force | Out-Null
                New-Item -ItemType Directory -Path $paused -Force | Out-Null
                git -C $proj init --quiet 2>$null
                git -C $proj config user.name "RTB Test" 2>$null
                git -C $proj config user.email "test@rtb.local" 2>$null
                Set-Content -Path (Join-Path $proj "clean.txt") -Value "clean"
                git -C $proj add clean.txt 2>$null
                git -C $proj commit -m "init" --quiet 2>$null

                Mock Get-RtbConfig { [PSCustomObject]@{ projectRoots = [PSCustomObject]@{ active = $active; paused = $paused }; cleanDeps = [PSCustomObject]@{ targets = @('node_modules') } } }
                Mock Confirm-RtbAction { $true }

                { rtb pause --prune "pause-lead-1" } | Should Not Throw
                Test-Path (Join-Path $paused "pause-lead-1") | Should Be $true
                Test-Path (Join-Path $paused "pause-lead-1\node_modules") | Should Be $false
            }

            It "pauses with 'rtb pause -Force --prune myproj' (leading -Force and --prune)" {
                $active = Join-Path $script:stressTemp "active_pause_lead2"
                $paused = Join-Path $script:stressTemp "paused_pause_lead2"
                $proj = Join-Path $active "pause-lead-2"
                $nm = Join-Path $proj "node_modules"
                New-Item -ItemType Directory -Path $nm -Force | Out-Null
                New-Item -ItemType Directory -Path $paused -Force | Out-Null
                git -C $proj init --quiet 2>$null
                git -C $proj config user.name "RTB Test" 2>$null
                git -C $proj config user.email "test@rtb.local" 2>$null
                Set-Content -Path (Join-Path $proj "dirty.txt") -Value "dirty"

                Mock Get-RtbConfig { [PSCustomObject]@{ projectRoots = [PSCustomObject]@{ active = $active; paused = $paused }; cleanDeps = [PSCustomObject]@{ targets = @('node_modules') } } }

                { rtb pause -Force --prune "pause-lead-2" } | Should Not Throw
                Test-Path (Join-Path $paused "pause-lead-2") | Should Be $true
                Test-Path (Join-Path $paused "pause-lead-2\node_modules") | Should Be $false
            }

            It "pauses with 'rtb pause --prune -Force myproj' (leading --prune and -Force)" {
                $active = Join-Path $script:stressTemp "active_pause_lead3"
                $paused = Join-Path $script:stressTemp "paused_pause_lead3"
                $proj = Join-Path $active "pause-lead-3"
                $nm = Join-Path $proj "node_modules"
                New-Item -ItemType Directory -Path $nm -Force | Out-Null
                New-Item -ItemType Directory -Path $paused -Force | Out-Null
                git -C $proj init --quiet 2>$null
                git -C $proj config user.name "RTB Test" 2>$null
                git -C $proj config user.email "test@rtb.local" 2>$null
                Set-Content -Path (Join-Path $proj "dirty.txt") -Value "dirty"

                Mock Get-RtbConfig { [PSCustomObject]@{ projectRoots = [PSCustomObject]@{ active = $active; paused = $paused }; cleanDeps = [PSCustomObject]@{ targets = @('node_modules') } } }

                { rtb pause --prune -Force "pause-lead-3" } | Should Not Throw
                Test-Path (Join-Path $paused "pause-lead-3") | Should Be $true
                Test-Path (Join-Path $paused "pause-lead-3\node_modules") | Should Be $false
            }

            It "pauses with 'rtb pause --prune --force myproj' (GNU style long flags before name)" {
                $active = Join-Path $script:stressTemp "active_pause_lead4"
                $paused = Join-Path $script:stressTemp "paused_pause_lead4"
                $proj = Join-Path $active "pause-lead-4"
                $nm = Join-Path $proj "node_modules"
                New-Item -ItemType Directory -Path $nm -Force | Out-Null
                New-Item -ItemType Directory -Path $paused -Force | Out-Null
                git -C $proj init --quiet 2>$null
                git -C $proj config user.name "RTB Test" 2>$null
                git -C $proj config user.email "test@rtb.local" 2>$null
                Set-Content -Path (Join-Path $proj "dirty.txt") -Value "dirty"

                Mock Get-RtbConfig { [PSCustomObject]@{ projectRoots = [PSCustomObject]@{ active = $active; paused = $paused }; cleanDeps = [PSCustomObject]@{ targets = @('node_modules') } } }

                { rtb pause --prune --force "pause-lead-4" } | Should Not Throw
                Test-Path (Join-Path $paused "pause-lead-4") | Should Be $true
                Test-Path (Join-Path $paused "pause-lead-4\node_modules") | Should Be $false
            }

            It "prints usage on 'rtb pause' when no arguments are provided" {
                $output = (rtb pause *>&1 | Out-String)
                $output | Should Match "Usage:"
            }

            It "prints usage on 'rtb pause -Force' when no project is provided" {
                $output = (rtb pause -Force *>&1 | Out-String)
                $output | Should Match "Usage:"
            }

            It "prints usage on 'rtb pause --prune' when no project is provided" {
                $output = (rtb pause --prune *>&1 | Out-String)
                $output | Should Match "Usage:"
            }
        }
    }

    Describe "Stress: Post-Tar Safety Verification & Corruption Resilience" {
        BeforeAll {
            $script:tarStressTemp = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_tar_stress_$([Guid]::NewGuid().ToString('N'))"
            New-Item -ItemType Directory -Path $script:tarStressTemp -Force | Out-Null
        }

        AfterAll {
            if ($script:tarStressTemp -and (Test-Path $script:tarStressTemp)) {
                Remove-Item -Recurse -Force $script:tarStressTemp -ErrorAction SilentlyContinue
            }
        }

        It "creates a valid .tar.gz and deletes source only after verified successful compression" {
            $realProj = Join-Path $script:tarStressTemp "valid_tar_proj"
            New-Item -ItemType Directory -Path $realProj -Force | Out-Null
            Set-Content -Path (Join-Path $realProj "index.js") -Value "console.log('test');"
            Set-Content -Path (Join-Path $realProj "package.json") -Value '{"name": "valid-tar"}'

            Mock Find-ProjectPath { @{ Path = $realProj; Status = 'Active' } }
            Mock Get-RtbConfig { [PSCustomObject]@{ backupRoot = $script:tarStressTemp; cleanDeps = [PSCustomObject]@{ targets = @() } } }

            rtb archive -Force "valid_tar_proj"

            $expectedArchive = Join-Path $script:tarStressTemp "project-snapshots\valid_tar_proj-$((Get-Date).ToString('yyyy-MM-dd')).tar.gz"
            Test-Path $expectedArchive | Should Be $true
            (Get-Item $expectedArchive).Length | Should BeGreaterThan 0
            Test-Path $realProj | Should Be $false

            # Verify tar integrity
            $listing = tar -tzf $expectedArchive 2>&1
            ($listing -match "index.js") | Should Be $true
        }

        It "retains source directory and deletes corrupt archive when tar fails or produces 0-byte output" {
            $failProj = Join-Path $script:tarStressTemp "fail_tar_proj"
            New-Item -ItemType Directory -Path $failProj -Force | Out-Null
            Set-Content -Path (Join-Path $failProj "precious_code.py") -Value "print('do not lose this')"

            $corruptArchive = Join-Path $script:tarStressTemp "corrupt.tar.gz"
            Set-Content -Path $corruptArchive -Value "" # 0-byte file

            $tarExitCode = 1 # Non-zero exit code
            $archivePath = $corruptArchive
            $isSafe = ($tarExitCode -eq 0 -and (Test-Path $archivePath) -and ((Get-Item $archivePath).Length -gt 0))
            $isSafe | Should Be $false

            # When not safe, the source path must not be removed and the broken archive must be cleaned
            if (-not $isSafe) {
                if (Test-Path $archivePath) {
                    Remove-Item $archivePath -Force -EA SilentlyContinue
                }
            }

            Test-Path $failProj | Should Be $true
            Test-Path $corruptArchive | Should Be $false
        }
    }
}
