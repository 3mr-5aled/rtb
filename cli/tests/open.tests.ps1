Describe "Rtb-Open Project Directory Command" {
    Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

    It "Displays error when non-existent project is specified" {
        $output = (Rtb-Open -ProjectName "non_existent_project_99999" *>&1) | Out-String
        $output | Should -Match "Project or path 'non_existent_project_99999' not found"
    }

    It "Exports Rtb-Open and Dev-Open functions" {
        (Get-Command Rtb-Open -ErrorAction SilentlyContinue) | Should -Not -BeNullOrEmpty
        (Get-Command Dev-Open -ErrorAction SilentlyContinue) | Should -Not -BeNullOrEmpty
    }
}
