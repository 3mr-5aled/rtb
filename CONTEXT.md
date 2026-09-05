# RTB (رتّب) Domain Model & Architectural Glossary

This document records the domain concepts and module definitions for the RTB Repository & Tooling Base.

## Core Domain Modules

### 1. Project Intelligence Engine (`ProjectInspector`)
The unified scanner and metadata inspector responsible for multi-runtime project discovery.
- **Interface**: `ProjectInspector.inspect(projectPath)` (TypeScript/Node.js) / `Inspect-Project -Path <String>` (PowerShell) / `ProjectInspector::inspect(path: &Path)` (Rust TUI).
- **Responsibilities**: Recursively parses package lockfiles (`pnpm`, `yarn`, `bun`, `npm`, `Cargo.toml`, `pyproject.toml`, `go.mod`), detects framework stacks (Next.js, React, Vue, Vite, Tailwind, Prisma, Express), builds monorepo workspace graphs, extracts runtime versions (`.nvmrc`, `rust-toolchain.toml`, `.python-version`), and gathers git telemetry.
- **Canonical Output**: Standardized JSON schema contract (`rtb list --json`).

### 2. Tab Controller (`TabController`)
The TUI view encapsulation seam.
- **Interface**: `TabController` trait (`handle_key`, `update`, `render`).
- **Responsibilities**: Encapsulates selection state, scroll positions, search filters, and tab-specific keyboard bindings for each Ratatui tab view (`DashboardTab`, `ProjectsTab`, `GitHealthTab`, `DepCleanerTab`, `MaintenanceTab`, `DevPortsTab`).
- **Seam Boundary**: `App` routes top-level events and global modals (CommandPalette, Toasts) to active `TabController` instances.

### 3. Maintenance Task Registry (`MaintenanceTaskRegistry`)
Centralized execution and safety engine for system operations.
- **Interface**: `MaintenanceTaskRegistry` (`core/src/services/maintenance.ts`, `rtb maintenance`) / `MaintenanceTaskRegistry::execute` (Rust).
- **Responsibilities**: Registers and executes modular workspace maintenance routines (guard, backup, env), provides `--full` and `--json` automation output, validates execution safety, and enforces workspace guard rules.

### 4. Agent Orchestrator (`AgentOrchestrator`)
AI agent discovery, context generation, and process execution engine.
- **Interface**: `AgentOrchestrator` (`core/src/commands/agent.ts`, `core/src/agent/context.ts`) / `AgentOrchestrator::launch(agent_id, project_path)` / `Rtb-Agent`, `rtb <agent-shorthand>`, `rtb goto --<agent>`.
- **Responsibilities**: Discovers installed AI agent CLIs (`agy`, `claude`, `gemini`, `codex`, `cursor`, `windsurf`, `aider`, `openhands`) in `PATH`, auto-generates transient project context payloads (`.rtb_context.md`), maps agent shorthand commands and `--<agent>` flags, and manages cross-platform process spawning.

### 5. Shell Integration Engine (`ShellIntegration`)
Cross-shell environment integration and directory switching engine.
- **Interface**: `rtb shell-init <bash|zsh|fish|pwsh>`.
- **Responsibilities**: Generates shell wrapper functions capturing `rtb goto` target paths to change the parent shell's current working directory natively across Unix and Windows shells.

## Installation & Delivery Glossary

**Installation Mode**: Either `repo` (developer running from a cloned source tree) or `standalone` (end user piping `install.ps1` or `install.sh`). Detected automatically: if source directories exist → repo mode; otherwise downloads release assets from GitHub Releases.

**Core CLI Runtime**: Node.js (>= 18) pure ESM distribution bundle (`rtb-cli.js` / `@3mr5aled/rtb` compiled from `core/`). Installed to `$RTB_DIR/lib/rtb.js` on Unix or `$script:scriptsDir\rtb.js` on Windows with native shell wrappers (`rtb`, `rtb.cmd`, `rtb.ps1`).

**Module Home**: The directory where the CLI files live after installation (`~/.config/rtb` or `$APPDATA\rtb`).

**Release Bundle** (`rtb-cli.zip` & `rtb-cli.js`): The canonical GitHub Release assets produced by CI. Contains the compiled Node.js CLI bundle, `rtbtui` binary, `logo.txt`, and uninstaller.

**User Configuration**: The unified `rtb.config.json` file at `~/.config/rtb/rtb.config.json` (`%USERPROFILE%\.config\rtb\rtb.config.json` on Windows, `$HOME/.config/rtb/rtb.config.json` on macOS and Linux). A user is considered **configured** when this file exists and `projectRoots.active.path` is a non-empty string.

**Config Gate**: The middleware in `rtb` that intercepts data-dependent subcommands before execution. If the user is not configured, it prints a message and offers `"Would you like to configure now? (Y/n)"`. Commands exempt from the gate: `help`, `--version`, `--help`, `init`, `config`, `doctor`, `shell-init`, `uninstall`, `upgrade`.

**Project Root Entry**: A single entry in `projectRoots` in `rtb.config.json`. Structured as `{ path: String, label: String, emoji: String }`. Represents one lifecycle folder (e.g. Active, Paused, Deployed). Replaces the previous flat string schema.

**Workspace Scaffold**: The directory tree created by `rtb init` under the user's chosen root. Folders are selected interactively via a multi-select list; each has a default emoji and label that the user may customize.

**Setup Wizard**: The interactive installation flow that collects user decisions (install path, shell hooks) and executes installer steps with real-time progress feedback. Implemented via `install.ps1` (Windows) and `install.sh` (POSIX Linux/macOS) with zero PowerShell prerequisite on Unix.
