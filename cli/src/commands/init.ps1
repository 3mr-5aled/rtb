function Rtb-Init {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0, ValueFromRemainingArguments = $true)]
        [string[]]$RemainingArgs,

        [switch]$Force
    )
    
    $allArgs = @()
    if ($RemainingArgs) { $allArgs += $RemainingArgs }
    $isForce = $Force.IsPresent -or ($allArgs -contains '-Force') -or ($allArgs -contains '--force') -or ($allArgs -contains '-f')

    Write-RtbHeader -Title "Initialize Configuration"
    
    $userHomeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
    $userConfigDir = Join-Path $userHomeDir '.config/rtb'
    $userConfigFile = Join-Path $userConfigDir 'rtb.config.json'
    
    if (-not (Test-Path $userConfigDir)) {
        New-Item -ItemType Directory -Path $userConfigDir -Force | Out-Null
        Write-Host "Created configuration directory at: $userConfigDir" -ForegroundColor Green
    }
    
    if ((Test-Path $userConfigFile) -and -not $isForce) {
        Write-Host "Configuration already exists at $userConfigFile." -ForegroundColor Yellow
        Write-Host "Use 'rtb init -Force' to overwrite with defaults." -ForegroundColor Gray
        return
    }
    
    $baseRoot = if ($env:RTB_WORKSPACE_ROOT) { $env:RTB_WORKSPACE_ROOT } else { Join-Path (if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }) 'rtb-projects' }

    $defaultConfig = @{
        version = "1.0.0"
        projectRoots = @{
            active     = "$baseRoot\02-Projects\01-Development\01-Active"
            paused     = "$baseRoot\02-Projects\01-Development\04-Paused"
            planning   = "$baseRoot\02-Projects\01-Development\02-Planning"
            testing    = "$baseRoot\02-Projects\01-Development\03-Testing"
            abandoned  = "$baseRoot\02-Projects\01-Development\05-Abandoned"
            production = "$baseRoot\02-Projects\02-Deployed\01-Production"
            staging    = "$baseRoot\02-Projects\02-Deployed\02-Staging"
            vibe       = "$baseRoot\02-Projects\03-Vibe-Coding"
            sandbox    = "$baseRoot\01-SandBox"
        }
        backupRoot = "$baseRoot\08-Backup"
        configRoot = "$baseRoot\05-Config"
        templateDir = "$baseRoot\05-Config\templates"
        cleanDeps = @{
            daysInactive = 60
            targets = @("node_modules", ".venv", ".next", "__pycache__", "dist", "build", "target")
        }
        staleThresholdDays = 90
        gitHealth = @{
            scanRoots = @("$baseRoot\02-Projects", "$baseRoot\01-SandBox")
        }
    }
    
    $json = $defaultConfig | ConvertTo-Json -Depth 5
    Set-Content -Path $userConfigFile -Value $json -Encoding UTF8
    Write-Host "Successfully initialized RTB configuration at: $userConfigFile" -ForegroundColor Green
}


function Initialize-RtbConfig { Rtb-Init @args }
