Describe "Outdated Dependency Auditor (Rtb-Deps)" {
    BeforeAll {
        . (Join-Path $PSScriptRoot '..\src\utils\helpers.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\deps.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\workspace.ps1')
    }

    It "Rtb-Deps returns declared dependencies for tui project" {
        $tuiPath = Join-Path $PSScriptRoot '..\..\tui'
        $deps = Rtb-Deps -ProjectName $tuiPath -Json
        $deps | Should Not BeNullOrEmpty
    }

    It "Rtb-Workspace detects project workspace configuration" {
        $ws = Rtb-Workspace -ProjectName 'rtb-command-tool' -Json
        $ws | Should Not BeNullOrEmpty
    }
}
