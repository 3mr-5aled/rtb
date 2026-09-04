# Specification: Phase 4 — npm Distribution, TUI Controller Consolidation, and Open Source Readiness

## Problem Statement

As RTB (رتّب) nears general availability as an open-source developer tool, three architectural and operational gaps prevent a smooth user and developer experience:

1. **Packaging & Delivery Barrier**: Developers and automated environments currently must install RTB through custom PowerShell or POSIX shell scripts (install.ps1, install.sh). In the modern developer ecosystem, users expect standard 
pm install -g @3mr-5aled/rtb or instant, zero-install execution via 
px @3mr-5aled/rtb.
2. **Incomplete TUI Controller Seam & Monolithic Coordinator**: While Phase 2 successfully resurrected the TabController trait with an action-passing protocol, 	ui/src/app.rs remains an oversized God Object (~1,700 lines). Tab-specific state fields are still flattened across the central application struct, and five monkey-patched handler modules remain in place. This prevents true unit testing of tab behaviors and clutters global event coordination.
3. **Absence of Community & Contribution Infrastructure**: The repository lacks formal contribution guidelines (CONTRIBUTING.md), detailed licensing and liability disclaimers for workspace file mutations, and structured onboarding documentation in README.md.

## Solution

Address these requirements across three cohesive architectural seams:

1. **First-Class npm Distribution**: Package @3mr-5aled/rtb for npm registry distribution with complete package metadata, binary declarations, bundle-only packaging (iles: ["dist"]), and automated release pack verification. Provide self-provisioning capabilities so that if a user running via npm executes tb ui without a local tbtui binary, RTB detects the platform and automatically downloads or guides fetching the precompiled release binary into the user config directory.
2. **Complete TUI Controller Encapsulation & Coordinator Slimming**: De-god-objectify the Rust TUI. Move all tab-specific state fields into their respective TabController structs (ProjectsTab, GitHealthTab, DepCleanerTab, MaintenanceTab, DevPortsTab, DashboardTab). Retire the five legacy monkey-patched handler files completely. Reduce 	ui/src/app.rs to an event coordinator (<400 lines) that handles global shortcuts, modal transitions, and action dispatching.
3. **Open Source Readiness & Community Architecture**: Author a comprehensive CONTRIBUTING.md defining development workflows, test running, Conventional Commits, and the release protocol. Reiterate and clarify the MIT License terms with explicit liability disclaimers regarding automated file operations. Update README.md with npm and npx quickstart guides and contribution paths.

## User Stories

1. As a JavaScript/TypeScript developer, I want to install RTB globally using 
pm install -g @3mr-5aled/rtb, so that I don't need to run curl or PowerShell scripts.
2. As a developer evaluating RTB, I want to run 
px @3mr-5aled/rtb list or 
px @3mr-5aled/rtb init, so that I can explore the tool with zero installation and zero permanent PATH modifications.
3. As a CI engineer, I want the published npm package to declare exact Node engine requirements and bundle all runtime dependencies, so that automated pipelines install and execute RTB deterministically.
4. As a user who installed RTB via npm, when I execute tb ui and the native terminal UI binary is not present on my machine, I want RTB to automatically detect my OS/architecture and offer to download the matching release binary into my user config directory, so that I can launch the TUI without installing a Rust compiler.
5. As an open-source developer discovering the project, I want a CONTRIBUTING.md file that guides me through local development setup, prerequisites, running test suites, and submitting PRs.
6. As an open-source contributor, I want clear instructions on the repository's Conventional Commits standard and release protocols, so that my contributions fit existing project automation.
7. As a project maintainer, I want the license and user documentation to clearly disclaim liability for data loss or unintended file system modifications caused by automated cleaning or workspace commands, protecting maintainers against legal claims.
8. As a reader of README.md, I want to see 
pm and 
px quickstart commands placed prominently alongside the shell one-liners, so that I can choose my preferred installation method.
9. As a TUI user navigating between tabs, I want each tab to retain its own selection indices, search filters, and threshold configurations independently when I switch away and return.
10. As a TUI user on the Projects tab, I want pressing navigation keys, search filters, and scaffold shortcuts to be handled entirely by the encapsulated ProjectsTab controller and emit declarative actions to the application coordinator.
11. As a TUI user on the Git Health tab, I want branch inspection, commit dialogs, and git status filters to be encapsulated within GitHealthTab without polluting the global application struct.
12. As a TUI user on the Dep Cleaner tab, I want dependency folder selection, threshold cycling, and pruning actions to be encapsulated within DepCleanerTab.
13. As a TUI user on the Maintenance tab, I want task triggers and status monitoring to be encapsulated within MaintenanceTab.
14. As a TUI user on the Dev Ports tab, I want port listing and kill-process actions to be handled within DevPortsTab.
15. As a TUI developer adding a new tab to tbtui, I want to implement the TabController trait without touching the central application coordinator or writing an impl App monkey-patch.
16. As an AI assistant or human developer maintaining 	ui/src/app.rs, I want the main coordinator module to be under 400 lines of code, so that global event routing and modal lifecycle management are immediately comprehensible.
17. As a test writer, I want to test each TUI tab controller in complete isolation by injecting synthetic key events and asserting on emitted application actions, without running a terminal emulator or mocking disk state.
18. As a test writer, I want to test the npm package bundle structure to verify that only necessary distribution files are published, keeping the package download lightweight.
19. As a maintainer running the automated release orchestrator, I want the release script to verify that 
pm pack succeeds before tagging and releasing a version.

## Implementation Decisions

### Decision 1: Package Manifest & Binary Distribution Architecture
- Configure the core CLI package metadata for registry distribution.
- Declare the executable binary mapping, package scope, repository metadata, license, and Node.js engine compatibility.
- Restrict published package files strictly to compiled distribution bundles and essential metadata (dist/, README.md, LICENSE), excluding development sources, test fixtures, and cache artifacts.
- Integrate package validation into the automated release pipeline (
pm pack dry-run verification).
- Add self-provisioning logic to the terminal UI launcher command: if the native binary is not detected on the host system, inspect GitHub Releases for the current version, download the platform-specific archive, extract the binary into the user config directory, and launch it transparently.

### Decision 2: Complete Controller Encapsulation and Coordinator Slimming in TUI
- Migrate all tab-specific state fields out of the central application struct and into dedicated view controllers implementing the view controller trait.
- Each view controller encapsulates its own active indices, filter predicates, threshold sliders, and local transient buffers.
- Retire all five monkey-patched handler modules completely. All tab-specific key handling and logic are folded directly into their respective controller implementations.
- The central application coordinator retains only global concerns: terminal setup/teardown, global hotkeys (quitting, tab switching, command palette toggle), modal stack routing, toast notifications, background thread message processing, and dispatching emitted actions from the active controller.
- Prototype Action Enum:
  `ust
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
  `
- The application coordinator is slimmed down to under 400 lines of code.

### Decision 3: Community Guidelines & Legal Disclaimers
- Establish a comprehensive contributing guide covering prerequisites, repository layout, workflow standards (Conventional Commits, atomic commits), test execution, and the mandatory pre-push release protocol.
- Retain the MIT License with prominent liability disclaimer language specifically addressing workspace and file mutation operations (such as dependency cleaning, project pausing, and archiving) in the project root documentation and user agreement sections.
- Modernize user-facing project documentation: provide side-by-side installation options (npm global, npx zero-install, and shell one-liners), update badges, and provide clear navigation to contributing resources.

## Testing Decisions

### What Makes a Good Test
- Tests must target external module interfaces at the defined seams, asserting on observable return values and filesystem outcomes rather than internal helper methods or intermediate private state.
- Tests should not mock the runtime environment (no spying on process.exit, no synthetic process environment overrides).

### Tested Modules & Seams
1. **npm Distribution Seam**: Packaged artifact validation (verifying that 
pm pack produces a clean archive with the declared binary executable and required bundle files, and executing the packed CLI via Node to verify --version and help commands).
2. **TUI Controller Seam**: Unit testing each tab view controller with synthetic key events, asserting on state mutations and returned actions without requiring a terminal backend or spawning processes.
3. **TUI Coordinator Seam**: Testing action routing and modal lifecycle management when actions are received from controllers.

### Prior Art
- TypeScript CLI lifecycle tests in core/tests/.
- TabController unit tests in 	ui/src/ui/tab.rs.
- Upgrade cycle tests in core/tests/commands-upgrade.test.ts.

## Out of Scope

- Rewriting the Ratatui rendering primitives or changing the visual theme styling.
- Native binary compilation of Node.js into a single-file C++ executable (e.g. pkg or sea).
- Publishing to operating system package managers (Homebrew, Chocolatey, Scoop, Winget, AUR) in this phase.

## Further Notes

- This spec completes the remaining candidates identified in the architecture review and fulfills all items tracked in 	odo.txt.
