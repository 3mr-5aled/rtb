# ADR 0001: Unified Native Rust Binary Engine

## Status
Accepted (Phase 1 Implemented; Phase 2 Migration Target)

## Context
RTB originated with a split architecture:
- **CLI Engine**: Implemented as a PowerShell script module (`cli/rtb.psm1` and `cli/src/commands/*.ps1`).
- **Interactive TUI**: Implemented in Rust with Ratatui (`tui/`).

While PowerShell provides great convenience for Windows-first scripting, it creates substantial friction on Unix-like operating systems (Linux and macOS):
1. **Runtime Prerequisite**: Unix systems rarely include PowerShell (`pwsh`) by default. Running `install.sh` requires prompting users to install a heavy ~150MB Microsoft runtime with root/sudo privileges via `apt`, `dnf`, or `brew`.
2. **Execution Latency**: Spawning `pwsh` for quick commands (like `rtb status` in shell prompts or `rtb list`) introduces noticeable cold-start latency compared to native binaries.
3. **Packaging Simplicity**: Distributing two separate layers (PowerShell scripts + a compiled Rust TUI binary) complicates installation scripts and release artifacts.

## Decision
We will consolidate all CLI subcommands (`rtb init`, `rtb config`, `rtb list`, `rtb goto`, `rtb run`, `rtb info`, etc.) and the interactive TUI (`rtb ui`) into a single, high-performance, statically-linked **Rust native binary (`rtb`)**.

### Architectural Core
1. **Single Multi-Call Binary (`rtb`)**:
   - `rtb <subcommand>` dispatches fast CLI operations.
   - `rtb ui` (or `rtbtui`) launches the interactive Ratatui terminal dashboard directly within the same process.
2. **Cross-Platform Compatibility**:
   - Compiles to native static binaries for Windows (`x86_64-pc-windows-msvc`), Linux (`x86_64-unknown-linux-gnu`), and macOS (`x86_64-apple-darwin` / `aarch64-apple-darwin`).
   - Zero external runtime dependencies (`pwsh`, Node.js, Python, or .NET not required to run `rtb`).
3. **Phased Migration**:
   - **Phase 1 (Current)**: Refined Setup Wizard (`install.ps1` and `install.sh`) with bug fixes and `rtb config` editor integration.
   - **Phase 2 (Target)**: Port CLI command handlers (`cli/src/commands/*`) into the Rust crate (`tui/src/cli/`).
   - **Phase 3**: Release universal standalone binary distribution where `install.sh` and `install.ps1` only need to download one single binary.

## Consequences

### Positive
- **Zero Prerequisite on Unix/macOS**: Users can install and run RTB with a single `curl | sh` without `pwsh` or `sudo`.
- **Ultra-Fast Prompt & CLI Execution**: Sub-millisecond startup time enables instantaneous prompt segments (`rtb status`) and instant fuzzy navigation (`rtb goto`).
- **Unified Codebase**: Shared data models (`rtb.config.json`, `ProjectInspector`, `AgentOrchestrator`) live in a single type-safe language.
- **Single Release Asset**: Packaging is simplified to per-platform binaries (`rtb-windows-amd64.exe`, `rtb-linux-amd64`, `rtb-macos-amd64`).

### Negative / Trade-offs
- **Rust Development Overhead**: Implementing CLI commands in Rust requires strict type handling and explicit filesystem / process spawning logic compared to dynamic PowerShell scripts.
- **Transitional Dual Maintenance**: During the migration phase, both the PowerShell CLI and Rust implementations must remain aligned until the Rust engine reaches full feature parity.
