# AI Agent Context Enrichment Unit Tests (Milestone M3)
# Compatible with Pester 3.4.0 and Pester 5+

Describe "New-RtbAgentContextFile" {
    BeforeAll {
        . (Join-Path $PSScriptRoot '..\src\utils\helpers.ps1')
        . (Join-Path $PSScriptRoot '..\src\commands\agent.ps1')
        $script:baseTemp = Join-Path ([System.IO.Path]::GetTempPath()) "agent_ctx_pester_$([Guid]::NewGuid().ToString('N'))"
        New-Item -ItemType Directory -Path $script:baseTemp -Force | Out-Null
    }

    AfterAll {
        if (Test-Path $script:baseTemp) {
            Remove-Item -Recurse -Force $script:baseTemp -ErrorAction SilentlyContinue
        }
    }

    It "returns null if project path does not exist" {
        $result = New-RtbAgentContextFile -ProjectPath (Join-Path $script:baseTemp "non_existent_folder")
        $result | Should -BeNullOrEmpty
    }

    It "creates .rtb_context.md with all required section headers" {
        $projDir = Join-Path $script:baseTemp "basic_project"
        New-Item -ItemType Directory -Path $projDir -Force | Out-Null

        $ctxPath = New-RtbAgentContextFile -ProjectPath $projDir -ProjectName "basic_project" -Status "Active" -Stack @("Node.js", "Express") -GitBranch "main" -ReadmePreview "# Basic Project"
        $ctxPath | Should -Not -BeNullOrEmpty
        Test-Path $ctxPath | Should -Be $true

        $content = Get-Content -Path $ctxPath -Raw
        $content | Should -Match "# RTB Agent Workspace Context: basic_project"
        $content | Should -Match "## Project Info"
        $content.Contains("- **Project Path**: $projDir") | Should -Be $true
        $content | Should -Match "- \*\*Status\*\*: Active"
        $content | Should -Match "- \*\*Detected Stack\*\*: Node\.js, Express"
        $content | Should -Match "- \*\*Git Branch\*\*: main"
        $content | Should -Match "- \*\*Generated At\*\*:"
        $content | Should -Match "## README Preview"
        $content | Should -Match "# Basic Project"
        $content | Should -Match "## Git Context"
        $content | Should -Match "### Last 10 Commits"
        $content | Should -Match "### Current Diff \(--stat HEAD\)"
        $content | Should -Match "## Dependencies"
    }

    It "provides correct fallbacks when project has no git, no stack, and no README" {
        $projDir = Join-Path $script:baseTemp "empty_project"
        New-Item -ItemType Directory -Path $projDir -Force | Out-Null

        $ctxPath = New-RtbAgentContextFile -ProjectPath $projDir
        $content = Get-Content -Path $ctxPath -Raw

        $content | Should -Match "- \*\*Detected Stack\*\*: Unknown"
        $content | Should -Match "- \*\*Git Branch\*\*: unknown"
        $content | Should -Match "\(no README\)"
        $content | Should -Match "\(not a git repository\)"
        $content | Should -Match "\(no recognised dependency manifest found\)"
    }

    It "extracts git commit log and diff stat when git repo exists" {
        $gitDir = Join-Path $script:baseTemp "git_project"
        New-Item -ItemType Directory -Path $gitDir -Force | Out-Null
        git -C $gitDir init --quiet 2>$null
        git -C $gitDir config user.name "RTB Context Test" 2>$null
        git -C $gitDir config user.email "test@rtb.local" 2>$null
        Set-Content -Path (Join-Path $gitDir "initial.txt") -Value "v1.0"
        git -C $gitDir add initial.txt 2>$null
        git -C $gitDir commit -m "feat: first commit message" --quiet 2>$null

        # Add uncommitted modification for diff stat
        Set-Content -Path (Join-Path $gitDir "initial.txt") -Value "v2.0 modified"

        $ctxPath = New-RtbAgentContextFile -ProjectPath $gitDir -ProjectName "git_project"
        $content = Get-Content -Path $ctxPath -Raw

        $content | Should -Match "feat: first commit message"
        $content | Should -Match "initial\.txt"
    }

    It "extracts dependencies and devDependencies from package.json" {
        $pkgDir = Join-Path $script:baseTemp "pkg_project"
        New-Item -ItemType Directory -Path $pkgDir -Force | Out-Null
        $pkgJson = @'
{
  "name": "sample-node-app",
  "dependencies": {
    "express": "^4.18.2",
    "cors": "^2.8.5"
  },
  "devDependencies": {
    "typescript": "^5.2.2",
    "jest": "^29.7.0"
  }
}
'@
        Set-Content -Path (Join-Path $pkgDir "package.json") -Value $pkgJson

        $ctxPath = New-RtbAgentContextFile -ProjectPath $pkgDir -ProjectName "pkg_project"
        $content = Get-Content -Path $ctxPath -Raw

        $content | Should -Match "\*\*package\.json deps:\*\* express, cors"
        $content | Should -Match "\*\*devDependencies:\*\* typescript, jest"
    }

    It "extracts dependencies from Cargo.toml" {
        $cargoDir = Join-Path $script:baseTemp "cargo_project"
        New-Item -ItemType Directory -Path $cargoDir -Force | Out-Null
        $cargoToml = @'
[package]
name = "sample-rust-app"
version = "0.1.0"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
serde = { version = "1.0", features = ["derive"] }
'@
        Set-Content -Path (Join-Path $cargoDir "Cargo.toml") -Value $cargoToml

        $ctxPath = New-RtbAgentContextFile -ProjectPath $cargoDir -ProjectName "cargo_project"
        $content = Get-Content -Path $ctxPath -Raw

        $content | Should -Match "\*\*Cargo\.toml crates:\*\* name, version, ratatui, crossterm, serde"
    }

    It "extracts dependencies from requirements.txt" {
        $pyDir = Join-Path $script:baseTemp "py_project"
        New-Item -ItemType Directory -Path $pyDir -Force | Out-Null
        $reqs = @'
# Core packages
flask>=2.0.0
requests==2.31.0
pytest>=7.0.0
'@
        Set-Content -Path (Join-Path $pyDir "requirements.txt") -Value $reqs

        $ctxPath = New-RtbAgentContextFile -ProjectPath $pyDir -ProjectName "py_project"
        $content = Get-Content -Path $ctxPath -Raw

        $content | Should -Match "\*\*requirements\.txt:\*\* flask>=2\.0\.0, requests==2\.31\.0, pytest>=7\.0\.0"
    }

    It "extracts dependencies from go.mod" {
        $goDir = Join-Path $script:baseTemp "go_project"
        New-Item -ItemType Directory -Path $goDir -Force | Out-Null
        $gomod = @'
module example.com/mygoapp

go 1.21

require (
	github.com/gin-gonic/gin v1.9.1
	github.com/google/uuid v1.4.0
)
'@
        Set-Content -Path (Join-Path $goDir "go.mod") -Value $gomod

        $ctxPath = New-RtbAgentContextFile -ProjectPath $goDir -ProjectName "go_project"
        $content = Get-Content -Path $ctxPath -Raw

        $content | Should -Match "\*\*go\.mod requires:\*\* github\.com/gin-gonic/gin v1\.9\.1, github\.com/google/uuid v1\.4\.0"
    }

    Context "Rust parity" {
        It "invokes native Rust binary for rtb agent --list, shell-init, and _goto-resolve" {
            $bin = Get-Command rtb -CommandType Application -ErrorAction SilentlyContinue
            if (-not $bin -and $env:_RTB_BIN -and (Test-Path $env:_RTB_BIN)) {
                $bin = Get-Item $env:_RTB_BIN
            }
            if (-not $bin) {
                $cargoTarget = Join-Path $PSScriptRoot "..\..\tui\target\debug\rtb.exe"
                if (Test-Path $cargoTarget) { $bin = Get-Item $cargoTarget }
            }
            if ($bin) {
                $binPath = if ($bin.Source) { $bin.Source } else { $bin.FullName }

                # 1. agent --list
                $listOut = (& $binPath agent --list) -join "`n"
                $listOut | Should -Match "Installed AI Agents"

                # 2. shell-init pwsh
                $shellOut = (& $binPath shell-init pwsh) -join "`n"
                $shellOut | Should -Match "function global:goto"

                # 3. _goto-resolve
                $projDir = Join-Path $script:baseTemp "parity_resolve_proj"
                New-Item -ItemType Directory -Path $projDir -Force | Out-Null
                $cfgFile = Join-Path $script:baseTemp "rtb.config.json"
                @{ version = "1.0.0"; projectRoots = @{ active = $script:baseTemp } } | ConvertTo-Json | Set-Content $cfgFile

                $resolved = (& $binPath --config $cfgFile _goto-resolve parity_resolve_proj) -join "`n"
                $resolved.Trim() | Should -Be $projDir
            }
        }
    }
}
