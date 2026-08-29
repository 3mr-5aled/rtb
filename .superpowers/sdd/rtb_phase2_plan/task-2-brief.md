# Task 2 Brief: AI Agent Discovery & CLI Launcher (`rtb agent`)

## Requirements

1. **Agent Discovery Engine (`tui/src/data/agents.rs` & `cli/src/commands/agent.ps1`):**
   - Implement agent CLI discovery looking for executables in system PATH: `agy` (Google Antigravity), `claude` (Claude Code), `gemini` (Gemini CLI), `codex` (Codex CLI).
   - Return list of installed agents with status (`installed: bool`, `command: String`, `name: String`).

2. **PowerShell CLI Cmdlet `Rtb-Agent` (`cli/src/commands/agent.ps1`):**
   - Cmdlet signature: `Rtb-Agent [-ProjectName <String>] [-Agent <String>]`
   - If `ProjectName` is specified, resolve project path. Default to current directory project if omitted.
   - If `Agent` is omitted, default to `agy` if available, or the first available installed agent.
   - Generate project context summary (Project Name, Stack, Git Branch, Status, README overview).
   - Launch the target agent CLI process in the project directory passing context.
   - Support `rtb agent` in `cli/rtb.psm1` dispatcher and tab completions (`cli/src/completions/rtb.completion.ps1`).

3. **TUI Integration (`tui/src/data/agents.rs` & `tui/src/app.rs`):**
   - Add shortcut `a` in TUI project list to launch selected project in default AI Agent.

4. **Testing & Report:**
   - Create Pester test suite `cli/tests/agent.tests.ps1` testing agent discovery and command dispatching.
   - Write full task report to `D:\02-Projects\01-Development\01-Active\rtb-command-tool\.superpowers\sdd\rtb_phase2_plan\task-2-report.md`.
