# Shared utility functions for RTB CLI

function Get-RtbConfig {
    $userConfigDir = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb' } else { Join-Path $env:HOME '.config/rtb' }
    $userConfigFile = Join-Path $userConfigDir 'rtb.config.json'
    
    $paths = @(
        $userConfigFile,
        (Join-Path $PSScriptRoot '..\..\..\config\rtb.config.json'),
        (Join-Path $PSScriptRoot '..\..\config\rtb.config.json'),
        (Join-Path $PSScriptRoot '..\..\..\config\dev.config.json')
    )
    foreach ($p in $paths) {
        if ($p -and (Test-Path $p)) {
            return Get-Content $p -Raw | ConvertFrom-Json
        }
    }
    Write-Error 'rtb config not found. Expected at rtb.config.json or user config directory'
    return $null
}

function Get-DevConfig {
    return Get-RtbConfig
}

function Get-AllProjectNames {
    $config = Get-RtbConfig
    if (-not $config) { return @() }
    
    $names = @()
    $roots = @(
        $config.projectRoots.active,
        $config.projectRoots.paused,
        $config.projectRoots.production,
        $config.projectRoots.staging,
        $config.projectRoots.vibe,
        $config.projectRoots.sandbox
    )
    
    foreach ($root in $roots) {
        if ($root -and (Test-Path $root)) {
            Get-ChildItem -Path $root -Directory -ErrorAction SilentlyContinue | ForEach-Object {
                $names += $_.Name
            }
        }
    }
    return $names | Sort-Object -Unique
}

function Get-ProjectsByStatus {
    param([string]$Status)
    $config = Get-RtbConfig
    if (-not $config) { return @() }
    
    $root = switch ($Status) {
        'active'     { $config.projectRoots.active }
        'paused'     { $config.projectRoots.paused }
        'production' { $config.projectRoots.production }
        'staging'    { $config.projectRoots.staging }
        'vibe'       { $config.projectRoots.vibe }
        'sandbox'    { $config.projectRoots.sandbox }
        'planning'   { $config.projectRoots.planning }
        'testing'    { $config.projectRoots.testing }
        'abandoned'  { $config.projectRoots.abandoned }
    }
    
    if ($root -and (Test-Path $root)) {
        return Get-ChildItem -Path $root -Directory -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Name
    }
    return @()
}

function Find-ProjectPath {
    param([string]$Name)
    $config = Get-RtbConfig
    if (-not $config) { return $null }
    
    $roots = @(
        @{ Path = $config.projectRoots.active; Status = 'Active' },
        @{ Path = $config.projectRoots.paused; Status = 'Paused' },
        @{ Path = $config.projectRoots.production; Status = 'Production' },
        @{ Path = $config.projectRoots.staging; Status = 'Staging' },
        @{ Path = $config.projectRoots.vibe; Status = 'Vibe' },
        @{ Path = $config.projectRoots.sandbox; Status = 'Sandbox' },
        @{ Path = $config.projectRoots.planning; Status = 'Planning' },
        @{ Path = $config.projectRoots.testing; Status = 'Testing' }
    )
    
    foreach ($entry in $roots) {
        if ($entry.Path) {
            $fullPath = Join-Path $entry.Path $Name
            if (Test-Path $fullPath) {
                return @{ Path = $fullPath; Status = $entry.Status }
            }
        }
    }
    
    # Fuzzy match - find projects containing the search term
    foreach ($entry in $roots) {
        if ($entry.Path -and (Test-Path $entry.Path)) {
            $match = Get-ChildItem -Path $entry.Path -Directory | Where-Object { $_.Name -like "*$Name*" } | Select-Object -First 1
            if ($match) {
                return @{ Path = $match.FullName; Status = $entry.Status }
            }
        }
    }
    
    return $null
}

function Write-RtbHeader {
    param([string]$Title)
    Write-Host '══════════════════════════════════════════' -ForegroundColor Cyan
    Write-Host "  rtb (ﺐﺘّﺭ) » $Title" -ForegroundColor Cyan
    Write-Host '══════════════════════════════════════════' -ForegroundColor Cyan
}

function Write-DevHeader {
    param([string]$Title)
    Write-RtbHeader -Title $Title
}
