# dev — Unified Developer Project Manager CLI

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![PowerShell](https://img.shields.io/badge/PowerShell-7+-blue.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)

Unified developer project manager CLI for D drive.

## Installation

Add the following line to your PowerShell `$PROFILE`:

```powershell
Import-Module 'D:\02-Projects\01-Development\01-Active\dev-tools\cli\dev.psd1' -Force
```

## Command Reference

The `dev` command provides the following subcommands:

| Command | Description |
|---|---|
| `dev goto <project>` | Tab-complete fuzzy project switcher |
| `dev new <name> [--stack react\|nextjs\|node\|python\|generic]` | Scaffold new project |
| `dev pause <name> [--prune]` | Pause project + optional dep pruning |
| `dev resume <name> [--install]` | Resume paused project + optional npm install |
| `dev deploy <name> [--prod\|--staging]` | Deploy project |
| `dev archive <name>` | Compress to .tar.gz in 08-Backup |
| `dev unarchive <name>` | Extract archive back to 01-Active |
| `dev list [--active\|--paused\|--deployed\|--vibe\|--all]` | Project listing with status |
| `dev health` | Git repo health scan |
| `dev clean [--force] [--days N]` | Dependency pruning |
| `dev index` | Generate PROJECT-INDEX.md |
| `dev backup` | Full config backup |
| `dev guard` | Root guardrail check |
| `dev env` | Backup .env files |
| `dev maintenance [--full]` | Run weekly maintenance |

## Tab Completion

The `dev` CLI includes full tab completion for project names, stacks, deployment environments, and flags. When you type `dev goto ` and press `Tab`, it will dynamically list available projects from the configured project roots.

## Configuration

The tool uses a JSON configuration file located at `config/dev.config.json` which maps project roots and defines settings for backups, templates, and dependency pruning.

## Contributing

Contributions, issues, and feature requests are welcome!

## License

This project is licensed under the [MIT License](LICENSE).
