<#
.SYNOPSIS
    Automated Release & Version Bumper for RTB.
.DESCRIPTION
    Bumps the SemVer version, synchronizes all codebase references,
    updates CHANGELOG.md, builds the core bundle, commits, and tags.
.PARAMETER Type
    Release increment: 'patch' (default), 'minor', 'major', or an explicit version (e.g., '0.5.2').
.PARAMETER Message
    Summary of changes for the changelog and release commit.
.PARAMETER NoCommit
    If specified, updates files without committing or creating a git tag.
.EXAMPLE
    .\scripts\release.ps1 -Type patch -Message "Fix TUI refresh key collision"
#>
[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$Type = 'patch',

    [Parameter(Position = 1)]
    [string]$Message = '',

    [switch]$NoCommit
)

$ErrorActionPreference = 'Stop'
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $repoRoot

$versionFile = Join-Path $repoRoot 'VERSION'
if (-not (Test-Path $versionFile)) {
    Write-Error "VERSION file not found at $versionFile"
}

$currentVersion = (Get-Content $versionFile -Raw).Trim().TrimStart('v')
Write-Host "══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  RTB Release & Version Orchestrator" -ForegroundColor Cyan
Write-Host "  Current Version: $currentVersion" -ForegroundColor Yellow
Write-Host "══════════════════════════════════════════════════════════════`n" -ForegroundColor Cyan

# 1. Calculate Target Version
$targetVersion = ''
if ($Type -match '^\d+\.\d+\.\d+$') {
    $targetVersion = $Type
} else {
    $parts = $currentVersion.Split('.')
    if ($parts.Count -lt 3) { $parts = @(0, 5, 0) }
    [int]$major = [int]$parts[0]
    [int]$minor = [int]$parts[1]
    [int]$patch = [int]$parts[2]

    switch ($Type.ToLower()) {
        'major' { $major++; $minor = 0; $patch = 0 }
        'minor' { $minor++; $patch = 0 }
        'patch' { $patch++ }
        default {
            Write-Error "Invalid version bump type: '$Type'. Use 'patch', 'minor', 'major', or 'X.Y.Z'."
        }
    }
    $targetVersion = "$major.$minor.$patch"
}

Write-Host "▶ Target Version: $targetVersion" -ForegroundColor Green

# 2. Write to canonical VERSION
Set-Content -Path $versionFile -Value $targetVersion -NoNewline -Encoding utf8
Write-Host "  ✓ Updated VERSION -> $targetVersion" -ForegroundColor Green

# 3. Synchronize all codebase version references
Write-Host "`n▶ Propagating version across codebase..." -ForegroundColor Cyan
node (Join-Path $PSScriptRoot 'sync-version.js')
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to synchronize version across codebase."
}

# 4. Update CHANGELOG.md
$changelogFile = Join-Path $repoRoot 'CHANGELOG.md'
$today = (Get-Date).ToString('yyyy-MM-dd')
if (Test-Path $changelogFile) {
    $changelog = Get-Content $changelogFile -Raw
    $tagHeader = "## [v$targetVersion]"
    if ($changelog -notmatch [regex]::Escape($tagHeader)) {
        Write-Host "`n▶ Updating CHANGELOG.md..." -ForegroundColor Cyan
        $entrySummary = $(if ($Message) { "- $Message" } else { "- Maintenance release and codebase updates." })
        $newSection = "## [v$targetVersion] - $today`n`n### Changed`n$entrySummary`n`n"
        # Insert before first release entry
        $idx = $changelog.IndexOf("`n## [v")
        if ($idx -ge 0) {
            $changelog = $changelog.Substring(0, $idx + 1) + $newSection + $changelog.Substring($idx + 1)
        } else {
            $changelog = $changelog + "`n`n" + $newSection
        }
        Set-Content -Path $changelogFile -Value $changelog -Encoding utf8
        Write-Host "  ✓ Appended [v$targetVersion] section to CHANGELOG.md" -ForegroundColor Green
    } else {
        Write-Host "`n  - CHANGELOG.md already contains section for [v$targetVersion]" -ForegroundColor DarkGray
    }
}

# 5. Build Core TypeScript Bundle
Write-Host "`n▶ Rebuilding core CLI bundle..." -ForegroundColor Cyan
npm --prefix core run build --silent
if ($LASTEXITCODE -ne 0) {
    Write-Error "Core TypeScript build failed!"
}
Write-Host "  ✓ Built core/dist/index.js" -ForegroundColor Green

# 5b. Verify npm Package Archive
Write-Host "`n▶ Validating npm package archive..." -ForegroundColor Cyan
Push-Location (Join-Path $repoRoot 'core')
try {
    npm pack --dry-run --silent
    if ($LASTEXITCODE -ne 0) {
        Write-Error "npm pack validation failed in core/!"
    }
    Write-Host "  ✓ Validated npm registry package archive (files: dist/)" -ForegroundColor Green
} finally {
    Pop-Location
}

# 6. Stage Standalone Release Assets
$releaseDir = Join-Path $repoRoot 'dist\release'
if (-not (Test-Path $releaseDir)) {
    New-Item -ItemType Directory -Path $releaseDir -Force | Out-Null
}

$coreBundle = Join-Path $repoRoot 'core\dist\index.js'
$standaloneJs = Join-Path $releaseDir 'rtb-cli.js'
Copy-Item $coreBundle $standaloneJs -Force
Copy-Item $versionFile (Join-Path $releaseDir 'VERSION') -Force
if (Test-Path (Join-Path $repoRoot 'logo.txt')) {
    Copy-Item (Join-Path $repoRoot 'logo.txt') (Join-Path $releaseDir 'logo.txt') -Force
}
if (Test-Path (Join-Path $repoRoot 'uninstall.ps1')) {
    Copy-Item (Join-Path $repoRoot 'uninstall.ps1') (Join-Path $releaseDir 'uninstall.ps1') -Force
}

$releaseZip = Join-Path $releaseDir 'rtb-cli.zip'
if (Test-Path $releaseZip) {
    Remove-Item $releaseZip -Force
}
Compress-Archive -Path (Join-Path $releaseDir '*') -DestinationPath $releaseZip -Force
Write-Host "  ✓ Staged standalone release assets in dist/release/" -ForegroundColor Green

# 7. Copy build to local distribution bin if installed
$userConfigBin = Join-Path $env:USERPROFILE '.config\rtb\bin'
if (Test-Path $userConfigBin) {
    Copy-Item (Join-Path $repoRoot 'core\dist\index.js') (Join-Path $userConfigBin 'rtb-cli.js') -Force
    if (Test-Path (Join-Path $userConfigBin 'rtb.js')) {
        Remove-Item (Join-Path $userConfigBin 'rtb.js') -Force
    }
    Copy-Item $versionFile (Join-Path $userConfigBin 'VERSION') -Force
    Write-Host "  ✓ Synchronized local ~/.config/rtb/bin launcher" -ForegroundColor Green
}

$dBin = 'D:\bin'
if ((Test-Path (Join-Path $dBin 'rtb.js')) -or (Test-Path (Join-Path $dBin 'rtb.ps1'))) {
    Copy-Item (Join-Path $repoRoot 'core\dist\index.js') (Join-Path $dBin 'rtb-cli.js') -Force
    if (Test-Path (Join-Path $dBin 'rtb.js')) {
        Remove-Item (Join-Path $dBin 'rtb.js') -Force
    }
    Copy-Item $versionFile (Join-Path $dBin 'VERSION') -Force
    Write-Host "  ✓ Synchronized local D:\bin launcher" -ForegroundColor Green
}

# 7. Git Commit & Tag
if ($NoCommit) {
    Write-Host "`n[!] -NoCommit specified. Files modified but not committed." -ForegroundColor Yellow
    return
}

Write-Host "`n▶ Preparing Git Commit & Tag..." -ForegroundColor Cyan
$commitMsg = $(if ($Message) { "chore(release): v$targetVersion - $Message" } else { "chore(release): v$targetVersion" })

git add -A
git commit -m $commitMsg
if ($LASTEXITCODE -ne 0) {
    Write-Warning "No new changes to commit or commit failed."
} else {
    Write-Host "  ✓ Committed: $commitMsg" -ForegroundColor Green
}

$tagName = "v$targetVersion"
$existingTag = git tag -l $tagName
if ($existingTag) {
    Write-Warning "Tag '$tagName' already exists locally."
} else {
    $tagMsg = if ($Message) { "Release $tagName - $Message" } else { "Release $tagName" }
    git tag -a $tagName -m $tagMsg
    Write-Host "  ✓ Tagged: $tagName" -ForegroundColor Green
}

Write-Host "`n══════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "  ✅ Release v$targetVersion Prepared Successfully!" -ForegroundColor Green
Write-Host "  To publish to GitHub and trigger CI/CD:" -ForegroundColor Yellow
Write-Host "    git push origin main --follow-tags" -ForegroundColor White
Write-Host "══════════════════════════════════════════════════════════════`n" -ForegroundColor Green
