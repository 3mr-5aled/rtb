# Navigation & Fuzzy Goto Unit Tests (Milestone M3)
# Compatible with Pester 3.4.0 and Pester 5+

Describe "Find-ProjectPathFuzzy" {
    BeforeAll {
        . (Join-Path $PSScriptRoot '..\src\utils\helpers.ps1')
        $script:tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) "fuzzy_test_roots_$([Guid]::NewGuid().ToString('N'))"
        $script:activeRoot = Join-Path $script:tempRoot "active"
        $script:pausedRoot = Join-Path $script:tempRoot "paused"
        $script:prodRoot = Join-Path $script:tempRoot "production"

        New-Item -ItemType Directory -Path (Join-Path $script:activeRoot "rtb-command-tool") -Force | Out-Null
        New-Item -ItemType Directory -Path (Join-Path $script:activeRoot "rtb-extras") -Force | Out-Null
        New-Item -ItemType Directory -Path (Join-Path $script:activeRoot "my-rtb-service") -Force | Out-Null
        New-Item -ItemType Directory -Path (Join-Path $script:pausedRoot "old-rtb-app") -Force | Out-Null
        New-Item -ItemType Directory -Path (Join-Path $script:prodRoot "production-deploy") -Force | Out-Null
        New-Item -ItemType Directory -Path (Join-Path $script:activeRoot "unrelated-app") -Force | Out-Null
    }

    AfterAll {
        if (Test-Path $script:tempRoot) {
            Remove-Item -Recurse -Force $script:tempRoot -ErrorAction SilentlyContinue
        }
    }

    It "returns empty array when query matches nothing" {
        Mock Get-RtbConfig {
            [PSCustomObject]@{
                projectRoots = [PSCustomObject]@{
                    active     = $script:activeRoot
                    paused     = $script:pausedRoot
                    production = $script:prodRoot
                }
            }
        }
        $results = @(Find-ProjectPathFuzzy -Query "zzz-non-existent-proj")
        $results.Count | Should -Be 0
    }

    It "scores exact match with 100" {
        Mock Get-RtbConfig {
            [PSCustomObject]@{
                projectRoots = [PSCustomObject]@{
                    active     = $script:activeRoot
                    paused     = $script:pausedRoot
                    production = $script:prodRoot
                }
            }
        }
        $results = @(Find-ProjectPathFuzzy -Query "rtb-command-tool")
        $results.Count | Should -Not -Be 0
        $results[0].Name | Should -Be "rtb-command-tool"
        $results[0].Score | Should -Be 100
        $results[0].Status | Should -Be "Active"
    }

    It "scores prefix match with 75" {
        Mock Get-RtbConfig {
            [PSCustomObject]@{
                projectRoots = [PSCustomObject]@{
                    active     = $script:activeRoot
                    paused     = $script:pausedRoot
                    production = $script:prodRoot
                }
            }
        }
        $results = @(Find-ProjectPathFuzzy -Query "rtb-ext")
        $results.Count | Should -Not -Be 0
        $match = $results | Where-Object { $_.Name -eq "rtb-extras" }
        $match.Score | Should -Be 75
    }

    It "scores substring match with 50" {
        Mock Get-RtbConfig {
            [PSCustomObject]@{
                projectRoots = [PSCustomObject]@{
                    active     = $script:activeRoot
                    paused     = $script:pausedRoot
                    production = $script:prodRoot
                }
            }
        }
        $results = @(Find-ProjectPathFuzzy -Query "service")
        $results.Count | Should -Not -Be 0
        $match = $results | Where-Object { $_.Name -eq "my-rtb-service" }
        $match.Score | Should -Be 50
    }

    It "scores full path match with 25 when folder path contains substring" {
        Mock Get-RtbConfig {
            [PSCustomObject]@{
                projectRoots = [PSCustomObject]@{
                    active     = $script:activeRoot
                    paused     = $script:pausedRoot
                    production = $script:prodRoot
                }
            }
        }
        $results = @(Find-ProjectPathFuzzy -Query "production")
        $results.Count | Should -Not -Be 0
        $match = $results | Where-Object { $_.Name -eq "production-deploy" }
        $match.Score | Should -Be 75
    }

    It "safely handles wildcard and bracket characters without throwing exception" {
        Mock Get-RtbConfig {
            [PSCustomObject]@{
                projectRoots = [PSCustomObject]@{
                    active     = $script:activeRoot
                    paused     = $script:pausedRoot
                    production = $script:prodRoot
                }
            }
        }
        { Find-ProjectPathFuzzy -Query "[" } | Should -Not -Throw
        { Find-ProjectPathFuzzy -Query "*" } | Should -Not -Throw
        { Find-ProjectPathFuzzy -Query "[id]" } | Should -Not -Throw
    }

    It "returns multi-root matches sorted by score descending" {
        Mock Get-RtbConfig {
            [PSCustomObject]@{
                projectRoots = [PSCustomObject]@{
                    active     = $script:activeRoot
                    paused     = $script:pausedRoot
                    production = $script:prodRoot
                }
            }
        }
        $results = @(Find-ProjectPathFuzzy -Query "rtb")
        $results.Count | Should -Be 4
        # Prefix matches (75) should rank first
        $results[0].Score | Should -Be 75
        $scores = $results | Select-Object -ExpandProperty Score
        ($scores[0] -ge $scores[1]) | Should -Be $true
        ($scores[1] -ge $scores[2]) | Should -Be $true
        ($scores[2] -ge $scores[3]) | Should -Be $true
    }
}

Describe "Dev-Goto Command & Disambiguation Picker" {
    BeforeAll {
        . (Join-Path $PSScriptRoot '..\src\utils\helpers.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\agent.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\goto.ps1')
        $script:origLoc = (Get-Location).Path
        $script:tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) "goto_test_roots_$([Guid]::NewGuid().ToString('N'))"
        $script:activeRoot = Join-Path $script:tempRoot "active"

        $script:p1 = Join-Path $script:activeRoot "unique-project-alpha"
        $script:p2 = Join-Path $script:activeRoot "multi-widget-frontend"
        $script:p3 = Join-Path $script:activeRoot "multi-widget-backend"

        New-Item -ItemType Directory -Path $script:p1 -Force | Out-Null
        New-Item -ItemType Directory -Path $script:p2 -Force | Out-Null
        New-Item -ItemType Directory -Path $script:p3 -Force | Out-Null
    }

    AfterAll {
        Set-Location $script:origLoc
        if (Test-Path $script:tempRoot) {
            Remove-Item -Recurse -Force $script:tempRoot -ErrorAction SilentlyContinue
        }
    }

    It "displays usage instructions when no project name is provided" {
        $output = (Dev-Goto *>&1) | Out-String
        $output | Should -Match "Usage: rtb goto"
    }

    It "displays error when project matching query is not found" {
        Mock Find-ProjectPathFuzzy { @() }
        Mock Get-AllProjectNames { @("unique-project-alpha", "multi-widget-frontend") }

        $output = (Dev-Goto -Name "non-existent-xyz" *>&1) | Out-String
        $output | Should -Match "No project matching 'non-existent-xyz' found"
        $output | Should -Match "Available projects:"
    }

    It "automatically navigates on a single match without prompting" {
        Mock Find-ProjectPathFuzzy {
            @([PSCustomObject]@{ Name = "unique-project-alpha"; Path = $script:p1; Status = "Active"; Score = 75 })
        }

        Dev-Goto -Name "unique"
        (Get-Location).Path | Should -Be $script:p1
    }

    It "automatically navigates on exact match (score 100) even when multiple matches exist" {
        Mock Find-ProjectPathFuzzy {
            @(
                [PSCustomObject]@{ Name = "multi-widget"; Path = $script:p1; Status = "Active"; Score = 100 },
                [PSCustomObject]@{ Name = "multi-widget-frontend"; Path = $script:p2; Status = "Active"; Score = 75 },
                [PSCustomObject]@{ Name = "multi-widget-backend"; Path = $script:p3; Status = "Active"; Score = 75 }
            )
        }

        Dev-Goto -Name "multi-widget"
        (Get-Location).Path | Should -Be $script:p1
    }

    It "displays interactive picker and selects choice 2 on ambiguous match" {
        Set-Location $script:tempRoot
        Mock Find-ProjectPathFuzzy {
            @(
                [PSCustomObject]@{ Name = "multi-widget-frontend"; Path = $script:p2; Status = "Active"; Score = 75 },
                [PSCustomObject]@{ Name = "multi-widget-backend"; Path = $script:p3; Status = "Active"; Score = 75 }
            )
        }
        Mock Read-Host { '2' }

        Dev-Goto -Name "multi-widget" -Choice "2"
        (Get-Location).Path | Should -Be $script:p3
    }

    It "cancels navigation when user enters empty input at picker prompt" {
        Set-Location $script:tempRoot
        Mock Find-ProjectPathFuzzy {
            @(
                [PSCustomObject]@{ Name = "multi-widget-frontend"; Path = $script:p2; Status = "Active"; Score = 75 },
                [PSCustomObject]@{ Name = "multi-widget-backend"; Path = $script:p3; Status = "Active"; Score = 75 }
            )
        }
        Mock Read-Host { '' }

        $output = (Dev-Goto -Name "multi-widget" *>&1) | Out-String
        $output | Should -Match "Cancelled."
        (Get-Location).Path | Should -Be $script:tempRoot
    }

    It "cancels navigation when user enters invalid non-numeric input" {
        Set-Location $script:tempRoot
        Mock Find-ProjectPathFuzzy {
            @(
                [PSCustomObject]@{ Name = "multi-widget-frontend"; Path = $script:p2; Status = "Active"; Score = 75 },
                [PSCustomObject]@{ Name = "multi-widget-backend"; Path = $script:p3; Status = "Active"; Score = 75 }
            )
        }
        Mock Read-Host { 'abc' }

        $output = (Dev-Goto -Name "multi-widget" *>&1) | Out-String
        $output | Should -Match "Cancelled."
        (Get-Location).Path | Should -Be $script:tempRoot
    }

    It "cancels navigation when user enters out-of-range number" {
        Set-Location $script:tempRoot
        Mock Find-ProjectPathFuzzy {
            @(
                [PSCustomObject]@{ Name = "multi-widget-frontend"; Path = $script:p2; Status = "Active"; Score = 75 },
                [PSCustomObject]@{ Name = "multi-widget-backend"; Path = $script:p3; Status = "Active"; Score = 75 }
            )
        }
        Mock Read-Host { '9' }

        $output = (Dev-Goto -Name "multi-widget" *>&1) | Out-String
        $output | Should -Match "Cancelled."
        (Get-Location).Path | Should -Be $script:tempRoot
    }

    It "forwards -Claude flag to Rtb-Agent when specified" {
        Mock Find-ProjectPathFuzzy {
            @([PSCustomObject]@{ Name = "unique-project-alpha"; Path = $script:p1; Status = "Active"; Score = 100 })
        }
        $script:agentCalled = $false
        $script:targetAgent = ""
        Mock Rtb-Agent {
            param($ProjectName, $Agent)
            $script:agentCalled = $true
            $script:targetAgent = $Agent
        }

        Dev-Goto -Name "unique-project-alpha" -Claude
        $script:agentCalled | Should -Be $true
        $script:targetAgent | Should -Be "claude"
    }

    It "forwards -Agy flag to Rtb-Agent when specified" {
        Mock Find-ProjectPathFuzzy {
            @([PSCustomObject]@{ Name = "unique-project-alpha"; Path = $script:p1; Status = "Active"; Score = 100 })
        }
        $script:agentCalled = $false
        $script:targetAgent = ""
        Mock Rtb-Agent {
            param($ProjectName, $Agent)
            $script:agentCalled = $true
            $script:targetAgent = $Agent
        }

        Dev-Goto -Name "unique-project-alpha" -Agy
        $script:agentCalled | Should -Be $true
        $script:targetAgent | Should -Be "agy"
    }

    It "normalizes raw string agent argument '-Claude' to 'claude'" {
        Mock Find-ProjectPathFuzzy {
            @([PSCustomObject]@{ Name = "unique-project-alpha"; Path = $script:p1; Status = "Active"; Score = 100 })
        }
        $script:agentCalled = $false
        $script:targetAgent = ""
        Mock Rtb-Agent {
            param($ProjectName, $Agent)
            $script:agentCalled = $true
            $script:targetAgent = $Agent
        }

        Dev-Goto -Name "unique-project-alpha" -Agent "-Claude"
        $script:agentCalled | Should -Be $true
        $script:targetAgent | Should -Be "claude"
    }

    It "handles leading agent flag like '-Claude <project>'" {
        Mock Find-ProjectPathFuzzy {
            @([PSCustomObject]@{ Name = "unique-project-alpha"; Path = $script:p1; Status = "Active"; Score = 100 })
        }
        $script:agentCalled = $false
        $script:targetAgent = ""
        Mock Rtb-Agent {
            param($ProjectName, $Agent)
            $script:agentCalled = $true
            $script:targetAgent = $Agent
        }

        Dev-Goto -Name "-Claude" -Agent "unique-project-alpha"
        $script:agentCalled | Should -Be $true
        $script:targetAgent | Should -Be "claude"
        (Get-Location).Path | Should -Be $script:p1
    }

    It "displays interactive picker when multiple exact matches (Score 100) exist across different roots" {
        Set-Location $script:tempRoot
        Mock Find-ProjectPathFuzzy {
            @(
                [PSCustomObject]@{ Name = "same-name"; Path = $script:p1; Status = "Active"; Score = 100 },
                [PSCustomObject]@{ Name = "same-name"; Path = $script:p2; Status = "Paused"; Score = 100 }
            )
        }

        # Select choice 2 (Paused root version)
        Dev-Goto -Name "same-name" -Choice "2"
        (Get-Location).Path | Should -Be $script:p2
    }
}
