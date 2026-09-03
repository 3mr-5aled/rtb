# AGENTS.md — Agent Guidelines & Skills Configuration

## Agent skills

### Issue tracker

Issues and specs live in GitHub Issues. See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context domain model (`CONTEXT.md` + `docs/adr/`). See `docs/agents/domain.md`.

## Pre-Push Release Protocol (Mandatory for all Agents)

Whenever making changes to code, agents and developers must adhere to the following workflow:

1. **Commit Frequently**:
   - Create focused, atomic commits for logical units of work (using Conventional Commits format: `feat:`, `fix:`, `docs:`, `chore:`).
2. **Never Push Directly Without Releasing**:
   - Code changes must **never** be pushed to `main` without bumping the project version and updating the documentation.
   - Run the automated release orchestrator before pushing:
     ```powershell
     .\scripts\release.ps1 -Type patch -Message "Concise summary of changes"
     ```
     *(Or `-Type minor` for new features)*
3. **Automated Verification**:
   - Running `.\scripts\release.ps1`:
     - Updates `VERSION`, `core/package.json`, `core/package-lock.json`, `cli/rtb.psd1`, `tui/Cargo.toml`, and the `README.md` badge.
     - Adds a categorized entry in `CHANGELOG.md` under today's date.
     - Rebuilds `core` (`npm --prefix core run build`).
     - Creates the Git release commit and tags `vX.Y.Z`.
4. **Push with Tags**:
   - Push commits and tags together:
     ```bash
     git push origin main --follow-tags
     ```
   - A Git `pre-push` hook protects `main` and blocks any push containing code changes if `VERSION` or `CHANGELOG.md` was omitted.


