---
title: RTB v2 Feature Enhancements Spec
status: ready-for-agent
labels:
  - ready-for-agent
  - enhancement
---

# RTB (rtbtui) — todo_v2 Specification

## Problem Statement

As developer workspaces grow, project management tools must maintain accurate branding, clear visual hierarchy, reliable execution feedback, and efficient Git telemetry filtering. Currently in RTB (`rtbtui`):
1. Startup and header banners contain legacy `DEV TUI` branding instead of `RTB` (Repository & Tooling Base).
2. Tech stack tags in the Projects tab detail view are displayed as plain monochrome strings, making technology stacks hard to distinguish at a glance.
3. The loading/progress screen spinner does not show on application launch or manual refresh when cache is present.
4. Git Health tab lacks filtering options, forcing users to scroll through all repositories without an easy way to isolate clean local repos or repos needing attention.
5. Users cannot run a project's dev server / live process directly from the TUI with per-project customized commands.
6. The CLI (`rtb`) lacks a dedicated interactive commit prompt, and TUI commit workflows need streamlining.
7. Re-scanning Git health does not display an active background progress indicator.

## Solution

Enhance the RTB developer tool suite (`rtbtui` Rust TUI & `rtb` PowerShell CLI) by delivering:
- Standardized `RTB` ASCII logo and consistent branding across all screens.
- Vibrant, color-coded tech stack badges (`tech_color()` helper) in the `ProjectsTab` detail view.
- Persistent loading and progress indicators for background project scans and manual refreshes (`r`).
- Comprehensive `GitFilter` in `GitHealthTab` (filtering by `All`, `Needs Attention`, `Local Clean`, `Synced`, `Non-Git`).
- Customizable live program runner (`[x]` key in TUI / `rtb run` in CLI) launching project dev commands (`npm run dev`, `cargo run`, etc.) in an interactive terminal window.
- Interactive commit prompt popup in PowerShell CLI (`rtb commit`) and enhanced TUI `CommitDialog`.
- Animated spinner re-scan progress indicator in `GitHealthTab`.

## User Stories

1. As a developer launching RTB, I want to see consistent `RTB` branding and ASCII logo art, so that I know I am using the official Repository & Tooling Base interface.
2. As a developer browsing projects, I want technology stack tags (React, Next.js, Rust, Tailwind, Python, etc.) to be color-coded in the project detail panel, so that I can immediately identify stack components visually.
3. As a developer refreshing the workspace, I want to see a visible loading/progress spinner, so that I know background directory scanning and Git telemetry extraction are active.
4. As a developer with dozens of repositories, I want to filter the Git Health tab by status (e.g. repos needing attention vs local clean repos), so that I can focus on uncommitted or unpushed work quickly.
5. As a developer selecting a project in RTB, I want to press a single key (`[x]`) to launch its dev server live in a new terminal window, so that I can start working instantly without manually navigating to the directory and typing run commands.
6. As a developer working in a project without standard scripts, I want to customize the dev run command for that project, so that RTB executes the exact start/dev command appropriate for that repository.
7. As a CLI user running `rtb commit`, I want an interactive pop-up or console prompt for the commit message when none is supplied, so that I can comfortably type and verify my commit message.
8. As a developer triggering a Git re-scan in `GitHealthTab`, I want an inline loading state spinner, so that I receive immediate feedback while Git telemetry is being updated in the background.

## Implementation Decisions

- **Domain Glossary Alignment**: Respect definitions in `CONTEXT.md` (`ProjectInspector`, `TabController`, `MaintenanceTaskRegistry`, `AgentOrchestrator`).
- **Branding Standard**: Standardize ASCII art and header titles across `loading.rs`, `dashboard.rs`, `help.rs`, and `ui/mod.rs` to display `RTB` and `RTB — ﺐﺘّﺭ`.
- **Technology Color Seam**: Introduce a `tech_color(tech: &str) -> Color` utility mapping standard stack keywords (Next.js, React, Rust, Vue, Vite, Tailwind, TypeScript, Python, Express, Docker, Go, etc.) to distinct Ratatui color tokens.
- **Git Health Filtering Model**: Implement `GitFilter` enum (`All`, `NeedsAttention`, `LocalClean`, `Synced`, `NonGit`) stored in `App` state and updated via `[f]` key binding in `GitHealthTab`.
- **Live Program Execution Seam**: Extend `Project` struct with `dev_command: Option<String>` with intelligent fallback detection based on package manifests (`package.json`, `Cargo.toml`, `pyproject.toml`). Spawn live sessions via `cmd /C start powershell -NoExit -Command ...` on Windows.
- **CLI Commit Popup**: Add `cli/src/commands/commit.ps1` in PowerShell CLI with optional GUI popup fallback (`[Microsoft.VisualBasic.Interaction]::InputBox`) or interactive read prompt.

## Testing Decisions

- **Testing Seams**:
  - **High-level App State Seam (`tui/src/app.rs`)**: Test state transitions, tab navigation, filter cycling, loading flags, and key handling.
  - **Data Inspection Seam (`tui/src/data/scanner.rs`)**: Test project stack parsing and dev command auto-detection.
  - **CLI Command Seam (`cli/src/commands/index.ps1`)**: Test PowerShell CLI command parsing and commit parameter handling.
- **Test Criteria**: Tests verify external behavior and state contracts without locking down private rendering layout coordinates.

## Out of Scope

- Remote Git hosting operations (creating GitHub PRs or managing GitHub action pipelines).
- Full integrated terminal emulator inside Ratatui (external terminal window launch is used for live program execution to ensure full terminal compatibility).

## Further Notes

- Specifications are tracked as local markdown files under `.scratch/todo_v2/spec.md`.
- Implementation plan is located at `C:\Users\devamr\.gemini\antigravity-cli\brain\faef9f94-6e77-49cd-bfb5-e810ac044a24\todo_v2_implementation_plan.md`.
