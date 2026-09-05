# rtbtui

> Interactive Terminal UI Project Operations Engine for D: Drive development environment.

Built with Rust + Ratatui. Companion to the [`rtb` CLI](../core).

## Features

- **6 Interactive Views** — Dashboard, Projects, Git Health, Dep Cleaner, Maintenance, Dev Ports
- **RTB Branding** — Clean Arabic/English header branding (`RTB — ﺐﺘّﺭ`)
- **Color Technology Badges** — Color-coded tech stack chips in Projects Tab detail view
- **Live Program Runner** — Press `x` to launch project dev servers (`npm run dev`, `cargo run`, `python main.py`) in an interactive terminal window
- **Git Telemetry & Filtering** — Press `f` in Git Health tab to filter repos (`ALL`, `Needs Attention`, `Local Clean`, `Synced`, `Non-Git`)
- **Arrow-key & Vim navigation** with `j/k` bindings
- **Fuzzy search** with `/`
- **Single binary** — Zero runtime dependencies

## Installation

Run the installer script:
```powershell
pwsh -File install.ps1
```

Or build manually with Cargo:
```powershell
cargo build --release -p rtbtui
```

Run directly from any terminal:
```powershell
rtbtui
```

## Keyboard Shortcuts

| Key | Action |
|:----|:-------|
| `1-6` | Switch tab (Dashboard, Projects, Git, Clean, Maint, Ports) |
| `Tab` / `Shift+Tab` | Next/previous tab |
| `↑/↓` or `j/k` | Navigate list |
| `x` | Launch Live Program dev server in terminal window |
| `f` (Git Health) | Cycle repository filters (`Needs Attention`, `Local Clean`, `Synced`, etc.) |
| `c` (Git Health) | Open Git Commit & Push dialog popup |
| `R` | Global Refresh workspace & trigger multi-threaded scan |
| `r` | View-specific action (Resume paused project in Projects, Re-scan in Git Health/Cleaner/Ports) |
| `/` | Global fuzzy search |
| `?` | Help overlay |
| `Esc` | Close overlay / modal |
| `q` | Quit |

## Views

1. **Dashboard** — Workspace pulse, recent projects quick jump, action items, tech ecosystem bar, disk usage
2. **Projects** — Split-pane browser with colored stack badges, git status, README preview, and live runner (`x`)
3. **Git Health** — Filterable Git telemetry table (`f`), inline commit dialog (`c`), push/pull shortcuts, and loading status
4. **Dep Cleaner** — Checkbox-based node_modules / target directory dependency pruner
5. **Maintenance** — Multi-task maintenance task execution engine with live log streaming
6. **Dev Ports** — Port manager scanning active local dev servers (`:3000`, `:5173`, `:8080`) with kill switch

## Tech Stack

- **Rust 1.93+**
- **ratatui 0.29** — TUI framework
- **crossterm 0.28** — Cross-platform terminal backend
- **rayon** — Parallel multi-core workspace scanning
- **serde_json** — Configuration & cache serialization

## License

MIT © 2026 devamr
