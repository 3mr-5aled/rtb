# Specification: Phase 4 — Primary npm Distribution, TUI Controller Consolidation, and Open Source Readiness

## Problem Statement

RTB (رتّب) is a TypeScript/Node.js application at its core, requiring Node.js (>= 18) to execute any CLI command. However, the current release and onboarding flow presents OS-specific PowerShell and shell download scripts (`install.ps1`, `install.sh`) as the primary installation method. These scripts download standalone bundles and generate wrapper scripts that invoke `node rtb.js`. 

Since Node.js is already an absolute runtime prerequisite, requiring developers to run curl or PowerShell scripts adds friction, PATH permission nuances, and platform divergence. In modern developer workflows, developers expect **`npm install -g @3mr-5aled/rtb`** or zero-install **`npx @3mr-5aled/rtb`** to be the **primary** installation method, while retaining standalone shell scripts as a secondary fallback.

Simultaneously, two architectural debts must be resolved before general release:
1. **Incomplete TUI Controller Seam & Monolithic Coordinator**: While Phase 2 resurrected the `TabController` trait, `tui/src/app.rs` remains an oversized God Object (~1,700 lines). Tab-specific state is still flattened on `App`, and five monkey-patched handler modules remain in place.
2. **Absence of Community & Contribution Infrastructure**: The repository lacks formal contribution guidelines (`CONTRIBUTING.md`), detailed licensing/liability disclaimers for automated workspace file mutations, and structured onboarding documentation.

## Solution

Address these requirements across three cohesive architectural seams, elevating npm/npx to the primary installation tier:

1. **Primary npm Distribution (`npm install -g @3mr-5aled/rtb` & `npx`)**:
   - Establish `@3mr-5aled/rtb` as the primary, official installation method.
   - Configure the package manifest with executable binary mapping (`"bin": { "rtb": "./dist/index.js" }`), bundle-only packaging (`"files": ["dist"]`), engine compatibility (`"node": ">=18.0.0"`), and comprehensive repository metadata.
   - Provide self-provisioning in `rtb ui`: when invoked from an npm installation without the Rust `rtbtui` binary present, automatically detect the OS and architecture, prompt to download the precompiled binary from GitHub Releases into `~/.config/rtb/bin/`, and launch transparently.
   - Reposition `install.ps1` and `install.sh` as secondary standalone installers for environments without global npm access.
2. **Complete TUI Controller Encapsulation & Coordinator Slimming**:
   - De-god-objectify the Rust TUI. Move all tab-specific state fields into dedicated `TabController` structs (`ProjectsTab`, `GitHealthTab`, `DepCleanerTab`, `MaintenanceTab`, `DevPortsTab`, `DashboardTab`).
   - Retire all five legacy monkey-patched handler files (`handlers/cleaner.rs`, `handlers/git_health.rs`, `handlers/maintenance.rs`, `handlers/ports.rs`, `handlers/projects.rs`).
   - Reduce `tui/src/app.rs` to an event coordinator (<400 lines) handling global hotkeys, modal routing, and action dispatching.
3. **Open Source Readiness & Community Architecture**:
   - Author a comprehensive `CONTRIBUTING.md` covering development workflows, test execution, Conventional Commits, and the release protocol.
   - Reiterate and clarify the MIT License terms with explicit liability disclaimers regarding automated workspace file operations (pruning, pausing, archiving).
   - Reorganize `README.md` to elevate `npm` and `npx` as the primary installation instructions, with shell scripts documented as secondary standalone alternatives.

## User Stories

1. As a developer with Node.js installed, I want to install RTB using `npm install -g @3mr-5aled/rtb`, so that it is instantly available in my PATH without needing custom shell scripts.
2. As a developer trying RTB for the first time, I want to run `npx @3mr-5aled/rtb init` or `npx @3mr-5aled/rtb list`, so that I can immediately explore the tool with zero installation and zero permanent PATH modifications.
3. As a user preferring alternative package managers, I want `pnpm add -g @3mr-5aled/rtb`, `yarn global add @3mr-5aled/rtb`, or `bun add -g @3mr-5aled/rtb` to work out-of-the-box using the same npm registry package.
4. As a CI engineer, I want the published npm package to declare exact Node engine requirements and bundle all runtime dependencies, so that automated pipelines install and execute RTB deterministically.
5. As a user who installed RTB via npm, when I execute `rtb ui` and the native terminal UI binary is not present on my machine, I want RTB to automatically detect my OS/architecture and offer to download the matching release binary into my user config directory, so that I can launch the TUI without installing a Rust compiler.
6. As a user without global npm write access, I want standalone curl/irm shell scripts (`install.ps1` and `install.sh`) to remain available as secondary installation options.
7. As an open-source developer discovering the project, I want a `CONTRIBUTING.md` file that guides me through local development setup, prerequisites, running test suites, and submitting PRs.
8. As an open-source contributor, I want clear instructions on the repository's Conventional Commits standard and release protocols, so that my contributions fit existing project automation.
9. As a project maintainer, I want the license and user documentation to clearly disclaim liability for data loss or unintended file system modifications caused by automated cleaning or workspace commands, protecting maintainers against legal claims.
10. As a reader of `README.md`, I want to see `npm install -g @3mr-5aled/rtb` and `npx` quickstart commands showcased as the primary installation method at the top of the Quick Start section.
11. As a TUI user navigating between tabs, I want each tab to retain its own selection indices, search filters, and threshold configurations independently when I switch away and return.
12. As a TUI user on the Projects tab, I want pressing navigation keys, search filters, and scaffold shortcuts to be handled entirely by the encapsulated `ProjectsTab` controller and emit declarative actions to the application coordinator.
13. As a TUI user on the Git Health tab, I want branch inspection, commit dialogs, and git status filters to be encapsulated within `GitHealthTab` without polluting the global application struct.
14. As a TUI user on the Dep Cleaner tab, I want dependency folder selection, threshold cycling, and pruning actions to be encapsulated within `DepCleanerTab`.
15. As a TUI user on the Maintenance tab, I want task triggers and status monitoring to be encapsulated within `MaintenanceTab`.
16. As a TUI user on the Dev Ports tab, I want port listing and kill-process actions to be handled within `DevPortsTab`.
17. As a TUI developer adding a new tab to `rtbtui`, I want to implement the `TabController` trait without touching the central application coordinator or writing an `impl App` monkey-patch.
18. As an AI assistant or human developer maintaining `tui/src/app.rs`, I want the main coordinator module to be under 400 lines of code, so that global event routing and modal lifecycle management are immediately comprehensible.
19. As a test writer, I want to test each TUI tab controller in complete isolation by injecting synthetic key events and asserting on emitted application actions, without running a terminal emulator or mocking disk state.
20. As a test writer, I want to test the npm package bundle structure to verify that only necessary distribution files are published, keeping the package download lightweight.
21. As a maintainer running the automated release orchestrator, I want the release script to verify that `npm pack` succeeds before tagging and releasing a version.

## Implementation Decisions

### Decision 1: Primary Package Manifest & Binary Publishing Architecture
- Configure the core CLI package metadata for registry distribution as the primary delivery channel.
- Declare the executable binary mapping (`"bin": { "rtb": "./dist/index.js" }`), package scope (`@3mr-5aled/rtb`), repository metadata, license, and Node.js engine compatibility (`>=18.0.0`).
- Restrict published package files strictly to compiled distribution bundles and essential metadata (`dist/`, `README.md`, `LICENSE`), excluding development sources, test fixtures, and cache artifacts via `"files": ["dist"]`.
- Integrate package validation into the automated release pipeline (`npm pack` dry-run verification in `scripts/release.ps1`).
- Add self-provisioning logic to the `rtb ui` launcher command: if the native binary is not detected on the host system, inspect GitHub Releases for the current version, download the platform-specific archive, extract the binary into the user config directory (`~/.config/rtb/bin/` or `%APPDATA%\rtb\bin\`), and launch it transparently.
- Maintain `install.ps1` and `install.sh` as secondary standalone installers.

### Decision 2: Complete Controller Encapsulation and Coordinator Slimming in TUI
- Migrate all tab-specific state fields out of the central application struct and into dedicated view controllers implementing the view controller trait.
- Each view controller encapsulates its own active indices, filter predicates, threshold sliders, and local transient buffers.
- Retire all five monkey-patched handler modules completely. All tab-specific key handling and logic are folded directly into their respective controller implementations.
- The central application coordinator retains only global concerns: terminal setup/teardown, global hotkeys (quitting, tab switching, command palette toggle), modal stack routing, toast notifications, background thread message processing, and dispatching emitted actions from the active controller.
- Prototype Action Enum:
  ```rust
  pub enum AppAction {
      None,
      Handled,
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
- The application coordinator is slimmed down to under 400 lines of code.

### Decision 3: Community Guidelines & Legal Disclaimers
- Establish a comprehensive `CONTRIBUTING.md` covering prerequisites, repository layout, workflow standards (Conventional Commits, atomic commits), test execution, and the mandatory pre-push release protocol.
- Retain the MIT License with prominent liability disclaimer language specifically addressing workspace and file mutation operations (such as dependency cleaning, project pausing, and archiving) in the project root documentation and user agreement sections.
- Modernize user-facing project documentation (`README.md`): promote `npm install -g @3mr-5aled/rtb` and `npx @3mr-5aled/rtb` as the primary installation method, update badges, and provide clear navigation to contributing resources.

## Testing Decisions

### What Makes a Good Test
- Tests must target external module interfaces at the defined seams, asserting on observable return values and filesystem outcomes rather than internal helper methods or intermediate private state.
- Tests should not mock the runtime environment (no spying on `process.exit`, no synthetic process environment overrides).

### Tested Modules & Seams
1. **npm Distribution Seam**: Packaged artifact validation (verifying that `npm pack` produces a clean archive with the declared binary executable and required bundle files, and executing the packed CLI via Node to verify `--version` and `help` commands).
2. **TUI Controller Seam**: Unit testing each tab view controller with synthetic key events, asserting on state mutations and returned actions without requiring a terminal backend or spawning processes.
3. **TUI Coordinator Seam**: Testing action routing and modal lifecycle management when actions are received from controllers.

### Prior Art
- TypeScript CLI lifecycle tests in `core/tests/`.
- TabController unit tests in `tui/src/ui/tab.rs`.
- Upgrade cycle tests in `core/tests/commands-upgrade.test.ts`.

## Out of Scope

- Rewriting the Ratatui rendering primitives or changing the visual theme styling.
- Native binary compilation of Node.js into a single-file C++ executable (e.g. `pkg` or `sea`).
- Publishing to operating system package managers (Homebrew, Chocolatey, Scoop, Winget, AUR) in this phase.

## Further Notes

- This spec completes the remaining candidates identified in the architecture review and fulfills all items tracked in `todo.txt`.
