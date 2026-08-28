Describe "Get-RtbConfig" {
    Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force

    It "Loads rtb.config.json from user config directory or fallback repository config" {
        $config = Get-RtbConfig
        $config.version | Should Be "1.0.0"
        $config.cleanDeps | Should Not BeNullOrEmpty
    }

    It "Exposes projectRoots object with active path" {
        $config = Get-RtbConfig
        $config.projectRoots.active | Should Not BeNullOrEmpty
    }
}
