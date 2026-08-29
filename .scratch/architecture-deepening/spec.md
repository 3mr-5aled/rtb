# Spec: Architecture Deepening & Seam Refactoring

## Problem Statement

As RTB has grown to encompass multi-runtime project intelligence, TUI dashboard views, dependency pruning, and AI agent discovery, critical core modules have become shallow or duplicated across client implementations:
1. Multi-runtime detection logic (Node.js, Rust, Python, Go, Java, .NET) is independently reimplemented in PowerShell CLI scripts and Rust TUI scanners, causing maintenance drift.
2. The TUI application (`app.rs`) has expanded into a 1,500+ line god object that routes events, manages state, and renders views for 6 distinct tabs and multiple modal overlays, degrading locality.
3. System maintenance commands consist of shallow pass-through wrappers that delegate to hardcoded absolute system paths, breaking portable execution across developer machines.
4. AI Agent discovery and process launching mix PATH lookups, process spawning, and context generation directly inside host command functions.

## Solution

Refactor shallow modules into deep, encapsulated modules centered around four clean domain seams:
1. **Project Intelligence Engine (`ProjectInspector`)**: A unified multi-runtime inspector that encapsulates filesystem traversal, lockfile parsing, AST evaluation, and git telemetry behind a canonical JSON-schema contract (`rtb list --json`).
2. **Tab Controller Seam (`TabController`)**: A modular trait interface for TUI views where each tab encapsulates its own key handling, selection state, and rendering, leaving `App` as a lightweight event router.
3. **Maintenance Task Registry (`MaintenanceTaskRegistry`)**: A portable engine that resolves maintenance scripts dynamically from `rtb.config.json` (with relative repo fallbacks) and enforces safety guard invariants.
4. **Agent Orchestrator (`AgentOrchestrator`)**: An engine for discovering installed AI agent CLIs (`agy`, `claude`, `gemini`, `codex`), generating transient project context summaries (`.rtb_context.md`), and managing cross-platform process execution.

## User Stories

1. As a CLI user running `rtb list --json`, I want structured JSON output describing all discovered project stacks, so that external scripts receive uniform metadata regardless of project runtime.
2. As a maintainer adding support for a new framework or runtime, I want to edit a single `ProjectInspector` module so that stack taxonomy rules never drift between CLI and TUI clients.
3. As a TUI user navigating project lists, I want keyboard shortcuts (up, down, filter, sort) to be handled exclusively by the active tab view, so that UI interaction remains responsive and isolated.
4. As a TUI developer creating a new view tab, I want to implement the `TabController` trait interface, so that I can add new tabs without modifying `App` state routing code.
5. As a developer running maintenance commands (`rtb maintenance`, `rtb guard`, `rtb backup`, `rtb env`), I want script paths to resolve dynamically from `rtb.config.json` with relative repository fallbacks, so that commands run portably on any system.
6. As a developer executing `rtb clean`, I want safety guard rules to automatically protect active and production workspace roots, so that dependency folders in active projects are never accidentally deleted.
7. As an AI-assisted developer invoking `rtb agent`, I want the orchestrator to automatically generate a transient `.rtb_context.md` project context summary before launching `agy` or `claude`, so that the target AI agent has instant workspace context.
8. As an engineer writing unit tests, I want to test project scanning, tab navigation, maintenance script resolution, and agent context assembly without executing real OS shell scripts or spawning terminal subprocesses.

## Implementation Decisions

- **Domain Glossary Alignment**: All module definitions follow the terms defined in `CONTEXT.md` (`ProjectInspector`, `TabController`, `MaintenanceTaskRegistry`, `AgentOrchestrator`).
- **Single Source of Truth**: The Rust scanner and `rtb list --json` CLI output serve as the canonical project metadata inspector. PowerShell CLI helper functions consume this JSON schema when project details are requested.
- **Top-Level Overlay Management**: Global interactive overlays (CommandPalette, ToastQueue) remain managed by `App` at the top level for cross-tab accessibility, while tab views implement the `TabController` trait.
- **Dynamic Task Resolution**: Maintenance task scripts resolve in priority order: (1) `rtb.config.json` overrides, (2) relative repository scripts directory (`cli/scripts/`), (3) system PATH.
- **Transient Context Payload**: The Agent Orchestrator generates a transient `.rtb_context.md` file in the target project directory upon launch containing stack details, git status, and README summary.

## Testing Decisions

- **Test Seam Principle**: Tests target only external interfaces at the 4 defined seams (`ProjectInspector`, `TabController::handle_key`, `Invoke-MaintenanceTask`, `AgentOrchestrator::launch`), avoiding assertions on internal private helpers.
- **Mock Filesystem Trees**: `ProjectInspector` and `DependencyCleaner` are tested against isolated temporary directory trees with dummy lockfiles and package manifests.
- **Synthetic Input Event Testing**: `TabController` implementations are tested by passing synthetic `KeyEvent` structs directly to `handle_key` to verify state transitions without launching ratatui terminal backends.
- **Prior Art**: Extends existing Pester tests (`cli/tests/info.tests.ps1`, `cli/tests/agent.tests.ps1`) and Rust Cargo unit tests (`tui/src/data/scanner.rs`).

## Out of Scope

- Modifying the underlying Ratatui terminal rendering backend or Crossterm event loop.
- Rewriting third-party maintenance scripts (`D:\06-Tools\scripts\...`) themselves.
- Adding web-based or GUI interfaces beyond the existing CLI and TUI.

## Further Notes

- Triage label applied: `ready-for-agent`
- Issue location: `.scratch/architecture-deepening/spec.md`
