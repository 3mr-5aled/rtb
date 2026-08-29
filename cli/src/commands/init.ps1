function Rtb-Init {
    [CmdletBinding()]
    param(
        [switch]$Force
    )
    
    Write-RtbHeader -Title "Initialize Configuration"
    
    $userConfigDir = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb' } else { Join-Path $env:HOME '.config/rtb' }
    $userConfigFile = Join-Path $userConfigDir 'rtb.config.json'
    
    if (-not (Test-Path $userConfigDir)) {
        New-Item -ItemType Directory -Path $userConfigDir -Force | Out-Null
        Write-Host "Created configuration directory at: $userConfigDir" -ForegroundColor Green
    }
    
    if ((Test-Path $userConfigFile) -and -not $Force) {
        Write-Host "Configuration already exists at $userConfigFile." -ForegroundColor Yellow
        Write-Host "Use 'rtb init -Force' to overwrite with defaults." -ForegroundColor Gray
        return
    }
    
    $defaultConfig = @{
        version = "1.0.0"
        projectRoots = @{
            active     = "D:\02-Projects\01-Development\01-Active"
            paused     = "D:\02-Projects\01-Development\04-Paused"
            planning   = "D:\02-Projects\01-Development\02-Planning"
            testing    = "D:\02-Projects\01-Development\03-Testing"
            abandoned  = "D:\02-Projects\01-Development\05-Abandoned"
            production = "D:\02-Projects\02-Deployed\01-Production"
            staging    = "D:\02-Projects\02-Deployed\02-Staging"
            vibe       = "D:\02-Projects\03-Vibe-Coding"
            sandbox    = "D:\01-SandBox"
        }
        backupRoot = "D:\08-Backup"
        configRoot = "D:\05-Config"
        templateDir = "D:\05-Config\templates"
        cleanDeps = @{
            daysInactive = 60
            targets = @("node_modules", ".venv", ".next", "__pycache__", "dist", "build", "target")
        }
        staleThresholdDays = 90
        gitHealth = @{
            scanRoots = @("D:\02-Projects", "D:\01-SandBox")
        }
    }
    
    $json = $defaultConfig | ConvertTo-Json -Depth 5
    Set-Content -Path $userConfigFile -Value $json -Encoding UTF8
    Write-Host "Successfully initialized RTB configuration at: $userConfigFile" -ForegroundColor Green
}


function Initialize-RtbConfig { Rtb-Init @args }
