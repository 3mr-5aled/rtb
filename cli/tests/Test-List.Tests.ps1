#Requires -Version 7
# Project List Tests: rtb list & -Json
# Compatible with Pester 3.4.0 and Pester 5+

Describe "Rtb-List Project Discovery" {
    BeforeAll {
        . (Join-Path $PSScriptRoot '..\src\utils\helpers.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\list.ps1')
        Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

        $script:testBase = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_list_test_$([Guid]::NewGuid().ToString('N'))"
        $script:activeRoot = Join-Path $script:testBase "01-Active"
        $script:pausedRoot = Join-Path $script:testBase "04-Paused"

        $script:proj1 = Join-Path $script:activeRoot "project-alpha"
        $script:proj2 = Join-Path $script:pausedRoot "project-beta"

        New-Item -ItemType Directory -Path $script:proj1 -Force | Out-Null
        New-Item -ItemType Directory -Path $script:proj2 -Force | Out-Null

        Set-Content -Path (Join-Path $script:proj1 "package.json") -Value '{"name":"project-alpha"}'
        Set-Content -Path (Join-Path $script:proj2 "Cargo.toml") -Value '[package]'

        $script:mockConfig = [PSCustomObject]@{
            projectRoots = [PSCustomObject]@{
                active     = $script:activeRoot
                paused     = $script:pausedRoot
                planning   = (Join-Path $script:testBase "02-Planning")
                testing    = (Join-Path $script:testBase "03-Testing")
                production = (Join-Path $script:testBase "02-Deployed\01-Production")
                staging    = (Join-Path $script:testBase "02-Deployed\02-Staging")
                vibe       = (Join-Path $script:testBase "03-Vibe")
                sandbox    = (Join-Path $script:testBase "01-SandBox")
                abandoned  = (Join-Path $script:testBase "05-Abandoned")
            }
        }
    }

    AfterAll {
        if (Test-Path $script:testBase) {
            Remove-Item -Recurse -Force $script:testBase -ErrorAction SilentlyContinue
        }
    }

    Context "Plain Text List Output" {
        It "lists projects grouped by category" {
            Mock Get-DevConfig { return $script:mockConfig }
            Mock Get-RtbConfig { return $script:mockConfig }
            $output = & { Rtb-List } 6>&1 | Out-String
            $output | Should -Match "project-alpha"
            $output | Should -Match "project-beta"
        }
    }

    Context "JSON Output Mode" {
        It "returns JSON list of all projects" {
            Mock Get-DevConfig { return $script:mockConfig }
            Mock Get-RtbConfig { return $script:mockConfig }
            $jsonStr = Rtb-List --json
            $data = $jsonStr | ConvertFrom-Json
            $data | Should -Not -BeNullOrEmpty
            ($data.Count -ge 2) | Should -Be $true
        }
    }

    Context "Rust parity" {
        It "invokes Rust binary when available and returns valid JSON structure" {
            $bin = Get-Command rtb -CommandType Application -ErrorAction SilentlyContinue
            if (-not $bin -and $env:_RTB_BIN -and (Test-Path $env:_RTB_BIN)) {
                $bin = Get-Item $env:_RTB_BIN
            }
            if ($bin) {
                $configPath = Join-Path $script:testBase "rtb.config.json"
                $rawConfig = @{
                    version = "1.0.0"
                    projectRoots = @{
                        active = $script:activeRoot
                        paused = $script:pausedRoot
                    }
                    backupRoot = ""
                    configRoot = ""
                    templateDir = ""
                    cleanDeps = @{ daysInactive = 30; targets = @() }
                    staleThresholdDays = 60
                    gitHealth = @{ scanRoots = @() }
                } | ConvertTo-Json -Depth 5
                Set-Content -Path $configPath -Value $rawConfig

                $jsonStr = & $bin.Source --config $configPath list --json
                $data = $jsonStr | ConvertFrom-Json
                $data | Should -Not -BeNullOrEmpty
                $names = $data | ForEach-Object { $_.name }
                ($names -contains "project-alpha") | Should -Be $true
                ($names -contains "project-beta") | Should -Be $true
            }
        }
    }
}
