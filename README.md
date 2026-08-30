# RTB — رتّب (Repository & Tooling Base)

[![Version](https://img.shields.io/badge/version-v0.3.0-blue.svg)](https://github.com/3mr-5aled/rtb/releases)
[![Status: Beta](https://img.shields.io/badge/status-BETA-orange.svg)](https://github.com/3mr-5aled/rtb/issues)
[![PowerShell](https://img.shields.io/badge/PowerShell-7+-blue.svg)](https://microsoft.com/powershell)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

> [!NOTE]
> **Beta Pre-Release**: RTB is currently in active beta testing. We welcome feedback, bug reports, and suggestions via [GitHub Issues](https://github.com/3mr-5aled/rtb/issues) and [Discussions](https://github.com/3mr-5aled/rtb/discussions)!

**RTB — رتّب** is a fast, cross-platform Developer Project Operations Tool featuring a PowerShell CLI (`rtb`), an interactive Rust Terminal UI (`rtbtui`), multi-runtime project intelligence, Git telemetry monitoring, customizable live execution, and safe workspace management.

---

## 📁 Repository Structure

```
rtb/
├── config/
│   └── rtb.config.json     # Default JSON configuration template
├── cli/                    # PowerShell CLI module source & commands
│   ├── rtb.psd1            # Module Manifest
│   ├── rtb.psm1            # Primary CLI Module Entrypoint
│   ├── src/
│   │   ├── commands/       # Subcommands (init, run, build, test, commit, goto, etc.)
│   │   ├── completions/    # Shell completion scripts (ps1, bash, zsh, fish)
│   │   └── utils/          # Helpers & config loaders
│   └── tests/              # Pester unit tests
├── tui/                    # Rust Ratatui interactive TUI source
│   ├── Cargo.toml
│   ├── Cargo.lock
│   └── src/
├── .github/                # Workflows & Issue templates
│   ├── workflows/release.yml
│   └── ISSUE_TEMPLATE/
├── install.ps1             # Automated installer & profile integrator
├── PROJECT.md              # Project metadata
├── LICENSE                 # MIT License
└── README.md
```

---

## 🚀 Quick Start & Installation

### Option 1: Automatic Installer (Recommended)

1. Clone the repository:
   ```bash
   git clone https://github.com/3mr-5aled/rtb.git
   cd rtb
   ```

2. Run the PowerShell setup script:
   ```powershell
   pwsh -File ./install.ps1
   ```

   This will:
   - Build `rtbtui` binary via Cargo (if Rust is installed) or configure existing binaries.
   - Set up the PowerShell module autoload in your `$PROFILE`.
   - Register dynamic tab completions for `rtb` commands.

3. Initialize your workspace roots:
   ```powershell
   rtb init
   ```

---

### Option 2: Pre-compiled Binaries

Download pre-built standalone binaries directly from [GitHub Releases](https://github.com/3mr-5aled/rtb/releases).

---

## 🛠️ CLI Command Reference (`rtb`)

| Command                                             | Description                                                       |
| --------------------------------------------------- | ----------------------------------------------------------------- |
| `rtb init`                                          | Initialize user configuration in `%APPDATA%\rtb\rtb.config.json`  |
| `rtb run [project]`                                 | Auto-detect and run project dev/start scripts                     |
| `rtb commit [-Message <str>] [-Amend] [-Push]`      | Interactive CLI prompt to stage, commit, and push git changes     |
| `rtb build [project]`                               | Auto-detect and run project build scripts                         |
| `rtb test [project]`                                | Auto-detect and run project test suites                           |
| `rtb goto <project>`                                | Tab-complete fuzzy project directory navigation                   |
| `rtb ui` / `rtbtui`                                 | Launch interactive Ratatui TUI operations center                  |
| `rtb list [--active\|--paused\|--deployed\|--vibe]` | Filtered project status listing                                   |
| `rtb new <name>`                                    | Scaffold a new project                                            |
| `rtb pause <name>`                                  | Move project to `04-Paused` and prune dependencies                |
| `rtb resume <name>`                                 | Move project to `01-Active`                                       |
| `rtb deploy <name>`                                 | Move project to `02-Deployed`                                     |
| `rtb archive <name>`                                | Compress project into `.tar.gz` backup archive                    |
| `rtb unarchive <name>`                              | Restore archived project archive                                  |
| `rtb health`                                        | Perform Git repository health overview scan                       |
| `rtb clean [--dry-run\|--force]`                    | Safe dependency pruning (`node_modules`, `target`, `.venv`)       |
| `rtb --version` / `rtb --help`                      | View version and available command details                        |

---

## 💻 Interactive TUI (`rtbtui` / `rtb ui`)

Launch the interactive dashboard with:
```bash
rtb ui
# or
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

## ⚙️ Configuration Hierarchy

RTB configuration (`rtb.config.json`) is dynamically loaded in order of priority:

1. **User Profile**: `%APPDATA%\rtb\rtb.config.json` (Windows) or `~/.config/rtb/rtb.config.json` (Linux/macOS)
2. **Repository Fallback**: `config/rtb.config.json`

To generate your personalized user configuration:

```powershell
rtb init
```

---

## 🧪 Beta Feedback & Bug Reports

Found a bug or have a suggestion?
- **Bug Reports**: Open an issue using the [Beta Bug Report Template](https://github.com/3mr-5aled/rtb/issues/new?template=bug_report.md).
- **Discussions & Ideas**: Join our [GitHub Discussions](https://github.com/3mr-5aled/rtb/discussions).

---

## 📄 License

Distributed under the [MIT License](LICENSE). © 2026 Amr Khaled ([@3mr-5aled](https://github.com/3mr-5aled)).
