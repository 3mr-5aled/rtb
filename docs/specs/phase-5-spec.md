# Specification: Phase 5 — Modern CLI UX, Interactive Wizard & Command Menu, Brand Design System, and Autocompletion Integrity

## Problem Statement

While RTB (رتّب) succeeded in transitioning to a pure ESM TypeScript core CLI (`core/`) and primary npm distribution (`@3mr5aled/rtb`), the interactive command-line experience retained several UX frictions and legacy behaviors:

1. **Static, Plain-Text Terminal Aesthetic**: Running bare `rtb` or `rtb --help` outputted an unstyled command list lacking visual brand presence, context awareness (e.g. current project or workspace roots), or modern interactive guidance.
2. **Branding Disconnect Between CLI and TUI**: The authentic 24-bit golden braille logo was rendered exclusively in the Rust Ratatui TUI, while the CLI was visually disconnected from the brand identity.
3. **Legacy Flat Prompt Installer**: The onboarding command (`rtb init`) relied on basic sequential readline questions rather than modern, styled multi-select wizards like those found in Next.js or Vercel CLIs.
4. **Absence of an Interactive Arrow-Navigable Command Menu**: Developers had to memorize subcommands and flags rather than having an interactive command palette (`rtb menu`) for daily operations like running dev servers, switching directories, and launching AI agents.
5. **Inconsistent & Broken Loading Spinners**: Long-running asynchronous operations (Git health scans, dependency audits, upgrade downloads) either printed raw console logs or lacked a unified, branded progress indicator with non-TTY safety.
6. **Autocompletion Inconsistencies**: Autocompletion in PowerShell exhibited switch fall-throughs, array concatenation bugs, and unescaped wildcard pattern errors, leading to degraded tab-completion reliability.

## Solution

Deliver a modern terminal UX overhaul across five cohesive architectural pillars:

1. **Multi-Tier Golden Braille Brand Logo System**:
   - Create `core/src/utils/logo.ts` with a multi-tier resolution strategy (`CWD` -> module root -> user config directory -> embedded fallback).
   - Convert braille Unicode art into true 24-bit ANSI RGB golden gradients (`#D4AF37` to `#F3E5AB`).
   - Strip UTF-8 BOM artifacts automatically.
2. **Context-Aware Hero Banner**:
   - Implement `HeroBanner` in `core/src/utils/banner.ts` displayed on bare `rtb` and `rtb --help` when interactive TTY is detected.
   - Senses current working directory: if invoked inside a managed project, dynamically shows project runtime stack, branch, and status alongside active workspace metrics.
   - Categorizes quick actions and tips with strict `--json` and `--quiet` suppression.
3. **Unified Ora TaskSpinner Utility**:
   - Implement `TaskSpinner` and `withSpinner` helper in `core/src/utils/spinner.ts`.
   - Uses golden braille spinner frames with elapsed time tracking.
   - Attached across `rtb health`, `rtb deps`, `rtb clean`, `rtb doctor`, and `rtb upgrade`.
   - Guaranteed silent, zero-noise fallback in non-TTY, CI, and `--json` modes.
4. **Interactive Setup Onboarding Wizard (`rtb init`)**:
   - Rebuild `rtb init` using `@clack/prompts` into an elegant 5-step guided flow:
     1. Brand intro with golden braille emblem.
     2. Workspace root path discovery and custom directory entry.
     3. 8-lifecycle folder multi-select scaffolding.
     4. Automated shell integration hooks (`pwsh`, `bash`, `zsh`, `fish`).
     5. Config outro with next-step command shortcuts.
   - Full non-interactive fallback for `--force`, CI, and headless executions.
5. **Interactive Command Cockpit (`rtb menu`)**:
   - Implement `core/src/commands/menu.ts` offering instant arrow-key selection for:
     - Run / Build / Test (interactive project and script execution)
     - Quick Goto (fuzzy directory jumping)
     - Launch Ratatui TUI (`rtbtui`)
     - Health Doctor (Git repository scans and toolchain verification)
     - AI Agent Cockpit (Google Antigravity, Claude, Gemini, Cursor)
     - Config direct editing
     - Exit
6. **Cross-Shell Autocompletion Integrity**:
   - Refactor `core/src/commands/completion.ts`.
   - Eliminate PowerShell `switch` fall-throughs by appending `break` to all branches.
   - Fix array string concatenation issues.
   - Escape wildcard search patterns via `[System.Management.Automation.WildcardPattern]::Escape`.
   - Register completer across all aliases (`rtb`, `rtb.cmd`, `rtb.ps1`, `dev`).
   - Support `dev` command and `menu` subcommand across Bash, Zsh, Fish, and PowerShell.

## User Stories

1. As a developer running `rtb` without arguments, I want to see a rich, golden braille hero banner showing my current workspace status and daily quick actions, so that the CLI feels modern, welcoming, and easy to navigate.
2. As a developer inside a project directory, I want `rtb` to recognize my active project and show its stack and branch status in the banner header, so that I immediately have situational awareness.
3. As a developer setting up RTB for the first time, I want an interactive `@clack/prompts` setup wizard (`rtb init`) that lets me pick my lifecycle folders with spacebar checkboxes and auto-configures my shell hooks, so that onboarding requires zero manual JSON editing.
4. As a developer navigating projects throughout the day, I want to run `rtb menu` and use arrow keys to pick actions and launch dev servers or AI agents, without typing lengthy flags.
5. As a developer waiting on `rtb health` or `rtb deps`, I want to see an elegant golden braille spinner with elapsed time, so that I know long-running asynchronous scans are actively progressing.
6. As a CI/CD engineer or script author, I want all banners, spinners, and interactive prompts to be strictly omitted when passing `--json` or `--quiet` or running in a non-interactive pipe, so that automation remains 100% deterministic and parseable.
7. As a PowerShell user, I want pressing `TAB` after `rtb goto ` or `dev ` to complete project names reliably without script errors or pattern mismatches, so that navigation is frictionless.
8. As a Bash, Zsh, or Fish user, I want `rtb completion <shell>` to output clean, valid completion scripts that include all subcommands and dynamic project name lookups.
9. As a tester, I want modular seams around `@clack/prompts` so that unit tests can verify interactive cancellation and step progression cleanly under pure ESM.

## Implementation Architecture

### 1. Brand Logo Resolution (`core/src/utils/logo.ts`)
The logo loader searches candidates in priority order:
1. `process.cwd()` (for local development checkouts)
2. Module bundle root (`path.resolve(__dirname, '..')`)
3. User configuration directory (`~/.config/rtb/logo.txt`)
4. Embedded fallback string

Any leading UTF-8 BOM (`\uFEFF`) is cleanly stripped, and lines are formatted with 24-bit truecolor ANSI escape sequences (`\x1b[38;2;...m`).

### 2. Context-Aware Hero Banner (`core/src/utils/banner.ts`)
Checks TTY status and suppresses rendering if `process.env.RTB_QUIET === '1'`, `--quiet`, or `--json` flags are present. Detects whether the current working directory is inside a managed project or workspace root, printing a dynamic status badge before displaying categorized quick actions.

### 3. Unified Ora TaskSpinner (`core/src/utils/spinner.ts`)
Wraps `ora` with custom braille frames:
`['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']` tinted in brand gold (`#D4AF37`). Exposes:
- `TaskSpinner` class (`start`, `succeed`, `fail`, `warn`, `info`, `stop`)
- `withSpinner<T>(title, fn)` functional wrapper with elapsed execution timing diagnostics.

### 4. Clack Prompts Adapter Pattern (`core/src/commands/init.ts` & `core/src/commands/menu.ts`)
To allow Vitest unit testing without mutating immutable ESM module namespaces, prompt methods are wrapped in an exported `prompts` object (`export const prompts = { intro, text, select, multiselect, ... }`). This provides a clean seam for unit testing with full coverage.

### 5. Multi-Shell Completion Registrations (`core/src/commands/completion.ts`)
Completion scripts generate dynamic lookups that invoke `rtb list --json` and cache results for fast keystroke response, supporting `rtb`, `rtb.cmd`, `rtb.ps1`, and the `dev` alias.

## Verification & Acceptance Criteria

- **100% Green Test Suite**: 35 test files passed (`197/197 tests`).
- **Clean Compilation**: `npm run typecheck` exits with code 0.
- **Pure ESM Distribution**: `tsup` compiles `core/dist/index.js` (<520 KB).
- **Packaging Contract**: `npm pack --dry-run` confirms clean tarball with only `dist/` and metadata.
- **Git State**: Clean tree tagged `v0.12.0` and pushed to `main`.
