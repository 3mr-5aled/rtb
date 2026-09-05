# RTB — رتّب (Repository & Tooling Base)

```text
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⢸⣿⣿⣿⣿⣿⣿⣷⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣷⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⠀⠀⠀⠀⠀
⠀⠀⢸⣿⣿⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠀⠀⠀⠀⠀
⠀⠀⢸⣿⡏⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⠀
⠀⠀⢸⣿⠃⣼⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿ ⣿ ⣿⣿⣿⣿⣿⣿⣿⡟⠀⠀
⠀⠀⢸⡟⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀
⠀⠀⢸⡇⢸⣿⣿⣿⣿⣿⣿ ⣿⣿⣿⣿ ⣿⣿⣿ ⣿⣿⣿ ⣿⣿⣿⡟⠀⠀⠀
⠀⠀⢸⠁⣿⣿⣿⣿⣿⣿⣿             ⣿⣿ ⣿⣿⣿⡟⠀⠀⠀
⠀⠀⠘⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿ ⣿⣿⣿⡟⠀⠀⠀⠀
⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿ ⣿⣿⣿⣿⣿⣿⣿ ⣿⣿⣿⡿⠀⠀⠀⠀
⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠇⠀⠀⠀⠀
⠀⠀⠈⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠀⠀⠀
```

[![Version](https://img.shields.io/badge/version-v0.13.5-blue.svg)](https://github.com/3mr-5aled/rtb/releases)
[![npm version](https://img.shields.io/npm/v/@3mr5aled/rtb.svg?color=red)](https://www.npmjs.com/package/@3mr5aled/rtb)
[![Status: Beta](https://img.shields.io/badge/status-BETA-orange.svg)](https://github.com/3mr-5aled/rtb/issues)
[![Node.js](https://img.shields.io/badge/Node.js-18+-green.svg)](https://nodejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5+-blue.svg)](https://www.typescriptlang.org/)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

> [!NOTE]
> **Beta Pre-Release**: RTB is currently in active beta testing. We welcome feedback, bug reports, and suggestions via [GitHub Issues](https://github.com/3mr-5aled/rtb/issues) and [Discussions](https://github.com/3mr-5aled/rtb/discussions)!

**RTB — رتّب** is a fast, developer-first cross-platform Project Operations Tool featuring a unified TypeScript/Node.js CLI (`rtb`), an interactive Rust Terminal UI (`rtbtui`), multi-runtime project intelligence, Git telemetry monitoring, AI agent orchestration, and automated workspace lifecycle management.

---

## 📁 Repository Structure

```
rtb/
├── config/
│   └── rtb.config.json     # Default JSON configuration template
├── core/                   # Unified Cross-Platform TypeScript/Node.js CLI engine
│   ├── package.json        # Pure ESM module (@3mr5aled/rtb)
│   ├── tsup.config.ts      # Multiplatform ESM bundler
│   ├── src/
│   │   ├── agent/          # AI agent discovery & .rtb_context.md generator
│   │   ├── commands/       # CLI commands (init, goto, agent, doctor, ui, run, build, test, etc.)
│   │   ├── config/         # Multi-tier config loader (~/.config/rtb/rtb.config.json)
│   │   ├── inspector/      # Multi-runtime project inspector (Node, Rust, Go, Python)
│   │   ├── navigation/     # Fuzzy scoring navigation engine
│   │   ├── services/       # Runner & Maintenance task registries
│   │   ├── utils/          # Golden braille logo, HeroBanner, and Ora TaskSpinners
│   │   └── index.ts        # CLI binary entrypoint
│   └── tests/              # Vitest test suite
├── tui/                    # Rust Ratatui interactive TUI source
│   ├── Cargo.toml
│   └── src/
├── .github/                # CI/CD Workflows & Issue templates
│   └── workflows/release.yml
├── install.ps1             # Windows interactive Setup Wizard (PowerShell / cmd)
├── install.sh              # Linux / macOS POSIX Setup Wizard (Node.js runtime)
├── uninstall.ps1           # Standalone automated uninstaller
├── CONTRIBUTING.md         # Developer setup & contribution guidelines
├── PROJECT.md              # Project architecture & milestone metadata
├── CONTEXT.md              # Domain model & architectural glossary
├── LICENSE                 # MIT License & Liability Disclaimers
└── README.md
```

---

## 🚀 Quick Start & Installation

### Option 1: 1-Line Setup & Install with npx (Recommended)

Run the interactive setup wizard directly with `npx`. It configures your workspace roots, lifecycle folders, terminal hooks, downloads the native TUI (or lets you defer it), deploys the CLI launcher to your PATH, and announces readiness:

```bash
npx @3mr5aled/rtb install
```

Or install globally via npm:

```bash
npm install -g @3mr5aled/rtb
```

You can also run any command on demand without permanent installation:

```bash
npx @3mr5aled/rtb menu
npx @3mr5aled/rtb ui
npx @3mr5aled/rtb doctor
```

> [!TIP]
> During `npx @3mr5aled/rtb install` (or `rtb init`), you can choose to download the native Terminal UI (`rtbtui`) immediately or download later on demand (or bypass via `--skip-ui`). When ready, `rtb` is immediately available in your terminal.

---

### Option 2: Standalone Setup Wizard (No Node.js Required)

If you prefer a standalone installation with native shell integration and automated PATH configuration:

#### 🪟 Windows (PowerShell 5.1+ / PowerShell 7+)
```powershell
irm https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.ps1 | iex
```

#### 🐧 Linux / 🍎 macOS
```bash
curl -fsSL https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.sh | sh
```

The interactive Setup Wizard will automatically:
- Detect your OS, architecture, and active shell environment (`pwsh`, `bash`, `zsh`, `fish`).
- Configure install directory (defaulting to `~/.config/rtb`).
- Offer the choice to download the native TUI binary (`rtbtui`) now or download later on first `rtb ui` launch.
- Configure your shell profile autoload with legacy duplicate cleanup.
- Prompt to immediately initialize your workspace via `rtb init`.

---

### Option 3: CI / Non-Interactive Automation

For headless pipelines (GitHub Actions, Docker, Azure Pipelines), pass quiet flags or environment variables:

**Windows PowerShell:**
```powershell
# Quiet installation with custom install path
pwsh -File ./install.ps1 -Quiet -InstallPath "C:\tools\rtb"

# Skip TUI binary download (lightweight CLI only)
pwsh -File ./install.ps1 -SkipUI -Quiet

# Environment variable option
$env:RTB_QUIET = "1"
$env:RTB_SKIP_UI = "1"
irm https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.ps1 | iex
```

**Linux / macOS:**
```bash
# Quiet installation
curl -fsSL https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.sh | RTB_QUIET=1 sh

# Skip TUI download via CLI flag or environment variable
curl -fsSL https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.sh | sh -s -- --skip-ui -q
curl -fsSL https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.sh | RTB_QUIET=1 RTB_SKIP_UI=1 sh

# Custom directory in CI
curl -fsSL https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.sh | RTB_QUIET=1 RTB_INSTALL_PATH="/opt/rtb" sh
```

---

### Option 4: Local Repository Installation (Developers & Contributors)

See [CONTRIBUTING.md](CONTRIBUTING.md) for full developer prerequisites, local build setup, and testing instructions.

1. Clone the repository:
   ```bash
   git clone https://github.com/3mr-5aled/rtb.git
   cd rtb
   ```

2. Run the local Setup Wizard:
   - **Windows:**
     ```powershell
     pwsh -File ./install.ps1
     ```
   - **Linux / macOS:**
     ```bash
     sh ./install.sh
     ```

3. Initialize your workspace:
   ```powershell
   rtb init
   ```

---

### 🔄 Upgrading RTB

To check for updates and self-upgrade:

```powershell
# Check if a new release is available
rtb upgrade --check

# Download and install the latest release automatically
rtb upgrade
```

---

### 🗑️ Uninstallation

To cleanly uninstall RTB, remove binaries, and clean up PowerShell profile imports:

- **Via CLI command:**
  ```powershell
  rtb uninstall
  # Or force clean without confirmation prompt:
  rtb uninstall -Force
  ```

- **Via standalone script:**
  ```powershell
  pwsh -File ./uninstall.ps1
  ```

Add the `-KeepConfig` flag if you wish to preserve your `%APPDATA%\rtb\rtb.config.json` user settings for future use.

---

## 🛠️ CLI Command Reference (`rtb`)

### Setup & Lifecycle
| Command | Description |
| :--- | :--- |
| `rtb menu` | Interactive prompt launcher (@clack/prompts) for instant project execution, navigation, diagnostics, and AI agents |
| `rtb init [--force]` | Interactive 5-step onboarding wizard (@clack/prompts) with root auto-detection, lifecycle scaffolding, and shell hooks |
| `rtb config` | Open active `rtb.config.json` configuration file in default editor |
| `rtb doctor` | System health check (validates config, roots, git, runtimes, agents, and TUI binary) |
| `rtb shell-init <bash\|zsh\|fish\|pwsh>` | Output shell wrapper function enabling directory change on `rtb goto` |
| `rtb completion <bash\|zsh\|fish\|pwsh>` | Output shell autocompletion script for tab completion of project names and subcommands |
| `rtb ui` | Launch native interactive Rust Terminal UI (`rtbtui`) |
| `rtb upgrade [--check] [--force]` | Check for newer releases and perform in-place self-upgrade |
| `rtb uninstall [--force] [-KeepConfig]` | Cleanly remove RTB binaries, module, and profile integrations |
| `rtb --version` / `rtb --help` | Display current version or context-aware HeroBanner & command guide |

### Navigation & Discovery
| Command | Description |
| :--- | :--- |
| `rtb goto <query> [--<agent>]` | Fuzzy project search & fast directory jump (with optional AI agent launch) |
| `rtb list [--active\|--paused\|--deployed\|--all] [--json]` | Filtered project status listing with last modified timestamps |
| `rtb status [--json]` | Fast one-line prompt status segment (`rtb » project (Active) [main ±1] Node.js`) |
| `rtb open <project>` | Open project directory in File Explorer |

### Project Intelligence & Operations
| Command | Description |
| :--- | :--- |
| `rtb run [project]` | Auto-detect and run dev/start scripts (`npm`, `pnpm`, `cargo`, `python`, etc.) |
| `rtb build [project]` | Auto-detect and run project build pipelines |
| `rtb test [project]` | Auto-detect and execute project test suites |
| `rtb info [project] [--json]` | Deep multi-runtime project intelligence inspection |
| `rtb deps [outdated] [project]` | Audit declared project dependencies and package lockfiles |
| `rtb workspace [project]` | Inspect monorepo workspace packages (pnpm, yarn, bun, npm, Cargo) |
| `rtb commit [-Message <str>] [-Amend] [-Push]` | Interactive CLI prompt to stage, commit, and push git changes |

### AI Agent Orchestration
| Command | Description |
| :--- | :--- |
| `rtb agy [project]` | Launch Google Antigravity CLI with auto-generated project context |
| `rtb claude\|gemini\|codex [project]` | Launch Claude, Gemini, or Codex CLI |
| `rtb cursor\|windsurf\|aider [project]` | Launch Cursor, Windsurf, or Aider |
| `rtb agent [project] [-List]` | List installed AI agent CLIs or launch targeted agent |

### Workspace Management & Safety
| Command | Description |
| :--- | :--- |
| `rtb new <name> [--stack <type>]` | Scaffold a new project in `01-Active` |
| `rtb pause <name> [--prune] [-Force]` | Move project to `04-Paused` with uncommitted changes check and dep pruning |
| `rtb resume <name> [--install]` | Move project back to `01-Active` (optionally reinstall dependencies) |
| `rtb deploy <name> [--prod\|--staging]` | Promote active project to production or staging |
| `rtb archive <name> [-Force]` | Safely compress project into `.tar.gz` backup archive |
| `rtb unarchive <archive-name>` | Restore project snapshot to `01-Active` |
| `rtb health` | Perform Git repository health overview scan |
| `rtb clean [--commit] [--dry-run]` | Safe dependency pruning (`node_modules`, `target`, `.venv`) with dry-run default |
| `rtb index` | Generate comprehensive `PROJECT-INDEX.md` markdown catalog |
| `rtb backup` / `rtb env` | Backup configurations or `.env` credential files |

---

## ✨ Modern CLI Experience & Interactive Cockpit

RTB introduces a modern, developer-first terminal visual experience built with `@clack/prompts`, 24-bit ANSI truecolor Golden Braille branding, and animated `ora` spinners.

### 🧭 Context-Aware Hero Banner
Whenever you execute bare `rtb` or `rtb --help` in an interactive terminal, RTB renders a rich, context-aware Hero Banner:
- **Golden Braille Brand Logo**: Authentic 24-bit RGB truecolor braille emblem loaded dynamically from module assets or user configuration paths.
- **Context Awareness**: Automatically senses your current working directory. If invoked inside a managed project, it displays project metadata (runtime stack, git branch, status) alongside active workspace metrics.
- **Categorized Quick Directory**: Presents daily development shortcuts grouped by operational category with arrow-key tip reminders.
- **Headless & Scripting Safety**: Automatically suppressed when standard output is non-TTY, redirected, or running with `--json` or `--quiet`.

### ⚡ Interactive Command Menu (`rtb menu`)
Launch the interactive command cockpit for instant arrow-key execution without memorizing flags:
```bash
rtb menu
```
The menu provides rapid access to:
- **Run / Build / Test**: Select any registered project and execute dev scripts with live output.
- **Quick Goto**: Interactive selector to switch directories into any active or paused project.
- **Launch TUI**: Start the full Ratatui interactive operations dashboard (`rtbtui`).
- **Health Doctor**: Run Git telemetry scans or toolchain health diagnostics with real-time Ora spinners.
- **AI Agent Cockpit**: Launch Google Antigravity, Claude, Gemini, or Cursor with auto-generated project context.
- **Configuration**: Edit `rtb.config.json` in your default code editor.

### 🧙 Interactive Onboarding Wizard (`rtb init`)
Setting up a new development environment is as simple as running:
```bash
rtb init
```
The Clack-powered wizard guides you through:
1. **Brand Intro**: Welcomes you with the Golden Braille emblem and configuration detection.
2. **Project Root Selection**: Auto-detects parent directories or allows typing a custom path.
3. **Lifecycle Scaffolding**: Interactive multi-select to choose which lifecycle folders to create (`01-Active`, `02-Backlog`, `03-Review`, `04-Paused`, `05-Archive`, `06-Prototypes`, `07-Templates`, `08-Lab`).
4. **Automated Shell Integration**: Detects active shell (`PowerShell`, `Bash`, `Zsh`, `Fish`) and installs directory switching wrappers with a single confirmation.
5. **Config Outro**: Displays the saved configuration path and immediate next steps.

### ⌨️ Multi-Shell Autocompletion
RTB provides native autocompletion for project names, subcommands, and flags across all major shells:
```bash
# PowerShell (add to $PROFILE)
rtb completion pwsh | Out-String | Invoke-Expression

# Bash (add to ~/.bashrc)
eval "$(rtb completion bash)"

# Zsh (add to ~/.zshrc)
eval "$(rtb completion zsh)"

# Fish (add to ~/.config/fish/config.fish)
rtb completion fish | source
```

---

## 💻 Interactive TUI (`rtbtui` / `rtb ui`)

Launch the interactive terminal dashboard with:
```bash
rtb ui
# or directly
rtbtui
```

### Key Features & Shortcuts
- **`1-6` / `Tab`**: Switch views (1: Dashboard, 2: Projects, 3: Git Health, 4: Dep Cleaner, 5: Maintenance, 6: Dev Ports)
- **`↑/↓` or `j/k`**: List navigation
- **`x`**: **Run Live Program** — Spawns project dev server (`npm run dev`, `cargo run`, `python main.py`) in an interactive terminal window
- **`f` (Git Health)**: Cycle Git repository filters (`ALL`, `Needs Attention`, `Local Clean`, `Synced`, `Non-Git`)
- **`c` (Git Health)**: Open quick commit dialog
- **`R`**: Multi-threaded workspace refresh with live spinner indicator
- **`/`**: Global fuzzy search modal
- **`?`**: Toggle help and keyboard shortcuts overlay
- **`v`**: Open interactive Markdown viewer (`README.md`)
- **`q`**: Gracefully exit

---

## ⚙️ Configuration & Direct Editing
 
RTB configuration (`rtb.config.json`) is dynamically loaded in order of priority:

1. **User Profile**: `~/.config/rtb/rtb.config.json` (`%USERPROFILE%\.config\rtb\rtb.config.json` on Windows, `$HOME/.config/rtb/rtb.config.json` on Linux/macOS)
2. **Local Repository Fallback**: `config/rtb.config.json`

### Direct Configuration Editing (`rtb config`)

Run `rtb config` to immediately launch your configuration in your default editor (`$env:EDITOR`, VS Code, or Notepad):

```powershell
rtb config
```

### Customizing Emojis, Labels & Project Roots

You can customize the emoji icon, display label, and physical folder path for any lifecycle root directly in `rtb.config.json`:

```json
{
  "version": "1.0.0",
  "projectRoots": {
    "active": {
      "path": "D:\\02-Projects\\01-Development\\01-Active",
      "label": "Active Projects",
      "emoji": "⚡"
    },
    "paused": {
      "path": "D:\\02-Projects\\01-Development\\04-Paused",
      "label": "On Hold",
      "emoji": "⏸️"
    },
    "production": {
      "path": "D:\\02-Projects\\02-Deployed\\01-Production",
      "label": "Production Apps",
      "emoji": "🚀"
    }
  }
}
```

To re-scaffold or generate your configuration interactively:

```powershell
rtb init
# Or force re-initialization:
rtb init -Force
```

---

## 🧪 Beta Feedback & Bug Reports

Found a bug or have a suggestion?
- **Bug Reports**: Open an issue using the [Beta Bug Report Template](https://github.com/3mr-5aled/rtb/issues/new?template=bug_report.md).
- **Discussions & Ideas**: Join our [GitHub Discussions](https://github.com/3mr-5aled/rtb/discussions).

---

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for development prerequisites, local setup, test execution, and commit conventions before submitting pull requests.

---

## 📄 License & Liability Disclaimer

Distributed under the [MIT License](LICENSE). © 2026 Amr Khaled ([@3mr-5aled](https://github.com/3mr-5aled)).

> [!CAUTION]
> **Workspace Operations Disclaimer**: RTB performs automated local file operations including dependency pruning (`node_modules`, `target`, build artifacts), project archival, directory moves, and Git synchronizations. While confirmation prompts safeguard destructive actions, users remain solely responsible for backing up critical data and configurations. The authors and maintainers assume no liability for data loss or repository corruption.
