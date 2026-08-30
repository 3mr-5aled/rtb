# rtb — RTB (رتّب) PowerShell CLI Module

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![PowerShell](https://img.shields.io/badge/PowerShell-7+-blue.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)

Unified developer project operations CLI (`rtb` / `dev`) for multi-runtime projects on D: Drive.

## Installation

Add the following line to your PowerShell `$PROFILE` (or run `install.ps1` to configure automatically):

```powershell
Import-Module '<path-to-repo>\cli\rtb.psd1' -Force
```

## Command Reference

The `rtb` (or `dev`) command provides the following subcommands:

| Command                                                         | Description                                  |
| --------------------------------------------------------------- | -------------------------------------------- |
| `rtb goto <project>`                                            | Tab-complete fuzzy project switcher          |
| `rtb new <name> [--stack react\|nextjs\|node\|python\|generic]` | Scaffold new project                         |
| `rtb commit [-Message <str>] [-Amend] [-Push]`                  | Interactive CLI pop up & prompt to commit    |
| `rtb pause <name> [--prune]`                                    | Pause project + optional dep pruning         |
| `rtb resume <name> [--install]`                                 | Resume paused project + optional npm install |
| `rtb deploy <name> [--prod\|--staging]`                         | Deploy project                               |
| `rtb archive <name>`                                            | Compress to .tar.gz in 08-Backup             |
| `rtb unarchive <name>`                                          | Extract archive back to 01-Active            |
| `rtb list [--active\|--paused\|--deployed\|--vibe\|--all]`      | Project listing with status                  |
| `rtb health`                                                    | Git repo health scan                         |
| `rtb clean [--force] [--days N]`                                | Dependency pruning                           |
| `rtb index`                                                     | Generate PROJECT-INDEX.md                    |
| `rtb backup`                                                    | Full config backup                           |
| `rtb guard`                                                     | Root guardrail check                         |
| `rtb env`                                                       | Backup .env files                            |
| `rtb maintenance [--full]`                                      | Run weekly maintenance                       |
| `rtb ui` / `rtbtui`                                             | Launch interactive TUI                       |

## Tab Completion

The `rtb` CLI includes full tab completion for project names, stacks, deployment environments, and flags. When you type `rtb goto ` and press `Tab`, it will dynamically list available projects from the configured project roots.

## Configuration

The tool uses a JSON configuration file located at `%APPDATA%\rtb\rtb.config.json` which maps project roots and defines settings for backups, templates, and dependency pruning.

## License

This project is licensed under the [MIT License](LICENSE).
