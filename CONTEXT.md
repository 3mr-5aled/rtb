# RTB (رتّب) Domain Model & Architectural Glossary

This document records the domain concepts and module definitions for the RTB Repository & Tooling Base.

## Core Domain Modules

### 1. Project Intelligence Engine (`ProjectInspector`)
The unified scanner and metadata inspector responsible for multi-runtime project discovery.
- **Interface**: `Inspect-Project -Path <String>` (PowerShell) / `ProjectInspector::inspect(path: &Path)` (Rust TUI).
- **Responsibilities**: Recursively parses package lockfiles (`pnpm`, `yarn`, `bun`, `npm`, `Cargo.toml`, `pyproject.toml`, `go.mod`), detects framework stacks (Next.js, React, Vue, Vite, Tailwind, Prisma, Express), builds monorepo workspace graphs, extracts runtime versions (`.nvmrc`, `rust-toolchain.toml`, `.python-version`), and gathers git telemetry.
- **Canonical Output**: Standardized JSON schema contract (`rtb list --json`).

### 2. Tab Controller (`TabController`)
The TUI view encapsulation seam.
- **Interface**: `TabController` trait (`handle_key`, `update`, `render`).
- **Responsibilities**: Encapsulates selection state, scroll positions, search filters, and tab-specific keyboard bindings for each Ratatui tab view (`DashboardTab`, `ProjectsTab`, `GitHealthTab`, `DepCleanerTab`, `MaintenanceTab`, `DevPortsTab`).
- **Seam Boundary**: `App` routes top-level events and global modals (CommandPalette, Toasts) to active `TabController` instances.

### 3. Maintenance Task Registry (`MaintenanceTaskRegistry`)
Centralized execution and safety engine for system operations.
- **Interface**: `Invoke-MaintenanceTask` (PowerShell) / `MaintenanceTaskRegistry::execute` (Rust).
- **Responsibilities**: Resolves task scripts from `rtb.config.json` (with relative fallbacks in `cli/scripts/`), validates execution safety flags, manages dry-run modes, streams process execution logs, and enforces workspace guard rules.

### 4. Agent Orchestrator (`AgentOrchestrator`)
AI agent discovery, context generation, and process execution engine.
- **Interface**: `AgentOrchestrator::launch(agent_id, project_path)` / `Rtb-Agent`, `rtb <agent-shorthand>`, `rtb goto --<agent>`.
- **Responsibilities**: Discovers installed AI agent CLIs (`agy`, `claude`, `gemini`, `codex`, `cursor`, `windsurf`, `aider`, `openhands`) in `PATH`, auto-generates transient project context payloads (`.rtb_context.md`), maps agent shorthand commands and `--<agent>` flags, and manages cross-platform process spawning.

## Installation & Delivery Glossary

**Installation Mode**: Either `repo` (developer running `pwsh -File ./install.ps1` from a cloned source tree) or `standalone` (end user piping `install.ps1` via `irm | iex`). Detected automatically: if `$PSScriptRoot` is empty or contains no `cli\` subfolder → standalone; otherwise → repo.

**Module Home**: The directory where the PowerShell CLI module (`rtb.psd1`, `rtb.psm1`, `src/`) lives after installation. In standalone mode: `%APPDATA%\rtb\module\`. In repo mode: `<repo>/cli/`. The `$PROFILE` `Import-Module` line always points here.

**Release Bundle** (`rtb-cli.zip`): The canonical GitHub Release asset produced by CI. Contains the full CLI module folder, `rtbtui.exe`, `logo.txt`, and `uninstall.ps1`. This is the only artifact a standalone installer downloads.

**User Configuration**: The `rtb.config.json` file at `~/.config/rtb/rtb.config.json` (`%USERPROFILE%\.config\rtb\rtb.config.json` on Windows, `$HOME/.config/rtb/rtb.config.json` on Unix). A user is considered **configured** when this file exists and `projectRoots.active.path` is a non-empty string.

**Config Gate**: The mechanism in `rtb` that intercepts data-dependent subcommands before execution. If the user is not configured, it prints a message and offers `"Would you like to configure now? (Y/n)"`. Commands exempt from the gate: `help`, `--version`, `--help`, `init`, `config`, `doctor`, `shell-init`, `uninstall`.

**Project Root Entry**: A single entry in `projectRoots` in `rtb.config.json`. Structured as `{ path: String, label: String, emoji: String }`. Represents one lifecycle folder (e.g. Active, Paused, Deployed). Replaces the previous flat string schema.

**Workspace Scaffold**: The directory tree created by `rtb init` under the user's chosen root. Folders are selected interactively via a multi-select list; each has a default emoji and label that the user may customize. `Vibe Coding` is not part of the standard scaffold — it is a user-defined custom folder type.

**Setup Wizard**: The interactive installation flow that collects user decisions (install path, profile targets) and executes installer steps with real-time progress feedback (spinners, colored step labels, a final summary box). Implemented as two entry-point scripts — `install.ps1` (Windows / pwsh) and `install.sh` (Linux / macOS) — that share the same UX conventions. The wizard is distinct from the **Installation Mode** concept: mode is about _where files come from_; the wizard is about _how the user experiences the process_.
