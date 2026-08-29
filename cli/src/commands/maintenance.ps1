function Resolve-MaintenanceScript {
    param([string]$TaskName)

    $config = Get-RtbConfig
    $scriptName = switch ($TaskName) {
        'guard'       { 'guard-d-drive.ps1' }
        'backup'      { 'backup-configs.ps1' }
        'env'         { 'backup-env-files.ps1' }
        'maintenance' { 'weekly-maintenance.ps1' }
        default       { "$TaskName.ps1" }
    }

    # 1. Config path
    if ($config -and $config.PSObject.Properties['maintenanceScripts'] -and $config.maintenanceScripts.$TaskName) {
        $customPath = $config.maintenanceScripts.$TaskName
        if (Test-Path $customPath) { return $customPath }
    }

    # 2. Repo script directory fallback
    $repoScript = Join-Path $PSScriptRoot "..\..\scripts\$scriptName"
    if (Test-Path $repoScript) { return (Resolve-Path $repoScript).Path }

    # 3. System fallback
    $systemScript = Join-Path 'D:\06-Tools\scripts' $scriptName
    if (Test-Path $systemScript) { return $systemScript }

    return $null
}

function Invoke-MaintenanceTask {
    param(
        [Parameter(Mandatory = $true)][string]$Task,
        [switch]$ReportOnly,
        [switch]$FullRun
    )

    $scriptPath = Resolve-MaintenanceScript -TaskName $Task
    if (-not $scriptPath) {
        Write-Host "Maintenance script for task '$Task' not found." -ForegroundColor Red
        return
    }

    $params = @{}
    if ($ReportOnly) { $params['ReportOnly'] = $true }
    if ($FullRun) { $params['FullRun'] = $true }

    & $scriptPath @params
}

function Dev-Maintenance {
    $full = $args -contains '--full'
    Write-RtbHeader 'Weekly Maintenance'
    Invoke-MaintenanceTask -Task 'maintenance' -FullRun:$full
}

function Invoke-RtbMaintenance { Dev-Maintenance @args }
