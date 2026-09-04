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

[![Version](https://img.shields.io/badge/version-v0.7.0-blue.svg)](https://github.com/3mr-5aled/rtb/releases)
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
│   ├── package.json        # Pure ESM module (@3mr-5aled/rtb)
│   ├── tsup.config.ts      # Multiplatform ESM bundler
│   ├── src/
│   │   ├── agent/          # AI agent discovery & .rtb_context.md generator
│   │   ├── commands/       # CLI commands (init, goto, agent, doctor, ui, run, build, test, etc.)
│   │   ├── config/         # Multi-tier config loader (~/.config/rtb/rtb.config.json)
│   │   ├── inspector/      # Multi-runtime project inspector (Node, Rust, Go, Python)
│   │   ├── navigation/     # Fuzzy scoring navigation engine
│   │   ├── services/       # Runner & Maintenance task registries
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
├── PROJECT.md              # Project architecture & milestone metadata
├── CONTEXT.md              # Domain model & architectural glossary
├── LICENSE                 # MIT License
└── README.md
```

---

## 🚀 Quick Start & Installation

### Option 1: Standalone One-Liner (Recommended)

#### 🪟 Windows (PowerShell 5.1+ / PowerShell 7+)
```powershell
irm https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.ps1 | iex
```

#### 🐧 Linux / 🍎 macOS
```bash
curl -fsSL https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.sh | sh
```

The interactive Setup Wizard will automatically:
- Display live animated progress and detect your OS, architecture, and shell environment.
- Prompt for install location (defaulting to `%APPDATA%\rtb` on Windows or `~/.config/rtb` on Unix).
- Download and configure the cross-platform TypeScript CLI engine (`rtb.js`, `rtb` wrapper scripts).
- Download the native TUI binary (`rtbtui`) and configure your system `PATH`.
- Configure module autoload in your shell configuration (`$PROFILE`, `.bashrc`, `.zshrc`, etc.) with legacy import deduplication.
- Prompt to immediately initialize your workspace via `rtb init`.

---

### Option 2: CI / Non-Interactive Automation

For headless pipelines (GitHub Actions, Docker, Azure Pipelines), pass quiet flags or environment variables:

**Windows PowerShell:**
```powershell
# Flag option
pwsh -File ./install.ps1 -Quiet -InstallPath "C:\tools\rtb"

# Environment variable option
$env:RTB_QUIET = "1"
irm https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.ps1 | iex
```

**Linux / macOS:**
```bash
# Quiet installation
curl -fsSL https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.sh | RTB_QUIET=1 sh

# Custom directory in CI
curl -fsSL https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.sh | RTB_QUIET=1 RTB_INSTALL_PATH="/opt/rtb" sh
```

---

### Option 3: Local Repository Installation (Developers & Contributors)

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
| `rtb init [--force]` | Interactive setup wizard (detects workspace root and scaffolds lifecycle folders) |
| `rtb config` | Open active `rtb.config.json` configuration file in default editor |
| `rtb doctor` | System health check (validates config, roots, git, runtimes, agents, and TUI binary) |
| `rtb shell-init <bash\|zsh\|fish\|pwsh>` | Output shell wrapper function enabling directory change on `rtb goto` |
| `rtb ui` | Launch native interactive Rust Terminal UI (`rtbtui`) |
| `rtb upgrade [--check] [--force]` | Check for newer releases and perform in-place self-upgrade |
| `rtb uninstall [--force] [-KeepConfig]` | Cleanly remove RTB binaries, module, and profile integrations |
| `rtb --version` / `rtb --help` | Display current version or command help menu |

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

## 📄 License

Distributed under the [MIT License](LICENSE). © 2026 Amr Khaled ([@3mr-5aled](https://github.com/3mr-5aled)).
