# Task 1 Brief: Extended Project Intelligence & CLI `--json`

## Requirements
1. **Extend Rust Scanner (`tui/src/data/scanner.rs` & `tui/src/data/project.rs`):**
   - Add `.NET` stack detection when `*.csproj` or `*.sln` files exist in project root.
   - Add Monorepo detection (`is_monorepo: bool`) when `pnpm-workspace.yaml`, `lerna.json`, `nx.json`, `turbo.json`, or `package.json` with a `"workspaces"` field is present.
   - Add CI/CD detection (`ci_cd: Option<String>`) for `.github/workflows` ("GitHub Actions"), `.gitlab-ci.yml` ("GitLab CI"), `azure-pipelines.yml` ("Azure Pipelines"), `.circleci` ("CircleCI").
   - Add Runtime version detection (`runtime_version: Option<String>`) from `.nvmrc`, `.python-version`, `rust-toolchain.toml`, or `package.json` engines field.

2. **Extend PowerShell Helpers & Commands (`cli/src/utils/helpers.ps1`, `cli/src/commands/list.ps1`, `cli/src/commands/info.ps1`):**
   - Update `cli/src/utils/helpers.ps1` to include `is_monorepo`, `ci_cd`, `runtime_version` in project objects returned.
   - Update `Rtb-List` (`cli/src/commands/list.ps1`) to support `-Json` / `--json` switch that outputs a formatted JSON array of projects.
   - Create `Rtb-Info` (`cli/src/commands/info.ps1`) to display detailed metadata for a single project (including stack, git status, monorepo status, CI/CD, and runtime version).
   - Export `info` in `cli/rtb.psm1` switch statement and completion scripts.

3. **Report Contract:**
   - Write full task report to `D:\02-Projects\01-Development\01-Active\dev-tools\.superpowers\sdd\rtb_phase2_plan\task-1-report.md`.
   - Return status `DONE` with commit hashes and test evidence (`cargo test` and `rtb test`).
