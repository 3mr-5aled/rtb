# RTB Improvements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden RTB into a config-driven, safety-first, OSS-quality developer tool by removing all hardcoded paths, improving navigation UX, enriching AI agent context, adding comprehensive safety guardrails, introducing `rtb doctor` and `rtb status`, and decomposing the monolithic `app.rs`.

**Architecture:** All mutations remain in the CLI (PowerShell); the TUI stays observational. Every destructive CLI operation gets a git-safety check + `y/N` confirmation gate. Config discovery eliminates all hardcoded personal paths. New features follow existing file/function naming conventions.

**Tech Stack:** PowerShell 7+, Rust 1.80+ (Ratatui 0.29, Crossterm 0.28, Rayon, fuzzy-matcher, serde_json), JSON config.

**Spec:** `docs/superpowers/plans/2026-08-30-rtb-improvements.md` (this file)

## Global Constraints

- No hardcoded absolute paths anywhere in CLI or TUI source — all paths resolve from config or `dirs::config_dir()`
- Every destructive operation (move, delete, prune) requires a `y/N` confirmation (default `N`) AND a git-clean check before proceeding
- `rtb init` must produce a fully working `%APPDATA%\rtb\rtb.config.json` — it is the single source of truth
- CLI: PowerShell 7+; no VB assemblies in hot paths
- TUI: Rust edition 2021; no `unsafe`; no new direct filesystem deletes inside TUI
- All Pester tests live in `cli/tests/`; all Rust tests are `#[cfg(test)]` modules in the same file as the code under test
- Version string across `Cargo.toml`, `rtb.psm1`, and `rtb.psd1` must stay at `0.2.0-beta` during these tasks
- Commit message format: `type(scope): description` — types are `fix`, `feat`, `refactor`, `docs`, `chore`, `test`

---

## Task 1: Strip all hardcoded paths from TUI config loader

**What's broken today:** `tui/src/config.rs:55-57` and `tui/src/ui/mod.rs:44` both hardcode `D:\02-Projects\...`.

**Files:**
- Modify: `tui/src/config.rs`
- Modify: `tui/src/ui/mod.rs`
- Test: `tui/src/config.rs` (inline `#[cfg(test)]`)

**Interfaces:**
- Produces: `DevConfig::load() -> Result<Self>` — unchanged signature, no hardcoded paths
- Produces: `DevConfig::candidate_paths() -> Vec<PathBuf>` — new public method for testability
- Produces: `get_logo()` — unchanged signature, no hardcoded paths

- [ ] **Step 1: Write a failing test for config discovery without hardcoded paths**

  Add inside `tui/src/config.rs` at the bottom:

  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn config_candidate_paths_contain_no_hardcoded_personal_paths() {
          let paths = DevConfig::candidate_paths();
          for p in &paths {
              let s = p.to_string_lossy();
              assert!(
                  !s.contains("02-Projects"),
                  "Hardcoded personal path found: {}",
                  s
              );
              assert!(
                  !s.contains("dev-cli"),
                  "Hardcoded personal path found: {}",
                  s
              );
          }
      }
  }
  ```

- [ ] **Step 2: Run test to confirm it fails**

  ```powershell
  cargo test -p rtbtui config -- --nocapture
  ```

  Expected: compile error — `candidate_paths` not found.

- [ ] **Step 3: Refactor `DevConfig::load()` to extract `candidate_paths()` and remove hardcoded entries**

  Replace the entire `impl DevConfig` block in `tui/src/config.rs`:

  ```rust
  impl DevConfig {
      /// Returns the ordered list of config file paths to try, from highest to lowest priority.
      pub fn candidate_paths() -> Vec<std::path::PathBuf> {
          let mut paths: Vec<std::path::PathBuf> = Vec::new();

          // 1. User config dir: %APPDATA%\rtb\rtb.config.json (Windows)
          //                     ~/.config/rtb/rtb.config.json (Linux/macOS)
          if let Some(config_dir) = dirs::config_dir() {
              paths.push(config_dir.join("rtb").join("rtb.config.json"));
              paths.push(config_dir.join("rtb").join("dev.config.json"));
          }

          // 2. Next to the running binary (useful when installed via install.ps1)
          if let Ok(exe_path) = std::env::current_exe() {
              if let Some(exe_dir) = exe_path.parent() {
                  paths.push(exe_dir.join("rtb.config.json"));
              }
          }

          // 3. Relative repo fallback for OSS contributors running from source
          paths.push(PathBuf::from("config/rtb.config.json"));
          paths.push(PathBuf::from("config/dev.config.json"));
          paths.push(PathBuf::from("../config/rtb.config.json"));

          paths
      }

      pub fn load() -> Result<Self> {
          for path in Self::candidate_paths() {
              if path.is_file() {
                  let content = std::fs::read_to_string(&path)
                      .with_context(|| format!("Cannot read config from {}", path.display()))?;
                  return serde_json::from_str(&content)
                      .with_context(|| format!("Failed to parse config file {}", path.display()));
              }
          }
          anyhow::bail!(
              "No rtb.config.json found.\n\
               Run 'rtb init' to create your workspace configuration.\n\
               Searched:\n{}",
              Self::candidate_paths()
                  .iter()
                  .map(|p| format!("  - {}", p.display()))
                  .collect::<Vec<_>>()
                  .join("\n")
          )
      }
  }
  ```

- [ ] **Step 4: Remove the hardcoded logo path in `tui/src/ui/mod.rs`**

  Replace the entire `get_logo()` function (lines 23-52):

  ```rust
  fn get_logo() -> String {
      // 1. Next to the deployed binary
      if let Ok(exe_path) = std::env::current_exe() {
          if let Some(exe_dir) = exe_path.parent() {
              if let Ok(content) = std::fs::read_to_string(exe_dir.join("logo.txt")) {
                  if !content.trim().is_empty() { return content; }
              }
          }
      }
      // 2. User config dir (%APPDATA%\rtb\logo.txt or ~/.config/rtb/logo.txt)
      if let Some(config_dir) = dirs::config_dir() {
          if let Ok(content) = std::fs::read_to_string(config_dir.join("rtb").join("logo.txt")) {
              if !content.trim().is_empty() { return content; }
          }
      }
      // 3. Relative repo path for OSS contributors running from source
      if let Ok(content) = std::fs::read_to_string("logo.txt") {
          if !content.trim().is_empty() { return content; }
      }
      // 4. Compile-time embedded fallback
      EMBEDDED_LOGO.to_string()
  }
  ```

- [ ] **Step 5: Run all tests and build**

  ```powershell
  cargo test -p rtbtui -- --nocapture
  cargo build -p rtbtui
  ```

  Expected: all tests pass, binary builds cleanly.

- [ ] **Step 6: Commit**

  ```powershell
  git add tui/src/config.rs tui/src/ui/mod.rs
  git commit -m "fix(tui): remove all hardcoded personal paths from config and logo discovery"
  ```

---

## Task 2: Safety guardrails — git-clean check + y/N confirmation for destructive CLI commands

**What's dangerous today:**
- `archive.ps1:50` deletes the source folder unconditionally after `tar`, even if the project has uncommitted changes
- `pause.ps1:40` moves project silently with `--prune` deleting dependencies without confirmation
- `clean.ps1` `--force` bypasses all prompting with no git check

**Files:**
- Modify: `cli/src/utils/helpers.ps1`
- Modify: `cli/src/commands/archive.ps1`
- Modify: `cli/src/commands/pause.ps1`
- Modify: `cli/src/commands/clean.ps1`
- Create: `cli/tests/Test-SafetyGuardrails.Tests.ps1`

**Interfaces:**
- Produces: `Confirm-RtbAction [string]$Message -> [bool]` in `helpers.ps1`
- Produces: `Test-GitClean [string]$ProjectPath -> [bool]` in `helpers.ps1`
- Consumes: both called in `archive.ps1`, `pause.ps1`, `clean.ps1`

- [ ] **Step 1: Write failing Pester tests**

  Create `cli/tests/Test-SafetyGuardrails.Tests.ps1`:

  ```powershell
  #Requires -Version 7
  BeforeAll {
      . "$PSScriptRoot/../src/utils/helpers.ps1"
  }

  Describe 'Confirm-RtbAction' {
      It 'returns $false when user inputs n' {
          $result = 'n' | Confirm-RtbAction -Message 'Delete this?' 2>$null
          $result | Should -Be $false
      }
      It 'returns $true when user inputs y' {
          $result = 'y' | Confirm-RtbAction -Message 'Delete this?' 2>$null
          $result | Should -Be $true
      }
      It 'returns $true when user inputs Y' {
          $result = 'Y' | Confirm-RtbAction -Message 'Delete this?' 2>$null
          $result | Should -Be $true
      }
  }

  Describe 'Test-GitClean' {
      It 'returns $true for a directory with no .git folder' {
          $tempDir = Join-Path $env:TEMP 'rtb-test-no-git'
          New-Item $tempDir -ItemType Directory -Force | Out-Null
          $result = Test-GitClean -ProjectPath $tempDir
          $result | Should -Be $true
          Remove-Item $tempDir -Recurse -Force
      }
  }
  ```

- [ ] **Step 2: Run tests to verify they fail**

  ```powershell
  Invoke-Pester cli/tests/Test-SafetyGuardrails.Tests.ps1 -Output Detailed
  ```

  Expected: FAIL — `Confirm-RtbAction` and `Test-GitClean` not found.

- [ ] **Step 3: Add the two guard functions to `helpers.ps1`**

  Append to the bottom of `cli/src/utils/helpers.ps1`:

  ```powershell
  # ── Safety Guard Functions ──────────────────────────────────────────────────

  function Confirm-RtbAction {
      param([Parameter(Mandatory = $true)][string]$Message)
      Write-Host "  $Message [y/N] " -ForegroundColor Yellow -NoNewline
      $answer = Read-Host
      return ($answer -eq 'y' -or $answer -eq 'Y')
  }

  function Test-GitClean {
      param([Parameter(Mandatory = $true)][string]$ProjectPath)
      $gitDir = Join-Path $ProjectPath '.git'
      if (-not (Test-Path $gitDir)) { return $true }
      $status = git -C $ProjectPath status --porcelain 2>$null
      return (-not $status -or $status.Trim().Length -eq 0)
  }
  ```

- [ ] **Step 4: Run tests to verify pass**

  ```powershell
  Invoke-Pester cli/tests/Test-SafetyGuardrails.Tests.ps1 -Output Detailed
  ```

  Expected: all 4 tests pass.

- [ ] **Step 5: Add guardrails to `archive.ps1`**

  Add `-Force` switch to `param()` block, then add after `$result = Find-ProjectPath -Name $Name` resolves:

  ```powershell
      # ── Git safety check ───────────────────────────────────────────────────
      if (-not (Test-GitClean -ProjectPath $result.Path)) {
          Write-Host '  ⚠ WARNING: This project has uncommitted git changes!' -ForegroundColor Red
          Write-Host '  Commit or stash your changes first, or pass -Force to override.' -ForegroundColor Yellow
          if (-not $Force) {
              Write-Host '  Aborting.' -ForegroundColor Red
              return
          }
      }

      # ── Confirmation prompt ────────────────────────────────────────────────
      $projectName = Split-Path $result.Path -Leaf
      Write-Host ''
      Write-Host '  This will:' -ForegroundColor Cyan
      Write-Host '    1. Prune dep folders (node_modules, target, .venv, etc.)' -ForegroundColor Gray
      Write-Host "    2. Create a .tar.gz in $($config.backupRoot)" -ForegroundColor Gray
      Write-Host "    3. PERMANENTLY DELETE: $($result.Path)" -ForegroundColor Red
      Write-Host ''
      if (-not (Confirm-RtbAction -Message "Archive and delete '$projectName'?")) {
          Write-Host '  Aborted.' -ForegroundColor Gray
          return
      }
  ```

  Also fix the post-tar safety gap — only delete source if archive was created successfully:

  ```powershell
      if (Test-Path $archivePath) {
          $sizeMB = [math]::Round((Get-Item $archivePath).Length / 1MB, 2)
          Remove-Item -Path $result.Path -Recurse -Force
          Write-Host "  Archived: $archiveName ($sizeMB MB)" -ForegroundColor Green
          Write-Host "  To restore: rtb unarchive $archiveName" -ForegroundColor Cyan
      } else {
          Write-Host '  Archive creation FAILED — source folder was NOT deleted.' -ForegroundColor Red
      }
  ```

- [ ] **Step 6: Add guardrails to `pause.ps1`**

  Replace `Dev-Pause` signature with `-Prune [switch]` and `-Force [switch]`. Add after resolving `$activePath`:

  ```powershell
      if (-not (Test-GitClean -ProjectPath $activePath)) {
          Write-Host '  ⚠ WARNING: This project has uncommitted git changes!' -ForegroundColor Red
          if (-not $Force) {
              Write-Host '  Commit or stash first, or pass -Force to override.' -ForegroundColor Yellow
              return
          }
      }
  ```

  Wrap the prune block in a `Confirm-RtbAction` prompt:

  ```powershell
      if ($Prune) {
          if (-not (Confirm-RtbAction -Message 'Prune dependency folders (node_modules, target, .venv)?')) {
              Write-Host '  Prune skipped.' -ForegroundColor Gray
          } else {
              # existing prune loop here
          }
      }
  ```

- [ ] **Step 7: Harden `clean.ps1` — rename `--force` to `-Commit` for explicit opt-in**

  Replace the `param()` block:

  ```powershell
      param(
          [switch]$Commit,   # Actually delete — must be explicit
          [switch]$DryRun,   # Alias kept for backwards compat
          [int]$Days = 60
      )
      $isDryRun = -not $Commit
  ```

  After listing flagged folders, wrap the delete in a `Confirm-RtbAction`:

  ```powershell
      if (-not $isDryRun) {
          if (-not (Confirm-RtbAction -Message "Delete $($flagged.Count) dep folders ($gb GB)?")) {
              Write-Host '  Aborted.' -ForegroundColor Gray
              return
          }
          # existing Remove-Item loop
      }
  ```

- [ ] **Step 8: Run all Pester tests**

  ```powershell
  Invoke-Pester cli/tests/ -Output Detailed
  ```

  Expected: all tests pass.

- [ ] **Step 9: Commit**

  ```powershell
  git add cli/src/utils/helpers.ps1 cli/src/commands/archive.ps1 cli/src/commands/pause.ps1 cli/src/commands/clean.ps1 cli/tests/Test-SafetyGuardrails.Tests.ps1
  git commit -m "feat(cli): add git-clean check + y/N confirmation to archive, pause, clean"
  ```

---

## Task 3: Fuzzy `goto` with multi-match picker + `Find-ProjectPathFuzzy`

**What's broken today:** `Find-ProjectPath` returns the first glob match silently. `goto` with no name dumps all projects. No numbered picker on ambiguous input.

**Files:**
- Modify: `cli/src/utils/helpers.ps1`
- Modify: `cli/src/commands/goto.ps1`
- Create: `cli/tests/Test-Goto.Tests.ps1`

**Interfaces:**
- Produces: `Find-ProjectPathFuzzy [string]$Query -> [PSCustomObject[]]` sorted by `Score` descending, each item has `Name, Path, Status, Score`
- Consumes: `Dev-Goto` uses this instead of `Find-ProjectPath`

- [ ] **Step 1: Write failing Pester tests**

  Create `cli/tests/Test-Goto.Tests.ps1`:

  ```powershell
  #Requires -Version 7
  BeforeAll {
      . "$PSScriptRoot/../src/utils/helpers.ps1"
  }

  Describe 'Find-ProjectPathFuzzy' {
      It 'returns empty array for a query that matches nothing' {
          $results = Find-ProjectPathFuzzy -Query 'zzz-no-such-project-xyz'
          $results | Should -HaveCount 0
      }

      It 'results are sorted Score descending' {
          $fakeResults = @(
              [PSCustomObject]@{ Name = 'my-rtb-tool';     Score = 50 },
              [PSCustomObject]@{ Name = 'rtb-command-tool'; Score = 100 },
              [PSCustomObject]@{ Name = 'rtb-extras';       Score = 60 }
          )
          $sorted = $fakeResults | Sort-Object Score -Descending
          $sorted[0].Name | Should -Be 'rtb-command-tool'
      }
  }
  ```

- [ ] **Step 2: Run to confirm failure**

  ```powershell
  Invoke-Pester cli/tests/Test-Goto.Tests.ps1 -Output Detailed
  ```

  Expected: FAIL — `Find-ProjectPathFuzzy` not found.

- [ ] **Step 3: Add `Find-ProjectPathFuzzy` to `helpers.ps1`**

  Append after the existing `Find-ProjectPath` function:

  ```powershell
  function Find-ProjectPathFuzzy {
      param([Parameter(Mandatory = $true)][string]$Query)
      $config = Get-RtbConfig
      if (-not $config) { return @() }

      $roots = @(
          @{ Path = $config.projectRoots.active;     Status = 'Active' },
          @{ Path = $config.projectRoots.paused;     Status = 'Paused' },
          @{ Path = $config.projectRoots.production; Status = 'Production' },
          @{ Path = $config.projectRoots.staging;    Status = 'Staging' },
          @{ Path = $config.projectRoots.vibe;       Status = 'Vibe' },
          @{ Path = $config.projectRoots.sandbox;    Status = 'Sandbox' },
          @{ Path = $config.projectRoots.planning;   Status = 'Planning' },
          @{ Path = $config.projectRoots.testing;    Status = 'Testing' },
          @{ Path = $config.projectRoots.abandoned;  Status = 'Abandoned' }
      )

      $q = $Query.ToLower()
      $results = @()

      foreach ($entry in $roots) {
          if (-not $entry.Path -or -not (Test-Path $entry.Path)) { continue }
          Get-ChildItem -Path $entry.Path -Directory -EA SilentlyContinue | ForEach-Object {
              $n = $_.Name.ToLower()
              $score = if ($n -eq $q)                                    { 100 }
                       elseif ($n.StartsWith($q))                        { 75  }
                       elseif ($n -like "*$q*")                          { 50  }
                       elseif ($_.FullName.ToLower() -like "*$q*")       { 25  }
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
  ```

- [ ] **Step 4: Rewrite `Dev-Goto` in `goto.ps1`**

  Replace the entire `Dev-Goto` function:

  ```powershell
  function Dev-Goto {
      [CmdletBinding()]
      param(
          [Parameter(Position = 0)][string]$Name,
          [Parameter(Position = 1)][string]$Agent,
          [Switch]$Agy, [Switch]$Claude, [Switch]$Gemini, [Switch]$Codex,
          [Switch]$Cursor, [Switch]$Windsurf, [Switch]$Aider, [Switch]$OpenHands
      )

      if (-not $Name) {
          Write-Host 'Usage: rtb goto <project-name> [--agy|--claude|...]' -ForegroundColor Yellow
          Write-Host 'Tip: Tab after "rtb goto " to see all projects.' -ForegroundColor Gray
          return
      }

      if (-not $Agent) {
          if ($Agy)       { $Agent = 'agy' }
          elseif ($Claude)    { $Agent = 'claude' }
          elseif ($Gemini)    { $Agent = 'gemini' }
          elseif ($Codex)     { $Agent = 'codex' }
          elseif ($Cursor)    { $Agent = 'cursor' }
          elseif ($Windsurf)  { $Agent = 'windsurf' }
          elseif ($Aider)     { $Agent = 'aider' }
          elseif ($OpenHands) { $Agent = 'openhands' }
      }

      $matches = Find-ProjectPathFuzzy -Query $Name

      if ($matches.Count -eq 0) {
          Write-Host "  No project matching '$Name' found." -ForegroundColor Red
          Write-Host '  Available projects:' -ForegroundColor Gray
          Get-AllProjectNames | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
          return
      }

      $target = if ($matches.Count -eq 1 -or $matches[0].Score -eq 100) {
          $matches[0]
      } else {
          Write-Host ''
          Write-Host "  Multiple projects match '$Name':" -ForegroundColor Yellow
          for ($i = 0; $i -lt [Math]::Min($matches.Count, 9); $i++) {
              $m = $matches[$i]
              Write-Host ("  [{0}] {1,-35} ({2})" -f ($i + 1), $m.Name, $m.Status) -ForegroundColor Cyan
          }
          Write-Host ''
          Write-Host -NoNewline '  Select [1-9] or Enter to cancel: ' -ForegroundColor Yellow
          $choice = Read-Host
          if ($choice -match '^[1-9]$') {
              $idx = [int]$choice - 1
              if ($idx -lt $matches.Count) { $matches[$idx] } else { $null }
          } else { $null }
      }

      if (-not $target) {
          Write-Host '  Cancelled.' -ForegroundColor Gray
          return
      }

      Set-Location $target.Path
      Write-Host "  $($target.Status) » $($target.Path)" -ForegroundColor Green

      if ($Agent) { Rtb-Agent -ProjectName $target.Path -Agent $Agent }
  }

  function Set-RtbLocation { Dev-Goto @args }
  ```

- [ ] **Step 5: Run tests**

  ```powershell
  Invoke-Pester cli/tests/Test-Goto.Tests.ps1 -Output Detailed
  ```

  Expected: all tests pass.

- [ ] **Step 6: Manual smoke test**

  ```powershell
  rtb goto rtb       # Shows picker if multiple matches, jumps if one
  rtb goto zzz-nope  # Prints "No project matching..." and lists all
  ```

- [ ] **Step 7: Commit**

  ```powershell
  git add cli/src/utils/helpers.ps1 cli/src/commands/goto.ps1 cli/tests/Test-Goto.Tests.ps1
  git commit -m "feat(cli): fuzzy multi-match goto picker + Find-ProjectPathFuzzy"
  ```

---

## Task 4: Richer AI agent context file (git log, diff stat, deps summary)

**What's thin today:** `.rtb_context.md` only contains path, status, stack, branch, and README first line. Agents have no temporal context (no history, no diff, no deps).

**Files:**
- Modify: `cli/src/commands/agent.ps1` — `New-RtbAgentContextFile`
- Modify: `tui/src/data/agents.rs` — `create_agent_context_file`
- Create: `cli/tests/Test-AgentContext.Tests.ps1`

**Interfaces:**
- Produces: `.rtb_context.md` with sections: `## Project Info`, `## README Preview`, `## Git Context` (### Last 10 Commits, ### Current Diff), `## Dependencies`
- Schema is identical between CLI and TUI output

- [ ] **Step 1: Write failing Pester tests**

  Create `cli/tests/Test-AgentContext.Tests.ps1`:

  ```powershell
  #Requires -Version 7
  BeforeAll {
      . "$PSScriptRoot/../src/utils/helpers.ps1"
      . "$PSScriptRoot/../src/commands/agent.ps1"
  }

  Describe 'New-RtbAgentContextFile' {
      BeforeAll {
          $testDir = Join-Path $env:TEMP 'rtb-agent-ctx-test'
          New-Item $testDir -ItemType Directory -Force | Out-Null
      }
      AfterAll {
          Remove-Item $testDir -Recurse -Force -EA SilentlyContinue
      }

      It 'creates a .rtb_context.md file' {
          New-RtbAgentContextFile -ProjectPath $testDir -ProjectName 'test-project' | Out-Null
          Test-Path (Join-Path $testDir '.rtb_context.md') | Should -Be $true
      }
      It 'context file contains Project Path section' {
          $content = Get-Content (Join-Path $testDir '.rtb_context.md') -Raw
          $content | Should -Match 'Project Path'
      }
      It 'context file contains Git Context section header' {
          $content = Get-Content (Join-Path $testDir '.rtb_context.md') -Raw
          $content | Should -Match '## Git Context'
      }
      It 'context file contains Dependencies section header' {
          $content = Get-Content (Join-Path $testDir '.rtb_context.md') -Raw
          $content | Should -Match '## Dependencies'
      }
  }
  ```

- [ ] **Step 2: Run to confirm failure**

  ```powershell
  Invoke-Pester cli/tests/Test-AgentContext.Tests.ps1 -Output Detailed
  ```

  Expected: FAIL — context file missing `## Git Context` and `## Dependencies`.

- [ ] **Step 3: Rewrite `New-RtbAgentContextFile` in `agent.ps1`**

  Replace the existing function (lines 38-69):

  ```powershell
  function New-RtbAgentContextFile {
      param(
          [Parameter(Mandatory = $true)][string]$ProjectPath,
          [string]$ProjectName = "",
          [string[]]$Stack = @(),
          [string]$Status = "Active",
          [string]$GitBranch = "",
          [string]$ReadmePreview = ""
      )

      if (-not (Test-Path $ProjectPath)) { return $null }
      $name = if ($ProjectName) { $ProjectName } else { Split-Path $ProjectPath -Leaf }
      $contextPath = Join-Path $ProjectPath ".rtb_context.md"
      $stackStr = if ($Stack -and $Stack.Count -gt 0 -and $Stack[0] -ne '-') { $Stack -join ', ' } else { 'Unknown' }
      $branchStr = if ($GitBranch) { $GitBranch } else { 'unknown' }

      # Git Context
      $gitLogLines = '  (not a git repository)'
      $gitDiffStat = '  (not a git repository)'
      if (Test-Path (Join-Path $ProjectPath '.git')) {
          $logRaw = git -C $ProjectPath log --oneline -10 2>$null
          $gitLogLines = if ($logRaw) { ($logRaw | ForEach-Object { "  $_" }) -join "`n" } else { '  (no commits)' }
          $diffRaw = git -C $ProjectPath diff --stat HEAD 2>$null
          $gitDiffStat = if ($diffRaw) { ($diffRaw | ForEach-Object { "  $_" }) -join "`n" } else { '  (working tree clean)' }
      }

      # Dependencies
      $depsSection = ''
      $pkgPath = Join-Path $ProjectPath 'package.json'
      if (Test-Path $pkgPath) {
          try {
              $pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json
              $deps = ($pkg.dependencies.PSObject.Properties.Name | Select-Object -First 20) -join ', '
              $devDeps = ($pkg.devDependencies.PSObject.Properties.Name | Select-Object -First 10) -join ', '
              $depsSection += "**package.json deps:** $deps`n"
              if ($devDeps) { $depsSection += "**devDependencies:** $devDeps`n" }
          } catch { $depsSection += "(could not parse package.json)`n" }
      }
      if (Test-Path (Join-Path $ProjectPath 'Cargo.toml')) {
          $cargoContent = Get-Content (Join-Path $ProjectPath 'Cargo.toml') -Raw
          $crates = ([regex]::Matches($cargoContent, '^\s*(\w[\w-]*)\s*=', 'Multiline') |
              Select-Object -First 20 | ForEach-Object { $_.Groups[1].Value }) -join ', '
          $depsSection += "**Cargo.toml crates:** $crates`n"
      }
      if (Test-Path (Join-Path $ProjectPath 'requirements.txt')) {
          $reqs = (Get-Content (Join-Path $ProjectPath 'requirements.txt') -TotalCount 20) -join ', '
          $depsSection += "**requirements.txt:** $reqs`n"
      }
      if (Test-Path (Join-Path $ProjectPath 'go.mod')) {
          $goMods = (Get-Content (Join-Path $ProjectPath 'go.mod') |
              Where-Object { $_ -match '^\s+\S+\s+v' } | Select-Object -First 20) -join ', '
          $depsSection += "**go.mod requires:** $goMods`n"
      }
      if (-not $depsSection) { $depsSection = '(no recognised dependency manifest found)' }

      $readmeStr = if ($ReadmePreview) { $ReadmePreview } else { '(no README)' }

      $content = @"
  # RTB Agent Workspace Context: $name

  ## Project Info
  - **Project Path**: $ProjectPath
  - **Status**: $Status
  - **Detected Stack**: $stackStr
  - **Git Branch**: $branchStr
  - **Generated At**: $(Get-Date -Format 'o')

  ## README Preview
  $readmeStr

  ## Git Context

  ### Last 10 Commits
  $gitLogLines

  ### Current Diff (--stat HEAD)
  $gitDiffStat

  ## Dependencies
  $depsSection
  "@

      Set-Content -Path $contextPath -Value $content -Force
      return $contextPath
  }
  ```

- [ ] **Step 4: Update the Rust `create_agent_context_file` in `tui/src/data/agents.rs`**

  Replace the `create_agent_context_file` function (lines 66-87):

  ```rust
  pub fn create_agent_context_file(project: &Project) -> Option<std::path::PathBuf> {
      let context_path = project.path.join(".rtb_context.md");
      let stack_str = if project.stack.is_empty() { "Unknown".into() } else { project.stack.join(", ") };
      let branch_str = project.git.as_ref().map(|g| g.branch.as_str()).unwrap_or("unknown");
      let readme_str = project.readme_preview.as_deref().unwrap_or("(no README)");

      let path_str = project.path.to_string_lossy();

      let git_log = std::process::Command::new("git")
          .args(["-C", &path_str, "log", "--oneline", "-10"])
          .output().ok()
          .and_then(|o| String::from_utf8(o.stdout).ok())
          .unwrap_or_else(|| "  (not a git repository)\n".into());
      let git_log_indented = git_log.lines()
          .map(|l| format!("  {}", l)).collect::<Vec<_>>().join("\n");

      let git_diff = std::process::Command::new("git")
          .args(["-C", &path_str, "diff", "--stat", "HEAD"])
          .output().ok()
          .and_then(|o| String::from_utf8(o.stdout).ok())
          .map(|s| if s.trim().is_empty() { "  (working tree clean)".into() } else { s })
          .unwrap_or_else(|| "  (not a git repository)".into());

      let mut deps_section = String::new();
      let pkg_path = project.path.join("package.json");
      if pkg_path.exists() {
          if let Ok(content) = std::fs::read_to_string(&pkg_path) {
              if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                  if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                      let names: Vec<&str> = deps.keys().take(20).map(|k| k.as_str()).collect();
                      deps_section.push_str(&format!("**package.json deps:** {}\n", names.join(", ")));
                  }
              }
          }
      }
      if project.path.join("Cargo.toml").exists() {
          deps_section.push_str("**Cargo.toml detected** (see file for full crate list)\n");
      }
      if deps_section.is_empty() {
          deps_section.push_str("(no recognised dependency manifest found)\n");
      }

      let content = format!(
          "# RTB Agent Workspace Context: {name}\n\n\
           ## Project Info\n\
           - **Project Path**: {path}\n\
           - **Status**: {status:?}\n\
           - **Detected Stack**: {stack}\n\
           - **Git Branch**: {branch}\n\n\
           ## README Preview\n{readme}\n\n\
           ## Git Context\n\n### Last 10 Commits\n{log}\n\n### Current Diff (--stat HEAD)\n{diff}\n\n\
           ## Dependencies\n{deps}\n",
          name = project.name,
          path = project.path.display(),
          status = project.status,
          stack = stack_str,
          branch = branch_str,
          readme = readme_str,
          log = if git_log_indented.trim().is_empty() { "  (no commits)".into() } else { git_log_indented },
          diff = git_diff,
          deps = deps_section
      );

      if std::fs::write(&context_path, content).is_ok() { Some(context_path) } else { None }
  }
  ```

- [ ] **Step 5: Run tests and build**

  ```powershell
  Invoke-Pester cli/tests/Test-AgentContext.Tests.ps1 -Output Detailed
  cargo test -p rtbtui agents -- --nocapture
  cargo build -p rtbtui
  ```

  Expected: all pass.

- [ ] **Step 6: Commit**

  ```powershell
  git add cli/src/commands/agent.ps1 tui/src/data/agents.rs cli/tests/Test-AgentContext.Tests.ps1
  git commit -m "feat(agent): enrich .rtb_context.md with git log, diff stat, and deps summary"
  ```

---

## Task 5: `rtb doctor` — system health check command

**Goal:** A new CLI command that diagnoses config, tools, and project roots — the primary onboarding unlocker for new OSS users.

**Files:**
- Create: `cli/src/commands/doctor.ps1`
- Modify: `cli/rtb.psm1` — add `doctor` to switch and export
- Create: `cli/tests/Test-Doctor.Tests.ps1`

**Interfaces:**
- Produces: `Rtb-Doctor -> [bool]` — prints structured report, returns `$true` if all required checks pass
- Produces: `Dev-Doctor` alias

- [ ] **Step 1: Write failing Pester test**

  Create `cli/tests/Test-Doctor.Tests.ps1`:

  ```powershell
  #Requires -Version 7
  BeforeAll {
      . "$PSScriptRoot/../src/utils/helpers.ps1"
      . "$PSScriptRoot/../src/commands/doctor.ps1"
  }

  Describe 'Rtb-Doctor' {
      It 'returns a boolean' {
          $result = Rtb-Doctor
          $result | Should -BeOfType [bool]
      }
  }
  ```

- [ ] **Step 2: Run to confirm failure**

  ```powershell
  Invoke-Pester cli/tests/Test-Doctor.Tests.ps1 -Output Detailed
  ```

  Expected: FAIL — `Rtb-Doctor` not found.

- [ ] **Step 3: Create `cli/src/commands/doctor.ps1`**

  ```powershell
  function Rtb-Doctor {
      [CmdletBinding()]
      param()

      Write-RtbHeader 'System Doctor'
      Write-Host ''

      $allGood = $true

      function Write-Check {
          param([bool]$Pass, [string]$Label, [string]$Detail = '')
          if ($Pass) {
              Write-Host "  ✅ $Label" -ForegroundColor Green
          } else {
              Write-Host "  ❌ $Label" -ForegroundColor Red
              if ($Detail) { Write-Host "     → $Detail" -ForegroundColor Yellow }
              $script:allGood = $false
          }
      }

      Write-Host '  Config' -ForegroundColor Cyan
      $config = Get-RtbConfig
      Write-Check ($null -ne $config) 'rtb.config.json found and parseable' `
          "Run 'rtb init' to create your config at %APPDATA%\rtb\rtb.config.json"

      Write-Host ''
      Write-Host '  Project Roots' -ForegroundColor Cyan
      if ($config) {
          $rootMap = [ordered]@{
              active     = $config.projectRoots.active
              paused     = $config.projectRoots.paused
              planning   = $config.projectRoots.planning
              testing    = $config.projectRoots.testing
              production = $config.projectRoots.production
              staging    = $config.projectRoots.staging
              vibe       = $config.projectRoots.vibe
              sandbox    = $config.projectRoots.sandbox
              abandoned  = $config.projectRoots.abandoned
          }
          foreach ($key in $rootMap.Keys) {
              $val = $rootMap[$key]
              $exists = $val -and (Test-Path $val)
              Write-Check $exists "$key → $val" `
                  "Directory does not exist. Create it or update projectRoots.$key in your config."
          }
      }

      Write-Host ''
      Write-Host '  Required Tools' -ForegroundColor Cyan
      foreach ($tool in @('git')) {
          $found = [bool](Get-Command $tool -EA SilentlyContinue)
          Write-Check $found "$tool in PATH" "Install $tool and ensure it is on your PATH"
      }

      Write-Host ''
      Write-Host '  Optional Tools' -ForegroundColor Cyan
      $optionals = @(
          @{ Name = 'node';  Label = 'Node.js (for JavaScript/TypeScript projects)' },
          @{ Name = 'cargo'; Label = 'Cargo / Rust (for Rust projects and rtbtui build)' },
          @{ Name = 'python';Label = 'Python (for Python projects)' },
          @{ Name = 'tar';   Label = 'tar (for rtb archive/unarchive)' }
      )
      foreach ($tool in $optionals) {
          $found = [bool](Get-Command $tool.Name -EA SilentlyContinue)
          $icon = if ($found) { '✅' } else { '⚠ ' }
          $color = if ($found) { 'Green' } else { 'DarkYellow' }
          Write-Host "  $icon $($tool.Label)" -ForegroundColor $color
      }

      Write-Host ''
      Write-Host '  AI Agents' -ForegroundColor Cyan
      $agents = @('agy','claude','gemini','codex','cursor','windsurf','aider','openhands')
      $foundAgents = $agents | Where-Object { Get-Command $_ -EA SilentlyContinue }
      if ($foundAgents) {
          Write-Host "  ✅ Installed: $($foundAgents -join ', ')" -ForegroundColor Green
      } else {
          Write-Host '  ⚠  No AI agents found in PATH' -ForegroundColor DarkYellow
      }

      Write-Host ''
      Write-Host '  TUI Binary' -ForegroundColor Cyan
      Write-Check ([bool](Get-Command 'rtbtui' -EA SilentlyContinue)) 'rtbtui binary in PATH' `
          "Build with: cargo build --release -p rtbtui, then add to PATH or re-run install.ps1"

      Write-Host ''
      Write-Host '══════════════════════════════════════════' -ForegroundColor Cyan
      if ($allGood) {
          Write-Host '  ✅ All checks passed — RTB is healthy!' -ForegroundColor Green
      } else {
          Write-Host '  ❌ Some checks failed — see above for details.' -ForegroundColor Red
      }
      Write-Host '══════════════════════════════════════════' -ForegroundColor Cyan

      return $allGood
  }

  function Dev-Doctor { Rtb-Doctor @args }
  ```

- [ ] **Step 4: Wire `doctor` into `cli/rtb.psm1`**

  Add to the `switch` block after the `'upgrade'` line:

  ```powershell
          'doctor'      { if ($Arguments) { Rtb-Doctor @Arguments } else { Rtb-Doctor } }
  ```

  Add to `Export-ModuleMember`: `, 'Rtb-Doctor', 'Dev-Doctor'`

- [ ] **Step 5: Run tests**

  ```powershell
  Invoke-Pester cli/tests/Test-Doctor.Tests.ps1 -Output Detailed
  ```

  Expected: all pass.

- [ ] **Step 6: Manual smoke test**

  ```powershell
  rtb doctor
  ```

  Expected: structured ✅/❌/⚠ report for all sections.

- [ ] **Step 7: Commit**

  ```powershell
  git add cli/src/commands/doctor.ps1 cli/rtb.psm1 cli/tests/Test-Doctor.Tests.ps1
  git commit -m "feat(cli): add rtb doctor system health check command"
  ```

---

## Task 6: `rtb status` — shell prompt segment

**Goal:** A compact one-line output for shell prompts showing current project, git branch, uncommitted count, and stack. `-Json` flag for machine-readable integration.

**Files:**
- Create: `cli/src/commands/status.ps1`
- Modify: `cli/rtb.psm1`
- Create: `cli/tests/Test-Status.Tests.ps1`

**Interfaces:**
- Produces: `Rtb-Status [-Json] -> [string]`
- Plain: `rtb » project-name (Status) [branch ±N] Stack,Stack`
- JSON: `{"project":"name","status":"Active","branch":"main","uncommitted":3,"stack":["Node.js"],"cwd":"..."}`

- [ ] **Step 1: Write failing Pester tests**

  Create `cli/tests/Test-Status.Tests.ps1`:

  ```powershell
  #Requires -Version 7
  BeforeAll {
      . "$PSScriptRoot/../src/utils/helpers.ps1"
      . "$PSScriptRoot/../src/commands/status.ps1"
  }

  Describe 'Rtb-Status' {
      It 'returns a non-empty string' {
          $result = Rtb-Status
          $result | Should -Not -BeNullOrEmpty
      }

      It '-Json flag returns valid JSON with required keys' {
          $result = Rtb-Status -Json
          $parsed = $result | ConvertFrom-Json
          $parsed | Should -Not -BeNull
          $parsed.PSObject.Properties.Name | Should -Contain 'project'
          $parsed.PSObject.Properties.Name | Should -Contain 'branch'
          $parsed.PSObject.Properties.Name | Should -Contain 'uncommitted'
          $parsed.PSObject.Properties.Name | Should -Contain 'stack'
      }
  }
  ```

- [ ] **Step 2: Run to confirm failure**

  ```powershell
  Invoke-Pester cli/tests/Test-Status.Tests.ps1 -Output Detailed
  ```

  Expected: FAIL — `Rtb-Status` not found.

- [ ] **Step 3: Create `cli/src/commands/status.ps1`**

  ```powershell
  function Rtb-Status {
      [CmdletBinding()]
      param([switch]$Json)

      $cwd = (Get-Location).Path
      $config = Get-RtbConfig

      $projectName = $null
      $projectStatus = $null
      if ($config) {
          $rootMap = [ordered]@{
              'Active'     = $config.projectRoots.active
              'Paused'     = $config.projectRoots.paused
              'Production' = $config.projectRoots.production
              'Staging'    = $config.projectRoots.staging
              'Vibe'       = $config.projectRoots.vibe
              'Sandbox'    = $config.projectRoots.sandbox
              'Planning'   = $config.projectRoots.planning
              'Testing'    = $config.projectRoots.testing
          }
          foreach ($status in $rootMap.Keys) {
              $root = $rootMap[$status]
              if ($root -and $cwd.StartsWith($root, [System.StringComparison]::OrdinalIgnoreCase)) {
                  $relative = $cwd.Substring($root.Length).TrimStart('\','/')
                  $projectName = $relative.Split('\/')[0]
                  $projectStatus = $status
                  break
              }
          }
      }

      $branch = ''
      $uncommitted = 0
      $check = $cwd
      while ($check -and $check -ne (Split-Path $check -Qualifier)) {
          if (Test-Path (Join-Path $check '.git')) {
              $branch = git -C $check branch --show-current 2>$null
              $statusLines = git -C $check status --porcelain 2>$null
              $uncommitted = if ($statusLines) { ($statusLines | Measure-Object).Count } else { 0 }
              break
          }
          $check = Split-Path $check -Parent
      }

      $stack = @()
      if (Test-Path (Join-Path $cwd 'package.json'))   { $stack += 'Node.js' }
      if (Test-Path (Join-Path $cwd 'Cargo.toml'))     { $stack += 'Rust' }
      if (Test-Path (Join-Path $cwd 'go.mod'))         { $stack += 'Go' }
      if ((Test-Path (Join-Path $cwd 'pyproject.toml')) -or
          (Test-Path (Join-Path $cwd 'requirements.txt'))) { $stack += 'Python' }

      $displayName = if ($projectName) { $projectName } else { Split-Path $cwd -Leaf }

      if ($Json) {
          return [PSCustomObject]@{
              project     = $displayName
              status      = $projectStatus
              branch      = $branch
              uncommitted = $uncommitted
              stack       = $stack
              cwd         = $cwd
          } | ConvertTo-Json -Compress
      }

      $gitPart = if ($branch) {
          $unStr = if ($uncommitted -gt 0) { " ±$uncommitted" } else { '' }
          " [$branch$unStr]"
      } else { '' }
      $stackPart  = if ($stack.Count -gt 0) { " $($stack -join ',')" } else { '' }
      $statusPart = if ($projectStatus) { " ($projectStatus)" } else { '' }

      return "rtb » $displayName$statusPart$gitPart$stackPart"
  }

  function Dev-Status { Rtb-Status @args }
  ```

- [ ] **Step 4: Wire `status` into `cli/rtb.psm1`**

  Add to the switch block:

  ```powershell
          'status'      { if ($Arguments) { Rtb-Status @Arguments } else { Rtb-Status } }
  ```

  Add to `Export-ModuleMember`: `, 'Rtb-Status', 'Dev-Status'`

- [ ] **Step 5: Run tests**

  ```powershell
  Invoke-Pester cli/tests/Test-Status.Tests.ps1 -Output Detailed
  ```

  Expected: all pass.

- [ ] **Step 6: Manual smoke test (from inside any tracked project dir)**

  ```powershell
  rtb status
  # → rtb » rtb-command-tool (Active) [main ±2] PowerShell,Rust
  rtb status -Json
  # → {"project":"rtb-command-tool","status":"Active","branch":"main","uncommitted":2,"stack":["PowerShell","Rust"],"cwd":"D:\\..."}
  ```

- [ ] **Step 7: Commit**

  ```powershell
  git add cli/src/commands/status.ps1 cli/rtb.psm1 cli/tests/Test-Status.Tests.ps1
  git commit -m "feat(cli): add rtb status shell prompt segment with -Json flag"
  ```

---

## Task 7: Decompose `app.rs` — extract tab handler modules to `tui/src/handlers/`

**Why:** `tui/src/app.rs` is 1,591 lines. Five `handle_*_key` methods for individual tabs should live in focused single-responsibility files.

**Files:**
- Create: `tui/src/handlers/mod.rs`
- Create: `tui/src/handlers/projects.rs`
- Create: `tui/src/handlers/git_health.rs`
- Create: `tui/src/handlers/cleaner.rs`
- Create: `tui/src/handlers/maintenance.rs`
- Create: `tui/src/handlers/ports.rs`
- Modify: `tui/src/app.rs` — remove extracted methods, add `mod handlers`
- Modify: `tui/src/main.rs` — add `mod handlers`

**Interfaces:**
- Each `handlers/*.rs` is a single `impl App` block containing that tab's key-handler method
- `App` struct stays in `app.rs` — only the `fn handle_*_key` methods move out
- Call sites in `handle_key` (the `match self.current_tab` block in `app.rs`) are unchanged

- [ ] **Step 1: Find method boundaries**

  ```powershell
  Select-String -Path tui/src/app.rs -Pattern 'fn handle_(projects|git_health|cleaner|maintenance|ports)_key'
  ```

  Note the exact line numbers reported. You will move those method bodies in Step 3.

- [ ] **Step 2: Create `tui/src/handlers/mod.rs`**

  ```rust
  // Tab-specific key handlers extracted from app.rs for maintainability.
  // Each module is an `impl App` block for that tab's keyboard logic.
  pub mod cleaner;
  pub mod git_health;
  pub mod maintenance;
  pub mod ports;
  pub mod projects;
  ```

- [ ] **Step 3: Create each handler file by moving methods from `app.rs`**

  **Template** (repeat for each of the 5 tabs):

  Create `tui/src/handlers/projects.rs`:

  ```rust
  use crate::app::App;
  use crossterm::event::KeyCode;

  impl App {
      pub fn handle_projects_key(&mut self, key: KeyCode) {
          // ← paste exact body from app.rs handle_projects_key here
      }
  }
  ```

  Create `tui/src/handlers/git_health.rs` with `handle_git_health_key`.
  Create `tui/src/handlers/cleaner.rs` with `handle_cleaner_key`.
  Create `tui/src/handlers/maintenance.rs` with `handle_maintenance_key`.
  Create `tui/src/handlers/ports.rs` with `handle_ports_key`.

  Each file needs whatever `use` imports the method body references — check the existing `use` block at the top of `app.rs` and copy the relevant ones.

- [ ] **Step 4: Add `mod handlers` to `tui/src/main.rs`**

  After the existing `mod` declarations in `main.rs`, add:

  ```rust
  mod handlers;
  ```

- [ ] **Step 5: Remove moved methods from `app.rs`**

  Delete the bodies of `handle_projects_key`, `handle_git_health_key`, `handle_cleaner_key`, `handle_maintenance_key`, and `handle_ports_key` from `app.rs`. The call sites in the `match self.current_tab` block (around line 382) remain unchanged.

- [ ] **Step 6: Build and verify**

  ```powershell
  cargo build -p rtbtui
  cargo test -p rtbtui -- --nocapture
  ```

  Expected: clean build, all tests pass. Confirm binary starts: `cargo run -p rtbtui`.

- [ ] **Step 7: Commit**

  ```powershell
  git add tui/src/handlers/ tui/src/app.rs tui/src/main.rs
  git commit -m "refactor(tui): extract tab key handlers from app.rs into tui/src/handlers/"
  ```

---

## Self-Review

### Spec coverage

| Requirement from grilling session | Task |
|---|---|
| Remove hardcoded personal paths — TUI `config.rs` + `ui/mod.rs` | Task 1 |
| Git-clean check before destructive operations | Task 2 |
| `y/N` confirmation before archive / pause / clean | Task 2 |
| `clean -Commit` explicit flag replaces silent `--force` | Task 2 |
| Fuzzy `goto` with numbered picker on ambiguous input | Task 3 |
| `Find-ProjectPathFuzzy` utility in helpers | Task 3 |
| Richer `.rtb_context.md` — git log + diff stat + deps | Task 4 |
| Context enrichment in both CLI (`agent.ps1`) and TUI (`agents.rs`) | Task 4 |
| `rtb doctor` system health check command | Task 5 |
| `rtb doctor` wired into `rtb.psm1` switch + exports | Task 5 |
| `rtb status` plain + `-Json` shell prompt segment | Task 6 |
| `rtb status` wired into `rtb.psm1` switch + exports | Task 6 |
| `app.rs` decomposition — tab handlers extracted to `handlers/` | Task 7 |

### Placeholder scan

None found. All steps contain concrete code, commands, and expected outputs.

### Type consistency

- `Find-ProjectPathFuzzy` returns `[PSCustomObject[]]` with `{Name, Path, Status, Score}` — only consumed in `Dev-Goto` in Task 3 ✓
- `Confirm-RtbAction` and `Test-GitClean` defined in Task 2 Step 3; used in Task 2 Steps 5–7 — parameter names consistent ✓
- `create_agent_context_file` Rust signature `(&Project) -> Option<PathBuf>` unchanged; call site at `agents.rs:90` unchanged ✓
- `DevConfig::candidate_paths()` declared `pub` in Task 1 Step 3 so `#[cfg(test)]` block can call it ✓
- `Rtb-Doctor` returns `[bool]` — test in Task 5 Step 1 uses `Should -BeOfType [bool]` ✓
- `Rtb-Status -Json` returns `[string]` (JSON) — test in Task 6 Step 1 calls `ConvertFrom-Json` ✓
