# Contributing to RTB (Repository & Tooling Base)

Thank you for your interest in contributing to **RTB**! Whether you are reporting a bug, proposing a new feature, improving documentation, or submitting pull requests, your contributions are warmly welcome.

---

## 🛠️ Development Environment Prerequisites

Before contributing code, ensure your workstation meets the following prerequisites:

- **Node.js**: `v18.0.0` or higher (Active LTS or Current recommended).
- **npm**: `v9.0.0` or higher.
- **Rust Toolchain**: `v1.80.0` or higher with Cargo (`rustup update stable`).
  - On Windows: Either `x86_64-pc-windows-msvc` (with MSVC Build Tools) or `x86_64-pc-windows-gnu`.
- **Git**: Installed and available in your `PATH`.
- **PowerShell**: PowerShell 5.1+ (Windows built-in) or PowerShell 7+ (cross-platform `pwsh`).

---

## 🚀 Quick Setup & Local Build

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/3mr-5aled/rtb.git
   cd rtb
   ```

2. **Install Core CLI Dependencies**:
   ```bash
   npm --prefix core install
   ```

3. **Build Core CLI Engine**:
   ```bash
   npm --prefix core run build
   ```
   To run in watch/development mode:
   ```bash
   npm --prefix core run dev
   ```

4. **Build Native Terminal UI (`rtbtui`)**:
   ```bash
   cargo build --manifest-path tui/Cargo.toml
   ```

---

## 🧪 Running Tests & Quality Verification

All PRs and commits must pass linting, type checks, and automated test suites:

### Core TypeScript CLI
- **Run Unit Tests**:
  ```bash
  npm --prefix core run test
  ```
- **Type Checking**:
  ```bash
  npm --prefix core run typecheck
  ```

### Rust Terminal UI (`rtbtui`)
- **Check Compilation & Test Syntax**:
  ```bash
  cargo check --tests --manifest-path tui/Cargo.toml
  ```
- **Run Rust Unit Tests**:
  ```bash
  cargo test --manifest-path tui/Cargo.toml
  ```

---

## 📝 Commit Conventions (Conventional Commits)

This repository strictly enforces the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>
```

### Supported Types:
- `feat`: A new user-facing feature or CLI command.
- `fix`: A bug fix.
- `docs`: Documentation updates or additions.
- `refactor`: Code restructurings that neither fix bugs nor add features.
- `chore`: Maintenance tasks, dependency bumps, tooling updates.
- `test`: Adding or correcting tests.

### Examples:
- `feat(cli): add --json output flag to rtb doctor`
- `fix(tui): prevent panic on empty port process list`
- `docs(community): add CONTRIBUTING.md guide`

---

## 📦 Pre-Push & Release Protocol (Mandatory)

To guarantee that version identifiers across npm (`core/package.json`), Cargo (`tui/Cargo.toml`), `VERSION`, and `CHANGELOG.md` remain strictly synchronized, **never push commits directly to `main` without running the release orchestrator**:

```powershell
# For patch updates (bug fixes, small refactors):
.\scripts\release.ps1 -Type patch -Message "Concise summary of changes"

# For minor updates (new features, new commands):
.\scripts\release.ps1 -Type minor -Message "New feature description"
```

The automated release script will:
1. Bump version numbers across all package manifests and the README badge.
2. Compile and typecheck the TypeScript core bundle.
3. Update `CHANGELOG.md` with categorized release notes.
4. Create the Git release commit and annotated tag `vX.Y.Z`.

Push commits and tags together:
```bash
git push origin main --follow-tags
```

---

## ⚠️ Important Liability Notice for Contributors

RTB provides automation capabilities for local directory manipulation, dependency directory pruning (`node_modules`, `target`, `vendor`), project archival compression, and Git workspace synchronizations. When contributing features that mutate files or directories:

1. **Always implement safety confirmations** for destructive actions (e.g., interactive prompt or confirmation dialog).
2. **Never execute hard-coded unrecoverable deletions** without an explicit opt-in mechanism.
3. **Respect user configuration**: All paths and targets must respect user configuration in `rtb.config.json`.
