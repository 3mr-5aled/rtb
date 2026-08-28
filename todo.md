# RTB — TODO & Roadmap

## 🔴 Core

- [x] Rename project from `dev-tools` → `RTB`
- [x] Rename CLI command from `dev` → `rtb`
- [x] Rename TUI command from `devtui` → `rtb ui`
- [x] Update repository structure and internal references
- [x] Update README and documentation
- [x] Create RTB branding / logo (`RTB — ﺐﺘّﺭ`)
- [x] Define official tagline ("Repository & Tooling Base")
- [x] Define CLI naming conventions

## 🎨 TUI / UX

- [x] Redesign clean and informative start page
- [x] Project overview dashboard
- [x] Recent projects
- [x] Project status summary
- [x] Git status summary
- [x] Keyboard shortcut hints
- [x] Contextual footer
- [x] Consistent loading states
- [x] Consistent empty states
- [x] Consistent error states
- [x] Confirmation dialogs
- [ ] Toast / notification system
- [ ] Command palette
- [x] Project fuzzy search
- [x] Project filtering
- [x] Project sorting
- [ ] Remember last project/tab
- [x] Handle terminal resizing

## 🧠 Project Intelligence

- [x] Automatic project type detection
- [x] Node.js detection
- [x] Python detection
- [x] Rust detection
- [x] Go detection
- [x] Java detection
- [ ] .NET detection
- [x] Package manager detection (`npm`, `pnpm`, `yarn`, `bun`, `pip`, `poetry`, `uv`, `cargo`)
- [x] Framework detection (`Next.js`, `React`, `Vue`, `Vite`, `Tailwind`, `Prisma`, `Express`, `Fastify`)
- [ ] Monorepo detection
- [ ] Workspace detection
- [x] Git detection
- [x] GitHub remote detection
- [x] Docker detection
- [x] Docker Compose detection
- [ ] CI/CD detection
- [x] `.env` detection
- [ ] Runtime version detection

## 🔀 Git

- [x] Git repository health
- [x] Branch information
- [x] Dirty/clean status
- [x] Ahead/behind status
- [x] Untracked files
- [x] Staged files
- [x] Modified files
- [x] Recent commits
- [x] Commit details popup
- [x] Commit diff viewer
- [x] Stage/unstage files
- [x] Commit from TUI
- [ ] Amend commit
- [x] Push/pull/fetch
- [x] Branch switcher
- [ ] Branch creation/deletion

## 🤖 AI / Agents

- [ ] Detect installed AI agents
- [x] Open project in Agent CLI (`agy`)
- [ ] Codex integration
- [ ] Claude Code integration
- [ ] Gemini CLI integration
- [ ] Configurable default agent
- [ ] Generate project context
- [ ] Generate project summary
- [ ] Generate Git summary
- [ ] Launch agent with project context

## 📦 Environment

- [x] Runtime environment detection
- [x] Package manager detection
- [x] Dependency status
- [ ] Outdated dependency detection
- [x] Virtual environment detection (`.venv`)
- [ ] Node version manager detection
- [x] Rust toolchain detection
- [x] Environment variable detection (`.env`, `EnvVault`)
- [x] `.env` / `.env.example` validation
- [x] Running development server detection
- [x] Port detection

## 📖 Documentation

- [x] Styled Markdown viewer
- [x] Syntax highlighting
- [x] README viewer
- [x] PROJECT.md viewer
- [x] Documentation browser
- [x] Open Markdown in `$EDITOR`

## ⚙️ Configuration

- [x] Custom project roots
- [x] Multiple project roots
- [x] Custom backup location
- [x] Custom editor
- [x] Custom terminal
- [x] Default agent
- [x] TUI preferences
- [ ] Custom keybindings
- [x] Configuration validation
- [x] Configuration migration
- [x] `rtb config` / `rtb init`

## 🛡️ Safety

- [x] `--dry-run`
- [x] Confirmation for destructive operations
- [x] Operation preview
- [x] Safe dependency cleanup
- [x] Archive validation
- [x] Backup before destructive operations
- [x] Path traversal protection
- [x] Prevent operations outside configured project roots
- [x] Operation logging
- [x] Better error recovery

## 🧪 Testing

- [x] CLI unit tests (Pester)
- [x] TUI tests (Cargo)
- [x] Configuration tests
- [x] Filesystem tests
- [x] Git integration tests
- [ ] End-to-end tests
- [ ] Regression tests
- [x] Windows testing
- [ ] Linux testing
- [ ] WSL testing
- [x] Empty-folder testing
- [x] Custom-folder-structure testing
- [x] Large project testing
- [x] Run formatter before release
- [x] Run linter before release
- [x] Run full test suite before release
- [x] Run smoke test before release

## 🌍 Compatibility

- [x] Empty folder compatibility
- [x] Custom folder structures
- [x] Multiple project roots
- [ ] Nested projects
- [ ] Monorepos
- [x] Windows
- [x] Linux (path & config fallback)
- [x] macOS (path & config fallback)
- [ ] WSL
- [x] PowerShell
- [x] Bash
- [x] Zsh
- [x] Fish

## ⚡ Performance

- [x] Fast CLI startup (0ms cache load)
- [x] Fast TUI startup
- [x] Lazy project scanning
- [x] Project metadata caching
- [x] Background Git scanning
- [x] Background dependency scanning
- [x] Test with 100+ projects
- [ ] Test with 1000+ projects

## 📦 Distribution

- [x] Semantic versioning (v1.0.0)
- [ ] GitHub Releases
- [ ] Automated builds
- [x] Windows binary
- [ ] Linux binary
- [ ] macOS binary
- [ ] Checksums
- [ ] CI/CD
- [ ] Upgrade command
- [x] `rtb --version`
- [x] `rtb --help`

## 📚 Documentation

- [x] Rewrite README for RTB
- [x] Quick-start guide
- [x] CLI reference
- [x] TUI keyboard reference
- [x] Configuration reference
- [x] Architecture documentation
- [x] Troubleshooting
- [ ] FAQ
- [ ] Changelog
- [ ] Roadmap

## 👨‍💻 Developer Experience

- [x] Consistent CLI output
- [x] Consistent error messages
- [ ] `--verbose`
- [ ] `--quiet`
- [ ] `--json`
- [x] Shell completions (`ps1`, `bash`, `zsh`, `fish`)
- [x] Command aliases (`dev` → `rtb`, `devtui` → `rtb ui`)
- [x] Helpful command suggestions