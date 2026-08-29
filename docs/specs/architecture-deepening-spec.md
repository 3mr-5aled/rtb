# Architecture Deepening & Seam Refactoring Specification

## Problem Statement

As RTB has expanded to support multi-runtime intelligence, TUI dashboard views, dependency pruning, and AI agent discovery, key modules have become shallow or duplicated:
1. Multi-runtime detection logic is duplicated across PowerShell CLI helper scripts and Rust TUI scanners, leading to maintenance drift.
2. The TUI application (`app.rs`) has grown into a 1,500+ line god object that routes events and manages state for 6 different tabs and multiple modal overlays, degrading locality.
3. System maintenance commands are paper-thin pass-through wrappers delegating to hardcoded absolute system paths, breaking execution portability.
4. AI Agent discovery and process launching mix PATH lookups, process spawning, and context generation directly within host command scripts.

## Solution

Restructure shallow modules into deep, encapsulated modules behind four clean domain seams:
1. **Project Intelligence Engine (`ProjectInspector`)**: A unified multi-runtime inspector that encapsulates filesystem traversal, lockfile parsing, AST evaluation, and git telemetry behind a single JSON-schema contract (`rtb list --json`).
2. **Tab Controller Seam (`TabController`)**: A modular trait interface for TUI views where each tab encapsulates its own key handling, selection state, and rendering, leaving `App` as a lightweight event router.
3. **Maintenance Task Registry (`MaintenanceTaskRegistry`)**: A portable engine that resolves maintenance scripts dynamically from `rtb.config.json` (with relative repo fallbacks) and enforces safety guard invariants.
4. **Agent Orchestrator (`AgentOrchestrator`)**: An engine for discovering installed AI agent CLIs (`agy`, `claude`, `gemini`, `codex`), generating transient project context summaries (`.rtb_context.md`), and managing cross-platform process execution.

## User Stories

1. As a developer using the `rtb` CLI, I want `rtb list --json` to return rich, consistent project intelligence across all supported languages (Node.js, Rust, Python, Go, Java, .NET) so that automated tooling receives uniform metadata.
2. As a developer maintaining the codebase, I want framework stack detection rules to live in a single canonical engine so that adding support for a new technology requires editing only one module.
3. As a TUI user, I want individual tabs (Projects, Dep Cleaner, Maintenance) to respond instantly to keyboard inputs without key event logic bleeding across tab views, so that UI navigation feels robust and maintainable.
4. As a developer creating new TUI features, I want to implement a single `TabController` interface so that I can add new tabs without modifying `app.rs`.
5. As a system administrator, I want maintenance tasks (`guard`, `backup`, `env`, `maintenance`) to resolve script paths from `rtb.config.json` with relative fallback paths, so that maintenance scripts run portably on any developer machine.
6. As a user running `rtb clean`, I want safety guard rules to automatically protect active and production workspace roots, so that dependencies are never accidentally pruned from active projects.
7. As an AI-assisted developer running `rtb agent`, I want the launcher to automatically generate a transient `.rtb_context.md` project summary before launching `agy` or `claude`, so that the AI agent starts with instant codebase context.
8. As a test writer, I want to test project scanning, tab navigation, and agent context generation without executing real OS shell scripts or spawning terminal subprocesses.

## Implementation Decisions

- **Domain Language Alignment**: All proposed refactors use terms strictly from `CONTEXT.md` (`ProjectInspector`, `TabController`, `MaintenanceTaskRegistry`, `AgentOrchestrator`).
- **Canonical Intelligence Core**: The Rust scanner and `rtb list --json` CLI output serve as the single source of truth for project metadata. PowerShell CLI helper functions consume this JSON schema when project details are requested.
- **Top-Level TUI Overlay Routing**: Global interactive overlays (CommandPalette, ToastQueue) remain managed by `App` at the top level for cross-tab accessibility, while tab views implement the `TabController` trait.
- **Dynamic Script Resolution**: Maintenance task scripts resolve dynamically in order of priority: (1) `rtb.config.json` overrides, (2) relative repository scripts directory (`cli/scripts/`), (3) system PATH.
- **Transient Context Payload**: The Agent Orchestrator generates a transient `.rtb_context.md` file in the target project directory upon launch containing stack details, git status, and README summary.

## Testing Decisions

- **Test Seam Principle**: Tests target only external interfaces at the 4 defined seams (`ProjectInspector`, `TabController::handle_key`, `Invoke-MaintenanceTask`, `AgentOrchestrator::launch`), avoiding assertions on internal private helpers.
- **Mock File System Fixtures**: `ProjectInspector` and `DependencyCleaner` are tested against isolated temporary directory trees with dummy lockfiles and package manifests.
- **Synthetic Input Event Testing**: `TabController` implementations are tested by passing synthetic `KeyEvent` structs directly to `handle_key` to verify state transitions without launching ratatui terminal backends.
- **Prior Art**: Extends existing Pester tests (`cli/tests/info.tests.ps1`, `cli/tests/agent.tests.ps1`) and Rust Cargo unit tests (`tui/src/data/scanner.rs`).

## Out of Scope

- Modifying the underlying Ratatui terminal rendering backend or Crossterm event loop.
- Rewriting third-party maintenance scripts (`D:\06-Tools\scripts\...`) themselves.
- Adding web-based or GUI interfaces beyond the existing CLI and TUI.

## Further Notes

Note: If you have an external issue tracker configured (e.g. GitHub Issues or Jira), run `/setup-matt-pocock-skills` to enable direct issue creation.
