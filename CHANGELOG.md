# Changelog

All notable changes to **RTB — رتّب (Repository & Tooling Base)** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.13.2] - 2026-09-05

### Changed
- add rtb install setup command with launcher deployment and ready banner

## [v0.13.1] - 2026-09-05

### Changed
- add interactive UI download choice to npx init workflow

## [v0.13.0] - 2026-09-05

### Changed
- allow choosing to download RTB UI now or download later

## [v0.12.4] - 2026-09-05

### Changed
- fix powershell array splatting and add goto shell shortcut

## [v0.12.3] - 2026-09-05

### Changed
- ci(npm): explicitly specify --access public on npm publish in release workflow

## [v0.12.2] - 2026-09-05

### Changed
- docs: add npm package README and LICENSE for @3mr5aled/rtb

## [v0.12.1] - 2026-09-05

### Changed
- docs: update README, PROJECT metadata, and reference specifications for Phase 5

## [v0.12.0] - 2026-09-05

### Changed
- feat: Phase 5 - Modern CLI UX, clack prompts wizard & menu, golden braille brand logo, ora spinners, and shell completions

## [v0.11.6] - 2026-09-05

### Changed
- fix(packaging): restore ./dist/index.js bin path in package.json

## [v0.11.5] - 2026-09-05

### Changed
- fix(pkg): clean bin script path in package.json

## [v0.11.4] - 2026-09-05

### Changed
- fix(uninstall): clean all PATH-discovered launchers, D:\bin, and npm globals

## [v0.11.3] - 2026-09-05

### Changed
- chore(pkg): update package scope to @3mr5aled/rtb matching npm username

## [v0.11.2] - 2026-09-05

### Changed
- chore(npm): configure public access and automated npm publishing workflow

## [v0.11.1] - 2026-09-05

### Changed
- fix(services): allow uninstalled agent fallback when launch is false

## [v0.11.0] - 2026-09-05

### Changed
- Phase 4 - Primary npm distribution, TUI TabController consolidation, and community readiness

## [v0.10.0] - 2026-09-05

### Changed
- Installer simplification, standalone asset packaging, and backwards compat cleanup (Phase 3)

## [v0.9.1] - 2026-09-05

### Changed
- Align shell profile cleaning across all shells and legacy artifacts

## [v0.9.0] - 2026-09-05

### Changed
- Architecture deepening and seam consolidation (Phase 2)

## [v0.8.6] - 2026-09-05

### Changed
- use pre-shaped connected Arabic glyphs in terminal headers

## [v0.8.5] - 2026-09-05

### Changed
- Fix installer and CLI version resolution when run outside repository

## [v0.8.4] - 2026-09-04

### Changed
- Clean upgrade command output and eliminate deprecation warning

## [v0.8.3] - 2026-09-04

### Changed
- Sync local D:\bin installation during release

## [v0.8.2] - 2026-09-04

### Changed
- Fix upgrade command npm noise and target path resolution

## [v0.8.1] - 2026-09-04

### Changed
- Fix TUI session state test race condition in CI

## [v0.8.0] - 2026-09-04

### Changed
- Shell autocompletion for commands, project names, and flags

## [v0.7.0] - 2026-09-04

### Changed
- Implement uninstall/upgrade commands, deploy promotion, resilient TUI config, and DEP0190 runner fix

## [v0.6.3] - 2026-09-04

### Changed
- test: add sandbox test environment generator and fix release.ps1

## [v0.6.2] - 2026-09-04

### Changed
- Fix PowerShell profile array binding error in shell-init

## [v0.6.1] - 2026-09-04

### Changed
- Stream project and health discovery in real time

## [v0.6.0] - 2026-09-04

### Changed
- Retire legacy cli/ directory and unify on TypeScript core engine
## [v0.5.3] - 2026-09-03

### Changed
- Assert RTB_VERSION dynamically in test suite
## [v0.5.2] - 2026-09-03

### Changed
- Change TUI refresh shortcut to [R] to prevent conflict with resume command
## [v0.5.1] - 2026-09-03

### Fixed

- **Config Path Resolution**:
  - Completely removed legacy `%APPDATA%\rtb` configuration fallback so `%USERPROFILE%\.config\rtb\rtb.config.json` is the sole, authoritative source.
  - Setup wizard (`install.ps1`) automatically cleans stale `%APPDATA%\rtb` and test directory entries from user `PATH` and migrates any remaining config.
- **Workspace Scaffold Duplication**:
  - Fixed `rtb init` path calculation when targeting an existing `02-Projects` or `Projects` folder to prevent nested `02-Projects/02-Projects` paths.
  - Added smart discovery for root-level `08-Backup`, `05-Config`, and `01-SandBox` directories.
- **`rtb config` Editor Opening**:
  - Restored editor-launch behavior for `rtb config` across Windows, macOS, and Linux, with `--show` / `--view` flags for terminal inspection.
- **Interactive Installer Module Caching**:
  - Setup wizard now cleans cached in-memory module instances and functions before invoking `rtb init`.

## [v0.5.0] - 2026-09-03

### Added

- **Unified Cross-Platform TypeScript/Node.js CLI Engine (`core/`)**:
  - Replaced PowerShell-exclusive CLI logic with a compiled Node.js engine (`@3mr-5aled/rtb`) bundled via `tsup`.
  - Zero-dependency runtime execution on Linux, macOS, and Windows via Node.js (>= 18).
- **Cross-Shell Integration (`rtb shell-init`)**:
  - Direct shell hooks for `bash`, `zsh`, `fish`, and `pwsh` enabling seamless directory navigation via `rtb goto`.
- **Multi-Runtime Project Inspector (`ProjectInspector`)**:
  - Full detection for Node.js (`package.json`, `pnpm`, `yarn`, `bun`), Rust (`Cargo.toml`), Go (`go.mod`), and Python (`pyproject.toml`, `requirements.txt`).
- **AI Agent Context Generation**:
  - Auto-discovers installed AI agents (`agy`, `claude`, `gemini`, `codex`, `cursor`, `windsurf`, `aider`, `openhands`) in PATH.
  - Automatically compiles `.rtb_context.md` project summaries with recent git logs and diff stat metrics.
- **Unified Cross-Platform Configuration Path**:
  - Standardized configuration directory to `~/.config/rtb/rtb.config.json` across Windows (`%USERPROFILE%\.config\rtb`), macOS, and Linux.
- **Updated Cross-Platform Setup Wizards (`install.ps1` & `install.sh`)**:
  - POSIX shell installer (`install.sh`) provisions Node.js CLI runtime without PowerShell requirement on Unix.
  - Windows installer (`install.ps1`) provisions both native shell wrappers (`rtb.cmd`, `rtb.ps1`) and PowerShell module.

---

## [v0.4.0] - 2026-08-31

### Added

- **Standalone One-Liner Installer (`irm ... | iex`)**:
  - Direct installation from GitHub Releases without requiring `git clone` or local source files.
  - Automatic release bundle packaging (`rtb-cli.zip`) in GitHub Actions workflow via `softprops/action-gh-release@v2`.
  - Automatic permanent user `PATH` configuration (in Windows Registry) and active session `$env:PATH` injection.
  - PowerShell `$PROFILE` integration targeting the centralized module home at `%APPDATA%\rtb\module\`.
- **Interactive Configuration Wizard (`rtb init`)**:
  - Smart workspace root detection (scans `~/Projects`, `~/dev`, `~/code`, `~/repos`, `~/workspace`, `D:\02-Projects`, etc.).
  - Interactive multi-select toggle list for scaffolding lifecycle folders (`Active`, `Paused`, `Deployed`, `Planning`, `Testing`, `Abandoned`, `Sandbox`).
  - Per-folder emoji and custom display label customization.
- **Config Gate**:
  - Intercepts data-dependent commands on unconfigured systems with a helpful prompt: `"Would you like to configure now? (Y/n)"`.
  - Exempts `help`, `doctor`, `init`, `uninstall`, `--version`, and `--help`.
- **Self-Upgrade Engine (`rtb upgrade`)**:
  - `rtb upgrade --check` compares local manifest version with GitHub Releases API.
  - `rtb upgrade` downloads and installs the latest `rtb-cli.zip` release bundle live.
- **Self-Contained Uninstaller (`rtb uninstall` / `uninstall.ps1`)**:
  - Dedicated prompt for PowerShell profile autoload cleanup with warning on manual retention.
  - Works on standalone systems without repository sources.
- **Rich Configuration Schema**:
  - `projectRoots` now supports `{ path, label, emoji }` with automatic backward-compatible normalization for legacy flat string paths across both PowerShell CLI and Rust TUI (`rtbtui`).

---

## [v0.3.0] - 2026-08-30

### Added

- **rtb doctor Command**: System health check tool that diagnoses configuration validity (
  rtb.config.json), project root paths existence, required tools (git), optional language runtimes (Node.js, Cargo, Python, ar), installed AI agent CLIs, and
  tbtui binary presence.
- **rtb status Command**: Compact one-line shell prompt segment for prompt integration (
  rtb » project (Status) [branch ±N] Stack). Includes -Json (--json, -j) flag for machine-readable JSON output.
- **Fuzzy Project Search & Selection (rtb goto)**: Upgraded
  rtb goto with Find-ProjectPathFuzzy scoring algorithm (100 exact, 75 prefix, 50 substring, 25 path match) and an interactive numbered picker ([1-9]) when multiple projects match a query.
- **Enriched AI Agent Workspace Context (.rtb_context.md)**:
  - Expanded context file generation in both PowerShell (gent.ps1) and Rust (gents.rs).
  - Added **Git Context**: last 10 commits (git log --oneline -10) and current diff summary (git diff --stat HEAD).
  - Added **Dependencies Summary**: auto-parsed manifests (package.json, Cargo.toml, go.mod,
    equirements.txt).

### Changed

- **CLI Safety Guardrails**:
  - rtb archive: Added mandatory y/N confirmation prompt and Test-GitClean uncommitted changes check (bypassed with -Force). Ensured source directory removal occurs strictly upon verified archive creation.
  - rtb pause: Added confirmation prompt prior to dependency pruning and uncommitted git changes check.
  - rtb clean: Updated deletion workflow to require explicit -Commit flag (defaults safely to -DryRun).
- **TUI Architecture Refactoring**:
  - Decomposed monolithic ui/src/app.rs by extracting view-specific keyboard event handlers into dedicated single-responsibility modules:
    -     ui/src/handlers/projects.rs
    -     ui/src/handlers/git_health.rs
    -     ui/src/handlers/cleaner.rs
    -     ui/src/handlers/maintenance.rs
    -     ui/src/handlers/ports.rs

### Fixed

- **Config & Logo Path Decoupling**: Completely removed all hardcoded personal absolute paths (D:\02-Projects\...) across TUI configuration discovery ( ui/src/config.rs), logo loader ( ui/src/ui/mod.rs),
  rtb init template generator (cli/src/commands/init.ps1), and project indexer (cli/src/commands/index.ps1). Configuration now resolves dynamically via system user directories (%APPDATA%\rtb\), binary relative paths, or local repository fallback.

---

## [v0.2.0-beta] - 2026-08-28

- Initial Beta Release featuring PowerShell CLI (
  rtb), Rust Terminal UI (
  rtbtui), project intelligence engine, git health monitoring, and AI agent launcher.
































