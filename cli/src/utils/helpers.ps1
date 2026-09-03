# Shared utility functions for RTB CLI

function Get-RtbRootPath {
    param([object]$RootEntry)
    if ($null -eq $RootEntry) { return $null }
    if ($RootEntry -is [string]) { return $RootEntry }
    if ($RootEntry.PSObject.Properties['path']) { return $RootEntry.path }
    return [string]$RootEntry
}

function Get-RtbConfig {
    $userHomeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
    $dotConfigDir = Join-Path $userHomeDir '.config/rtb'
    $appDataPath = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb/rtb.config.json' } else { $null }

    $paths = @(
        $env:RTB_CONFIG,
        $appDataPath,
        (Join-Path $dotConfigDir 'rtb.config.json'),
        (Join-Path (Get-Location) 'config\rtb.config.json'),
        (Join-Path (Get-Location) 'config\dev.config.json'),
        (Join-Path $PSScriptRoot '..\config\rtb.config.json'),
        (Join-Path $PSScriptRoot '..\..\config\rtb.config.json'),
        (Join-Path $PSScriptRoot '..\..\..\config\rtb.config.json'),
        (Join-Path $PSScriptRoot '..\..\..\config\dev.config.json')
    )
    foreach ($p in $paths) {
        if ($p -and (Test-Path $p)) {
            $cfg = Get-Content $p -Raw | ConvertFrom-Json
            if ($cfg -and $cfg.projectRoots) {
                # Normalize projectRoots entries to objects with { path, label, emoji }
                foreach ($prop in $cfg.projectRoots.PSObject.Properties) {
                    $val = $prop.Value
                    if ($val -is [string]) {
                        $prop.Value = [PSCustomObject]@{
                            path  = $val
                            label = $prop.Name
                            emoji = '📁'
                        }
                    } elseif ($val -is [PSCustomObject] -and $val.PSObject.Properties['path']) {
                        if (-not $val.PSObject.Properties['label']) {
                            $val | Add-Member -NotePropertyName 'label' -NotePropertyValue $prop.Name -Force
                        }
                        if (-not $val.PSObject.Properties['emoji']) {
                            $val | Add-Member -NotePropertyName 'emoji' -NotePropertyValue '📁' -Force
                        }
                    }
                }
            }
            return $cfg
        }
    }
    return $null
}

function Get-DevConfig {
    return Get-RtbConfig
}

function Test-RtbConfigured {
    try {
        $cfg = Get-RtbConfig -ErrorAction SilentlyContinue
        if (-not $cfg) {
            return $false
        }
        if ($cfg.projectRoots -and $cfg.projectRoots.active) {
            $activePath = Get-RtbRootPath $cfg.projectRoots.active
            return (-not [string]::IsNullOrWhiteSpace($activePath))
        }
        if ($cfg.projectRoots -or $cfg.backupRoot -or $cfg.cleanDeps) {
            return $true
        }
        return $false
    } catch {
        return $false
    }
}

function Get-AllProjectNames {
    $config = Get-RtbConfig
    if (-not $config) { return @() }
    
    $names = @()
    $roots = @(
        (Get-RtbRootPath $config.projectRoots.active),
        (Get-RtbRootPath $config.projectRoots.paused),
        (Get-RtbRootPath $config.projectRoots.production),
        (Get-RtbRootPath $config.projectRoots.staging),
        (Get-RtbRootPath $config.projectRoots.vibe),
        (Get-RtbRootPath $config.projectRoots.sandbox),
        (Get-RtbRootPath $config.projectRoots.planning),
        (Get-RtbRootPath $config.projectRoots.testing),
        (Get-RtbRootPath $config.projectRoots.abandoned)
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
    
    $root = switch ($Status.ToLower()) {
        'active'     { Get-RtbRootPath $config.projectRoots.active }
        'paused'     { Get-RtbRootPath $config.projectRoots.paused }
        'production' { Get-RtbRootPath $config.projectRoots.production }
        'staging'    { Get-RtbRootPath $config.projectRoots.staging }
        'vibe'       { Get-RtbRootPath $config.projectRoots.vibe }
        'sandbox'    { Get-RtbRootPath $config.projectRoots.sandbox }
        'planning'   { Get-RtbRootPath $config.projectRoots.planning }
        'testing'    { Get-RtbRootPath $config.projectRoots.testing }
        'abandoned'  { Get-RtbRootPath $config.projectRoots.abandoned }
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
        @{ Path = (Get-RtbRootPath $config.projectRoots.active);     Status = 'Active' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.paused);     Status = 'Paused' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.production); Status = 'Production' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.staging);    Status = 'Staging' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.vibe);       Status = 'Vibe' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.sandbox);    Status = 'Sandbox' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.planning);   Status = 'Planning' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.testing);    Status = 'Testing' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.abandoned);  Status = 'Abandoned' }
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

function Find-ProjectPathFuzzy {
    param([Parameter(Mandatory = $true)][AllowEmptyString()][AllowNull()][string]$Query = '')
    $config = Get-RtbConfig
    if (-not $config) { return @() }

    $roots = @(
        @{ Path = (Get-RtbRootPath $config.projectRoots.active);     Status = 'Active' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.paused);     Status = 'Paused' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.production); Status = 'Production' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.staging);    Status = 'Staging' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.vibe);       Status = 'Vibe' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.sandbox);    Status = 'Sandbox' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.planning);   Status = 'Planning' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.testing);    Status = 'Testing' },
        @{ Path = (Get-RtbRootPath $config.projectRoots.abandoned);  Status = 'Abandoned' }
    )

    $q = $Query.ToLower()
    $results = @()

    foreach ($entry in $roots) {
        if (-not $entry.Path -or -not (Test-Path $entry.Path)) { continue }
        Get-ChildItem -Path $entry.Path -Directory -ErrorAction SilentlyContinue | ForEach-Object {
            $n = $_.Name.ToLower()
            $score = if ($n -eq $q)                                    { 100 }
                     elseif ($n.StartsWith($q))                        { 75  }
                     elseif ($n.Contains($q))                          { 50  }
                     elseif ($_.FullName.ToLower().Contains($q))       { 25  }
                     else { 0 }
            if ($score -gt 0) {
                $results += [PSCustomObject]@{
                    Name   = $_.Name
                    Path   = $_.FullName
                    Status = $entry.Status
                    Score  = $score
                }
            }
        }
    }
    return $results | Sort-Object Score -Descending
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
    $lastFile = Get-ChildItem $ProjectPath -Recurse -File -ErrorAction SilentlyContinue |
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
        @{ Name = 'Active';     Status = 'Active';     Path = (Get-RtbRootPath $config.projectRoots.active);     Show = $Filter -in 'all','active' },
        @{ Name = 'Paused';     Status = 'Paused';     Path = (Get-RtbRootPath $config.projectRoots.paused);     Show = $Filter -in 'all','paused' },
        @{ Name = 'Production'; Status = 'Production'; Path = (Get-RtbRootPath $config.projectRoots.production); Show = $Filter -in 'all','deployed' },
        @{ Name = 'Staging';    Status = 'Staging';    Path = (Get-RtbRootPath $config.projectRoots.staging);    Show = $Filter -in 'all','deployed' },
        @{ Name = 'Vibe';       Status = 'Vibe';       Path = (Get-RtbRootPath $config.projectRoots.vibe);       Show = $Filter -in 'all','vibe' },
        @{ Name = 'Sandbox';    Status = 'Sandbox';    Path = (Get-RtbRootPath $config.projectRoots.sandbox);    Show = $Filter -in 'all','all' },
        @{ Name = 'Planning';   Status = 'Planning';   Path = (Get-RtbRootPath $config.projectRoots.planning);   Show = $Filter -in 'all','all' },
        @{ Name = 'Testing';    Status = 'Testing';    Path = (Get-RtbRootPath $config.projectRoots.testing);    Show = $Filter -in 'all','all' },
        @{ Name = 'Abandoned';  Status = 'Abandoned';  Path = (Get-RtbRootPath $config.projectRoots.abandoned);  Show = $Filter -in 'all','all' }
    )

    $results = [System.Collections.Generic.List[PSCustomObject]]::new()
    foreach ($cat in $categories) {
        if (-not $cat.Show -or -not $cat.Path -or -not (Test-Path $cat.Path)) { continue }
        $dirs = Get-ChildItem -Path $cat.Path -Directory -ErrorAction SilentlyContinue
        foreach ($d in $dirs) {
            $details = Get-ProjectDetails -ProjectPath $d.FullName -Status $cat.Status
            $results.Add($details)
        }
    }
    return $results.ToArray()
}

# ── Safety Guard Functions ──────────────────────────────────────────────────

function Confirm-RtbAction {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true, Position = 0)][string]$Message,
        [Parameter(ValueFromPipeline = $true)][string]$Answer
    )
    process {
        if (-not $PSBoundParameters.ContainsKey('Answer') -and [string]::IsNullOrEmpty($Answer)) {
            Write-Host "  $Message [y/N] " -ForegroundColor Yellow -NoNewline
            $Answer = Read-Host
        }
        if ([string]::IsNullOrWhiteSpace($Answer)) { return $false }
        $ans = $Answer.Trim().ToLower()
        return ($ans -eq 'y' -or $ans -eq 'yes')
    }
}

function Test-GitClean {
    param([Parameter(Mandatory = $true, Position = 0)][string]$ProjectPath)
    if (-not (Test-Path $ProjectPath)) { return $true }
    $gitDir = Join-Path $ProjectPath '.git'
    if (-not (Test-Path $gitDir)) { return $true }
    try {
        $status = git -C $ProjectPath status --porcelain 2>$null
        return (-not $status -or $status.Trim().Length -eq 0)
    } catch {
        return $true
    }
}
