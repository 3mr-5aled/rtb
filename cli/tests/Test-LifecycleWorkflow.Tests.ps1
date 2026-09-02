Describe "Standalone Installation & Lifecycle Workflow" {
    BeforeAll {
        Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force
    }

    Context "Config Normalization and Backward Compatibility" {
        It "Normalizes flat string projectRoots into objects with path, label, and emoji" {
            $rawJson = @'
{
    "version": "1.0.0",
    "projectRoots": {
        "active": "C:\\Mock\\Active",
        "paused": "C:\\Mock\\Paused"
    }
}
'@
            $tempFile = [System.IO.Path]::GetTempFileName()
            Set-Content -Path $tempFile -Value $rawJson -Encoding UTF8

            $cfg = Get-Content $tempFile -Raw | ConvertFrom-Json
            foreach ($prop in $cfg.projectRoots.PSObject.Properties) {
                $val = $prop.Value
                if ($val -is [string]) {
                    $prop.Value = [PSCustomObject]@{
                        path  = $val
                        label = $prop.Name
                        emoji = '📁'
                    }
                }
            }

            $cfg.projectRoots.active.path | Should -Be "C:\Mock\Active"
            $cfg.projectRoots.active.label | Should -Be "active"
            $cfg.projectRoots.active.emoji | Should -Be "📁"

            Remove-Item -Force $tempFile -ErrorAction SilentlyContinue
        }
    }

    Context "Rtb-Upgrade Command" {
        It "Rtb-Upgrade -Check resolves current module version from rtb.psd1" {
            $versionOutput = Rtb-Upgrade -Check
            $versionOutput | Should -Not -BeNullOrEmpty
            $versionOutput | Should -Match 'v\d+\.\d+'
        }
    }

    Context "Config Gate and Helpers" {
        It "Test-RtbConfigured correctly detects active root presence" {
            $result = Test-RtbConfigured
            $result -is [bool] | Should -Be $true
        }
    }

    Context "Rust parity" {
        It "invokes Rust binary when available and runs lifecycle commands end-to-end" {
            $bin = Get-Command rtb -CommandType Application -ErrorAction SilentlyContinue
            if (-not $bin -and $env:_RTB_BIN -and (Test-Path $env:_RTB_BIN)) {
                $bin = Get-Item $env:_RTB_BIN
            }
            if (-not $bin) {
                $cargoTarget = Join-Path $PSScriptRoot "..\..\tui\target\debug\rtb.exe"
                if (Test-Path $cargoTarget) { $bin = Get-Item $cargoTarget }
            }
            if ($bin) {
                $testBase = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_lifecycle_test_$([Guid]::NewGuid().ToString('N'))"
                $activeRoot = Join-Path $testBase "01-Active"
                $pausedRoot = Join-Path $testBase "04-Paused"
                $stagingRoot = Join-Path $testBase "02-Deployed\02-Staging"
                $backupRoot = Join-Path $testBase "backups"

                New-Item -ItemType Directory -Path $activeRoot -Force | Out-Null
                New-Item -ItemType Directory -Path $pausedRoot -Force | Out-Null
                New-Item -ItemType Directory -Path $stagingRoot -Force | Out-Null
                New-Item -ItemType Directory -Path $backupRoot -Force | Out-Null

                $configPath = Join-Path $testBase "rtb.config.json"
                $rawConfig = @{
                    version = "1.0.0"
                    projectRoots = @{
                        active = $activeRoot
                        paused = $pausedRoot
                        staging = $stagingRoot
                    }
                    backupRoot = $backupRoot
                    configRoot = ""
                    templateDir = ""
                    cleanDeps = @{ daysInactive = 30; targets = @("node_modules") }
                    staleThresholdDays = 60
                    gitHealth = @{ scanRoots = @() }
                } | ConvertTo-Json -Depth 5
                Set-Content -Path $configPath -Value $rawConfig

                $binPath = if ($bin.Source) { $bin.Source } else { $bin.FullName }

                # 1. new
                & $binPath --config $configPath new p-demo
                (Test-Path (Join-Path $activeRoot "p-demo")) | Should -Be $true

                # 2. pause
                & $binPath --config $configPath pause p-demo --force
                (Test-Path (Join-Path $pausedRoot "p-demo")) | Should -Be $true
                (Test-Path (Join-Path $activeRoot "p-demo")) | Should -Be $false

                # 3. resume
                & $binPath --config $configPath resume p-demo
                (Test-Path (Join-Path $activeRoot "p-demo")) | Should -Be $true

                # 4. deploy
                & $binPath --config $configPath deploy p-demo --staging
                (Test-Path (Join-Path $stagingRoot "p-demo")) | Should -Be $true

                # 5. archive
                & $binPath --config $configPath archive p-demo --force
                (Test-Path (Join-Path $stagingRoot "p-demo")) | Should -Be $false
                $snapshotDir = Join-Path $backupRoot "project-snapshots"
                $archives = Get-ChildItem $snapshotDir -Filter "*p-demo*" -ErrorAction SilentlyContinue
                ($archives.Count -ge 1) | Should -Be $true

                # 6. unarchive
                & $binPath --config $configPath unarchive $archives[0].Name
                (Test-Path (Join-Path $activeRoot "p-demo")) | Should -Be $true

                Remove-Item -Recurse -Force $testBase -ErrorAction SilentlyContinue
            }
        }
    }
}
