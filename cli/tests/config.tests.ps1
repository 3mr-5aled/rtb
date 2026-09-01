Describe "Get-RtbConfig and Test-RtbConfigured" {
    Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

    It "Loads rtb.config.json from user config directory or fallback repository config" {
        $config = Get-RtbConfig
        $config.version | Should -Be "1.0.0"
        $config.cleanDeps | Should -Not -BeNullOrEmpty
    }

    It "Exposes projectRoots object with active path, label, and emoji" {
        $config = Get-RtbConfig
        $config.projectRoots.active | Should -Not -BeNullOrEmpty
        $config.projectRoots.active.path | Should -Not -BeNullOrEmpty
        $config.projectRoots.active.label | Should -Not -BeNullOrEmpty
        $config.projectRoots.active.emoji | Should -Not -BeNullOrEmpty
    }

    It "Test-RtbConfigured returns boolean status" {
        $configured = Test-RtbConfigured
        $configured -is [bool] | Should -Be $true
    }

    It "Rtb-Config and Dev-Config commands exist and resolve config path" {
        (Get-Command Rtb-Config -ErrorAction SilentlyContinue) | Should -Not -BeNullOrEmpty
        (Get-Command Dev-Config -ErrorAction SilentlyContinue) | Should -Not -BeNullOrEmpty
        
        Mock Start-Process {}
        Mock Invoke-Item {}
        { Rtb-Config } | Should -Not -Throw
    }

    Context "Rust parity" {
        It "invokes Rust binary when available and creates default config scaffold" {
            $bin = Get-Command rtb -CommandType Application -ErrorAction SilentlyContinue
            if (-not $bin -and $env:_RTB_BIN -and (Test-Path $env:_RTB_BIN)) {
                $bin = Get-Item $env:_RTB_BIN
            }
            if (-not $bin) {
                $targetBin = Join-Path $PSScriptRoot "..\..\tui\target\debug\rtb.exe"
                if (Test-Path $targetBin) { $bin = Get-Item $targetBin }
            }
            if ($bin) {
                $tempConfigDir = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_cfg_test_$([Guid]::NewGuid().ToString('N'))"
                New-Item -ItemType Directory -Path $tempConfigDir -Force | Out-Null
                try {
                    $configPath = Join-Path $tempConfigDir "rtb.config.json"
                    $binPath = if ($bin.Source) { $bin.Source } else { $bin.FullName }
                    $env:RTB_NON_INTERACTIVE = "1"
                    & $binPath --config $configPath config
                    $LASTEXITCODE | Should -Be 0
                    (Test-Path $configPath) | Should -Be $true
                } finally {
                    Remove-Item -Recurse -Force $tempConfigDir -ErrorAction SilentlyContinue
                }
            }
        }
    }
}


