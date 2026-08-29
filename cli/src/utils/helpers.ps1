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

function Get-ProjectDetails {
    param(
        [Parameter(Mandatory = $true)][string]$ProjectPath,
        [string]$Status = 'Active'
    )

    if (-not (Test-Path $ProjectPath)) { return $null }

    $name = Split-Path $ProjectPath -Leaf

    # Detect stack
    $stack = @()

    $pkgPath = Join-Path $ProjectPath 'package.json'
    if (Test-Path $pkgPath) {
        try {
            $pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json
            $deps = if ($pkg.dependencies) { $pkg.dependencies.PSObject.Properties.Name } else { @() }
            $devDeps = if ($pkg.devDependencies) { $pkg.devDependencies.PSObject.Properties.Name } else { @() }
            $allKeys = @($deps) + @($devDeps)

            if ($allKeys -contains 'next') { $stack += 'Next.js' }
            elseif ($allKeys -contains 'react') { $stack += 'React' }
            elseif ($allKeys -contains 'vue') { $stack += 'Vue' }
            elseif ($allKeys -contains 'vite') { $stack += 'Vite' }

            if ($allKeys -contains 'tailwindcss') { $stack += 'Tailwind' }
            if ($allKeys -contains 'prisma' -or $allKeys -contains '@prisma/client') { $stack += 'Prisma' }
            if ($allKeys -contains 'typescript') { $stack += 'TypeScript' }
            if ($allKeys -contains 'express') { $stack += 'Express' }
            elseif ($allKeys -contains 'fastify') { $stack += 'Fastify' }
        } catch {}

        if (-not ($stack | Where-Object { $_ -in 'Next.js', 'React', 'Vue', 'Vite', 'Node.js' })) {
            $stack += 'Node.js'
        }
    }

    if ((Test-Path (Join-Path $ProjectPath 'uv.lock')) -or (Test-Path (Join-Path $ProjectPath 'poetry.lock')) -or (Test-Path (Join-Path $ProjectPath 'requirements.txt')) -or (Test-Path (Join-Path $ProjectPath 'pyproject.toml'))) {
        $stack += 'Python'
    }

    if (Test-Path (Join-Path $ProjectPath 'Cargo.toml')) { $stack += 'Rust' }
    if (Test-Path (Join-Path $ProjectPath 'go.mod')) { $stack += 'Go' }
    if ((Test-Path (Join-Path $ProjectPath 'pom.xml')) -or (Test-Path (Join-Path $ProjectPath 'build.gradle'))) { $stack += 'Java' }
    if (Test-Path (Join-Path $ProjectPath 'Dockerfile')) { $stack += 'Docker' }
    if ((Test-Path (Join-Path $ProjectPath 'docker-compose.yml')) -or (Test-Path (Join-Path $ProjectPath 'docker-compose.yaml'))) { $stack += 'Compose' }
    if ((Test-Path (Join-Path $ProjectPath 'rtb.psm1')) -or (Test-Path (Join-Path $ProjectPath 'rtb.psd1')) -or (Test-Path (Join-Path $ProjectPath 'dev.psm1'))) { $stack += 'PowerShell' }

    $hasDotnet = Get-ChildItem -Path $ProjectPath -File -ErrorAction SilentlyContinue | Where-Object { $_.Extension -in '.csproj', '.sln' }
    if ($hasDotnet) { $stack += '.NET' }

    if ($stack.Count -eq 0) { $stack += '-' }

    # Detect Monorepo
    $isMonorepo = (Test-Path (Join-Path $ProjectPath 'pnpm-workspace.yaml')) -or
                  (Test-Path (Join-Path $ProjectPath 'lerna.json')) -or
                  (Test-Path (Join-Path $ProjectPath 'nx.json')) -or
                  (Test-Path (Join-Path $ProjectPath 'turbo.json'))

    if (-not $isMonorepo -and (Test-Path $pkgPath)) {
        try {
            $pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json
            if ($pkg.PSObject.Properties['workspaces']) {
                $isMonorepo = $true
            }
        } catch {}
    }

    # Detect CI/CD
    $ciCd = $null
    if (Test-Path (Join-Path $ProjectPath '.github\workflows')) {
        $ciCd = 'GitHub Actions'
    } elseif (Test-Path (Join-Path $ProjectPath '.gitlab-ci.yml')) {
        $ciCd = 'GitLab CI'
    } elseif (Test-Path (Join-Path $ProjectPath 'azure-pipelines.yml')) {
        $ciCd = 'Azure Pipelines'
    } elseif (Test-Path (Join-Path $ProjectPath '.circleci')) {
        $ciCd = 'CircleCI'
    }

    # Detect Runtime Version
    $runtimeVersion = $null
    $nvmrcPath = Join-Path $ProjectPath '.nvmrc'
    $pyverPath = Join-Path $ProjectPath '.python-version'
    $rusttcPath = Join-Path $ProjectPath 'rust-toolchain.toml'

    if (Test-Path $nvmrcPath) {
        $runtimeVersion = (Get-Content $nvmrcPath -Raw).Split("`r`n")[0].Trim()
    } elseif (Test-Path $pyverPath) {
        $runtimeVersion = (Get-Content $pyverPath -Raw).Split("`r`n")[0].Trim()
    } elseif (Test-Path $rusttcPath) {
        $content = Get-Content $rusttcPath -Raw
        foreach ($line in $content.Split("`r`n")) {
            if ($line.Trim().StartsWith('channel')) {
                $parts = $line.Split('=')
                if ($parts.Count -gt 1) {
                    $runtimeVersion = $parts[1].Trim(" `'`"")
                    break
                }
            }
        }
    } elseif (Test-Path $pkgPath) {
        try {
            $pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json
            if ($pkg.PSObject.Properties['engines'] -and $pkg.engines.PSObject.Properties['node']) {
                $runtimeVersion = $pkg.engines.node
            }
        } catch {}
    }

    # Git info
    $gitInfo = $null
    if (Test-Path (Join-Path $ProjectPath '.git')) {
        try {
            $branch = git -C $ProjectPath branch --show-current 2>$null
            if (-not $branch) { $branch = 'unknown' }
            $statusLines = git -C $ProjectPath status --porcelain 2>$null
            $uncommitted = if ($statusLines) { ($statusLines | Measure-Object).Count } else { 0 }
            $unpushedLines = git -C $ProjectPath log '@{u}..' --oneline 2>$null
            $unpushed = if ($unpushedLines) { ($unpushedLines | Measure-Object).Count } else { 0 }
            $lastCommitMsg = git -C $ProjectPath log -1 --format='%s' 2>$null
            $lastCommitRelative = git -C $ProjectPath log -1 --format='%cr' 2>$null
            $remotes = git -C $ProjectPath remote 2>$null
            $hasRemote = [bool]($remotes -and $remotes.Trim())

            $gitInfo = [PSCustomObject]@{
                branch               = $branch
                uncommitted          = [uint32]$uncommitted
                unpushed             = [uint32]$unpushed
                last_commit_msg      = $lastCommitMsg
                last_commit_relative = $lastCommitRelative
                has_remote           = $hasRemote
            }
        } catch {}
    }

    # Readme preview
    $readmePreview = $null
    foreach ($rName in @('README.md', 'readme.md', 'README.txt')) {
        $rPath = Join-Path $ProjectPath $rName
        if (Test-Path $rPath) {
            $lines = Get-Content $rPath -TotalCount 6 -ErrorAction SilentlyContinue
            if ($lines) { $readmePreview = $lines -join "`n" }
            break
        }
    }

    # Last modified
    $lastFile = Get-ChildItem $ProjectPath -Recurse -Depth 3 -File -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -notmatch 'node_modules|\.git|dist|build|\.next|target' } |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1
    $lastMod = if ($lastFile) { $lastFile.LastWriteTime.ToString('yyyy-MM-ddTHH:mm:ss') } else { $null }

    [PSCustomObject]@{
        name             = $name
        path             = $ProjectPath
        status           = $Status
        stack            = $stack
        last_modified    = $lastMod
        total_size_bytes = 0
        dep_size_bytes   = 0
        git              = $gitInfo
        readme_preview   = $readmePreview
        is_monorepo      = [bool]$isMonorepo
        ci_cd            = $ciCd
        runtime_version  = $runtimeVersion
    }
}

function Get-AllProjectsDetails {
    param([string]$Filter = 'all')
    $config = Get-DevConfig
    if (-not $config) { return @() }

    $categories = @(
        @{ Name = 'Active';     Status = 'Active';     Path = $config.projectRoots.active;     Show = $Filter -in 'all','active' },
        @{ Name = 'Paused';     Status = 'Paused';     Path = $config.projectRoots.paused;     Show = $Filter -in 'all','paused' },
        @{ Name = 'Production'; Status = 'Production'; Path = $config.projectRoots.production; Show = $Filter -in 'all','deployed' },
        @{ Name = 'Staging';    Status = 'Staging';    Path = $config.projectRoots.staging;    Show = $Filter -in 'all','deployed' },
        @{ Name = 'Vibe';       Status = 'Vibe';       Path = $config.projectRoots.vibe;       Show = $Filter -in 'all','vibe' },
        @{ Name = 'Sandbox';    Status = 'Sandbox';    Path = $config.projectRoots.sandbox;    Show = $Filter -in 'all','all' },
        @{ Name = 'Planning';   Status = 'Planning';   Path = $config.projectRoots.planning;   Show = $Filter -in 'all','all' },
        @{ Name = 'Testing';    Status = 'Testing';    Path = $config.projectRoots.testing;    Show = $Filter -in 'all','all' }
    )

    $results = [System.Collections.Generic.List[PSCustomObject]]::new()
    foreach ($cat in $categories) {
        if (-not $cat.Show -or -not (Test-Path $cat.Path)) { continue }
        $dirs = Get-ChildItem -Path $cat.Path -Directory -ErrorAction SilentlyContinue
        foreach ($d in $dirs) {
            $details = Get-ProjectDetails -ProjectPath $d.FullName -Status $cat.Status
            $results.Add($details)
        }
    }
    return $results.ToArray()
}
