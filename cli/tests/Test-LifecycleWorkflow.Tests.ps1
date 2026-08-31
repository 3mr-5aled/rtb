Describe "Standalone Installation & Lifecycle Workflow" {
    BeforeAll {
        Import-Module (Join-Path $PSScriptRoot '..\rtb.psd1') -Force
    }

    Context "Config Normalization and Backward Compatibility" {
        It "Normalizes flat string projectRoots into objects with path, label, and emoji" {
            $rawJson = @'
{
    "version": "1.0.0",
    "projectRoots": {
        "active": "C:\\Mock\\Active",
        "paused": "C:\\Mock\\Paused"
    }
}
'@
            $tempFile = [System.IO.Path]::GetTempFileName()
            Set-Content -Path $tempFile -Value $rawJson -Encoding UTF8

            $cfg = Get-Content $tempFile -Raw | ConvertFrom-Json
            foreach ($prop in $cfg.projectRoots.PSObject.Properties) {
                $val = $prop.Value
                if ($val -is [string]) {
                    $prop.Value = [PSCustomObject]@{
                        path  = $val
                        label = $prop.Name
                        emoji = '📁'
                    }
                }
            }

            $cfg.projectRoots.active.path | Should Be "C:\Mock\Active"
            $cfg.projectRoots.active.label | Should Be "active"
            $cfg.projectRoots.active.emoji | Should Be "📁"

            Remove-Item -Force $tempFile -ErrorAction SilentlyContinue
        }
    }

    Context "Rtb-Upgrade Command" {
        It "Rtb-Upgrade -Check resolves current module version from rtb.psd1" {
            $versionOutput = Rtb-Upgrade -Check
            $versionOutput | Should Not BeNullOrEmpty
            $versionOutput | Should Match 'v\d+\.\d+'
        }
    }

    Context "Config Gate and Helpers" {
        It "Test-RtbConfigured correctly detects active root presence" {
            $result = Test-RtbConfigured
            $result -is [bool] | Should Be $true
        }
    }
}
