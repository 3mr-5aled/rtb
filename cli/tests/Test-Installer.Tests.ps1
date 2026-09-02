#Requires -Version 7
# Test Suite: RTB Setup Wizard (install.ps1)
# Compatible with Pester 3.4.0 and Pester 5+

Describe "Setup Wizard (install.ps1)" {
    BeforeAll {
        $script:origEnvPath = $env:PATH
        if ($PROFILE -and (Test-Path $PROFILE)) {
            $script:origProfileContent = Get-Content $PROFILE -Raw
        }
        $script:repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
        $script:installerPath = Join-Path $script:repoRoot 'install.ps1'
        . $script:installerPath -NoExec
        $script:NoExitOnFail = $true
        $script:testBase = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_installer_test_$([Guid]::NewGuid().ToString('N'))"
        New-Item -ItemType Directory -Path $script:testBase -Force | Out-Null
    }

    AfterAll {
        # Restore environment PATH and clean up test-loaded module instances
        $env:PATH = $script:origEnvPath
        if ($script:origProfileContent -and $PROFILE) {
            $script:origProfileContent | Set-Content $PROFILE -Encoding UTF8
        }
        Get-Module rtb | Where-Object { $_.Path -notlike "*$([System.IO.Path]::DirectorySeparatorChar)cli$([System.IO.Path]::DirectorySeparatorChar)*" } | Remove-Module -Force -ErrorAction SilentlyContinue

        if ($script:testBase -and (Test-Path -LiteralPath $script:testBase)) {
            Remove-Item -Recurse -Force -LiteralPath $script:testBase -ErrorAction SilentlyContinue
        }
    }

    Context "UI Formatting & Helper Functions" {
        It "Esc returns ANSI escape code when ANSI is enabled" {
            $script:ANSI = $true
            $code = Esc '36m'
            $code | Should -Be "$([char]27)[36m"
        }

        It "Esc returns empty string when ANSI is disabled" {
            $script:ANSI = $false
            $code = Esc '36m'
            $code | Should -Be ''
        }

        It "Write-Step renders numbered step in quiet mode without error" {
            $script:QUIET = $true
            { Write-Step 1 5 "Test Step" } | Should -Not -Throw
        }

        It "Write-Step renders numbered step in ANSI mode without error" {
            $script:QUIET = $false
            $script:ANSI = $true
            { Write-Step 2 5 "Deploy Module" } | Should -Not -Throw
        }

        It "Write-Warn outputs warning message without error" {
            { Write-Warn "Non-critical test warning" } | Should -Not -Throw
        }

        It "Show-Header renders banner in normal and quiet modes" {
            $script:QUIET = $false
            { Show-Header } | Should -Not -Throw
            $script:QUIET = $true
            { Show-Header } | Should -Not -Throw
        }

        It "Show-Summary renders installation summary box" {
            $fakeProfiles = @("C:\fake\profile1.ps1", "C:\fake\profile2.ps1")
            { Show-Summary "C:\fake\install\dir" $fakeProfiles } | Should -Not -Throw
        }
    }

    Context "Animated Spinner Engine" {
        It "Start-Spinner returns a valid context in quiet mode" {
            $script:QUIET = $true
            $ctx = Start-Spinner "Testing quiet spinner"
            $ctx | Should -Not -BeNullOrEmpty
            $ctx.Label | Should -Be "Testing quiet spinner"
            $ctx.Type | Should -Be "Quiet"
        }

        It "Stop-Spinner handles quiet context without error" {
            $ctx = @{ Type = 'Quiet'; Label = 'Quiet test'; Job = $null }
            { Stop-Spinner $ctx $true } | Should -Not -Throw
            { Stop-Spinner $ctx $false } | Should -Not -Throw
        }

        It "Start-Spinner and Stop-Spinner runspace lifecycle executes cleanly" {
            $script:QUIET = $false
            $script:ANSI = $true
            $ctx = Start-Spinner "Testing live runspace spinner"
            $ctx | Should -Not -BeNullOrEmpty
            Start-Sleep -Milliseconds 150
            { Stop-Spinner $ctx $true } | Should -Not -Throw
        }

        It "Stop-Spinner handles failure flag correctly" {
            $script:QUIET = $false
            $script:ANSI = $true
            $ctx = Start-Spinner "Testing failure spinner"
            Start-Sleep -Milliseconds 100
            { Stop-Spinner $ctx $false } | Should -Not -Throw
        }
    }

    Context "Interactive Prompts" {
        It "Prompt-InstallPath returns default when input is empty" {
            $script:QUIET = $false
            $script:ForceInteractive = $true
            $script:InstallPath = ''
            Mock Read-Host { return "" }
            $res = Prompt-InstallPath "C:\default\rtb"
            $res | Should -Be "C:\default\rtb"
        }

        It "Prompt-InstallPath returns custom path when user provides input" {
            $script:QUIET = $false
            $script:ForceInteractive = $true
            $script:InstallPath = ''
            Mock Read-Host { return "D:\custom\rtb" }
            $res = Prompt-InstallPath "C:\default\rtb"
            $res | Should -Be "D:\custom\rtb"
        }

        It "Prompt-InstallPath returns default immediately in quiet mode" {
            $script:QUIET = $true
            $script:ForceInteractive = $false
            $script:InstallPath = ''
            $res = Prompt-InstallPath "C:\default\rtb"
            $res | Should -Be "C:\default\rtb"
        }

        It "Prompt-InstallPath returns passed script:InstallPath directly" {
            $script:QUIET = $true
            $script:ForceInteractive = $false
            $script:InstallPath = 'D:\explicit\install\path'
            $res = Prompt-InstallPath "C:\default\rtb"
            $res | Should -Be 'D:\explicit\install\path'
            $script:InstallPath = ''
        }

        It "Prompt-Profiles filters candidates based on user answers" {
            $script:QUIET = $false
            $script:ForceInteractive = $true
            $candidates = @("C:\p1.ps1", "C:\p2.ps1", "C:\p3.ps1")
            $script:callCount = 0
            Mock Read-Host {
                $script:callCount++
                if ($script:callCount -eq 2) { return "n" }
                return "y"
            }
            $selected = Prompt-Profiles $candidates
            $selected | Should -Contain "C:\p1.ps1"
            $selected | Should -Not -Contain "C:\p2.ps1"
            $selected | Should -Contain "C:\p3.ps1"
        }

        It "Prompt-Profiles selects all valid candidates in quiet mode" {
            $script:QUIET = $true
            $script:ForceInteractive = $false
            $candidates = @("C:\p1.ps1", "C:\p2.ps1")
            $selected = Prompt-Profiles $candidates
            $selected.Count | Should -Be 2
        }

        It "Prompt-RunInit returns true when user enters y" {
            $script:QUIET = $false
            $script:ForceInteractive = $true
            Mock Read-Host { return "y" }
            (Prompt-RunInit) | Should -Be $true
        }

        It "Prompt-RunInit returns false when user enters n" {
            $script:QUIET = $false
            $script:ForceInteractive = $true
            Mock Read-Host { return "n" }
            (Prompt-RunInit) | Should -Be $false
        }

        It "Prompt-RunInit returns false immediately in quiet mode" {
            $script:QUIET = $true
            $script:ForceInteractive = $false
            (Prompt-RunInit) | Should -Be $false
        }
    }

    Context "Sandboxed Stepped Installation (Repo Mode)" {
        BeforeEach {
            $script:sandboxDir = Join-Path $script:testBase "sandbox_repo_$([Guid]::NewGuid().ToString('N'))"
            $script:userConfigDir = $script:sandboxDir
            $script:moduleHome = Join-Path $script:userConfigDir 'module'
            $script:scriptsDir = Join-Path $script:userConfigDir 'bin'
            $script:testProfile = Join-Path $script:sandboxDir 'test_profile.ps1'
            $script:resolvedProfiles = @($script:testProfile)
            $script:QUIET = $true
            $script:scriptRoot = $script:repoRoot
            $script:isStandaloneOverride = $false
        }

        It "Step 1 creates target directories on disk" {
            Install-Steps
            Test-Path $script:userConfigDir | Should -Be $true
            Test-Path $script:scriptsDir | Should -Be $true
        }

        It "Step 2 deploys rtb binary" {
            Install-Steps
            Test-Path (Join-Path $script:scriptsDir 'rtb.exe') | Should -Be $true
        }

        It "Step 3 updates environment PATH safely" {
            Install-Steps
            Test-Path $script:scriptsDir | Should -Be $true
            ($env:PATH -split ';') | Should -Contain $script:scriptsDir
        }

        It "Step 4 injects shell-init into target profile" {
            Install-Steps
            Test-Path $script:testProfile | Should -Be $true
            $content = Get-Content $script:testProfile -Raw
            $content | Should -Match '# RTB Shell Integration'
            $content | Should -Match 'Invoke-Expression \(& rtb shell-init pwsh\)'
        }

        It "Step 4 is idempotent and does not duplicate integration blocks on repeated runs" {
            Install-Steps
            Install-Steps
            $lines = Get-Content $script:testProfile
            $matchCount = ($lines | Where-Object { $_ -match '# RTB Shell Integration' }).Count
            $matchCount | Should -Be 1
        }
    }

    Context "Profile Cleanup & Legacy Import Removal" {
        It "removes legacy dev-tools and old rtb import statements before adding current integration" {
            $cleanSandbox = Join-Path $script:testBase "sandbox_clean_$([Guid]::NewGuid().ToString('N'))"
            New-Item -ItemType Directory -Path $cleanSandbox -Force | Out-Null
            $cleanProfile = Join-Path $cleanSandbox 'profile_with_legacy.ps1'

            $legacyContent = @"
# User Custom Functions
function prompt { "PS> " }

# RTB CLI Module
Import-Module 'C:\old\path\dev-tools\rtb.psd1' -DisableNameChecking -Force
Import-Module 'D:\legacy\rtb-command-tool\cli\rtb.psd1' -Force

# Other tool
Import-Module 'SomeOtherModule'
"@
            $legacyContent | Set-Content -Path $cleanProfile -Encoding UTF8

            $script:userConfigDir = $cleanSandbox
            $script:moduleHome = Join-Path $cleanSandbox 'module'
            $script:scriptsDir = Join-Path $cleanSandbox 'bin'
            $script:resolvedProfiles = @($cleanProfile)
            $script:QUIET = $true
            $script:scriptRoot = $script:repoRoot
            $script:isStandaloneOverride = $false

            Install-Steps

            $resultContent = Get-Content $cleanProfile -Raw
            $resultContent | Should -Not -Match 'C:\\old\\path\\dev-tools'
            $resultContent | Should -Not -Match 'D:\\legacy\\rtb-command-tool'
            $resultContent | Should -Match 'SomeOtherModule'
            $resultContent | Should -Match '# RTB Shell Integration'
            $resultContent | Should -Match 'Invoke-Expression \(& rtb shell-init pwsh\)'
        }

        It "silently removes Phase 2 module directory if present" {
            $phase2Sandbox = Join-Path $script:testBase "sandbox_phase2_$([Guid]::NewGuid().ToString('N'))"
            $oldModuleHome = Join-Path $phase2Sandbox 'module'
            New-Item -ItemType Directory -Path $oldModuleHome -Force | Out-Null
            '# Dummy psd1' | Set-Content (Join-Path $oldModuleHome 'rtb.psd1')

            $script:userConfigDir = $phase2Sandbox
            $script:moduleHome = $oldModuleHome
            $script:scriptsDir = Join-Path $phase2Sandbox 'bin'
            $script:resolvedProfiles = @(Join-Path $phase2Sandbox 'profile.ps1')
            $script:QUIET = $true
            $script:scriptRoot = $script:repoRoot
            $script:isStandaloneOverride = $false

            Install-Steps
            Test-Path $oldModuleHome | Should -Be $false
        }
    }

    Context "Standalone Mode Simulation" {
        It "simulates standalone binary download when running outside repo" {
            $standaloneSandbox = Join-Path $script:testBase "sandbox_standalone_$([Guid]::NewGuid().ToString('N'))"
            New-Item -ItemType Directory -Path $standaloneSandbox -Force | Out-Null

            Mock Invoke-WebRequest {
                param($Uri, $OutFile)
                if ($Uri -match 'rtb-windows-amd64') {
                    '# Mock Binary Content' | Set-Content $OutFile
                }
            }

            $script:userConfigDir = Join-Path $standaloneSandbox 'installed'
            $script:moduleHome = Join-Path $script:userConfigDir 'module'
            $script:scriptsDir = Join-Path $script:userConfigDir 'bin'
            $script:resolvedProfiles = @(Join-Path $standaloneSandbox 'standalone_profile.ps1')
            $script:QUIET = $true
            $script:isStandaloneOverride = $true

            Install-Steps
            Test-Path (Join-Path $script:scriptsDir 'rtb.exe') | Should -Be $true
        }
    }

    Context "Subprocess & CI Execution (-Quiet)" {
        It "executes install.ps1 in a clean subprocess with -Quiet flag and exits 0" {
            $quietSandbox = Join-Path $script:testBase "sandbox_quiet_proc_$([Guid]::NewGuid().ToString('N'))"
            $psi = New-Object System.Diagnostics.ProcessStartInfo
            $psi.FileName = "pwsh"
            $psi.Arguments = "-NoProfile -File `"$script:installerPath`" -Quiet -InstallPath `"$quietSandbox`""
            $psi.RedirectStandardOutput = $true
            $psi.RedirectStandardError = $true
            $psi.UseShellExecute = $false
            $psi.EnvironmentVariables["USERPROFILE"] = $quietSandbox
            $psi.EnvironmentVariables["APPDATA"] = (Join-Path $quietSandbox 'AppData\Roaming')
            $psi.EnvironmentVariables["HOME"] = $quietSandbox
            $proc = [System.Diagnostics.Process]::Start($psi)
            $proc.WaitForExit(30000) | Should -Be $true
            $proc.ExitCode | Should -Be 0

            Test-Path (Join-Path $quietSandbox 'bin\rtb.exe') | Should -Be $true
        }
    }
}
