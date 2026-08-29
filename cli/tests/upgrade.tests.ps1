Describe "Self-Upgrade Engine (Rtb-Upgrade)" {
    BeforeAll {
        . (Join-Path $PSScriptRoot '..\src\utils\helpers.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\upgrade.ps1')
    }

    It "Rtb-Upgrade -Check reports version status" {
        $result = Rtb-Upgrade -Check
        $result | Should Not BeNullOrEmpty
    }
}
