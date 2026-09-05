# @3mr5aled/rtb

> **RTB — رتّب (Repository & Tooling Base)**  
> Unified developer workspace manager, modern CLI UX cockpit, AI launcher, and project telemetry engine.

```text
⠀⠀⢸⣿⣿⣿⣿⣿⣿⣷⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣷⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⠀
⠀⠀⢸⣿⣿⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠀⠀⠀⠀⠀
⠀⠀⢸⣿⡏⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟
⠀ ⢸⣿⠃⣼⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿ ⣿ ⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀
⠀⠀⢸⡟⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀
⠀⠀⢸⡇⢸⣿⣿⣿⣿⣿⣿ ⣿⣿⣿⣿ ⣿⣿⣿ ⣿⣿⣿⣿ ⣿⣿⣿⡟⠀⠀⠀
⠀⠀⢸⠁⣿⣿⣿⣿⣿⣿⣿          ⣿⣿⣿ ⣿⣿⣿⡟⠀⠀⠀
⠀⠀⢸⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿ ⣿⣿⣿⡟⠀⠀⠀⠀
⠀⠀⠘⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿ ⣿⣿⣿⣿⣿⣿⣿  ⣿⣿⣿⡿⠀⠀⠀⠀
⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠇⠀⠀⠀⠀
⠀ ⠈⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉   
```

[![npm version](https://img.shields.io/npm/v/@3mr5aled/rtb.svg?color=red)](https://www.npmjs.com/package/@3mr5aled/rtb)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/3mr-5aled/rtb/blob/main/LICENSE)
[![Node.js](https://img.shields.io/badge/Node.js-18+-green.svg)](https://nodejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5+-blue.svg)](https://www.typescriptlang.org/)
[![GitHub](https://img.shields.io/badge/GitHub-3mr--5aled%2Frtb-blue.svg)](https://github.com/3mr-5aled/rtb)

---

## ⚡ Quick Start

### Global Installation (Recommended)
Install globally with npm to have `rtb` available across your terminal:
```bash
npm install -g @3mr5aled/rtb
```

### Instant Execution with `npx`
Run directly without permanent installation:
```bash
# Launch the interactive prompt launcher
npx @3mr5aled/rtb menu

# Initialize your workspace
npx @3mr5aled/rtb init

# Open native Ratatui TUI
npx @3mr5aled/rtb ui
```

---

## ✨ Features

- ⚡ **Interactive Command Menu (`rtb menu`)**: Arrow-navigable command cockpit powered by `@clack/prompts` to run scripts, switch directories, and launch agents with zero memorization.
- 🧙 **Modern Onboarding Wizard (`rtb init`)**: 5-step guided setup with workspace root discovery, 8 lifecycle multi-select folders (`01-Active`, `02-Backlog`, etc.), and automated shell hook integration.
- 🧭 **Context-Aware Hero Banner**: Golden Braille brand logo, current workspace metrics, and local project environment detection on bare `rtb` or `rtb --help`.
- 🔄 **Unified Ora Task Spinners**: Custom animated golden braille spinner frames with elapsed time diagnostics across long-running async tasks.
- 🚀 **Multi-Runtime Project Operations**: Auto-detects and runs dev/build/test scripts across Node.js (`npm`, `pnpm`, `yarn`, `bun`), Rust (`cargo`), Python (`pytest`, `poetry`, `uv`), and Go.
- 🔍 **Fuzzy Directory Navigation (`rtb goto`)**: Fast fuzzy scoring to jump straight into any project directory across all configured roots.
- 🤖 **AI Agent Orchestration**: Launch Google Antigravity (`rtb agy`), Claude (`rtb claude`), Gemini (`rtb gemini`), or Cursor (`rtb cursor`) with auto-generated project context (`.rtb_context.md`).
- 🩺 **System Diagnostics (`rtb doctor` & `rtb health`)**: Verify toolchain requirements and scan Git repositories across all roots for uncommitted changes or missing remotes.
- 🖥️ **Native Terminal UI (`rtb ui`)**: Interactive Rust Ratatui dashboard with multi-pane project browsing, Git telemetry, and dependency cleanup.
- ⌨️ **Cross-Shell Autocompletion**: Tab completion for project names and subcommands across PowerShell, Bash, Zsh, and Fish.

---

## 🎮 Interactive CLI Experience

### 1. Interactive Command Menu (`rtb menu`)
```bash
rtb menu
```
Navigate with arrow keys and press Enter:
- **Run / Build / Test**: Select a project and execute its detected start, build, or test scripts.
- **Quick Goto**: Interactive project picker to switch directories.
- **Launch TUI**: Start the full Ratatui interactive operations dashboard (`rtbtui`).
- **Health Doctor**: Run toolchain diagnostics or Git telemetry health scans.
- **AI Agent Cockpit**: Launch Google Antigravity, Claude, Gemini, or Cursor with context.
- **Configuration**: Edit `rtb.config.json` in your favorite editor.

### 2. Guided Workspace Setup (`rtb init`)
```bash
rtb init
```
1. **Brand Intro**: Welcomes you with the Golden Braille emblem.
2. **Project Root Selection**: Auto-detects workspace directory or prompts for custom path.
3. **Lifecycle Scaffolding**: Checkbox multi-select for lifecycle folders (`01-Active`, `04-Paused`, `05-Archive`, etc.).
4. **Automated Shell Integration**: Detects active shell (`PowerShell`, `Bash`, `Zsh`, `Fish`) and installs directory switching wrappers.
5. **Verification**: Displays saved configuration path and immediate next steps.

---

## 🛠️ Command Reference

| Command | Description |
| :--- | :--- |
| `rtb menu` | Launch interactive prompt command cockpit |
| `rtb init [--force]` | Run interactive 5-step onboarding wizard |
| `rtb goto <name> [--<agent>]` | Fuzzy project search & fast directory jump |
| `rtb run [project]` | Auto-detect and run dev/start scripts |
| `rtb build [project]` | Auto-detect and run project build pipelines |
| `rtb test [project]` | Auto-detect and run project test suites |
| `rtb list [--active\|--paused\|--json]` | List managed projects with status & timestamps |
| `rtb status [--json]` | Fast one-line prompt status segment |
| `rtb doctor` | System health check (roots, git, runtimes, agents) |
| `rtb health` | Git repository health scan across project roots |
| `rtb deps [project]` | Audit declared project dependencies |
| `rtb new <name> [--stack <type>]` | Scaffold a new project in `01-Active` |
| `rtb pause <name> [--prune]` | Move project to `04-Paused` |
| `rtb resume <name> [--install]` | Move project back to `01-Active` |
| `rtb ui` | Launch native interactive Rust Terminal UI (`rtbtui`) |
| `rtb agy\|claude\|gemini [project]` | Launch AI agent with auto-generated context |
| `rtb config` | Open `rtb.config.json` in default editor |
| `rtb completion <shell>` | Output autocompletion script (pwsh, bash, zsh, fish) |
| `rtb shell-init <shell>` | Output directory changing wrapper function |

---

## ⌨️ Shell Integration

To enable directory changes on `rtb goto` and tab autocompletion in your shell:

### PowerShell (`$PROFILE`)
```powershell
# Directory switching function
Invoke-Expression (& rtb shell-init pwsh)

# Autocompletion
rtb completion pwsh | Out-String | Invoke-Expression
```

### Bash (`~/.bashrc`)
```bash
eval "$(rtb shell-init bash)"
eval "$(rtb completion bash)"
```

### Zsh (`~/.zshrc`)
```bash
eval "$(rtb shell-init zsh)"
eval "$(rtb completion zsh)"
```

### Fish (`~/.config/fish/config.fish`)
```fish
rtb shell-init fish | source
rtb completion fish | source
```

---

## ⚙️ Configuration

Configuration is stored at `~/.config/rtb/rtb.config.json` (`%USERPROFILE%\.config\rtb\rtb.config.json` on Windows):

```json
{
  "version": "1.0.0",
  "projectRoots": {
    "active": {
      "path": "D:\\Projects\\01-Active",
      "label": "Active Projects",
      "emoji": "⚡"
    },
    "paused": {
      "path": "D:\\Projects\\04-Paused",
      "label": "On Hold",
      "emoji": "⏸️"
    }
  }
}
```

Run `rtb config` anytime to edit your configuration.

---

## 📄 License

Distributed under the [MIT License](https://github.com/3mr-5aled/rtb/blob/main/LICENSE).  
Copyright © 2026 Amr Khaled ([@3mr-5aled](https://github.com/3mr-5aled)).
