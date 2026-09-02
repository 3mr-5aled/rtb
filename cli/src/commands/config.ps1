<#
.SYNOPSIS
    Rtb-Config — Open RTB configuration file in default editor.
.DESCRIPTION
    Resolves the active rtb.config.json path and launches it in the user's default editor ($env:EDITOR, VS Code, Notepad, or OS default).
#>

function Get-RtbConfigFilePath {
    $userHomeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
    $appDataPath = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb\rtb.config.json' } else { $null }
    $dotConfigPath = Join-Path $userHomeDir '.config\rtb\rtb.config.json'

    if ($appDataPath -and (Test-Path $appDataPath)) { return $appDataPath }
    if (Test-Path $dotConfigPath) { return $dotConfigPath }
    
    if ($appDataPath) { return $appDataPath }
    return $dotConfigPath
}

function Rtb-Config {
    [CmdletBinding()]
    param()

    $configFile = Get-RtbConfigFilePath
    $configDir = Split-Path $configFile -Parent

    if (-not (Test-Path $configFile)) {
        if (-not (Test-Path $configDir)) {
            New-Item -ItemType Directory -Path $configDir -Force | Out-Null
        }
        $defaultConfig = [ordered]@{
            version      = "1.0.0"
            projectRoots = [ordered]@{
                active     = [ordered]@{ path = ""; label = "Active";     emoji = "📁" }
                paused     = [ordered]@{ path = ""; label = "Paused";     emoji = "⏸️" }
                production = [ordered]@{ path = ""; label = "Production"; emoji = "🚀" }
            }
        }
        $defaultConfig | ConvertTo-Json -Depth 5 | Set-Content -Path $configFile -Encoding UTF8
    }

    Write-Host "Opening RTB configuration..." -ForegroundColor Cyan
    Write-Host "  Config file: $configFile" -ForegroundColor Gray

    if ($env:RTB_NON_INTERACTIVE -or $env:CI -or $env:GITHUB_ACTIONS) {
        return
    }

    # 1. Check $env:EDITOR / $env:VISUAL
    $editor = if ($env:EDITOR) { $env:EDITOR } elseif ($env:VISUAL) { $env:VISUAL } else { $null }
    if ($editor) {
        try {
            Start-Process $editor -ArgumentList "`"$configFile`""
            return
        } catch {}
    }

    # 2. Fallbacks by OS
    try {
        if ($IsWindows -or $env:OS -like '*Windows*') {
            if (Get-Command code -ErrorAction SilentlyContinue) {
                Start-Process 'code' -ArgumentList "`"$configFile`""
            } else {
                Start-Process 'notepad.exe' -ArgumentList "`"$configFile`""
            }
        } elseif ($IsMacOS) {
            Start-Process 'open' -ArgumentList "`"$configFile`""
        } else {
            Start-Process 'xdg-open' -ArgumentList "`"$configFile`""
        }
    } catch {
        Invoke-Item $configFile
    }
}

function Dev-Config {
    Rtb-Config @args
}

function Invoke-RtbConfig { Rtb-Config @args }
function Edit-RtbConfig { Rtb-Config @args }
