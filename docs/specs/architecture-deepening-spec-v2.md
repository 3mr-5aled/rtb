# Architecture Deepening & Seam Consolidation Specification (Phase 2)

## Problem Statement

As RTB has scaled into a multi-runtime developer tool with 34 CLI commands and a Ratatui terminal dashboard, architectural friction has accumulated across three primary areas:

1. **Fragmented Project Lifecycle & Ad-Hoc Target Resolution**: Operations that mutate project lifecycle state (`new`, `pause`, `resume`, `archive`, `unarchive`, `deploy`) are scattered across individual command files that depend horizontally on each other. Target project resolution is duplicated across 9 separate locations, leading to inconsistent matching rules, silent failures, and unhandled CLI flags (such as ignored agent flags in `rtb goto`).
2. **The TUI App God Object & The Abandoned `TabController`**: In the Rust TUI, `App` has expanded into a 1,667-line God Object maintaining 30+ flattened state fields for all six tabs and interactive overlays. The domain-documented `TabController` trait was abandoned because it lacked a mechanism for tabs to emit application-level side effects (modals, toasts, background scans). As a result, five handler files monkey-patch `App` using `impl App` blocks, preventing isolated tab testing and duplicating view-handling logic.
3. **Test-Polluting Command Execution & Paper-Thin Wrappers**: CLI commands invoke `process.exit(1)` directly upon failure. To prevent tests from terminating prematurely, production code was retrofitted with `if (process.env.VITEST) return;` escapes across nine files. Furthermore, multiple command files (`build`, `run`, `test`, `backup`, `guard`, `env`) are paper-thin pass-through wrappers that pass the deletion test with zero loss of functionality.

## Solution

Deepen the architecture around three canonical seams:

1. **Deep Project Lifecycle Engine (`ProjectLifecycle`)**: A single, deep domain module that encapsulates all workspace lifecycle transitions (`create`, `pause`, `resume`, `archive`, `unarchive`, `deploy`), invariant checks (git cleanliness, duplicate naming, path validation), dependency pruning, snapshot compression, and post-restore package manager installation. Individual CLI commands become 10-line input/output adapters.
2. **Action-Dispatched `TabController` Seam in `rtbtui`**: Redesign `TabController` around an action-passing protocol (`handle_key -> AppAction`). Each tab becomes an independent, encapsulated module owning its selection state, filter modes, and keybindings. `App` becomes a pure coordinator that routes top-level hotkeys and dispatches emitted `AppAction` events.
3. **Unified Command Execution Envelope (`CommandEnvelope`)**: Introduce a standardized command execution envelope that intercepts typed domain errors, guarantees a consistent dual-mode JSON/ANSI contract, and assigns `process.exitCode` cleanly without calling `process.exit` or requiring test escapes. Collapse shallow pass-through commands into their respective domain modules (Runner and Maintenance).

## User Stories

1. As a developer using the `rtb` CLI, I want `rtb pause <project>` to safely verify git cleanliness, prune temporary dependency folders, and move the project to Paused without leaving dangling files.
2. As a developer resuming a project with `rtb resume <project> --install`, I want the tool to automatically detect the project's ecosystem (Node.js or Python) and install dependencies cleanly.
3. As an automated script calling `rtb archive <project> --force --json`, I want a standardized JSON payload detailing archive creation and source cleanup, so that CI pipelines receive consistent metadata.
4. As a developer navigating projects with `rtb goto <project> --agy`, I want the CLI to resolve the project and seamlessly launch the specified AI agent upon arrival, rather than dropping the flag.
5. As a developer typing project names with partial or kebab-cased names, I want all commands (`pause`, `resume`, `archive`, `open`, `deps`, `goto`) to resolve projects using the exact same fuzzy matching rules.
6. As a TUI user navigating the Projects, Dep Cleaner, or Git Health tabs, I want keyboard navigation and list scrolling to feel instantaneous and maintain independent selection states across tab switches.
7. As a TUI user pressing `Enter` on a project in the Dashboard, I want it to trigger the exact same editor opening behavior as the Projects tab without duplicated code paths.
8. As a developer adding a new tab to `rtbtui`, I want to implement the `TabController` interface without touching the main `App` struct or editing unrelated handler files.
9. As a developer maintaining `rtbtui`, I want the main application loop to be a lightweight coordinator under 400 lines of code, so that global event routing is transparent and AI-navigable.
10. As a developer running `rtb build <project>`, I want build command resolution and execution to be managed directly by the unified Runner module without jumping through paper-thin command wrappers.
11. As a system administrator running workspace maintenance, I want `rtb maintenance <task>` and individual shortcuts (`rtb backup`, `rtb guard`, `rtb env`) to share identical execution pipelines and safety validations.
12. As a CLI user receiving an error, I want structured JSON output (`{ success: false, error: { code, message } }`) whenever `--json` is passed, regardless of which subcommand failed.
13. As a test writer, I want to test project lifecycle transitions directly against temporary directory fixtures without spawning child processes or parsing Commander CLI arguments.
14. As a test writer, I want to test tab keyboard navigation and filter cycling by passing synthetic input events to the tab controller and asserting on the returned `AppAction`, without loading disk configurations or scanning system network ports.
15. As a developer running the unit test suite, I want zero production files to contain `if (process.env.VITEST) return;` or process termination escapes.
16. As an AI assistant reading the codebase, I want commands to depend vertically on deep domain modules rather than horizontally importing helper functions from sibling command files.

## Implementation Decisions

### Decision 1: Project Lifecycle Domain Seam (`ProjectLifecycle`)
- Create a deep `ProjectLifecycle` domain module responsible for all workspace directory mutations and lifecycle transitions.
- The module encapsulates:
  - Canonical project target resolution using fuzzy matching and exact path verification across all configured roots.
  - Name sanitization and kebab-case transformation.
  - Pre-flight safety checks (verifying working trees are clean via git status before destructive or move operations unless overridden).
  - Destination collision detection.
  - Recursive dependency and build artifact pruning (`node_modules`, `.venv`, `.next`, `target`, `dist`, `build`).
  - Snapshot compression into timestamped archives (`.tar.gz`).
  - Post-restore package manager installation (`npm install`, `pip install`).
- Existing command files (`new`, `pause`, `resume`, `archive`, `unarchive`, `deploy`) become thin adapters whose sole responsibility is declaring CLI options, calling `ProjectLifecycle`, and formatting responses.
- Horizontal dependencies between sibling command files are completely eliminated.

### Decision 2: Action-Dispatched `TabController` Protocol in Rust TUI
- Redefine `TabController` as an active trait at the view seam that communicates back to the application coordinator via an action-passing protocol.
- Prototype action enum:
  ```rust
  pub enum AppAction {
      None,
      SwitchTab(Tab),
      OpenModal(ModalKind),
      CloseModal,
      ShowToast(String, ToastLevel),
      StartScan(&'static str),
      OpenEditor(PathBuf),
      OpenExplorer(PathBuf),
      ExecuteCommand(String, Vec<String>),
      Quit,
  }
  ```
- Each tab view (`ProjectsTab`, `GitHealthTab`, `DepCleanerTab`, `MaintenanceTab`, `DevPortsTab`, `DashboardTab`) becomes an independent struct owning its selection indices, search filters, and threshold configurations.
- The main `App` coordinator routes top-level keybindings (quitting, global tab cycling, command palette) first. If not handled, it delegates key events to the active `TabController` and executes the returned `AppAction`.
- The five monkey-patch handler modules (`handlers/cleaner.rs`, `handlers/git_health.rs`, `handlers/maintenance.rs`, `handlers/ports.rs`, `handlers/projects.rs`) are removed.

### Decision 3: Command Execution Envelope (`CommandEnvelope`)
- Wrap all CLI command action handlers in a deep execution envelope.
- The envelope intercepts thrown domain errors (e.g. `ProjectNotFoundError`, `DirtyGitError`, `ConfigMissingError`).
- In JSON mode, it formats errors as `{ error: true, code: string, message: string }`. In human-readable mode, it formats errors with standard ANSI styling.
- Sets `process.exitCode = 1` rather than calling `process.exit()`, completely removing `if (process.env.VITEST) return;` escapes from production files.
- The paper-thin runner commands (`build`, `run`, `test`) are registered directly from the Runner domain module.
- The maintenance command shortcuts (`backup`, `guard`, `env`) are registered directly as aliases to the Maintenance Task Registry.

### Decision 4: Wire Agent Flags in Navigation Engine
- Update the navigation engine so that agent flags passed to `rtb goto` (`--agy`, `--claude`, `--gemini`, `--cursor`, etc.) trigger the Agent Orchestrator to generate the `.rtb_context.md` project context and launch the agent process upon directory arrival.

## Testing Decisions

### What Makes a Good Test
- Tests must target external module interfaces at the defined seams, asserting on observable return values and filesystem outcomes rather than internal helper methods or intermediate private state.
- Tests should not mock the runtime environment (no spying on `process.exit`, no synthetic process environment overrides).

### Tested Modules & Seams
1. **`ProjectLifecycle` Seam**: Tested directly using isolated temporary directory fixtures. Tests verify creation, pausing (with and without dependency pruning), resuming (with package manager detection), archiving (with archive creation and directory deletion), and deploy promotion.
2. **`TabController` Seam**: Tested by feeding synthetic `KeyCode` and `KeyModifiers` events to tab structs and asserting on the returned `AppAction` enum and internal selection state transitions, completely isolated from terminal backends and system processes.
3. **`CommandEnvelope` Seam**: Tested by invoking the envelope with failing and succeeding actions, verifying correct JSON serialization, error code formatting, and non-zero exit code setting without process termination.

### Prior Art
- TypeScript lifecycle tests in `core/tests/lifecycle.test.ts`.
- Command runner tests in `core/tests/commands-runner.test.ts`.
- TUI session state tests in `tui/src/app.rs`.

## Out of Scope

- Rewriting the underlying Ratatui terminal rendering backend or Crossterm event loop.
- Modifying the core JSON schema emitted by `rtb list --json`.
- Modifying external system maintenance scripts (`D:\06-Tools\scripts\...`).
- Rewriting shell installers (`install.ps1`, `install.sh`) into Node.js (Candidate 5 remains speculative).

## Implementation Tickets

The specification is broken down into 5 tracer bullets tracked in GitHub Issues:

1. **Ticket 1 (#57)**: [`feat(cli): implement CommandEnvelope and eliminate process.exit test escapes`](https://github.com/3mr-5aled/rtb/issues/57) *(Blocked by: None)*
2. **Ticket 5 (#58)**: [`feat(tui): resurrect TabController seam using action-passing event loop`](https://github.com/3mr-5aled/rtb/issues/58) *(Blocked by: None)*
3. **Ticket 2 (#59)**: [`feat(core): implement ProjectLifecycle engine with core transitions (new, pause, resume)`](https://github.com/3mr-5aled/rtb/issues/59) *(Blocked by: #57)*
4. **Ticket 3 (#60)**: [`feat(core): extend ProjectLifecycle engine with snapshots and promotion (archive, unarchive, deploy)`](https://github.com/3mr-5aled/rtb/issues/60) *(Blocked by: #59)*
5. **Ticket 4 (#61)**: [`fix(cli): unify target resolution across commands and wire agent flags in goto`](https://github.com/3mr-5aled/rtb/issues/61) *(Blocked by: #59)*
