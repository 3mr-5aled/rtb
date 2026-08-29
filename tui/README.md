# devtui

> Interactive terminal project manager for your D drive development environment.

Built with Rust + Ratatui. Companion to the [`dev` CLI](../cli).

## Features

- **5 interactive views** — Dashboard, Projects, Git Health, Dep Cleaner, Maintenance
- **Arrow-key navigation** with `j/k` vim bindings
- **Fuzzy search** with `/`
- **Split-pane project browser** with git status, stack detection, README preview
- **Git health table** across all repositories
- **Tab completion** between views with `1-5` or `Tab`
- **Single binary** — no runtime deps

## Installation

```powershell
cd D:\02-Projects\01-Development\01-Active\devtui
cargo build --release
# Copy binary to D:\06-Tools\scripts\
Copy-Item target\release\devtui.exe D:\06-Tools\scripts\devtui.exe
```

Add to `$PROFILE` or run directly:
```powershell
devtui
```

## Keyboard Shortcuts

| Key | Action |
|:----|:-------|
| `1-5` | Switch tab |
| `Tab` / `Shift+Tab` | Next/previous tab |
| `↑/↓` or `j/k` | Navigate list |
| `/` | Fuzzy search |
| `r` | Refresh |
| `?` | Help overlay |
| `Esc` | Close overlay |
| `q` | Quit |

## Views

1. **Dashboard** — Project counts, recent activity, git attention alerts
2. **Projects** — Split-pane browser with detail panel (git, size, README)
3. **Git Health** — Table of all repos with status
4. **Dep Cleaner** — Interactive checkbox pruning *(coming soon)*
5. **Maintenance** — Live task runner *(coming soon)*

## Configuration

Reads `D:\02-Projects\01-Development\01-Active\rtb-command-tool\config\dev.config.json` — same config shared with the `dev` PowerShell CLI.

## Tech Stack

- **Rust 1.93+**
- **ratatui 0.29** — TUI framework
- **crossterm 0.28** — Cross-platform terminal backend
- **walkdir** — Directory scanning
- **serde_json** — Config parsing

## License

MIT © 2026 devamr
