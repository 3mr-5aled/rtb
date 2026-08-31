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
### 5. Unified Binary Engine (`RtbEngine` / `rtb`)
The high-performance, single standalone compiled binary unifying all CLI operations and TUI dashboards across platforms.
- **Interface**: `rtb <subcommand> [args]` / `rtb ui` / `RtbEngine::dispatch()`.
- **Responsibilities**: Consolidates project lifecycle management, configuration parsing (`rtb.config.json`), agent context launching, and the Ratatui interactive dashboard into a zero-dependency static executable. Eliminates runtime interpreter prerequisites (e.g. `pwsh` on Linux/macOS).

## Installation & Delivery Glossary

**Installation Mode**: Either `repo` (developer running `pwsh -File ./install.ps1` or `sh ./install.sh` from a cloned source tree) or `standalone` (end user piping via `irm | iex` on Windows or `curl | sh` on Unix). Detected automatically.

**Module Home**: The directory where the CLI module (`rtb.psd1`, `rtb.psm1`, `src/`) lives after installation. In standalone mode: `%APPDATA%\rtb\module\` or `~/.config/rtb/module/`. In repo mode: `<repo>/cli/`.

**Release Bundle** (`rtb-cli.zip`): The canonical GitHub Release asset produced by CI. Contains the full CLI module folder, native binary (`rtbtui`), `logo.txt`, and `uninstall.ps1`.

**Unified Binary Engine**: The target architecture consolidating the PowerShell command scripts and Rust TUI into a single static compiled binary (`rtb`), providing native performance on Windows, Linux, and macOS without external runtime requirements. See `docs/adr/0001-unified-native-rust-engine.md`.

**User Configuration**: The `rtb.config.json` file at `%APPDATA%\rtb\rtb.config.json` (Windows) or `~/.config/rtb/rtb.config.json` (Unix). A user is considered **configured** when this file exists and `projectRoots.active.path` is a non-empty string. Can be opened directly in the user's default editor via `rtb config`.

**Config Gate**: The mechanism in `rtb.psm1` that intercepts data-dependent subcommands before execution. If the user is not configured, it prints a message and offers `"Would you like to configure now? (Y/n)"`. Commands exempt from the gate: `help`, `config`, `--version`, `--help`, `init`, `doctor`, `uninstall`.

**Project Root Entry**: A single entry in `projectRoots` in `rtb.config.json`. Structured as `{ path: String, label: String, emoji: String }`. Represents one lifecycle folder (e.g. Active, Paused, Deployed). Replaces the previous flat string schema.

**Workspace Scaffold**: The directory tree created by `rtb init` under the user's chosen root. Folders are selected interactively via a multi-select list; each has a default emoji and label that the user may customize via `rtb config`. `Vibe Coding` is not part of the standard scaffold — it is a user-defined custom folder type.

**Setup Wizard**: The interactive installation flow that collects user decisions (install path, profile targets) and executes installer steps with real-time progress feedback (spinners, colored step labels, a final summary box). Implemented as two entry-point scripts — `install.ps1` (Windows / pwsh) and `install.sh` (Linux / macOS) — that share the same UX conventions. The wizard is distinct from the **Installation Mode** concept: mode is about _where files come from_; the wizard is about _how the user experiences the process_.
