Describe "Get-RtbConfig and Test-RtbConfigured" {
    Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

    It "Loads rtb.config.json from user config directory or fallback repository config" {
        $config = Get-RtbConfig
        $config.version | Should Be "1.0.0"
        $config.cleanDeps | Should Not BeNullOrEmpty
    }

    It "Exposes projectRoots object with active path, label, and emoji" {
        $config = Get-RtbConfig
        $config.projectRoots.active | Should Not BeNullOrEmpty
        $config.projectRoots.active.path | Should Not BeNullOrEmpty
        $config.projectRoots.active.label | Should Not BeNullOrEmpty
        $config.projectRoots.active.emoji | Should Not BeNullOrEmpty
    }

    It "Test-RtbConfigured returns boolean status" {
        $configured = Test-RtbConfigured
        $configured -is [bool] | Should Be $true
    }
}
