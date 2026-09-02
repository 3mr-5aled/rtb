#Requires -Version 7
# Diagnostic Command Tests: rtb doctor (Milestone M4)
# Compatible with Pester 3.4.0 and Pester 5+

Describe "Rtb-Doctor Diagnostic Command" {
    BeforeAll {
        . (Join-Path $PSScriptRoot '..\src\utils\helpers.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\doctor.ps1')
        Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

        $script:tempBase = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_doctor_test_$([Guid]::NewGuid().ToString('N'))"
        New-Item -ItemType Directory -Path $script:tempBase -Force | Out-Null
    }

    BeforeEach {
        Mock Get-Command { return $null }
    }

    AfterAll {
        if (Test-Path $script:tempBase) {
            Remove-Item -Recurse -Force $script:tempBase -ErrorAction SilentlyContinue
        }
    }

    It "returns a boolean value" {
        $res = Rtb-Doctor
        $res | Should -Not -BeNullOrEmpty
        ($res -is [bool]) | Should -Be $true
    }

    It "Dev-Doctor alias exists and returns a boolean" {
        $res = Dev-Doctor
        ($res -is [bool]) | Should -Be $true
    }

    It "Test-RtbDoctor alias exists and returns a boolean" {
        $res = Test-RtbDoctor
        ($res -is [bool]) | Should -Be $true
    }

    It "handles missing config gracefully without throwing" {
        Mock Get-RtbConfig { return $null }
        $res = Rtb-Doctor
        $res | Should -Be $false
    }

    It "handles malformed config gracefully without throwing" {
        Mock Get-RtbConfig { throw [System.Exception]::new("Invalid JSON syntax") }
        $res = Rtb-Doctor
        $res | Should -Be $false
    }

    It "fails when project root does not exist on disk" {
        $nonExistentDir = Join-Path $script:tempBase 'non_existent_active_root'
        Mock Get-RtbConfig {
            return [PSCustomObject]@{
                projectRoots = [PSCustomObject]@{
                    active     = $nonExistentDir
                    paused     = $script:tempBase
                    planning   = $script:tempBase
                    testing    = $script:tempBase
                    production = $script:tempBase
                    staging    = $script:tempBase
                    vibe       = $script:tempBase
                    sandbox    = $script:tempBase
                    abandoned  = $script:tempBase
                }
            }
        }
        $res = Rtb-Doctor
        $res | Should -Be $false
    }

    It "passes project roots check when all roots exist" {
        $validRoots = [PSCustomObject]@{
            active     = $script:tempBase
            paused     = $script:tempBase
            planning   = $script:tempBase
            testing    = $script:tempBase
            production = $script:tempBase
            staging    = $script:tempBase
            vibe       = $script:tempBase
            sandbox    = $script:tempBase
            abandoned  = $script:tempBase
        }
        Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = $validRoots } }
        $res = Rtb-Doctor
        ($res -is [bool]) | Should -Be $true
    }

    It "fails when required git tool is missing" {
        $validRoots = [PSCustomObject]@{
            active     = $script:tempBase
            paused     = $script:tempBase
            planning   = $script:tempBase
            testing    = $script:tempBase
            production = $script:tempBase
            staging    = $script:tempBase
            vibe       = $script:tempBase
            sandbox    = $script:tempBase
            abandoned  = $script:tempBase
        }
        Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = $validRoots } }
        Mock Get-Command { return $null } -ParameterFilter { $Name -eq 'git' }
        $res = Rtb-Doctor
        $res | Should -Be $false
    }

    It "fails when rtb binary is missing" {
        $validRoots = [PSCustomObject]@{
            active     = $script:tempBase
            paused     = $script:tempBase
            planning   = $script:tempBase
            testing    = $script:tempBase
            production = $script:tempBase
            staging    = $script:tempBase
            vibe       = $script:tempBase
            sandbox    = $script:tempBase
            abandoned  = $script:tempBase
        }
        Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = $validRoots } }
        Mock Get-Command { return [PSCustomObject]@{ Name = 'git' } } -ParameterFilter { $Name -eq 'git' }
        Mock Get-Command { return $null } -ParameterFilter { $Name -eq 'rtb' }
        Mock Test-Path { return $false } -ParameterFilter { $Path -like '*rtb*' }
        Mock Test-Path { return $true } -ParameterFilter { $Path -notlike '*rtb*' }
        $res = Rtb-Doctor
        $res | Should -Be $false
    }

    It "optional tools missing does not cause health check failure" {
        $validRoots = [PSCustomObject]@{
            active     = $script:tempBase
            paused     = $script:tempBase
            planning   = $script:tempBase
            testing    = $script:tempBase
            production = $script:tempBase
            staging    = $script:tempBase
            vibe       = $script:tempBase
            sandbox    = $script:tempBase
            abandoned  = $script:tempBase
        }
        Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = $validRoots } }
        Mock Get-Command { return [PSCustomObject]@{ Name = $Name; Source = $Name } } -ParameterFilter { $Name -in @('git', 'rtb') }
        Mock Get-Command { return $null } -ParameterFilter { $Name -in @('node', 'cargo', 'python', 'tar') }
        $res = Rtb-Doctor
        $res | Should -Be $true
    }

    It "missing AI agents does not cause health check failure" {
        $validRoots = [PSCustomObject]@{
            active     = $script:tempBase
            paused     = $script:tempBase
            planning   = $script:tempBase
            testing    = $script:tempBase
            production = $script:tempBase
            staging    = $script:tempBase
            vibe       = $script:tempBase
            sandbox    = $script:tempBase
            abandoned  = $script:tempBase
        }
        Mock Get-RtbConfig { return [PSCustomObject]@{ projectRoots = $validRoots } }
        Mock Get-Command { return [PSCustomObject]@{ Name = $Name; Source = $Name } } -ParameterFilter { $Name -in @('git', 'rtb') }
        Mock Get-Command { return $null } -ParameterFilter { $Name -in @('agy','claude','gemini','codex','cursor','windsurf','aider','openhands') }
        $res = Rtb-Doctor
        $res | Should -Be $true
    }

    It "does not pollute pipeline with extra objects" {
        $res = @(Rtb-Doctor)
        $res.Count | Should -Be 1
        ($res[0] -is [bool]) | Should -Be $true
    }

    It "executes via rtb dispatcher and dev alias cleanly" {
        { rtb doctor } | Should -Not -Throw
        { dev doctor } | Should -Not -Throw
    }
}
