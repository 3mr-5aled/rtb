Describe "Extended Project Intelligence & CLI --json" {
    Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

    BeforeAll {
        $script:tempTestDir = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_ps_test_$([Guid]::NewGuid().ToString('N'))"
        New-Item -ItemType Directory -Path $script:tempTestDir -Force | Out-Null
        $script:activeRoot = Join-Path $script:tempTestDir "active"
        $script:projectName = "rtb-command-tool"
        $script:projectPath = Join-Path $script:activeRoot $script:projectName
        New-Item -ItemType Directory -Path $script:projectPath -Force | Out-Null
        Set-Content -Path (Join-Path $script:projectPath "package.json") -Value '{"name":"rtb-command-tool","dependencies":{"next":"14.0.0"}}'

        $script:testConfig = [PSCustomObject]@{
            projectRoots = [PSCustomObject]@{
                active = [PSCustomObject]@{
                    path  = $script:activeRoot
                    label = "active"
                    emoji = "📁"
                }
            }
        }
    }

    AfterAll {
        if (Test-Path $script:tempTestDir) {
            Remove-Item -Recurse -Force $script:tempTestDir -ErrorAction SilentlyContinue
        }
    }

    BeforeEach {
        Mock -ModuleName rtb Get-RtbConfig { return $script:testConfig }
        Mock -ModuleName rtb Get-DevConfig { return $script:testConfig }
    }

    It "Detects .NET stack, Monorepo, CI/CD, and Runtime version in Get-ProjectDetails" {
        $projDir = Join-Path $script:tempTestDir "TestProject"
        New-Item -ItemType Directory -Path $projDir -Force | Out-Null

        New-Item -ItemType File -Path (Join-Path $projDir "App.csproj") -Force | Out-Null
        New-Item -ItemType File -Path (Join-Path $projDir "pnpm-workspace.yaml") -Force | Out-Null
        $workflows = Join-Path $projDir ".github\workflows"
        New-Item -ItemType Directory -Path $workflows -Force | Out-Null
        Set-Content -Path (Join-Path $projDir ".nvmrc") -Value "v20.10.0"
        @{ name = "test-app"; dependencies = @{ next = "14.0.0"; tailwindcss = "3.0.0" } } | ConvertTo-Json | Set-Content (Join-Path $projDir "package.json")

        $details = Get-ProjectDetails -ProjectPath $projDir -Status "Active"
        $details.name | Should -Be "TestProject"
        ($details.stack -contains ".NET") | Should -Be $true
        ($details.stack -contains "Next.js") | Should -Be $true
        ($details.stack -contains "Tailwind") | Should -Be $true
        $details.is_monorepo | Should -Be $true
        $details.ci_cd | Should -Be "GitHub Actions"
        $details.runtime_version | Should -Be "v20.10.0"
    }

    It "Rtb-List outputs valid JSON array when --json flag is passed" {
        $jsonStr = Rtb-List --json | Out-String
        $jsonStr | Should -Not -BeNullOrEmpty
        $parsed = $jsonStr | ConvertFrom-Json
        $parsed | Should -Not -BeNullOrEmpty
    }

    It "Rtb-Info returns detailed metadata object when --json flag is passed" {
        $jsonStr = Rtb-Info $script:projectName --json | Out-String
        $jsonStr | Should -Not -BeNullOrEmpty
        $parsed = $jsonStr | ConvertFrom-Json
        $parsed.name | Should -Be $script:projectName
        ($parsed.is_monorepo -ne $null) | Should -Be $true
    }
}
