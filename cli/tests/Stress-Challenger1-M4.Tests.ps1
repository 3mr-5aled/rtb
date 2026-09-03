#Requires -Version 7
# Milestone M4 Stress & Empirical Challenge Suite (Challenger 1)
# Testing: rtb doctor, config resiliency, project roots, pipeline isolation, tool checks

Describe "Milestone M4: Challenger 1 Empirical Stress Tests - rtb doctor" {
    BeforeAll {
        . (Join-Path $PSScriptRoot '..\src\utils\helpers.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\doctor.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\status.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\clean.ps1')
        Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

        $script:tempBase = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_m4_stress_$([Guid]::NewGuid().ToString('N'))"
        New-Item -ItemType Directory -Path $script:tempBase -Force | Out-Null

        # Create 9 valid dummy directories for testing healthy state
        $script:rootDirs = @('active','paused','planning','testing','production','staging','vibe','sandbox','abandoned')
        $script:validRoots = [ordered]@{}
        foreach ($r in $script:rootDirs) {
            $dir = Join-Path $script:tempBase $r
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
            $script:validRoots[$r] = $dir
        }
    }

    BeforeEach {
        Mock Get-Command { return $null }
    }

    AfterAll {
        if (Test-Path $script:tempBase) {
            Remove-Item -Recurse -Force $script:tempBase -ErrorAction SilentlyContinue
        }
    }

    Context "1. Config Resiliency & Error Handling" {
        It "returns $false and does not throw when Get-RtbConfig returns $null" {
            Mock Get-RtbConfig { return $null }
            $res = Rtb-Doctor
            $res | Should -Be $false
        }

        It "returns $false and does not throw when Get-RtbConfig throws an exception" {
            Mock Get-RtbConfig { throw [System.IO.InvalidDataException]::new("Corrupted JSON data at line 1") }
            $res = Rtb-Doctor
            $res | Should -Be $false
        }

        It "returns $false when config is an empty hashtable or missing projectRoots" {
            Mock Get-RtbConfig { return [PSCustomObject]@{} }
            $res = Rtb-Doctor
            $res | Should -Be $false
        }

        It "returns $false when projectRoots is null" {
            Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = $null } }
            $res = Rtb-Doctor
            $res | Should -Be $false
        }

        It "returns $false when projectRoots has all empty string paths" {
            $emptyRoots = [ordered]@{}
            foreach ($r in $script:rootDirs) { $emptyRoots[$r] = "" }
            Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = [PSCustomObject]$emptyRoots } }
            $res = Rtb-Doctor
            $res | Should -Be $false
        }
    }

    Context "2. Project Roots Edge Cases & Paths" {
        It "returns $false when one of 9 project roots is missing" {
            foreach ($missingKey in $script:rootDirs) {
                $customRoots = [ordered]@{}
                foreach ($r in $script:rootDirs) {
                    if ($r -eq $missingKey) {
                        $customRoots[$r] = Join-Path $script:tempBase "non_existent_$r"
                    } else {
                        $customRoots[$r] = $script:validRoots[$r]
                    }
                }
                Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = [PSCustomObject]$customRoots } }
                $res = Rtb-Doctor
                $res | Should -Be $false
            }
        }

        It "handles project root paths with spaces and brackets properly without syntax errors" {
            $specialDir = Join-Path $script:tempBase "path with spaces and [brackets] (1)"
            New-Item -ItemType Directory -Path $specialDir -Force | Out-Null

            $customRoots = [ordered]@{}
            foreach ($r in $script:rootDirs) {
                $customRoots[$r] = $specialDir
            }
            Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = [PSCustomObject]$customRoots } }
            Mock Get-Command { return [PSCustomObject]@{ Name = $Name } } -ParameterFilter { $Name -in @('git', 'rtbtui') }

            $res = Rtb-Doctor
            $res | Should -Be $true
        }

        It "returns $false when a project root is null or whitespace" {
            $customRoots = [ordered]@{}
            foreach ($r in $script:rootDirs) { $customRoots[$r] = $script:validRoots[$r] }
            $customRoots['vibe'] = '   '
            Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = [PSCustomObject]$customRoots } }

            $res = Rtb-Doctor
            $res | Should -Be $false
        }
    }

    Context "3. Tool Presence & Failure Modes" {
        It "returns $false when required tool 'git' is missing" {
            Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = [PSCustomObject]$script:validRoots } }
            Mock Get-Command { return [PSCustomObject]@{ Name = 'rtbtui' } } -ParameterFilter { $Name -eq 'rtbtui' }
            Mock Get-Command { return $null } -ParameterFilter { $Name -eq 'git' }

            $res = Rtb-Doctor
            $res | Should -Be $false
        }

        It "returns $false when required tool 'rtbtui' is missing" {
            Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = [PSCustomObject]$script:validRoots } }
            Mock Get-Command { return [PSCustomObject]@{ Name = 'git' } } -ParameterFilter { $Name -eq 'git' }
            Mock Get-Command { return $null } -ParameterFilter { $Name -eq 'rtbtui' }

            $res = Rtb-Doctor
            $res | Should -Be $false
        }

        It "returns $true when required tools exist even if all optional tools and AI agents are missing" {
            Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = [PSCustomObject]$script:validRoots } }
            Mock Get-Command { return [PSCustomObject]@{ Name = $Name } } -ParameterFilter { $Name -in @('git', 'rtbtui') }
            Mock Get-Command { return $null } -ParameterFilter { $Name -notin @('git', 'rtbtui') }

            $res = Rtb-Doctor
            $res | Should -Be $true
        }

        It "returns $true when all required and optional tools and AI agents exist" {
            Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = [PSCustomObject]$script:validRoots } }
            Mock Get-Command { return [PSCustomObject]@{ Name = $Name } }

            $res = Rtb-Doctor
            $res | Should -Be $true
        }
    }

    Context "4. Pipeline Return Type Isolation & Strictness" {
        It "returns exactly 1 pipeline item of type [bool] on success" {
            Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = [PSCustomObject]$script:validRoots } }
            Mock Get-Command { return [PSCustomObject]@{ Name = $Name } } -ParameterFilter { $Name -in @('git', 'rtbtui') }

            $output = @(Rtb-Doctor)
            $output.Count | Should -Be 1
            ($output[0] -is [bool]) | Should -Be $true
            $output[0] | Should -Be $true
        }

        It "returns exactly 1 pipeline item of type [bool] on failure" {
            Mock Get-RtbConfig { return $null }

            $output = @(Rtb-Doctor)
            $output.Count | Should -Be 1
            ($output[0] -is [bool]) | Should -Be $true
            $output[0] | Should -Be $false
        }

        It "aliases Dev-Doctor, Test-RtbDoctor, and Test-RtbEnvironment preserve single boolean output" {
            Mock Get-RtbConfig { return $null }

            $d1 = @(Dev-Doctor)
            $d1.Count | Should -Be 1
            $d1[0] | Should -Be $false

            $d2 = @(Test-RtbDoctor)
            $d2.Count | Should -Be 1
            $d2[0] | Should -Be $false

            $d3 = @(Test-RtbEnvironment)
            $d3.Count | Should -Be 1
            $d3[0] | Should -Be $false
        }

        It "pipes cleanly to Where-Object or Foreach-Object without stdout leakage" {
            Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = [PSCustomObject]$script:validRoots } }
            Mock Get-Command { return [PSCustomObject]@{ Name = $Name } } -ParameterFilter { $Name -in @('git', 'rtbtui') }

            $types = Rtb-Doctor | ForEach-Object { $_.GetType().FullName }
            @($types).Count | Should -Be 1
            $types | Should -Be 'System.Boolean'
        }
    }

    Context "5. Sequential Stress & No State Leakage" {
        It "survives 30 alternating success/failure invocations without state leakage" {
            for ($i = 0; $i -lt 30; $i++) {
                if ($i % 2 -eq 0) {
                    Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = [PSCustomObject]$script:validRoots } }
                    Mock Get-Command { return [PSCustomObject]@{ Name = $Name } } -ParameterFilter { $Name -in @('git', 'rtbtui') }
                    $res = Rtb-Doctor
                    $res | Should -Be $true
                } else {
                    Mock Get-RtbConfig { return $null }
                    $res = Rtb-Doctor
                    $res | Should -Be $false
                }
            }
        }
    }

    Context "6. Dispatcher & End-to-End CLI execution" {
        It "executes 'rtb doctor' and 'dev doctor' through module dispatcher without throwing" {
            Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = [PSCustomObject]$script:validRoots } }
            Mock Get-Command { return [PSCustomObject]@{ Name = $Name } } -ParameterFilter { $Name -in @('git', 'rtbtui') }

            { rtb doctor } | Should -Not -Throw
            { dev doctor } | Should -Not -Throw
        }

        It "passes extra arguments to rtb doctor gracefully" {
            Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = [PSCustomObject]$script:validRoots } }
            Mock Get-Command { return [PSCustomObject]@{ Name = $Name } } -ParameterFilter { $Name -in @('git', 'rtbtui') }

            { rtb doctor -Verbose } | Should -Not -Throw
            { rtb doctor --extra-flag } | Should -Not -Throw
        }
    }
}
