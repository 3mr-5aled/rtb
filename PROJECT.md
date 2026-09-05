# Project: RTB CLI & TUI Improvements

## Architecture
RTB (Repository & Tooling Base) is a developer workspace management tool with:
- **Core CLI (TypeScript / Node.js 18+)**: `core/src/` providing cross-platform commands (`init`, `goto`, `agent`, `doctor`, `ui`, `shell-init`, `run`, `build`, `test`, `deps`, `workspace`, `open`, `health`, `maintenance`, `backup`, `env`, `guard`, `deploy`, `pause`, `resume`, `archive`) compiled via `tsup`.
- **TUI (Rust Ratatui/Crossterm)**: `tui/src/` with an observational dashboard for project overview, git health, dependency cleaner, maintenance, ports, and agent launch.
- **Config**: JSON-based config stored in `~/.config/rtb/rtb.config.json` (`%USERPROFILE%\.config\rtb\rtb.config.json` on Windows, `$HOME/.config/rtb/rtb.config.json` on Unix), falling back to local paths for development.

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | R1: Config Path Decoupling & Logo fallback | Strip `D:\02-Projects\...` hardcodes; add `candidate_paths()` and multi-tier logo lookup | M1 | ORIGINAL_REQUEST.md & Plan Task 1 |
| 2 | R2: CLI Safety Guardrails | `Confirm-RtbAction` and `Test-GitClean` in `helpers.ps1`, integrate into `archive.ps1`, `pause.ps1`, `clean.ps1` (`-Commit` flag) | M2 | ORIGINAL_REQUEST.md & Plan Task 2 |
| 3 | R3a: Fuzzy Multi-Match `goto` | `Find-ProjectPathFuzzy` with scoring and interactive numbered picker in `goto.ps1` | M3 | ORIGINAL_REQUEST.md & Plan Task 3 |
| 4 | R3b: AI Agent Context Enrichment | Rich `.rtb_context.md` generation with git log, diff stat, deps in `agent.ps1` and `agents.rs` | M3 | ORIGINAL_REQUEST.md & Plan Task 4 |
| 5 | R4a: `rtb doctor` Command | Comprehensive system health check in `doctor.ps1` and `rtb.psm1` | M4 | ORIGINAL_REQUEST.md & Plan Task 5 |
| 6 | R4b: `rtb status` Prompt Segment | One-line shell status and `-Json` output in `status.ps1` and `rtb.psm1` | M4 | ORIGINAL_REQUEST.md & Plan Task 6 |
| 7 | R5: TUI Architecture Refactoring | Extract tab key handlers from `app.rs` into `tui/src/handlers/{mod,projects,git_health,cleaner,maintenance,ports}.rs` | M5 | ORIGINAL_REQUEST.md & Plan Task 7 |
| 8 | E2E & Full Verification | Pass 100% Pester tests, Rust tests, cargo build with zero warnings, zero personal paths, verify CLI commands | M6 | ORIGINAL_REQUEST.md Acceptance Criteria |
| 9 | Cross-Platform TypeScript/Node.js CLI Replacement | Pure ESM Node.js CLI engine with multi-runtime inspection, fuzzy goto, shell hooks, agent orchestration, and automated CI release | M7 | Wayfinder Map Issue #34 |
| 10 | Multi-Tier Golden Braille Logo & Hero Banner | Context-aware greeting with 24-bit truecolor braille emblem, dynamic project detection, and directory guide | M8 | Wayfinder Map Issue #74 / #75 |
| 11 | Unified Ora TaskSpinner Utility | Golden braille animated spinner frames, timing diagnostics, and strict headless/JSON suppression | M8 | Wayfinder Map Issue #74 / #76 |
| 12 | Cross-Shell Autocompletion Integrity | Robust project name completion across pwsh, bash, zsh, fish; switch fall-through & wildcard pattern fixes | M8 | Wayfinder Map Issue #74 / #77 |
| 13 | Interactive Setup Wizard (@clack/prompts) | Modern 5-step onboarding wizard for root detection, multi-select lifecycle folders, and shell hook installation | M8 | Wayfinder Map Issue #74 / #78 |
| 14 | Interactive Command Menu (@clack/prompts) | Arrow-navigable command cockpit (`rtb menu`) for project running, navigation, TUI, health, and agent environments | M8 | Wayfinder Map Issue #74 / #79 |
| 15 | E2E & Packaging Acceptance Verification | 100% green tests across 35 test files (197 tests), clean typecheck, and npm package distribution contract | M8 | Wayfinder Map Issue #74 / #80 |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Config Path Decoupling & TUI Clean-up | `tui/src/config.rs`, `tui/src/ui/mod.rs` | None | DONE |
| M2 | CLI Safety Guardrails | `cli/src/utils/helpers.ps1`, `cli/src/commands/{archive,pause,clean}.ps1`, `cli/tests/Test-SafetyGuardrails.Tests.ps1` | None | DONE |
| M3 | Navigation & AI Agent Context Enrichment | `cli/src/commands/goto.ps1`, `cli/src/commands/agent.ps1`, `tui/src/data/agents.rs`, `cli/tests/Test-Goto.Tests.ps1`, `cli/tests/Test-AgentContext.Tests.ps1` | M1, M2 | DONE |
| M4 | Diagnostic & Utility Commands | `cli/src/commands/doctor.ps1`, `cli/src/commands/status.ps1`, `cli/rtb.psm1`, `cli/tests/Test-Doctor.Tests.ps1`, `cli/tests/Test-Status.Tests.ps1` | M2, M3 | DONE |
| M5 | TUI Architecture Refactoring | `tui/src/handlers/*.rs`, `tui/src/app.rs`, `tui/src/main.rs` | M1 | DONE |
| M6 | Full Verification & E2E Acceptance | All tests in `cli/tests/`, `cargo test -p rtbtui`, `cargo build -p rtbtui`, forensic integrity audit | M1..M5 | DONE |
| M7 | Cross-Platform TypeScript Engine (v0.5.0) | `core/`, `install.sh`, `install.ps1`, `.github/workflows/release.yml`, Vitest & Pester suites | M1..M6 | DONE |
| M8 | Modern CLI UX & Design System (v0.12.0) | `core/src/utils/{logo,banner,spinner}.ts`, `core/src/commands/{init,menu,completion}.ts`, Clack prompts, Ora spinners | M7 | DONE |

## Code Layout
- `core/src/index.ts` — TypeScript/Node.js CLI entrypoint
- `core/src/commands/` — Cross-platform subcommands (init, menu, goto, agent, doctor, ui, run, build, test, etc.)
- `core/src/config/loader.ts` — Cross-platform multi-tier config loader
- `core/src/inspector/` — Multi-runtime project and workspace inspector
- `core/src/navigation/fuzzy.ts` — Fuzzy project scoring and path discovery
- `core/src/services/` — Execution and maintenance task registries
- `core/src/utils/logo.ts` — Multi-tier logo loader and 24-bit ANSI converter
- `core/src/utils/banner.ts` — Context-aware HeroBanner renderer
- `core/src/utils/spinner.ts` — Centralized `TaskSpinner` & `withSpinner` Ora wrapper
- `tui/src/config.rs` — Config discovery and deserialization
- `tui/src/ui/` — Ratatui UI components and logo rendering
- `tui/src/data/agents.rs` — Agent context file generation in Rust
- `tui/src/handlers/` — Tab-specific key handlers for TUI
- `tui/src/app.rs` — Core TUI application state and event dispatch
- `tui/src/main.rs` — TUI binary entry point
