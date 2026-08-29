Describe "AI Agent Discovery & CLI Launcher (Rtb-Agent)" {
    Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

    It "Get-InstalledAgents returns array with expected agent objects and properties" {
        $agents = Get-InstalledAgents
        $agents | Should Not BeNullOrEmpty
        $agents.Count | Should Be 4

        $commands = $agents | Select-Object -ExpandProperty command
        ($commands -contains 'agy') | Should Be $true
        ($commands -contains 'claude') | Should Be $true
        ($commands -contains 'gemini') | Should Be $true
        ($commands -contains 'codex') | Should Be $true

        foreach ($a in $agents) {
            $a.name | Should Not BeNullOrEmpty
            $a.command | Should Not BeNullOrEmpty
            ($a.installed -eq $true -or $a.installed -eq $false) | Should Be $true
        }
    }

    It "Rtb-Agent -List returns list of agents" {
        $result = Rtb-Agent -List
        $result | Should Not BeNullOrEmpty
        $result.Count | Should Be 4
    }

    It "Rtb-Agent displays error message when non-existent project is specified" {
        $output = (Rtb-Agent -ProjectName "non_existent_project_99999" *>&1) | Out-String
        $output | Should Match "Project or path 'non_existent_project_99999' not found"
    }

    It "Rtb-Agent displays error when invalid agent is specified" {
        $output = (Rtb-Agent -Agent "unknown_agent_xyz" *>&1) | Out-String
        $output | Should Match "Specified agent 'unknown_agent_xyz' is not recognized"
    }

    It "Generates transient .rtb_context.md file in project folder before launch" {
        $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) "rtb_agent_context_test_$([Guid]::NewGuid().ToString('N'))"
        New-Item -Path $tempDir -ItemType Directory -Force | Out-Null

        $contextFile = Join-Path $tempDir ".rtb_context.md"
        New-RtbAgentContextFile -ProjectPath $tempDir -ProjectName "test-proj" -Stack @("Rust", "Ratatui") -Status "Active"

        Test-Path $contextFile | Should Be $true
        (Get-Content $contextFile -Raw) | Should Match "Rust, Ratatui"

        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
