#Requires -Version 7
# Dev Workflow Tests: run, build, test, deps, workspace, clean & Rust parity

Describe "Dev Workflow Commands & Parity" {
    BeforeAll {
        Get-Module rtb | Remove-Module -Force -ErrorAction SilentlyContinue
        Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

        $script:testBase = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_dev_wf_$([Guid]::NewGuid().ToString('N'))"
        $script:activeRoot = Join-Path $script:testBase "01-Active"
        $script:proj1 = Join-Path $script:activeRoot "demo-node-app"

        New-Item -ItemType Directory -Path $script:proj1 -Force | Out-Null

        $pkgJson = @{
            name = "demo-node-app"
            workspaces = @("packages/*")
            scripts = @{
                dev = "echo dev"
                build = "echo build"
                test = "echo test"
            }
            dependencies = @{
                express = "^4.18.0"
            }
        } | ConvertTo-Json -Depth 5
        Set-Content -Path (Join-Path $script:proj1 "package.json") -Value $pkgJson

        $script:configPath = Join-Path $script:testBase "rtb.config.json"
        $rawConfig = @{
            version = "1.0.0"
            projectRoots = @{ active = $script:activeRoot }
            backupRoot = ""
            configRoot = ""
            templateDir = ""
            cleanDeps = @{ daysInactive = 0; targets = @("node_modules") }
            staleThresholdDays = 0
            gitHealth = @{ scanRoots = @() }
        } | ConvertTo-Json -Depth 5
        Set-Content -Path $script:configPath -Value $rawConfig
    }

    AfterAll {
        if (Test-Path $script:testBase) {
            Remove-Item -Recurse -Force $script:testBase -ErrorAction SilentlyContinue
        }
    }

    Context "Rust parity dev workflow" {
        It "invokes Rust binary for deps, workspace, clean, run, build, test" {
            $bin = Get-Command rtb -CommandType Application -ErrorAction SilentlyContinue
            if (-not $bin -and $env:_RTB_BIN -and (Test-Path $env:_RTB_BIN)) {
                $bin = Get-Item $env:_RTB_BIN
            }
            if (-not $bin) {
                $cargoTarget = Join-Path $PSScriptRoot "..\..\tui\target\debug\rtb.exe"
                if (Test-Path $cargoTarget) { $bin = Get-Item $cargoTarget }
            }
            if ($bin) {
                $binPath = if ($bin.Source) { $bin.Source } else { $bin.FullName }

                # 1. deps --json
                $depsJson = (& $binPath --config $script:configPath deps demo-node-app --json) -join "`n"
                $depsData = $depsJson | ConvertFrom-Json
                $depsData | Should -Not -BeNullOrEmpty

                # 2. workspace --json
                $wsJson = (& $binPath --config $script:configPath workspace demo-node-app --json) -join "`n"
                $wsData = $wsJson | ConvertFrom-Json
                $wsData.is_monorepo | Should -Be $true

                # 3. clean --dry-run
                $cleanOut = (& $binPath --config $script:configPath clean --dry-run --days 0) -join "`n"
                $cleanOut | Should -Match "DRY RUN MODE"

                # 4. run
                $runOut = (& $binPath --config $script:configPath run demo-node-app) -join "`n"
                $runOut | Should -Match "Run Project"

                # 5. build
                $buildOut = (& $binPath --config $script:configPath build demo-node-app) -join "`n"
                $buildOut | Should -Match "Build Project"

                # 6. test
                $testOut = (& $binPath --config $script:configPath test demo-node-app) -join "`n"
                $testOut | Should -Match "Test Project"
            }
        }
    }
}
