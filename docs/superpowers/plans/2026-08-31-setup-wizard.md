# Setup Wizard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the silent, auto-executing install scripts with a rich interactive Setup Wizard that guides users through installation with spinners, colored steps, prompts, and a final summary — on Windows (PowerShell), Linux, and macOS (Bash).

**Architecture:** Two entry-point scripts (`install.ps1` + `install.sh`) share the same UX conventions and step sequence. Each script contains its own spinner engine (pure PowerShell job / pure Bash background process), a linear step runner, and interactive prompts. The Unix shell script bootstraps `pwsh` if missing before delegating setup logic. The standalone download path fetches `rtb-cli.zip` (PowerShell module) plus the correct per-OS binary from the GitHub release.

**Tech Stack:** PowerShell Core 7+ (`install.ps1`), Bash 3.2+ / POSIX sh (`install.sh`), GitHub Releases API, ANSI escape codes (auto-detected), `curl` / `Invoke-WebRequest`.

**Spec:** `docs/superpowers/plans/2026-08-31-setup-wizard.md` (this file) + grilling session decisions in [`CONTEXT.md`](../../CONTEXT.md).

## Global Constraints

- No external dependencies beyond `pwsh` and standard Unix tools (`curl`, `uname`, `sed`, `grep`).
- Spinner: language-native only — no compiled helpers, no `oh-my-posh`.
- All ANSI output is gated: auto-detect; honour `--quiet` / `RTB_QUIET=1`.
- Unix install path: `$XDG_CONFIG_HOME/rtb` → `~/.config/rtb`.
- Windows install path: `$env:APPDATA\rtb` (user-overridable via prompt).
- Critical step failures abort; non-critical warn and continue.
- Branch: `feat/setup-wizard`. Deliver via PR to `main`.
- `install.ps1` is replaced in-place. `install.sh` is new at repo root.

---

## File Map

| File | Action | Responsibility |
|---|---|---|
| `install.ps1` | **Replace** | Windows Setup Wizard (spinner, prompts, steps, summary) |
| `install.sh` | **Create** | Unix Setup Wizard (pwsh bootstrap, spinner, prompts, steps, summary) |
| `CONTEXT.md` | **Updated** (done) | "Setup Wizard" domain term |
| `README.md` | **Modify** | Update install one-liners |

---

## Task 1: PowerShell Spinner & Step-Runner Infrastructure

**Files:**
- Modify: `install.ps1` (full rewrite)

**Interfaces:**
- Produces:
  - `Start-Spinner([string]$Label) → [hashtable]` — starts background job, returns ctx handle
  - `Stop-Spinner([hashtable]$ctx, [bool]$success)` — stops job, prints ✅ or ❌
  - `Write-Step([int]$n, [int]$total, [string]$label)` — prints `[N/T] ◆ LABEL` in cyan
  - `Write-Warn([string]$msg)` — yellow warning
  - `Write-Fail([string]$msg)` — red error + exit 1
  - `$script:QUIET` — bool from `--quiet` / `$env:RTB_QUIET`
  - `$script:ANSI` — bool terminal capability

- [ ] **Step 1: Create manual test harness** at `.scratch/test-spinner.ps1`

  ```powershell
  # Run: pwsh -File .scratch/test-spinner.ps1
  # After install.ps1 exposes functions via dot-source, validate spinner visually
  ```

- [ ] **Step 2: Rewrite `install.ps1` — param block + ANSI detection**

  ```powershell
  #Requires -Version 5.1
  # RTB (رتّب) Setup Wizard — Windows / PowerShell
  param(
      [string]$InstallPath = '',
      [switch]$Quiet
  )
  $ErrorActionPreference = 'Stop'

  $script:QUIET = $Quiet.IsPresent -or ($env:RTB_QUIET -eq '1')
  $script:ANSI  = (-not $script:QUIET) -and (
      $PSVersionTable.PSVersion.Major -ge 7 -or
      $env:TERM -match 'xterm|screen|256color' -or
      [bool]($Host.UI.RawUI.ForegroundColor -ne -1)
  )
  function script:Esc([string]$code) { if ($script:ANSI) { "`e[$code" } else { '' } }
  ```

- [ ] **Step 3: Implement `Write-Step`, `Write-Warn`, `Write-Fail`**

  ```powershell
  function script:Write-Step([int]$n, [int]$total, [string]$label) {
      if ($script:QUIET) { Write-Host "[$n/$total] $label"; return }
      $c = script:Esc '36m'; $b = script:Esc '1m'; $r = script:Esc '0m'
      Write-Host "  ${b}${c}[$n/$total]${r} ◆ $label"
  }
  function script:Write-Warn([string]$msg) {
      Write-Host "  $(script:Esc '33m')⚠  $msg$(script:Esc '0m')"
  }
  function script:Write-Fail([string]$msg) {
      Write-Host "  $(script:Esc '31m')✗  $msg$(script:Esc '0m')"; exit 1
  }
  ```

- [ ] **Step 4: Implement `Start-Spinner` / `Stop-Spinner`**

  ```powershell
  $script:SPINNER_FRAMES = '⠋','⠙','⠹','⠸','⠼','⠴','⠦','⠧','⠇','⠏'

  function script:Start-Spinner([string]$Label) {
      if ($script:QUIET) { Write-Host "  … $Label"; return @{ Job = $null; Label = $Label } }
      $frames = $script:SPINNER_FRAMES
      $job = Start-Job -ScriptBlock {
          param($frames, $label)
          $i = 0
          while ($true) {
              [Console]::Write("`r  $($frames[$i % $frames.Count])  $label")
              Start-Sleep -Milliseconds 80; $i++
          }
      } -ArgumentList $frames, $Label
      return @{ Job = $job; Label = $Label }
  }

  function script:Stop-Spinner([hashtable]$ctx, [bool]$success) {
      if ($ctx.Job) {
          Stop-Job $ctx.Job -EA SilentlyContinue
          Remove-Job $ctx.Job -Force -EA SilentlyContinue
          [Console]::Write("`r" + (" " * 60) + "`r")
      }
      $icon  = if ($success) { '✅' } else { '❌' }
      $color = if ($success) { script:Esc '32m' } else { script:Esc '31m' }
      Write-Host "  $icon  ${color}$($ctx.Label)$(script:Esc '0m')"
  }
  ```

- [ ] **Step 5: Smoke-test spinner manually**

  ```powershell
  pwsh -File .scratch/test-spinner.ps1
  ```
  Expected: animated frames → ✅ / ❌ labels.

- [ ] **Step 6: Commit**

  ```bash
  git add install.ps1 .scratch/test-spinner.ps1
  git commit -m "feat(wizard): PS spinner & step-runner infrastructure"
  ```

---

## Task 2: PowerShell Header, Prompts & Summary

**Files:**
- Modify: `install.ps1`

**Interfaces:**
- Consumes: Task 1 helpers
- Produces: `Show-Header`, `Prompt-InstallPath`, `Prompt-Profiles`, `Prompt-RunInit`, `Show-Summary`, `Main` function (sets `$script:userConfigDir`, `$script:moduleHome`, `$script:scriptsDir`, `$script:resolvedProfiles`)

- [ ] **Step 1: Implement `Show-Header`**

  ```powershell
  function script:Show-Header {
      if ($script:QUIET) { Write-Host 'RTB Setup Wizard'; return }
      $c = script:Esc '36m'; $b = script:Esc '1m'; $r = script:Esc '0m'
      Write-Host ""; Write-Host "  ${b}${c}██████╗ ████████╗██████╗ ${r}"
      Write-Host "  ${b}${c}██╔══██╗╚══██╔══╝██╔══██╗${r}"
      Write-Host "  ${b}${c}██████╔╝   ██║   ██████╔╝${r}"
      Write-Host "  ${b}${c}██╔══██╗   ██║   ██╔══██╗${r}"
      Write-Host "  ${b}${c}██║  ██║   ██║   ██████╔╝${r}"
      Write-Host "  ${b}${c}╚═╝  ╚═╝   ╚═╝   ╚═════╝ ${r}  Setup Wizard"
      Write-Host ""; Write-Host "  ${c}رتّب — Repository & Tooling Base${r}"
      Write-Host "  $(script:Esc '90m')Windows / PowerShell installer$(script:Esc '0m')"
      Write-Host ""
  }
  ```

- [ ] **Step 2: Implement `Prompt-InstallPath`, `Prompt-Profiles`, `Prompt-RunInit`**

  ```powershell
  function script:Prompt-InstallPath([string]$default) {
      if ($script:QUIET -or $InstallPath) { return if ($InstallPath) { $InstallPath } else { $default } }
      Write-Host "  $(script:Esc '32m')?$(script:Esc '0m') Install location $(script:Esc '90m')(Enter to accept)$(script:Esc '0m')"
      Write-Host "    $(script:Esc '90m')$default$(script:Esc '0m')"
      Write-Host -NoNewline '  › '
      $in = Read-Host
      return if ($in.Trim()) { $in.Trim() } else { $default }
  }

  function script:Prompt-Profiles([string[]]$candidates) {
      if ($script:QUIET) { return $candidates }
      $selected = @()
      Write-Host ""
      Write-Host "  $(script:Esc '32m')?$(script:Esc '0m') Which PowerShell profiles should RTB auto-load into?"
      foreach ($p in $candidates) {
          if (-not $p) { continue }
          Write-Host -NoNewline "    Include $(script:Esc '90m')$p$(script:Esc '0m')? [Y/n] "
          if ((Read-Host) -notmatch '^[Nn]') { $selected += $p }
      }
      return $selected
  }

  function script:Prompt-RunInit {
      if ($script:QUIET) { return $false }
      Write-Host ""; Write-Host -NoNewline "  $(script:Esc '32m')?$(script:Esc '0m') Run 'rtb init' now? [Y/n] "
      return ((Read-Host) -notmatch '^[Nn]')
  }
  ```

- [ ] **Step 3: Implement `Show-Summary`**

  ```powershell
  function script:Show-Summary([string]$installPath, [string[]]$profiles) {
      $g = script:Esc '32m'; $b = script:Esc '1m'; $c = script:Esc '36m'
      $d = script:Esc '90m'; $r = script:Esc '0m'
      Write-Host ""; Write-Host "  ${b}${g}✔ RTB installed successfully!${r}"; Write-Host ""
      Write-Host "  ${c}Install path:${r}  $installPath"
      foreach ($p in $profiles) { Write-Host "  ${c}Profile:${r}       $p" }
      Write-Host ""; Write-Host "  ${b}Next steps:${r}"
      Write-Host "    ${g}rtb init${r}  ${d}— configure your project workspace${r}"
      Write-Host "    ${g}rtb help${r}  ${d}— explore available commands${r}"
      Write-Host "    ${g}rtb ui${r}    ${d}— open the terminal dashboard${r}"
      Write-Host ""
  }
  ```

- [ ] **Step 4: Wire `Main` orchestrator**

  ```powershell
  function script:Main {
      Show-Header
      $default = if ($env:APPDATA) { Join-Path $env:APPDATA 'rtb' } else { Join-Path $env:HOME '.config/rtb' }
      $script:userConfigDir    = Prompt-InstallPath $default
      $script:moduleHome       = Join-Path $script:userConfigDir 'module'
      $script:scriptsDir       = if ($env:RTB_BIN_DIR) { $env:RTB_BIN_DIR } else { Join-Path $script:userConfigDir 'bin' }
      $docs                    = [Environment]::GetFolderPath('MyDocuments')
      $allProfiles             = @($PROFILE, (Join-Path $docs 'WindowsPowerShell\Microsoft.PowerShell_profile.ps1'), (Join-Path $docs 'PowerShell\Microsoft.PowerShell_profile.ps1')) | Select-Object -Unique
      $script:resolvedProfiles = Prompt-Profiles $allProfiles
      Install-Steps  # defined in Task 3
      Show-Summary $script:userConfigDir $script:resolvedProfiles
      if (Prompt-RunInit) { rtb init }
  }

  Main
  ```

- [ ] **Step 5: Test prompts manually**

  ```powershell
  pwsh -File .\install.ps1
  # Accept all defaults; verify header, prompts, summary render correctly
  ```

- [ ] **Step 6: Commit**

  ```bash
  git add install.ps1
  git commit -m "feat(wizard): PS header, prompts, and summary box"
  ```

---

## Task 3: PowerShell Stepped Installation Logic

**Files:**
- Modify: `install.ps1`

**Interfaces:**
- Consumes: all Task 1+2 helpers; `$script:userConfigDir`, `$script:moduleHome`, `$script:scriptsDir`, `$script:resolvedProfiles`
- Produces: `Install-Steps` function — 5-step installer (dirs, module, binary, PATH, profiles)

- [ ] **Step 1: Step 1/5 — Create directories**

  ```powershell
  function script:Install-Steps {
      $TOTAL = 5
      $scriptRoot   = $PSScriptRoot
      $isStandalone = (-not $scriptRoot) -or (-not (Test-Path (Join-Path $scriptRoot 'cli\rtb.psd1')))

      Write-Step 1 $TOTAL 'Creating directories'
      $ctx = Start-Spinner 'Setting up install directories'
      try {
          foreach ($d in @($script:scriptsDir, $script:userConfigDir, $script:moduleHome)) {
              if (-not (Test-Path $d)) { New-Item -ItemType Directory -Path $d -Force | Out-Null }
          }
          Stop-Spinner $ctx $true
      } catch { Stop-Spinner $ctx $false; Write-Fail "Cannot create directories: $_" }
  ```

- [ ] **Step 2: Step 2/5 — Deploy module (standalone downloads zip; repo copies local)**

  ```powershell
      Write-Step 2 $TOTAL 'Deploying RTB module'
      if ($isStandalone) {
          $zipUrl  = 'https://github.com/3mr-5aled/rtb/releases/latest/download/rtb-cli.zip'
          $tmpZip  = Join-Path ([IO.Path]::GetTempPath()) "rtb-install-$(Get-Random).zip"
          $tmpExt  = Join-Path ([IO.Path]::GetTempPath()) "rtb-install-$(Get-Random)"
          $ctx = Start-Spinner 'Downloading rtb-cli.zip'
          try {
              Invoke-WebRequest -Uri $zipUrl -OutFile $tmpZip -UseBasicParsing -TimeoutSec 60 -EA Stop
              Stop-Spinner $ctx $true
              $ctx = Start-Spinner 'Extracting module files'
              Expand-Archive -Path $tmpZip -DestinationPath $tmpExt -Force
              $ec = Join-Path $tmpExt 'cli'
              if (Test-Path $ec) { Copy-Item "$ec\*" $script:moduleHome -Recurse -Force }
              foreach ($f in @('logo.txt','uninstall.ps1')) {
                  $src = Join-Path $tmpExt $f
                  if (Test-Path $src) { Copy-Item $src "$script:scriptsDir\$f" -Force }
              }
              if (Test-Path (Join-Path $tmpExt 'uninstall.ps1')) {
                  Copy-Item (Join-Path $tmpExt 'uninstall.ps1') "$script:userConfigDir\uninstall.ps1" -Force
              }
              Stop-Spinner $ctx $true
          } catch {
              Stop-Spinner $ctx $false
              Write-Fail "Download failed: $_`nCheck https://github.com/3mr-5aled/rtb/releases"
          } finally {
              Remove-Item $tmpZip,$tmpExt -Recurse -Force -EA SilentlyContinue
          }
      } else {
          $ctx = Start-Spinner 'Copying local CLI module'
          $src = Join-Path $scriptRoot 'cli'
          if (Test-Path $src) { Copy-Item "$src\*" $script:moduleHome -Recurse -Force; Stop-Spinner $ctx $true }
          else { Stop-Spinner $ctx $false; Write-Fail 'cli\ not found.' }
          foreach ($f in @('logo.txt','uninstall.ps1')) {
              $s = Join-Path $scriptRoot $f
              if (Test-Path $s) { Copy-Item $s "$script:scriptsDir\$f" -Force }
          }
          if (Test-Path (Join-Path $scriptRoot 'uninstall.ps1')) {
              Copy-Item (Join-Path $scriptRoot 'uninstall.ps1') "$script:userConfigDir\uninstall.ps1" -Force
          }
      }
  ```

- [ ] **Step 3: Step 3/5 — TUI binary (non-critical, per-OS download in standalone)**

  ```powershell
      Write-Step 3 $TOTAL 'Installing rtbtui binary'
      if ($isStandalone) {
          $binUrl = 'https://github.com/3mr-5aled/rtb/releases/latest/download/rtbtui-windows-amd64.exe'
          $tmpBin = Join-Path ([IO.Path]::GetTempPath()) "rtbtui-$(Get-Random).exe"
          $ctx = Start-Spinner 'Downloading rtbtui.exe'
          try {
              Invoke-WebRequest -Uri $binUrl -OutFile $tmpBin -UseBasicParsing -TimeoutSec 180 -EA Stop
              Copy-Item $tmpBin "$script:scriptsDir\rtbtui.exe" -Force
              Copy-Item $tmpBin "$script:scriptsDir\devtui.exe" -Force -EA SilentlyContinue
              Stop-Spinner $ctx $true
          } catch {
              Stop-Spinner $ctx $false
              Write-Warn "TUI binary download failed — 'rtb ui' unavailable, CLI is fine."
          } finally { Remove-Item $tmpBin -Force -EA SilentlyContinue }
      } else {
          $tuiDir = Join-Path $scriptRoot 'tui'
          $cargo  = Get-Command cargo -EA SilentlyContinue
          if ($cargo -and (Test-Path (Join-Path $tuiDir 'Cargo.toml'))) {
              $ctx = Start-Spinner 'Building rtbtui with Cargo'
              Push-Location $tuiDir
              try {
                  cargo build --release 2>&1 | Out-Null
                  $bin = Join-Path $tuiDir 'target\release\rtbtui.exe'
                  if (Test-Path $bin) {
                      Copy-Item $bin "$script:scriptsDir\rtbtui.exe" -Force
                      Copy-Item $bin "$script:scriptsDir\devtui.exe" -Force -EA SilentlyContinue
                  }
                  Stop-Spinner $ctx $true
              } catch { Stop-Spinner $ctx $false; Write-Warn 'Cargo build failed — retaining existing binary.' }
              finally { Pop-Location }
          } else {
              $pre = Join-Path $tuiDir 'target\release\rtbtui.exe'
              if (Test-Path $pre) { Copy-Item $pre "$script:scriptsDir\rtbtui.exe" -Force; Write-Warn 'cargo not found — copied prebuilt binary.' }
              else { Write-Warn "cargo not found and no prebuilt binary — 'rtb ui' will not work." }
          }
      }
  ```

- [ ] **Step 4: Step 4/5 — PATH (Registry + session)**

  ```powershell
      Write-Step 4 $TOTAL 'Configuring PATH'
      $ctx = Start-Spinner 'Updating User PATH'
      try {
          $cur = [Environment]::GetEnvironmentVariable('PATH', 'User')
          if (-not (($cur -split ';') -contains $script:scriptsDir)) {
              [Environment]::SetEnvironmentVariable('PATH', (if ($cur) { "$cur;$script:scriptsDir" } else { $script:scriptsDir }), 'User')
          }
          if (($env:PATH -split ';') -notcontains $script:scriptsDir) { $env:PATH = "$script:scriptsDir;$env:PATH" }
          Stop-Spinner $ctx $true
      } catch { Stop-Spinner $ctx $false; Write-Warn "PATH update failed — add '$script:scriptsDir' manually." }
  ```

- [ ] **Step 5: Step 5/5 — Profile injection**

  ```powershell
      Write-Step 5 $TOTAL 'Configuring PowerShell profile(s)'
      $psd = Join-Path $script:moduleHome 'rtb.psd1'
      if (Test-Path $psd) {
          $line = "Import-Module '$psd' -DisableNameChecking -Force"
          foreach ($p in $script:resolvedProfiles) {
              if (-not $p) { continue }
              $ctx = Start-Spinner "Updating $([IO.Path]::GetFileName($p))"
              try {
                  $dir = Split-Path $p -Parent
                  if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
                  if (-not (Test-Path $p))   { New-Item -ItemType File -Path $p -Force | Out-Null }
                  $clean = @(Get-Content $p -EA SilentlyContinue | Where-Object {
                      $_ -notmatch 'Import-Module\s+.*?[''"].*?(rtb|dev-tools|dev-cli|rtb-command-tool).*?\.psd1[''"]' -and
                      $_ -notmatch '#\s*RTB.*?Module'
                  })
                  ($clean + @('', '# RTB CLI Module', $line)) -join "`r`n" | Set-Content $p -Encoding UTF8
                  Stop-Spinner $ctx $true
              } catch { Stop-Spinner $ctx $false; Write-Warn "Could not update $p — $_" }
          }
          Import-Module $psd -DisableNameChecking -Force -EA SilentlyContinue
      }
  }   # end Install-Steps
  ```

- [ ] **Step 6: End-to-end test — repo mode**

  ```powershell
  pwsh -File .\install.ps1
  # Expected: 5 steps each with spinner → ✅, summary box, prompt for rtb init
  ```

- [ ] **Step 7: End-to-end test — standalone simulation**

  ```powershell
  Rename-Item .\cli .\cli_bak
  pwsh -File .\install.ps1
  # Expected: Step 2 shows download spinner, Steps 3-5 continue normally
  Rename-Item .\cli_bak .\cli
  ```

- [ ] **Step 8: Test --quiet flag**

  ```powershell
  pwsh -File .\install.ps1 -Quiet
  # Expected: plain [1/5] text, no spinners, no prompts, exits 0
  ```

- [ ] **Step 9: Commit**

  ```bash
  git add install.ps1
  git commit -m "feat(wizard): PS stepped installation logic (Steps 1-5)"
  ```

---

## Task 4: Unix Shell Wizard (`install.sh`)

**Files:**
- Create: `install.sh`

**Interfaces:**
- Produces: POSIX `sh`-compatible script invoked via `curl -fsSL <url> | sh`
- Env vars: `RTB_INSTALL_PATH`, `RTB_QUIET=1`
- Exit: 0 on success, 1 on critical failure

- [ ] **Step 1: ANSI helpers + `write_step` / `write_warn` / `write_fail`**

  Create `install.sh`:
  ```sh
  #!/usr/bin/env sh
  # RTB (رتّب) Setup Wizard — Linux / macOS
  # Usage: curl -fsSL https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.sh | sh
  set -e
  RTB_QUIET="${RTB_QUIET:-0}"

  if [ "$RTB_QUIET" = "1" ] || [ ! -t 1 ]; then ANSI=0
  else
      case "$TERM" in xterm*|screen*|*256color*|*color*) ANSI=1 ;; *) ANSI=0 ;; esac
  fi

  esc() { [ "$ANSI" = "1" ] && printf '\033[%sm' "$1" || true; }
  write_step() { printf '  %s%s[%s/%s]%s ◆ %s\n' "$(esc '1m')" "$(esc '36m')" "$1" "$2" "$(esc '0m')" "$3"; }
  write_warn()  { printf '  %s⚠  %s%s\n' "$(esc '33m')" "$1" "$(esc '0m')"; }
  write_fail()  { printf '  %s✗  %s%s\n' "$(esc '31m')" "$1" "$(esc '0m')"; exit 1; }
  ```

- [ ] **Step 2: Spinner (background process)**

  ```sh
  SPINNER_PID=""
  start_spinner() {
      label="$1"
      [ "$RTB_QUIET" = "1" ] && { printf '  … %s\n' "$label"; return; }
      (while true; do
          for f in '⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏'; do
              printf '\r  %s  %s%s%s' "$f" "$(esc '90m')" "$label" "$(esc '0m')"
              sleep 0.08
          done
      done) &
      SPINNER_PID=$!
  }
  stop_spinner() {
      success="$1"; label="$2"
      [ -n "$SPINNER_PID" ] && { kill "$SPINNER_PID" 2>/dev/null; wait "$SPINNER_PID" 2>/dev/null; SPINNER_PID=""; printf '\r%60s\r' ''; }
      icon='❌'; color="$(esc '31m')"
      [ "$success" = "1" ] && { icon='✅'; color="$(esc '32m')"; }
      printf '  %s  %s%s%s\n' "$icon" "$color" "$label" "$(esc '0m')"
  }
  ```

- [ ] **Step 3: Header + prompts + summary**

  ```sh
  show_header() {
      [ "$RTB_QUIET" = "1" ] && { printf 'RTB Setup Wizard\n'; return; }
      printf '\n  %s%s██████╗ ████████╗██████╗ %s\n'  "$(esc '1m')" "$(esc '36m')" "$(esc '0m')"
      printf '  %s%s██╔══██╗╚══██╔══╝██╔══██╗%s\n'   "$(esc '1m')" "$(esc '36m')" "$(esc '0m')"
      printf '  %s%s██████╔╝   ██║   ██████╔╝%s\n'   "$(esc '1m')" "$(esc '36m')" "$(esc '0m')"
      printf '  %s%s██╔══██╗   ██║   ██╔══██╗%s\n'   "$(esc '1m')" "$(esc '36m')" "$(esc '0m')"
      printf '  %s%s██║  ██║   ██║   ██████╔╝%s\n'   "$(esc '1m')" "$(esc '36m')" "$(esc '0m')"
      printf '  %s%s╚═╝  ╚═╝   ╚═╝   ╚═════╝ %s  Setup Wizard\n' "$(esc '1m')" "$(esc '36m')" "$(esc '0m')"
      printf '\n  %sرتّب — Repository & Tooling Base%s\n' "$(esc '36m')" "$(esc '0m')"
      printf '  %sLinux / macOS installer%s\n\n' "$(esc '90m')" "$(esc '0m')"
  }

  prompt_install_path() {
      default="$1"
      [ "$RTB_QUIET" = "1" ] || [ -n "$RTB_INSTALL_PATH" ] && { echo "${RTB_INSTALL_PATH:-$default}"; return; }
      printf '  %s?%s Install location %s(Enter to accept)%s\n    %s%s%s\n  › ' \
          "$(esc '32m')" "$(esc '0m')" "$(esc '90m')" "$(esc '0m')" "$(esc '90m')" "$default" "$(esc '0m')"
      read -r ans; echo "${ans:-$default}"
  }

  prompt_run_init() {
      [ "$RTB_QUIET" = "1" ] && { echo n; return; }
      printf '\n  %s?%s Run '"'"'rtb init'"'"' now? [Y/n] ' "$(esc '32m')" "$(esc '0m')"
      read -r ans
      case "$ans" in [Nn]*) echo n ;; *) echo y ;; esac
  }

  show_summary() {
      ipath="$1"
      printf '\n  %s%s✔ RTB installed successfully!%s\n\n' "$(esc '1m')" "$(esc '32m')" "$(esc '0m')"
      printf '  %sInstall path:%s  %s\n\n' "$(esc '36m')" "$(esc '0m')" "$ipath"
      printf '  %sNext steps:%s\n' "$(esc '1m')" "$(esc '0m')"
      printf '    %srtb init%s  %s— configure workspace%s\n' "$(esc '32m')" "$(esc '0m')" "$(esc '90m')" "$(esc '0m')"
      printf '    %srtb help%s  %s— explore commands%s\n'    "$(esc '32m')" "$(esc '0m')" "$(esc '90m')" "$(esc '0m')"
      printf '    %srtb ui%s    %s— terminal dashboard%s\n\n' "$(esc '32m')" "$(esc '0m')" "$(esc '90m')" "$(esc '0m')"
  }
  ```

- [ ] **Step 4: OS detection + `ensure_pwsh`**

  ```sh
  detect_os_arch() {
      OS="$(uname -s)"; ARCH="$(uname -m)"
      case "$OS" in Linux) OS_SLUG=linux ;; Darwin) OS_SLUG=macos ;; *) write_fail "Unsupported OS: $OS" ;; esac
      case "$ARCH" in x86_64|amd64) ARCH_SLUG=amd64 ;; arm64|aarch64) ARCH_SLUG=arm64 ;; *) ARCH_SLUG=amd64 ;; esac
  }

  ensure_pwsh() {
      command -v pwsh > /dev/null 2>&1 && return
      write_warn "PowerShell (pwsh) not found — RTB requires it."
      printf '  %s?%s Install PowerShell now? [Y/n] ' "$(esc '32m')" "$(esc '0m')"
      read -r ans
      case "$ans" in [Nn]*) write_fail "Install pwsh from https://aka.ms/install-powershell then re-run." ;; esac
      if command -v brew   > /dev/null 2>&1; then brew install --cask powershell
      elif command -v apt-get > /dev/null 2>&1; then
          wget -q "https://packages.microsoft.com/config/ubuntu/$(lsb_release -rs)/packages-microsoft-prod.deb" -O /tmp/ms-prod.deb
          sudo dpkg -i /tmp/ms-prod.deb && sudo apt-get update -q && sudo apt-get install -y powershell
      elif command -v dnf  > /dev/null 2>&1; then sudo dnf install -y powershell
      else write_fail "Cannot auto-install pwsh. Install manually from https://aka.ms/install-powershell"; fi
      command -v pwsh > /dev/null 2>&1 || write_fail "pwsh install failed. Install manually."
  }
  ```

- [ ] **Step 5: 5-step `install_steps` function**

  ```sh
  TOTAL=5

  install_steps() {
      XDG_CFG="${XDG_CONFIG_HOME:-$HOME/.config}"
      RTB_DIR="$(prompt_install_path "$XDG_CFG/rtb")"
      MODULE_HOME="$RTB_DIR/module"; BIN_DIR="$RTB_DIR/bin"

      # 1 — Directories
      write_step 1 $TOTAL 'Creating directories'
      start_spinner 'Setting up install directories'
      mkdir -p "$RTB_DIR" "$MODULE_HOME" "$BIN_DIR" \
          || { stop_spinner 0 'Create directories'; write_fail 'Cannot create install directories.'; }
      stop_spinner 1 'Created install directories'

      # 2 — Module (zip)
      write_step 2 $TOTAL 'Downloading RTB module'
      ZIP_URL='https://github.com/3mr-5aled/rtb/releases/latest/download/rtb-cli.zip'
      TMP_ZIP="/tmp/rtb-$$.zip"; TMP_EXT="/tmp/rtb-ext-$$"
      start_spinner 'Downloading rtb-cli.zip'
      curl -fsSL --max-time 120 "$ZIP_URL" -o "$TMP_ZIP" 2>/dev/null \
          || { stop_spinner 0 'Download rtb-cli.zip'; rm -f "$TMP_ZIP"; write_fail "Download failed from $ZIP_URL"; }
      stop_spinner 1 'Downloaded rtb-cli.zip'
      start_spinner 'Extracting module files'
      mkdir -p "$TMP_EXT"
      command -v unzip > /dev/null 2>&1 \
          && unzip -q "$TMP_ZIP" -d "$TMP_EXT" \
          || pwsh -NoProfile -Command "Expand-Archive -Path '$TMP_ZIP' -DestinationPath '$TMP_EXT' -Force"
      [ -d "$TMP_EXT/cli" ]          && cp -r "$TMP_EXT/cli/." "$MODULE_HOME/"
      [ -f "$TMP_EXT/logo.txt" ]     && cp "$TMP_EXT/logo.txt" "$BIN_DIR/logo.txt"
      [ -f "$TMP_EXT/uninstall.ps1" ] && cp "$TMP_EXT/uninstall.ps1" "$RTB_DIR/uninstall.ps1"
      stop_spinner 1 'Extracted module files'
      rm -f "$TMP_ZIP"; rm -rf "$TMP_EXT"

      # 3 — TUI binary (non-critical)
      write_step 3 $TOTAL 'Installing rtbtui binary'
      BIN_URL="https://github.com/3mr-5aled/rtb/releases/latest/download/rtbtui-${OS_SLUG}-${ARCH_SLUG}"
      start_spinner "Downloading rtbtui ($OS_SLUG/$ARCH_SLUG)"
      if curl -fsSL --max-time 180 "$BIN_URL" -o "$BIN_DIR/rtbtui" 2>/dev/null; then
          chmod +x "$BIN_DIR/rtbtui"
          stop_spinner 1 'Installed rtbtui binary'
      else
          stop_spinner 0 'Download rtbtui binary'
          write_warn "TUI binary unavailable — 'rtb ui' will not work, but CLI is fine."
      fi

      # 4 — PATH + shell alias injection
      write_step 4 $TOTAL 'Configuring shell PATH'
      start_spinner 'Updating shell rc files'
      EXPORT_LINE="export PATH=\"\$PATH:$BIN_DIR\""
      ALIAS_LINE="alias rtb='pwsh -NoProfile -NonInteractive -Command rtb'"
      _inject() {
          rc="$1"; [ -f "$rc" ] || return
          grep -qF "$BIN_DIR" "$rc" 2>/dev/null && return
          printf '\n# RTB CLI\n%s\n%s\n' "$EXPORT_LINE" "$ALIAS_LINE" >> "$rc"
      }
      _inject "$HOME/.bashrc"; _inject "$HOME/.bash_profile"
      _inject "$HOME/.zshrc";  _inject "$HOME/.profile"
      export PATH="$PATH:$BIN_DIR"
      stop_spinner 1 'Shell configuration updated'

      # 5 — pwsh profile
      write_step 5 $TOTAL 'Configuring PowerShell profile'
      PWSH_PROFILE_DIR="${XDG_CFG}/powershell"
      PWSH_PROFILE="$PWSH_PROFILE_DIR/Microsoft.PowerShell_profile.ps1"
      MODULE_PSD="$MODULE_HOME/rtb.psd1"
      start_spinner 'Injecting Import-Module into pwsh profile'
      if [ -f "$MODULE_PSD" ]; then
          mkdir -p "$PWSH_PROFILE_DIR"; touch "$PWSH_PROFILE"
          grep -qF "rtb.psd1" "$PWSH_PROFILE" 2>/dev/null \
              || printf '\n# RTB CLI Module\nImport-Module '"'"'%s'"'"' -DisableNameChecking -Force\n' "$MODULE_PSD" >> "$PWSH_PROFILE"
          stop_spinner 1 'PowerShell profile configured'
      else
          stop_spinner 0 'PowerShell profile'
          write_warn "rtb.psd1 not found — skipping profile injection."
      fi
  }
  ```

- [ ] **Step 6: `main` entrypoint**

  ```sh
  main() {
      show_header
      detect_os_arch
      ensure_pwsh
      install_steps
      show_summary "$RTB_DIR"
      [ "$(prompt_run_init)" = "y" ] && pwsh -NoProfile -NonInteractive -Command "Import-Module '$MODULE_PSD'; rtb init" || true
  }

  main
  ```

- [ ] **Step 7: Make executable + smoke test on Linux/macOS (or WSL)**

  ```bash
  chmod +x install.sh
  sh ./install.sh
  # Expected: header → 5 steps with spinners → ✅ each → summary box
  ```

- [ ] **Step 8: Test quiet/piped mode**

  ```bash
  RTB_QUIET=1 sh ./install.sh
  # Expected: plain [1/5] lines, exits 0, no spinner escape codes in output
  ```

- [ ] **Step 9: Commit**

  ```bash
  git add install.sh
  git commit -m "feat(wizard): Unix install.sh — spinner, pwsh bootstrap, 5-step setup"
  ```

---

## Task 5: README Update & Pull Request

**Files:**
- Modify: `README.md`
- Verify: `CONTEXT.md` (already updated)

- [ ] **Step 1: Update install one-liners in README**

  Find the installation section and replace with:
  ```markdown
  ## Installation

  **Windows** — open PowerShell and run:
  ```powershell
  irm https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.ps1 | iex
  ```

  **Linux / macOS** — open a terminal and run:
  ```bash
  curl -fsSL https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.sh | sh
  ```

  Both installers are interactive — they'll guide you through choosing an install path and configuring your shell profile.
  Pass `RTB_QUIET=1` (or `-Quiet` on Windows) for a silent non-interactive install.
  ```

- [ ] **Step 2: Final pre-PR checklist**

  - [ ] `pwsh -File install.ps1` — repo mode — all 5 steps ✅
  - [ ] `pwsh -File install.ps1 -Quiet` — silent, exits 0
  - [ ] `sh install.sh` — Linux/macOS — all 5 steps ✅
  - [ ] `RTB_QUIET=1 sh install.sh` — silent, exits 0
  - [ ] Standalone simulation (rename `cli/`) passes for PS
  - [ ] No `.scratch/` test files committed to branch
  - [ ] `CONTEXT.md` has Setup Wizard term ✅

- [ ] **Step 3: Commit README**

  ```bash
  git add README.md
  git commit -m "docs: update install one-liners for cross-platform Setup Wizard"
  ```

- [ ] **Step 4: Push and open PR**

  ```bash
  git push -u origin feat/setup-wizard
  ```
  Open PR on GitHub:
  - **Base:** `main` → **Compare:** `feat/setup-wizard`
  - **Title:** `feat: cross-platform Setup Wizard (install.ps1 + install.sh)`
  - **Body:** paste the decisions table from the grilling session

---

## Self-Review

| Spec requirement | Task |
|---|---|
| Spinners | 1 (PS), 4 (sh) |
| Numbered colored step labels | 1, 4 (`write_step`) |
| Final summary box | 2, 4 (`show_summary`) |
| Both standalone + repo modes | 3 (PS), 4 (sh) |
| Interactive install path prompt | 2, 4 |
| Interactive profile prompt | 2 (PS), 4 (sh alias + pwsh profile) |
| Offer `rtb init` at end | 2, 4 |
| Critical abort / non-critical warn | 3, 4 |
| Auto-detect ANSI + `RTB_QUIET` / `-Quiet` | 1, 4 |
| Unix entry via `curl … \| sh` | 4 + 5 README |
| XDG install path on Unix | 4 |
| pwsh bootstrap on Unix | 4 (`ensure_pwsh`) |
| Shell alias injection (.bashrc/.zshrc) | 4 |
| Per-OS binary download | 3 (PS), 4 (sh) |
| `feat/setup-wizard` branch + PR | 5 |
| `CONTEXT.md` Setup Wizard term | Done (pre-plan) |
