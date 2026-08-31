# Stress & Edge Case Tests for Milestone M4 (Challenger 2)
# Tests: rtb status, -Json, nested subdirectories, outside paths, clean/dirty uncommitted counts, detached HEAD, stacks
# Compatible with Pester 3.4.0 and Pester 5+

Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

InModuleScope rtb {
    Describe "Milestone M4 Stress: rtb status Deep Subdirectories & Path Discovery" {
        BeforeAll {
            $script:stressBase = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_m4_stress_$([Guid]::NewGuid().ToString('N'))"
            $script:activeRoot = Join-Path $script:stressBase "01-Active"
            $script:pausedRoot = Join-Path $script:stressBase "04-Paused"
            $script:stagingRoot = Join-Path $script:stressBase "02-Deployed\02-Staging"
            $script:abandonedRoot = Join-Path $script:stressBase "05-Abandoned"

            New-Item -ItemType Directory -Path $script:activeRoot -Force | Out-Null
            New-Item -ItemType Directory -Path $script:pausedRoot -Force | Out-Null
            New-Item -ItemType Directory -Path $script:stagingRoot -Force | Out-Null
            New-Item -ItemType Directory -Path $script:abandonedRoot -Force | Out-Null

            # Setup Project 1: mega-app in Active root with 6-level deep subdirectory
            $script:proj1 = Join-Path $script:activeRoot "mega-app"
            $script:deepSub1 = Join-Path $script:proj1 "src\commands\sub\deep\level5\level6"
            New-Item -ItemType Directory -Path $script:deepSub1 -Force | Out-Null

            git -C $script:proj1 init --quiet 2>$null
            git -C $script:proj1 config user.name "Tester" 2>$null
            git -C $script:proj1 config user.email "tester@local" 2>$null
            Set-Content -Path (Join-Path $script:proj1 "package.json") -Value '{"name":"mega-app"}'
            Set-Content -Path (Join-Path $script:proj1 "Cargo.toml") -Value '[package]'
            Set-Content -Path (Join-Path $script:proj1 "rtb.psm1") -Value '# ps module'
            git -C $script:proj1 add . 2>$null
            git -C $script:proj1 commit -m "init mega-app" --quiet 2>$null
            $script:proj1Branch = (git -C $script:proj1 branch --show-current).Trim()

            # Setup Project 2: dirty-app in Paused root (1 staged, 1 modified, 1 untracked = 3 uncommitted)
            $script:proj2 = Join-Path $script:pausedRoot "dirty-app"
            New-Item -ItemType Directory -Path $script:proj2 -Force | Out-Null
            git -C $script:proj2 init --quiet 2>$null
            git -C $script:proj2 config user.name "Tester" 2>$null
            git -C $script:proj2 config user.email "tester@local" 2>$null
            Set-Content -Path (Join-Path $script:proj2 "initial.txt") -Value 'v1'
            git -C $script:proj2 add initial.txt 2>$null
            git -C $script:proj2 commit -m "init dirty-app" --quiet 2>$null
            Set-Content -Path (Join-Path $script:proj2 "initial.txt") -Value 'v2-modified'
            Set-Content -Path (Join-Path $script:proj2 "staged.txt") -Value 'staged-file'
            git -C $script:proj2 add staged.txt 2>$null
            Set-Content -Path (Join-Path $script:proj2 "untracked.txt") -Value 'untracked-file'

            # Setup Project 3: detached-app in Abandoned root (Detached HEAD)
            $script:proj3 = Join-Path $script:abandonedRoot "detached-app"
            New-Item -ItemType Directory -Path $script:proj3 -Force | Out-Null
            git -C $script:proj3 init --quiet 2>$null
            git -C $script:proj3 config user.name "Tester" 2>$null
            git -C $script:proj3 config user.email "tester@local" 2>$null
            Set-Content -Path (Join-Path $script:proj3 "c1.txt") -Value 'c1'
            git -C $script:proj3 add . 2>$null
            git -C $script:proj3 commit -m "commit 1" --quiet 2>$null
            Set-Content -Path (Join-Path $script:proj3 "c2.txt") -Value 'c2'
            git -C $script:proj3 add . 2>$null
            git -C $script:proj3 commit -m "commit 2" --quiet 2>$null
            $commit1Hash = (git -C $script:proj3 rev-parse HEAD~1).Trim()
            git -C $script:proj3 checkout $commit1Hash --quiet 2>$null

            # Setup Project 4: pygo-app in Staging (Non-git: Python + Go)
            $script:proj4 = Join-Path $script:stagingRoot "pygo-app"
            New-Item -ItemType Directory -Path $script:proj4 -Force | Out-Null
            Set-Content -Path (Join-Path $script:proj4 "requirements.txt") -Value "flask`nrequests"
            Set-Content -Path (Join-Path $script:proj4 "go.mod") -Value "module pygo"

            # Setup Outside Directory
            $script:outsideDir = Join-Path $script:stressBase "completely-outside\some\nested\folder"
            New-Item -ItemType Directory -Path $script:outsideDir -Force | Out-Null

            $script:mockConfig = [PSCustomObject]@{
                projectRoots = [PSCustomObject]@{
                    active     = $script:activeRoot
                    paused     = $script:pausedRoot
                    production = (Join-Path $script:stressBase "02-Deployed\01-Production")
                    staging    = $script:stagingRoot
                    vibe       = (Join-Path $script:stressBase "03-Vibe")
                    sandbox    = (Join-Path $script:stressBase "01-SandBox")
                    planning   = (Join-Path $script:stressBase "02-Planning")
                    testing    = (Join-Path $script:stressBase "03-Testing")
                    abandoned  = $script:abandonedRoot
                }
            }

            $script:origLocation = (Get-Location).Path
        }

        AfterAll {
            Set-Location $script:origLocation
            if ($script:stressBase -and (Test-Path $script:stressBase)) {
                Remove-Item -Recurse -Force $script:stressBase -ErrorAction SilentlyContinue
            }
        }

        Context "Deeply Nested Subdirectories" {
            It "resolves project name, status, branch, and stacks from 6 levels deep" {
                Push-Location $script:deepSub1
                try {
                    Mock Get-RtbConfig { return $script:mockConfig }
                    $plain = Rtb-Status
                    $plain | Should -Match "^rtb » mega-app \(Active\)"
                    $plain | Should -Match "\[$($script:proj1Branch)\]"
                    $plain | Should -Match "Node\.js"
                    $plain | Should -Match "Rust"
                    $plain | Should -Match "PowerShell"
                } finally {
                    Pop-Location
                }
            }

            It "resolves JSON fields correctly from 6 levels deep" {
                Push-Location $script:deepSub1
                try {
                    Mock Get-RtbConfig { return $script:mockConfig }
                    $jsonStr = Rtb-Status -Json
                    $obj = $jsonStr | ConvertFrom-Json
                    $obj.project | Should -Be "mega-app"
                    $obj.status | Should -Be "Active"
                    $obj.branch | Should -Be $script:proj1Branch
                    $obj.uncommitted | Should -Be 0
                    $obj.cwd | Should -Be $script:deepSub1
                    ($obj.stack -contains 'Node.js') | Should -Be $true
                    ($obj.stack -contains 'Rust') | Should -Be $true
                    ($obj.stack -contains 'PowerShell') | Should -Be $true
                } finally {
                    Pop-Location
                }
            }
        }

        Context "Execution Outside Project Roots" {
            It "falls back to leaf directory when outside any project root" {
                Push-Location $script:outsideDir
                try {
                    Mock Get-RtbConfig { return $script:mockConfig }
                    $plain = Rtb-Status
                    $plain | Should -Match "^rtb » folder"
                    $plain -notmatch "\((Active|Paused|Staging)\)" | Should -Be $true

                    $jsonStr = Rtb-Status -Json
                    $obj = $jsonStr | ConvertFrom-Json
                    $obj.project | Should -Be "folder"
                    $obj.status | Should -BeNullOrEmpty
                    $obj.cwd | Should -Be $script:outsideDir
                } finally {
                    Pop-Location
                }
            }

            It "executes gracefully at Drive Root (e.g. D:\)" {
                Push-Location "D:\"
                try {
                    Mock Get-RtbConfig { return $script:mockConfig }
                    $plain = Rtb-Status
                    $plain | Should -Not -BeNullOrEmpty

                    $jsonStr = Rtb-Status -Json
                    $obj = $jsonStr | ConvertFrom-Json
                    $obj.cwd | Should -Be "D:\"
                } finally {
                    Pop-Location
                }
            }

            It "executes gracefully in `$env:TEMP" {
                Push-Location $env:TEMP
                try {
                    Mock Get-RtbConfig { return $script:mockConfig }
                    $plain = Rtb-Status
                    $plain | Should -Not -BeNullOrEmpty

                    $jsonStr = Rtb-Status -Json
                    $obj = $jsonStr | ConvertFrom-Json
                    $obj.cwd | Should -Be (Get-Item $env:TEMP).FullName
                } finally {
                    Pop-Location
                }
            }
        }

        Context "JSON Flag Variants and Schema Integrity" {
            It "supports -Json, --json, -j, -J, -json with identical full schema" {
                Push-Location $script:proj1
                try {
                    Mock Get-RtbConfig { return $script:mockConfig }
                    $flags = @('-Json', '--json', '-j', '-J', '-json')

                    foreach ($f in $flags) {
                        $raw = if ($f -eq '-Json') { Rtb-Status -Json } else { Rtb-Status $f }
                        $obj = $raw | ConvertFrom-Json
                        $obj | Should -Not -BeNullOrEmpty
                        $obj.project | Should -Be "mega-app"
                        $obj.status | Should -Be "Active"
                        $obj.branch | Should -Be $script:proj1Branch
                        $obj.uncommitted | Should -Be 0
                        $obj.cwd | Should -Be $script:proj1

                        ($obj.PSObject.Properties.Name -contains 'project') | Should -Be $true
                        ($obj.PSObject.Properties.Name -contains 'status') | Should -Be $true
                        ($obj.PSObject.Properties.Name -contains 'branch') | Should -Be $true
                        ($obj.PSObject.Properties.Name -contains 'uncommitted') | Should -Be $true
                        ($obj.PSObject.Properties.Name -contains 'stack') | Should -Be $true
                        ($obj.PSObject.Properties.Name -contains 'cwd') | Should -Be $true
                    }
                } finally {
                    Pop-Location
                }
            }
        }

        Context "Uncommitted Changes Counter (Clean vs Dirty)" {
            It "returns 0 uncommitted on clean repository" {
                Push-Location $script:proj1
                try {
                    Mock Get-RtbConfig { return $script:mockConfig }
                    $plain = Rtb-Status
                    $plain -notmatch "±" | Should -Be $true

                    $jsonStr = Rtb-Status -Json
                    $obj = $jsonStr | ConvertFrom-Json
                    $obj.uncommitted | Should -Be 0
                } finally {
                    Pop-Location
                }
            }

            It "counts staged, modified, and untracked changes accurately (3 uncommitted)" {
                Push-Location $script:proj2
                try {
                    Mock Get-RtbConfig { return $script:mockConfig }
                    $plain = Rtb-Status
                    $plain | Should -Match "±3"

                    $jsonStr = Rtb-Status -Json
                    $obj = $jsonStr | ConvertFrom-Json
                    $obj.uncommitted | Should -Be 3
                    $obj.status | Should -Be "Paused"
                } finally {
                    Pop-Location
                }
            }

            It "dynamically reflects newly added untracked files (7 uncommitted)" {
                Push-Location $script:proj2
                try {
                    Mock Get-RtbConfig { return $script:mockConfig }
                    1..4 | ForEach-Object { Set-Content -Path (Join-Path $script:proj2 "stress_file_$_.tmp") -Value "test" }

                    $plain = Rtb-Status
                    $plain | Should -Match "±7"

                    $jsonStr = Rtb-Status -Json
                    $obj = $jsonStr | ConvertFrom-Json
                    $obj.uncommitted | Should -Be 7

                    # Clean up the 4 files
                    1..4 | ForEach-Object { Remove-Item (Join-Path $script:proj2 "stress_file_$_.tmp") -Force -ErrorAction SilentlyContinue }
                } finally {
                    Pop-Location
                }
            }
        }

        Context "Detached HEAD and Non-Git Repositories" {
            It "handles detached HEAD state by showing HEAD@<hash>" {
                Push-Location $script:proj3
                try {
                    Mock Get-RtbConfig { return $script:mockConfig }
                    $plain = Rtb-Status
                    $plain | Should -Match "\[HEAD@[0-9a-f]+\]"
                    $plain | Should -Match "\(Abandoned\)"

                    $jsonStr = Rtb-Status -Json
                    $obj = $jsonStr | ConvertFrom-Json
                    $obj.branch | Should -Match "^HEAD@[0-9a-f]+"
                    $obj.status | Should -Be "Abandoned"
                } finally {
                    Pop-Location
                }
            }

            It "handles non-git project in Staging root" {
                Push-Location $script:proj4
                try {
                    Mock Get-RtbConfig { return $script:mockConfig }
                    $plain = Rtb-Status
                    $plain | Should -Match "^rtb » pygo-app \(Staging\)"
                    $plain -notmatch "\[" | Should -Be $true
                    $plain | Should -Match "Python"
                    $plain | Should -Match "Go"

                    $jsonStr = Rtb-Status -Json
                    $obj = $jsonStr | ConvertFrom-Json
                    $obj.project | Should -Be "pygo-app"
                    $obj.status | Should -Be "Staging"
                    $obj.branch | Should -Be ""
                    $obj.uncommitted | Should -Be 0
                    ($obj.stack -contains 'Python') | Should -Be $true
                    ($obj.stack -contains 'Go') | Should -Be $true
                } finally {
                    Pop-Location
                }
            }
        }

        Context "Top-Level CLI & Alias Dispatchers" {
            It "dispatches 'rtb status' and 'dev status'" {
                Push-Location $script:proj1
                try {
                    Mock Get-RtbConfig { return $script:mockConfig }
                    $rtbOut = rtb status
                    $rtbOut | Should -Match "^rtb » mega-app \(Active\)"

                    $devOut = dev status
                    $devOut | Should -Match "^rtb » mega-app \(Active\)"
                } finally {
                    Pop-Location
                }
            }

            It "dispatches 'rtb status -Json', 'rtb status -j', 'rtb status --json'" {
                Push-Location $script:proj1
                try {
                    Mock Get-RtbConfig { return $script:mockConfig }

                    $j1 = (rtb status -Json) | ConvertFrom-Json
                    $j1.project | Should -Be "mega-app"

                    $j2 = (rtb status -j) | ConvertFrom-Json
                    $j2.project | Should -Be "mega-app"

                    $j3 = (rtb status --json) | ConvertFrom-Json
                    $j3.project | Should -Be "mega-app"

                    $j4 = (dev status -Json) | ConvertFrom-Json
                    $j4.project | Should -Be "mega-app"
                } finally {
                    Pop-Location
                }
            }
        }
    }
}
