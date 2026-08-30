# Project: RTB CLI & TUI Improvements

## Architecture
RTB (Repository & Tooling Base) is a developer workspace management tool with:
- **CLI (PowerShell 7+)**: `cli/src/` with commands for workspace navigation, project lifecycle (archive, pause, clean, goto, agent, doctor, status), and helper utilities (`helpers.ps1`, `rtb.psm1`).
- **TUI (Rust Ratatui/Crossterm)**: `tui/src/` with an observational dashboard for project overview, git health, dependency cleaner, maintenance, ports, and agent launch.
- **Config**: JSON-based config stored in `%APPDATA%\rtb\rtb.config.json` (Windows) or `~/.config/rtb/rtb.config.json` (Unix), falling back to relative paths for local development.

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

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Config Path Decoupling & TUI Clean-up | `tui/src/config.rs`, `tui/src/ui/mod.rs` | None | DONE |
| M2 | CLI Safety Guardrails | `cli/src/utils/helpers.ps1`, `cli/src/commands/{archive,pause,clean}.ps1`, `cli/tests/Test-SafetyGuardrails.Tests.ps1` | None | DONE |
| M3 | Navigation & AI Agent Context Enrichment | `cli/src/commands/goto.ps1`, `cli/src/commands/agent.ps1`, `tui/src/data/agents.rs`, `cli/tests/Test-Goto.Tests.ps1`, `cli/tests/Test-AgentContext.Tests.ps1` | M1, M2 | DONE |
| M4 | Diagnostic & Utility Commands | `cli/src/commands/doctor.ps1`, `cli/src/commands/status.ps1`, `cli/rtb.psm1`, `cli/tests/Test-Doctor.Tests.ps1`, `cli/tests/Test-Status.Tests.ps1` | M2, M3 | DONE |
| M5 | TUI Architecture Refactoring | `tui/src/handlers/*.rs`, `tui/src/app.rs`, `tui/src/main.rs` | M1 | DONE |
| M6 | Full Verification & E2E Acceptance | All tests in `cli/tests/`, `cargo test -p rtbtui`, `cargo build -p rtbtui`, forensic integrity audit | M1..M5 | DONE |

## Code Layout
- `cli/src/utils/helpers.ps1` — Common CLI helper functions
- `cli/src/commands/` — Individual command implementations
- `cli/tests/` — Pester test suite
- `cli/rtb.psm1` — Main PowerShell module entry point
- `tui/src/config.rs` — Config discovery and deserialization
- `tui/src/ui/` — Ratatui UI components and logo rendering
- `tui/src/data/agents.rs` — Agent context file generation in Rust
- `tui/src/handlers/` — Tab-specific key handlers for TUI
- `tui/src/app.rs` — Core TUI application state and event dispatch
- `tui/src/main.rs` — TUI binary entry point
