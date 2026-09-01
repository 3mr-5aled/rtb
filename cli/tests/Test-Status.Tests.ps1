#Requires -Version 7
# Shell Prompt Status Tests: rtb status & -Json (Milestone M4)
# Compatible with Pester 3.4.0 and Pester 5+

Describe "Rtb-Status Shell Prompt Integration" {
    BeforeAll {
        . (Join-Path $PSScriptRoot '..\src\utils\helpers.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\status.ps1')
        Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

        $script:testBase = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_status_test_$([Guid]::NewGuid().ToString('N'))"
        $script:activeRoot = Join-Path $script:testBase "01-Active"
        $script:testProj = Join-Path $script:activeRoot "sample-project"
        $script:nestedDir = Join-Path $script:testProj "src\nested\deep\folder"

        New-Item -ItemType Directory -Path $script:nestedDir -Force | Out-Null

        # Initialize git repo in sample-project
        git -C $script:testProj init --quiet 2>$null
        git -C $script:testProj config user.name "RTB Test" 2>$null
        git -C $script:testProj config user.email "test@rtb.local" 2>$null
        Set-Content -Path (Join-Path $script:testProj "package.json") -Value '{"name":"sample-project"}'
        git -C $script:testProj add package.json 2>$null
        git -C $script:testProj commit -m "initial commit" --quiet 2>$null

        $script:mockConfig = [PSCustomObject]@{
            projectRoots = [PSCustomObject]@{
                active     = $script:activeRoot
                paused     = (Join-Path $script:testBase "04-Paused")
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

    Context "Plain Text Output Mode" {
        It "returns formatted string from project root directory" {
            Push-Location $script:testProj
            try {
                Mock Get-RtbConfig { return $script:mockConfig }
                $status = Rtb-Status
                $status | Should -Match "^rtb » sample-project \(Active\)"
                $status | Should -Match "Node\.js"
            } finally {
                Pop-Location
            }
        }

        It "detects project name, status, and git branch from deeply nested directory" {
            Push-Location $script:nestedDir
            try {
                Mock Get-RtbConfig { return $script:mockConfig }
                $status = Rtb-Status
                $status | Should -Match "^rtb » sample-project \(Active\)"
            } finally {
                Pop-Location
            }
        }

        It "displays uncommitted counter when modifications exist" {
            Push-Location $script:testProj
            try {
                Mock Get-RtbConfig { return $script:mockConfig }
                Set-Content -Path (Join-Path $script:testProj "uncommitted.txt") -Value "dirty"
                $status = Rtb-Status
                $status | Should -Match "±1"
                Remove-Item (Join-Path $script:testProj "uncommitted.txt") -Force -ErrorAction SilentlyContinue
            } finally {
                Pop-Location
            }
        }

        It "falls back gracefully when CWD is outside project roots" {
            $outsideDir = Join-Path $script:testBase "outside_workspace"
            New-Item -ItemType Directory -Path $outsideDir -Force | Out-Null
            Push-Location $outsideDir
            try {
                Mock Get-RtbConfig { return $script:mockConfig }
                $status = Rtb-Status
                $status | Should -Match "^rtb » outside_workspace"
            } finally {
                Pop-Location
            }
        }

        It "Dev-Status alias returns matching plain text output" {
            Push-Location $script:testProj
            try {
                Mock Get-RtbConfig { return $script:mockConfig }
                $status = Dev-Status
                $status | Should -Match "^rtb » sample-project"
            } finally {
                Pop-Location
            }
        }

        It "Get-RtbStatus alias returns matching plain text output" {
            Push-Location $script:testProj
            try {
                Mock Get-RtbConfig { return $script:mockConfig }
                $status = Get-RtbStatus
                $status | Should -Match "^rtb » sample-project"
            } finally {
                Pop-Location
            }
        }
    }

    Context "JSON Output Mode (-Json, --json, -j)" {
        It "returns valid JSON string with all required schema keys" {
            Push-Location $script:testProj
            try {
                Mock Get-RtbConfig { return $script:mockConfig }
                $jsonStr = Rtb-Status -Json
                $jsonStr | Should -Not -BeNullOrEmpty

                $data = $jsonStr | ConvertFrom-Json
                $data | Should -Not -BeNullOrEmpty
                $data.project | Should -Be "sample-project"
                $data.status | Should -Be "Active"
                ($data.PSObject.Properties.Name -contains 'branch') | Should -Be $true
                ($data.PSObject.Properties.Name -contains 'uncommitted') | Should -Be $true
                ($data.PSObject.Properties.Name -contains 'stack') | Should -Be $true
                ($data.PSObject.Properties.Name -contains 'cwd') | Should -Be $true
            } finally {
                Pop-Location
            }
        }

        It "resolves project name and status in JSON mode from deeply nested directory" {
            Push-Location $script:nestedDir
            try {
                Mock Get-RtbConfig { return $script:mockConfig }
                $jsonStr = Rtb-Status -Json
                $data = $jsonStr | ConvertFrom-Json
                $data.project | Should -Be "sample-project"
                $data.status | Should -Be "Active"
                $data.cwd | Should -Match "nested"
            } finally {
                Pop-Location
            }
        }

        It "preserves stack as a JSON array" {
            Push-Location $script:testProj
            try {
                Mock Get-RtbConfig { return $script:mockConfig }
                $jsonStr = Rtb-Status -Json
                $data = $jsonStr | ConvertFrom-Json
                ($data.stack -is [System.Array] -or $data.stack -is [System.Collections.IEnumerable]) | Should -Be $true
                ($data.stack -contains 'Node.js') | Should -Be $true
            } finally {
                Pop-Location
            }
        }

        It "supports --json GNU flag" {
            Push-Location $script:testProj
            try {
                Mock Get-RtbConfig { return $script:mockConfig }
                $jsonStr = Rtb-Status '--json'
                $data = $jsonStr | ConvertFrom-Json
                $data.project | Should -Be "sample-project"
                $data.status | Should -Be "Active"
            } finally {
                Pop-Location
            }
        }

        It "supports -j shorthand flag" {
            Push-Location $script:testProj
            try {
                Mock Get-RtbConfig { return $script:mockConfig }
                $jsonStr = Rtb-Status '-j'
                $data = $jsonStr | ConvertFrom-Json
                $data.project | Should -Be "sample-project"
                $data.status | Should -Be "Active"
            } finally {
                Pop-Location
            }
        }

        It "handles missing config gracefully in JSON mode" {
            Push-Location $script:testProj
            try {
                Mock Get-RtbConfig { return $null }
                $jsonStr = Rtb-Status -Json
                $data = $jsonStr | ConvertFrom-Json
                $data.project | Should -Be "sample-project"
                $data.status | Should -BeNullOrEmpty
            } finally {
                Pop-Location
            }
        }

        It "handles malformed config gracefully in JSON mode" {
            Push-Location $script:testProj
            try {
                Mock Get-RtbConfig { throw [System.Exception]::new("Corrupted JSON") }
                $jsonStr = Rtb-Status -Json
                $data = $jsonStr | ConvertFrom-Json
                $data.project | Should -Be "sample-project"
            } finally {
                Pop-Location
            }
        }
    }

    Context "Stack Detection" {
        It "detects multi-stack technologies (Rust, Go, Python, PowerShell)" {
            $multiProj = Join-Path $script:testBase "multi-stack-proj"
            New-Item -ItemType Directory -Path $multiProj -Force | Out-Null
            Set-Content -Path (Join-Path $multiProj "Cargo.toml") -Value '[package]'
            Set-Content -Path (Join-Path $multiProj "go.mod") -Value 'module test'
            Set-Content -Path (Join-Path $multiProj "pyproject.toml") -Value '[tool.poetry]'
            Set-Content -Path (Join-Path $multiProj "rtb.psm1") -Value '# module'

            Push-Location $multiProj
            try {
                Mock Get-RtbConfig { return $null }
                $jsonStr = Rtb-Status -Json
                $data = $jsonStr | ConvertFrom-Json
                ($data.stack -contains 'Rust') | Should -Be $true
                ($data.stack -contains 'Go') | Should -Be $true
                ($data.stack -contains 'Python') | Should -Be $true
                ($data.stack -contains 'PowerShell') | Should -Be $true
            } finally {
                Pop-Location
            }
        }
    }

    Context "CLI Dispatcher Integration" {
        It "executes 'rtb status' and 'dev status' via dispatcher" {
            Push-Location $script:testProj
            try {
                Mock -ModuleName rtb Get-RtbConfig { return $script:mockConfig }
                Mock Get-RtbConfig { return $script:mockConfig }
                $plain = rtb status
                $plain | Should -Match "^rtb » sample-project"

                $devPlain = dev status
                $devPlain | Should -Match "^rtb » sample-project"
            } finally {
                Pop-Location
            }
        }

        It "executes 'rtb status -Json' and 'rtb status --json' via dispatcher" {
            Push-Location $script:testProj
            try {
                Mock -ModuleName rtb Get-RtbConfig { return $script:mockConfig }
                Mock Get-RtbConfig { return $script:mockConfig }
                $json1 = rtb status -Json
                $data1 = $json1 | ConvertFrom-Json
                $data1.project | Should -Be "sample-project"

                $json2 = rtb status --json
                $data2 = $json2 | ConvertFrom-Json
                $data2.project | Should -Be "sample-project"
            } finally {
                Pop-Location
            }
        }
    }

    Context "Rust parity" {
        It "invokes Rust binary when available and returns matching JSON contract" {
            $bin = Get-Command rtb -CommandType Application -ErrorAction SilentlyContinue
            if (-not $bin -and $env:_RTB_BIN -and (Test-Path $env:_RTB_BIN)) {
                $bin = Get-Item $env:_RTB_BIN
            }
            if ($bin) {
                Push-Location $script:testProj
                try {
                    $configPath = Join-Path $script:testBase "rtb.config.json"
                    $rawConfig = @{
                        version = "1.0.0"
                        projectRoots = @{
                            active = $script:activeRoot
                        }
                        backupRoot = ""
                        configRoot = ""
                        templateDir = ""
                        cleanDeps = @{ daysInactive = 30; targets = @() }
                        staleThresholdDays = 60
                        gitHealth = @{ scanRoots = @() }
                    } | ConvertTo-Json -Depth 5
                    Set-Content -Path $configPath -Value $rawConfig

                    $jsonStr = & $bin.Source --config $configPath status --json
                    $data = $jsonStr | ConvertFrom-Json
                    $data.project | Should -Be "sample-project"
                    $data.status | Should -Be "Active"
                } finally {
                    Pop-Location
                }
            }
        }
    }
}
