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
