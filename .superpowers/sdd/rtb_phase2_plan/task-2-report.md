# Task 2 Report: AI Agent Discovery & CLI Launcher (`rtb agent`)

**Task Name:** AI Agent Discovery & CLI Launcher (`rtb agent`)  
**Plan:** RTB Phase 2 Plan  
**Status:** COMPLETED  
**Date:** 2026-08-28  

---

## Executive Summary

Task 2 implements AI Agent discovery and dispatching capabilities for RTB, supporting CLI discovery for Google Antigravity (`agy`), Claude Code (`claude`), Gemini CLI (`gemini`), and Codex CLI (`codex`). Users can discover installed AI agents, pass project context summaries, and launch targeted AI agents via `rtb agent` CLI cmdlet or `a` keybinding in the TUI interface.

---

## Work Accomplished

### 1. Agent Discovery Engine
- **PowerShell Helper (`cli/src/commands/agent.ps1`)**:
  - Implemented `Get-InstalledAgents` returning structured status objects for `agy`, `claude`, `gemini`, and `codex`.
  - Performs PATH lookup using `Get-Command` with silent error handling.
- **Rust Discovery Engine (`tui/src/data/agents.rs`)**:
  - Created `AgentInfo` struct with `name`, `command`, and `installed` status fields.
  - Implemented `is_command_installed` to scan system PATH and Windows executable extensions (`.exe`, `.cmd`, `.bat`, `.ps1`).
  - Implemented `get_installed_agents()` and `get_default_agent()` prioritizing `agy` if present, falling back to the first available installed agent.

### 2. PowerShell CLI Cmdlet & Integration
- **Cmdlet `Rtb-Agent` (`cli/src/commands/agent.ps1`)**:
  - Signature: `Rtb-Agent [-ProjectName <String>] [-Agent <String>] [-List]`.
  - Supports `-List` flag to view installed status of all supported AI agents.
  - Resolves target project using `Find-ProjectPath` or defaults to current directory.
  - Resolves target agent using `-Agent` parameter or defaults to `agy`/first installed agent.
  - Displays structured project context summary (Name, Path, Stack, Git Branch, Status, README overview).
  - Launches agent CLI process in target project directory.
  - Aliased as `Dev-Agent`.
- **Dispatcher & Completions (`cli/rtb.psm1`, `cli/src/completions/`)**:
  - Registered `agent` case in `rtb` switch dispatcher in `cli/rtb.psm1`.
  - Exported `Rtb-Agent`, `Dev-Agent`, and `Get-InstalledAgents`.
  - Added `agent` subcommand and argument completions in `rtb.completion.ps1` and `dev.completion.ps1`.

### 3. TUI Integration & Keybindings
- **TUI Integration (`tui/src/app.rs`, `tui/src/ui/projects.rs`, `tui/src/ui/help.rs`)**:
  - Wired `a` shortcut in TUI Projects view to launch the selected project in the default AI agent.
  - Updated Archive project keybinding to `A` (Shift+A) for consistency with capitalized action shortcuts (`N`, `E`, `D`).
  - Added `🤖 [a] Agent` indicator to Projects view action bar and updated help modal documentation.

### 4. Testing & Verification
- **PowerShell Pester Test Suite (`cli/tests/agent.tests.ps1`)**:
  - Created unit tests verifying `Get-InstalledAgents`, `Rtb-Agent -List`, non-existent project error handling, and invalid agent validation.
  - Verified 4/4 passing tests in `agent.tests.ps1` and 9/9 passing tests across full `rtb test` suite.
- **Rust Unit Tests (`tui/src/data/agents.rs`)**:
  - Added unit tests for `get_installed_agents` and `is_command_installed`.

---

## File Changes Summary

| File | Status | Description |
|---|---|---|
| `cli/src/commands/agent.ps1` | Created | Implemented `Get-InstalledAgents`, `Rtb-Agent`, and `Dev-Agent` |
| `cli/tests/agent.tests.ps1` | Created | Pester unit test suite for agent discovery and launch validation |
| `cli/rtb.psm1` | Modified | Registered `agent` command case and exported functions |
| `cli/src/commands/test.ps1` | Modified | Added `cli/tests` path support for `rtb test` |
| `cli/src/completions/rtb.completion.ps1` | Modified | Added `agent` subcommand and agent parameter completions |
| `cli/src/completions/dev.completion.ps1` | Modified | Added `agent` subcommand and agent completions |
| `tui/src/data/agents.rs` | Created | Implemented Rust agent discovery engine, launcher, and unit tests |
| `tui/src/data/mod.rs` | Modified | Exported `agents` module |
| `tui/src/app.rs` | Modified | Wired `a` keybinding to AI agent launcher |
| `tui/src/ui/projects.rs` | Modified | Added `🤖 [a] Agent` to action bar |
| `tui/src/ui/help.rs` | Modified | Documented `a` AI Agent launcher shortcut |

---

## Verification Evidence

```powershell
Describing AI Agent Discovery & CLI Launcher (Rtb-Agent)
 [+] Get-InstalledAgents returns array with expected agent objects and properties 418ms
 [+] Rtb-Agent -List returns list of agents 163ms
 [+] Rtb-Agent displays error message when non-existent project is specified 213ms
 [+] Rtb-Agent displays error when invalid agent is specified 122ms

Describing Get-RtbConfig
 [+] Loads rtb.config.json from user config directory or fallback repository config 276ms
 [+] Exposes projectRoots object with active path 10ms

Describing Extended Project Intelligence & CLI --json
 [+] Detects .NET stack, Monorepo, CI/CD, and Runtime version in Get-ProjectDetails 190ms
 [+] Rtb-List outputs valid JSON array when --json flag is passed 19.19s
 [+] Rtb-Info returns detailed metadata object when --json flag is passed 272ms

Tests completed in 20.86s
Passed: 9 Failed: 0 Skipped: 0 Pending: 0 Inconclusive: 0
```

---

## Conclusion

Task 2 is fully implemented, verified with Pester tests, and ready for integration.
