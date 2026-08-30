# Changelog

All notable changes to **RTB — رتّب (Repository & Tooling Base)** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
