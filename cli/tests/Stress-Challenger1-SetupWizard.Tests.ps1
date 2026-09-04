#Requires -Version 7
# Test Suite: Setup Wizard Empirical Stress Testing (Challenger 1)
# Verifies path edge cases, unicode, profile deduplication, standalone fallbacks, quiet flags, and runspace spinner lifecycle.

Describe "Setup Wizard Adversarial Stress Tests (Challenger 1)" {
    BeforeAll {
        $script:origEnvPath = $env:PATH
        if ($PROFILE -and (Test-Path $PROFILE)) {
            $script:origProfileContent = Get-Content $PROFILE -Raw
        }
        $script:repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
        $script:installerPath = Join-Path $script:repoRoot 'install.ps1'
        . $script:installerPath -NoExec
        $script:NoExitOnFail = $true
        $script:stressBase = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_stress_c1_$([Guid]::NewGuid().ToString('N'))"
        New-Item -ItemType Directory -Path $script:stressBase -Force | Out-Null
    }

    AfterAll {
        $env:PATH = $script:origEnvPath
        if ($script:origProfileContent -and $PROFILE) {
            $script:origProfileContent | Set-Content $PROFILE -Encoding UTF8
        }
        Get-Module rtb | Where-Object { $_.Path -notlike "*$([System.IO.Path]::DirectorySeparatorChar)cli$([System.IO.Path]::DirectorySeparatorChar)*" } | Remove-Module -Force -ErrorAction SilentlyContinue

        if ($script:stressBase -and (Test-Path -LiteralPath $script:stressBase)) {
            Remove-Item -Recurse -Force -LiteralPath $script:stressBase -ErrorAction SilentlyContinue
        }
    }

    Context "Stress Dimension 1: Path Handling (Spaces, Unicode, Deep Nesting)" {
        It "handles installation paths with multiple spaces and special characters" {
            $spacesPath = Join-Path $script:stressBase "RTB Install Path With Spaces and & + (1)"
            $profilePath = Join-Path $spacesPath "Profile With Spaces.ps1"

            $script:userConfigDir = $spacesPath
            $script:scriptsDir = Join-Path $spacesPath 'bin'
            $script:resolvedProfiles = @($profilePath)
            $script:QUIET = $true
            $script:scriptRoot = $script:repoRoot
            $script:isStandaloneOverride = $false

            { Install-Steps } | Should -Not -Throw

            Test-Path -LiteralPath $script:userConfigDir | Should -Be $true
            Test-Path -LiteralPath (Join-Path $script:scriptsDir 'rtb.js') | Should -Be $true
            Test-Path -LiteralPath (Join-Path $script:scriptsDir 'logo.txt') | Should -Be $true
            Test-Path -LiteralPath $profilePath | Should -Be $true

            $profContent = Get-Content -LiteralPath $profilePath -Raw
            $profContent | Should -Match "Invoke-Expression \(& rtb shell-init pwsh\)"
        }

        It "handles installation paths with Unicode, RTL/Arabic, CJK, and accented characters" {
            $unicodePath = Join-Path $script:stressBase "rtb_تبت_測試_café_🚀"
            $profilePath = Join-Path $unicodePath "profile_árabe_مرحبا.ps1"

            $script:userConfigDir = $unicodePath
            $script:scriptsDir = Join-Path $unicodePath 'bin'
            $script:resolvedProfiles = @($profilePath)
            $script:QUIET = $true
            $script:scriptRoot = $script:repoRoot
            $script:isStandaloneOverride = $false

            { Install-Steps } | Should -Not -Throw

            Test-Path -LiteralPath $script:userConfigDir | Should -Be $true
            Test-Path -LiteralPath (Join-Path $script:scriptsDir 'rtb.js') | Should -Be $true
            Test-Path -LiteralPath $profilePath | Should -Be $true

            $profContent = Get-Content -LiteralPath $profilePath -Raw -Encoding UTF8
            $profContent | Should -Match "Invoke-Expression \(& rtb shell-init pwsh\)"
        }

        It "handles deeply nested directories (10+ levels deep) where parent directories do not exist" {
            $deepPath = $script:stressBase
            1..10 | ForEach-Object { $deepPath = Join-Path $deepPath "sub_lvl_$_" }
            $deepPath = Join-Path $deepPath "rtb_install"
            $deepProfile = Join-Path $deepPath "nested_profile\deep_profile.ps1"

            $script:userConfigDir = $deepPath
            $script:scriptsDir = Join-Path $deepPath 'bin'
            $script:resolvedProfiles = @($deepProfile)
            $script:QUIET = $true
            $script:scriptRoot = $script:repoRoot
            $script:isStandaloneOverride = $false

            { Install-Steps } | Should -Not -Throw

            Test-Path -LiteralPath $script:userConfigDir | Should -Be $true
            Test-Path -LiteralPath (Join-Path $script:scriptsDir 'rtb.js') | Should -Be $true
            Test-Path -LiteralPath $deepProfile | Should -Be $true
        }
    }

    Context "Stress Dimension 2: Profile Corruption, Multi-Legacy & Deduplication" {
        It "handles a non-existent profile whose parent folder does not exist" {
            $nonExistentProfile = Join-Path $script:stressBase "non_existent_folder_1\non_existent_folder_2\profile.ps1"

            $script:userConfigDir = Join-Path $script:stressBase "sandbox_nonexist"
            $script:scriptsDir = Join-Path $script:userConfigDir 'bin'
            $script:resolvedProfiles = @($nonExistentProfile)
            $script:QUIET = $true
            $script:scriptRoot = $script:repoRoot
            $script:isStandaloneOverride = $false

            { Install-Steps } | Should -Not -Throw
            Test-Path -LiteralPath $nonExistentProfile | Should -Be $true
            $content = Get-Content -LiteralPath $nonExistentProfile -Raw
            $content | Should -Match '# RTB Shell Integration'
        }

        It "handles completely empty (0-byte) profile cleanly" {
            $emptyProfile = Join-Path $script:stressBase "empty_profile.ps1"
            New-Item -ItemType File -Path $emptyProfile -Force | Out-Null

            $script:userConfigDir = Join-Path $script:stressBase "sandbox_empty"
            $script:scriptsDir = Join-Path $script:userConfigDir 'bin'
            $script:resolvedProfiles = @($emptyProfile)
            $script:QUIET = $true
            $script:scriptRoot = $script:repoRoot
            $script:isStandaloneOverride = $false

            { Install-Steps } | Should -Not -Throw
            $content = Get-Content -LiteralPath $emptyProfile -Raw
            $content | Should -Match '# RTB Shell Integration'
            $content | Should -Match "Invoke-Expression \(& rtb shell-init pwsh\)"
        }

        It "cleans up multiple mixed legacy formats while preserving arbitrary user functions and environment vars" {
            $dirtyProfile = Join-Path $script:stressBase "dirty_profile.ps1"
            $dirtyContent = @"
# Custom User Setup
`$env:MY_CUSTOM_VAR = "hello world"

function Invoke-CustomTool {
    Write-Output "custom"
}

# RTB CLI Module
Import-Module 'C:\old\path\dev-tools\rtb.psd1' -DisableNameChecking -Force
Import-Module 'D:\old\rtb-command-tool\cli\rtb.psd1' -Force
Import-Module "E:\another\dev-cli\tools\rtb.psd1" -DisableNameChecking

# Other essential modules
Import-Module posh-git
Import-Module PSReadLine

# RTB CLI Module
Import-Module 'F:\legacy\rtb\module\rtb.psd1' -DisableNameChecking -Force

# End of custom setup
"@
            $dirtyContent | Set-Content -LiteralPath $dirtyProfile -Encoding UTF8

            $script:userConfigDir = Join-Path $script:stressBase "sandbox_dirty"
            $script:moduleHome = Join-Path $script:userConfigDir 'module'
            $script:scriptsDir = Join-Path $script:userConfigDir 'bin'
            $script:resolvedProfiles = @($dirtyProfile)
            $script:QUIET = $true
            $script:scriptRoot = $script:repoRoot
            $script:isStandaloneOverride = $false

            Install-Steps

            $cleaned = Get-Content -LiteralPath $dirtyProfile -Raw
            $cleaned | Should -Match '\$env:MY_CUSTOM_VAR = "hello world"'
            $cleaned | Should -Match 'function Invoke-CustomTool'
            $cleaned | Should -Match 'Import-Module posh-git'
            $cleaned | Should -Match 'Import-Module PSReadLine'
            $cleaned | Should -Not -Match 'C:\\old\\path\\dev-tools'
            $cleaned | Should -Not -Match 'D:\\old\\rtb-command-tool'
            $cleaned | Should -Not -Match 'E:\\another\\dev-cli'
            $cleaned | Should -Not -Match 'F:\\legacy\\rtb'

            # Ensure exactly ONE # RTB Shell Integration comment block
            $lines = Get-Content -LiteralPath $dirtyProfile
            $commentCount = ($lines | Where-Object { $_ -match '#\s*RTB Shell Integration' }).Count
            $commentCount | Should -Be 1
        }

        It "is strictly idempotent over 5 consecutive installation runs" {
            $idempotentProfile = Join-Path $script:stressBase "idempotent_profile.ps1"
            "Write-Output 'Initial Profile Content'" | Set-Content -LiteralPath $idempotentProfile -Encoding UTF8

            $script:userConfigDir = Join-Path $script:stressBase "sandbox_idempotent"
            $script:scriptsDir = Join-Path $script:userConfigDir 'bin'
            $script:resolvedProfiles = @($idempotentProfile)
            $script:QUIET = $true
            $script:scriptRoot = $script:repoRoot
            $script:isStandaloneOverride = $false

            1..5 | ForEach-Object {
                Install-Steps
            }

            $lines = Get-Content -LiteralPath $idempotentProfile
            $headerCount = ($lines | Where-Object { $_ -match '#\s*RTB Shell Integration' }).Count
            $importCount = ($lines | Where-Object { $_ -match "Invoke-Expression \(& rtb shell-init pwsh\)" }).Count

            $headerCount | Should -Be 1
            $importCount | Should -Be 1
            ($lines | Where-Object { $_ -match 'Initial Profile Content' }).Count | Should -Be 1
        }
    }

    Context "Stress Dimension 3: Standalone Simulation & Network Resilience" {
        It "throws cleanly when standalone zip download fails (fatal error)" {
            $fatalSandbox = Join-Path $script:stressBase "sandbox_fatal_net"
            New-Item -ItemType Directory -Path $fatalSandbox -Force | Out-Null

            Mock Invoke-WebRequest {
                throw [System.Net.WebException]::new("Simulated Network Down 503")
            }

            $script:userConfigDir = $fatalSandbox
            $script:scriptsDir = Join-Path $fatalSandbox 'bin'
            $script:resolvedProfiles = @(Join-Path $fatalSandbox 'p.ps1')
            $script:QUIET = $true
            $script:isStandaloneOverride = $true

            { Install-Steps } | Should -Throw
        }

        It "continues gracefully when TUI binary download throws 404 / WebException" {
            $tuiFailSandbox = Join-Path $script:stressBase "sandbox_tui_net_fail"
            New-Item -ItemType Directory -Path $tuiFailSandbox -Force | Out-Null

            # Create synthetic mock script for CLI engine
            $mockJs = Join-Path $tuiFailSandbox 'mock-rtb.js'
            '// Mock RTB JS' | Set-Content $mockJs

            Mock Invoke-WebRequest {
                param($Uri, $OutFile)
                if ($Uri -match 'rtb-cli\.js') {
                    Copy-Item $mockJs $OutFile -Force
                } elseif ($Uri -match 'VERSION') {
                    '0.5.3' | Set-Content $OutFile
                } else {
                    throw [System.Net.WebException]::new("404 Not Found")
                }
            }

            $script:userConfigDir = $tuiFailSandbox
            $script:scriptsDir = Join-Path $tuiFailSandbox 'bin'
            $script:resolvedProfiles = @(Join-Path $tuiFailSandbox 'p.ps1')
            $script:QUIET = $true
            $script:isStandaloneOverride = $true

            { Install-Steps } | Should -Not -Throw
            Test-Path -LiteralPath (Join-Path $script:scriptsDir 'rtb.js') | Should -Be $true
        }
    }

    Context "Stress Dimension 4: Process Isolation & Non-Interactive / Quiet Flags" {
        It "executes in fresh subprocess with RTB_QUIET=1 environment variable" {
            $procSandbox = Join-Path $script:stressBase "proc_quiet_env"
            $psi = New-Object System.Diagnostics.ProcessStartInfo
            $psi.FileName = "pwsh"
            $psi.Arguments = "-NoProfile -File `"$script:installerPath`" -InstallPath `"$procSandbox`""
            $psi.RedirectStandardOutput = $true
            $psi.RedirectStandardError = $true
            $psi.UseShellExecute = $false
            $psi.EnvironmentVariables["RTB_QUIET"] = "1"
            $psi.EnvironmentVariables["USERPROFILE"] = $procSandbox
            $psi.EnvironmentVariables["APPDATA"] = (Join-Path $procSandbox 'AppData\Roaming')
            $psi.EnvironmentVariables["HOME"] = $procSandbox

            $proc = [System.Diagnostics.Process]::Start($psi)
            $completed = $proc.WaitForExit(30000)
            $completed | Should -Be $true
            $proc.ExitCode | Should -Be 0
            Test-Path -LiteralPath (Join-Path $procSandbox 'bin\rtb.js') | Should -Be $true
        }

        It "executes in fresh subprocess with RTB_NON_INTERACTIVE=true environment variable" {
            $procSandbox = Join-Path $script:stressBase "proc_nonint_env"
            $psi = New-Object System.Diagnostics.ProcessStartInfo
            $psi.FileName = "pwsh"
            $psi.Arguments = "-NoProfile -File `"$script:installerPath`" -InstallPath `"$procSandbox`""
            $psi.RedirectStandardOutput = $true
            $psi.RedirectStandardError = $true
            $psi.UseShellExecute = $false
            $psi.EnvironmentVariables["RTB_NON_INTERACTIVE"] = "true"
            $psi.EnvironmentVariables["USERPROFILE"] = $procSandbox
            $psi.EnvironmentVariables["APPDATA"] = (Join-Path $procSandbox 'AppData\Roaming')
            $psi.EnvironmentVariables["HOME"] = $procSandbox

            $proc = [System.Diagnostics.Process]::Start($psi)
            $completed = $proc.WaitForExit(30000)
            $completed | Should -Be $true
            $proc.ExitCode | Should -Be 0
            Test-Path -LiteralPath (Join-Path $procSandbox 'bin\rtb.js') | Should -Be $true
        }

        It "executes in fresh subprocess with CI=true environment variable" {
            $procSandbox = Join-Path $script:stressBase "proc_ci_env"
            $psi = New-Object System.Diagnostics.ProcessStartInfo
            $psi.FileName = "pwsh"
            $psi.Arguments = "-NoProfile -File `"$script:installerPath`" -InstallPath `"$procSandbox`""
            $psi.RedirectStandardOutput = $true
            $psi.RedirectStandardError = $true
            $psi.UseShellExecute = $false
            $psi.EnvironmentVariables["CI"] = "true"
            $psi.EnvironmentVariables["USERPROFILE"] = $procSandbox
            $psi.EnvironmentVariables["APPDATA"] = (Join-Path $procSandbox 'AppData\Roaming')
            $psi.EnvironmentVariables["HOME"] = $procSandbox

            $proc = [System.Diagnostics.Process]::Start($psi)
            $completed = $proc.WaitForExit(30000)
            $completed | Should -Be $true
            $proc.ExitCode | Should -Be 0
            Test-Path -LiteralPath (Join-Path $procSandbox 'bin\rtb.js') | Should -Be $true
        }
    }

    Context "Stress Dimension 5: Runspace Spinner Rapid Lifecycle & Concurrency" {
        It "survives 30 rapid sequential start and stop cycles without deadlock or leak" {
            $script:QUIET = $false
            $script:ANSI = $true

            1..30 | ForEach-Object {
                $ctx = Start-Spinner "Rapid Spinner Cycle $_"
                $ctx | Should -Not -BeNullOrEmpty
                Start-Sleep -Milliseconds 10
                { Stop-Spinner $ctx ($true) } | Should -Not -Throw
            }
        }

        It "handles Stop-Spinner gracefully on `$null`, invalid hashtables, or already-stopped contexts" {
            { Stop-Spinner $null $true } | Should -Not -Throw
            { Stop-Spinner @{} $false } | Should -Not -Throw
            { Stop-Spinner @{ Type = 'Unknown'; Label = 'orphan' } $true } | Should -Not -Throw

            # Double stop on runspace
            $ctx = Start-Spinner "Double stop test"
            Start-Sleep -Milliseconds 15
            Stop-Spinner $ctx $true
            { Stop-Spinner $ctx $true } | Should -Not -Throw
        }
    }
}
