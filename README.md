# RTB — رتّب (Repository & Tooling Base)

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![PowerShell](https://img.shields.io/badge/PowerShell-7+-blue.svg)
![Rust](https://img.shields.io/badge/Rust-1.93+-orange.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)

**RTB — رتّب** is a professional, cross-platform Developer Project Operations Tool featuring a robust PowerShell CLI (`rtb`), an interactive Rust Terminal UI (`rtbtui`), multi-runtime project intelligence, lifecycle management, and safety-first operations.

---

## 📁 Repository Structure

```
dev-tools/
├── config/
│   └── rtb.config.json     # Default JSON configuration template
├── cli/                    # PowerShell CLI module source & commands
│   ├── rtb.psd1            # Module Manifest
│   ├── rtb.psm1            # Primary CLI Module Entrypoint
│   ├── src/
│   │   ├── commands/       # Subcommands (init, run, build, test, goto, etc.)
│   │   ├── completions/    # Shell completion scripts (ps1, bash, zsh, fish)
│   │   └── utils/          # Helpers & config loaders
│   └── tests/              # Pester unit tests
├── tui/                    # Rust Ratatui interactive TUI source
│   ├── Cargo.toml
│   ├── Cargo.lock
│   └── src/
├── install.ps1             # Installer & profile integrator script
├── PROJECT.md              # Project metadata
└── README.md
```

---

## 🚀 Installation & Setup

Run the installer script in PowerShell:

```powershell
pwsh -File 'D:\02-Projects\01-Development\01-Active\dev-tools\install.ps1'
```

This will automatically:
1. Verify/copy the built `rtbtui.exe` binary into `D:\06-Tools\scripts\` (in PATH).
2. Configure `Import-Module '.../cli/rtb.psd1' -Force` in your PowerShell `$PROFILE`.
3. Export `rtb` CLI commands and tab completion in your active terminal session.

---

## 🛠️ CLI Reference (`rtb`)

| Command | Description |
|---|---|
| `rtb init` | Initialize user configuration in `%APPDATA%\rtb\rtb.config.json` |
| `rtb run [project]` | Auto-detect and run project dev/start scripts |
| `rtb build [project]` | Auto-detect and run project build scripts |
| `rtb test [project]` | Auto-detect and run project test suites |
| `rtb goto <project>` | Tab-complete fuzzy project navigation |
| `rtb ui` / `rtbtui` | Launch interactive Ratatui TUI operations center |
| `rtb list [--active\|--paused\|--deployed\|--vibe]` | Filtered project status listing |
| `rtb new <name>` | Scaffold a new project |
| `rtb pause <name>` | Move project to `04-Paused` |
| `rtb resume <name>` | Move project to `01-Active` |
| `rtb deploy <name>` | Move project to `02-Deployed` |
| `rtb archive <name>` | Compress project into `.tar.gz` backup archive |
| `rtb unarchive <name>` | Restore archived project archive |
| `rtb health` | Perform Git repository health overview scan |
| `rtb clean [--dry-run\|--force]` | Dependency pruning with safety dry-run preview |
| `rtb --version` / `rtb --help` | View version and available command details |

---

## 💻 Interactive TUI (`rtbtui` / `rtb ui`)

Launch by running `rtb ui` or `rtbtui` directly in your terminal.

- **`1-6` / `Tab`**: Switch views (Dashboard, Projects, Git Health, Dep Cleaner, Maintenance, Dev Ports)
- **`j/k` or `↑/↓`**: Navigate lists with 1:1 precision
- **`/`**: Global fuzzy search modal
- **`?`**: Toggle help and keyboard shortcuts overlay
- **`v`**: Open interactive Markdown viewer (`README.md`, `PROJECT.md`)
- **`q`**: Gracefully exit without terminal state corruption

---

## ⚙️ Configuration Hierarchy

RTB configuration (`rtb.config.json`) is dynamically loaded in order of priority:
1. User Profile Configuration (`%APPDATA%\rtb\rtb.config.json` or `~/.config/rtb/rtb.config.json`)
2. Repository Fallback (`config/rtb.config.json`)

To generate your personalized user configuration, run:
```bash
rtb init
```

---

## 📄 License

[MIT License](LICENSE) © 2026 devamr
