# RTB Codebase Architecture Deepening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor shallow modules in RTB CLI (`cli/`) and TUI (`tui/`) into deep, testable modules with clean domain seams as recorded in `CONTEXT.md`.

**Architecture:** 
1. Build a schema-backed **Project Intelligence Engine** (`ProjectInspector`) powering both CLI JSON output and TUI scanner.
2. Decouple `tui/src/app.rs` using a `TabController` trait seam forRatatui views.
3. Consolidate shallow maintenance commands into a portable **Maintenance Task Registry** (`MaintenanceTaskRegistry`).
4. Implement an **Agent Orchestrator** (`AgentOrchestrator`) for discovery, transient context injection, and process lifecycle management.

**Tech Stack:** PowerShell 7+, Rust 2021, Ratatui, Crossterm, Rayon, Serde JSON, Pester (cli tests), Cargo test.

**Spec:** `CONTEXT.md`

## Global Constraints

- Preserve all existing public CLI command entry points (`rtb`, `dev`, `rtb list`, `rtb info`, `rtb agent`, `rtb clean`, `rtb maintenance`).
- All new Rust code must pass `cargo test` clean without warnings or errors.
- All PowerShell changes must pass Pester unit tests (`cli/tests/*.tests.ps1`).
- No hardcoded absolute user directory paths; use `rtb.config.json` resolution or relative repo fallbacks.

---

### Task 1: Project Intelligence Engine (`ProjectInspector`)

**Files:**
- Create/Modify: `cli/src/utils/helpers.ps1`
- Modify: `tui/src/data/scanner.rs`
- Modify: `tui/src/data/project.rs`
- Test: `cli/tests/info.tests.ps1`
- Test: `tui/src/data/scanner.rs` (inline test module)

**Interfaces:**
- Consumes: Raw filesystem paths (`PathBuf` / `[string]$ProjectPath`)
- Produces: `Project` struct in Rust and PSCustomObject matching JSON contract in PowerShell (`rtb list --json`)

- [ ] **Step 1: Write failing Pester test for `Get-ProjectDetails` extended stack detection**

```powershell
Describe "Project Intelligence Engine" {
    It "Detects framework stack and monorepo workspace correctly" {
        $tempDir = Join-Path $env:TEMP "rtb_test_inspector"
        New-Item -Path $tempDir -ItemType Directory -Force | Out-Null
        @{ name = "test-app"; dependencies = @{ next = "14.0.0"; tailwindcss = "3.0.0" } } | ConvertTo-Json | Set-Content (Join-Path $tempDir "package.json")
        Set-Content (Join-Path $tempDir "pnpm-workspace.yaml") "packages: ['*']"

        $details = Get-ProjectDetails -ProjectPath $tempDir -Status 'Active'
        $details.stack | Should -Contain 'Next.js'
        $details.stack | Should -Contain 'Tailwind'
        $details.is_monorepo | Should -BeTrue

        Remove-Item -Path $tempDir -Recurse -Force
    }
}
```

- [ ] **Step 2: Run Pester test to verify setup**

Run: `powershell -NoProfile -Command "Invoke-Pester cli/tests/info.tests.ps1"`
Expected: Tests run cleanly and verify existing assertions.

- [ ] **Step 3: Refactor `Get-ProjectDetails` in `cli/src/utils/helpers.ps1` into a deep inspection pipeline**

Refactor `Get-ProjectDetails` to encapsulate stack taxonomy, monorepo workspace detection, CI/CD runners, and runtime version extraction in structured helper functions inside `helpers.ps1`.

- [ ] **Step 4: Update `tui/src/data/scanner.rs` unit tests for `ProjectInspector`**

```rust
#[test]
fn test_project_inspector_detects_nextjs_and_monorepo() {
    let temp_dir = std::env::temp_dir().join("rtb_rust_inspector_test");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let pkg_json = r#"{"name":"my-app","dependencies":{"next":"14.0.0","tailwindcss":"3.0.0"}}"#;
    fs::write(temp_dir.join("package.json"), pkg_json).unwrap();
    fs::write(temp_dir.join("pnpm-workspace.yaml"), "packages: ['*']").unwrap();

    let project = scan_project(&temp_dir, ProjectStatus::Active);
    assert!(project.stack.contains(&"Next.js".to_string()));
    assert!(project.stack.contains(&"Tailwind".to_string()));
    assert!(project.is_monorepo);

    let _ = fs::remove_dir_all(&temp_dir);
}
```

- [ ] **Step 5: Run Cargo test to verify Rust scanner**

Run: `powershell -NoProfile -Command "cargo test --manifest-path tui/Cargo.toml"`
Expected: PASS

- [ ] **Step 6: Commit Task 1**

```bash
git add cli/src/utils/helpers.ps1 tui/src/data/scanner.rs cli/tests/info.tests.ps1
git commit -m "refactor(engine): establish deep Project Intelligence Engine for CLI and TUI"
```

---

### Task 2: Decouple TUI App Monolith into `TabController` Modules

**Files:**
- Create: `tui/src/ui/tab.rs`
- Modify: `tui/src/ui/mod.rs`
- Modify: `tui/src/app.rs`
- Modify: `tui/src/ui/projects.rs`
- Modify: `tui/src/ui/dep_cleaner.rs`

**Interfaces:**
- Consumes: Crossterm `KeyCode`, `KeyModifiers`, Ratatui `Frame`, `Rect`
- Produces: `TabController` trait trait implementations for TUI views:
  ```rust
  pub trait TabController {
      fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool;
      fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &crate::app::App);
  }
  ```

- [ ] **Step 1: Create `tui/src/ui/tab.rs` defining `TabController` trait**

Define `TabController` trait and `TabOutcome` enum (`Handled`, `Ignored`, `QuitRequested`).

- [ ] **Step 2: Implement `TabController` for `ProjectsTab` view**

Encapsulate project list navigation (`Up`, `Down`, search filtering, sorting) inside `ProjectsTabController` in `tui/src/ui/projects.rs`.

- [ ] **Step 3: Update `App::handle_key` in `tui/src/app.rs` to delegate to `current_tab`**

Refactor `App::handle_key` to route unhandled keys to `self.get_active_tab_controller().handle_key(...)`.

- [ ] **Step 4: Verify TUI build and run tests**

Run: `powershell -NoProfile -Command "cargo check --manifest-path tui/Cargo.toml"`
Expected: Clean build without errors.

- [ ] **Step 5: Commit Task 2**

```bash
git add tui/src/ui/tab.rs tui/src/ui/mod.rs tui/src/ui/projects.rs tui/src/app.rs
git commit -m "refactor(tui): introduce TabController trait seam and decouple App key handler"
```

---

### Task 3: Maintenance Task Registry (`MaintenanceTaskRegistry`)

**Files:**
- Create/Modify: `cli/src/commands/maintenance.ps1`
- Modify: `tui/src/data/maintenance.rs`
- Modify: `cli/src/commands/guard.ps1`
- Modify: `cli/src/commands/backup.ps1`
- Modify: `cli/src/commands/env.ps1`

**Interfaces:**
- Consumes: Task name (`guard`, `backup`, `env`, `maintenance`) and configuration paths from `rtb.config.json`.
- Produces: `Invoke-MaintenanceTask` execution result and log streams.

- [ ] **Step 1: Refactor `cli/src/commands/maintenance.ps1` to implement `Invoke-MaintenanceTask`**

Add dynamic script path resolution checking `$config.maintenanceScripts.<task>`, falling back to `Join-Path $PSScriptRoot '../../scripts/<task>.ps1'`.

- [ ] **Step 2: Update pass-through commands (`guard.ps1`, `backup.ps1`, `env.ps1`) to use `Invoke-MaintenanceTask`**

Replace hardcoded path string invocations with `Invoke-MaintenanceTask -Task "guard" @args`.

- [ ] **Step 3: Update Rust `tui/src/data/maintenance.rs` task resolver**

Update `TaskDef::get_default_tasks()` in Rust to resolve relative script paths from working directory or config.

- [ ] **Step 4: Run Cargo and PowerShell tests**

Run: `powershell -NoProfile -Command "cargo test --manifest-path tui/Cargo.toml"`
Expected: PASS

- [ ] **Step 5: Commit Task 3**

```bash
git add cli/src/commands/maintenance.ps1 cli/src/commands/guard.ps1 cli/src/commands/backup.ps1 cli/src/commands/env.ps1 tui/src/data/maintenance.rs
git commit -m "refactor(maintenance): centralize portable script resolution behind MaintenanceTaskRegistry"
```

---

### Task 4: AI Agent Orchestrator & Context Injection (`AgentOrchestrator`)

**Files:**
- Modify: `cli/src/commands/agent.ps1`
- Modify: `tui/src/data/agents.rs`
- Test: `cli/tests/agent.tests.ps1`

**Interfaces:**
- Consumes: Project details (`Project` / `[PSCustomObject]`) and target agent name (`agy`, `claude`, `gemini`, `codex`).
- Produces: Transient `.rtb_context.md` file in target directory and process launch handle.

- [ ] **Step 1: Write Pester test for `Rtb-Agent` context file generation**

```powershell
Describe "Agent Orchestrator Context Generator" {
    It "Generates transient .rtb_context.md file in project folder before launch" {
        $tempDir = Join-Path $env:TEMP "rtb_agent_context_test"
        New-Item -Path $tempDir -ItemType Directory -Force | Out-Null

        $contextFile = Join-Path $tempDir ".rtb_context.md"
        # Test helper function
        New-RtbAgentContextFile -ProjectPath $tempDir -ProjectName "test-proj" -Stack @("Rust", "Ratatui")

        Test-Path $contextFile | Should -BeTrue
        (Get-Content $contextFile -Raw) | Should -Match "Rust, Ratatui"

        Remove-Item -Path $tempDir -Recurse -Force
    }
}
```

- [ ] **Step 2: Run Pester test to verify failure**

Run: `powershell -NoProfile -Command "Invoke-Pester cli/tests/agent.tests.ps1"`

- [ ] **Step 3: Implement `New-RtbAgentContextFile` in `agent.ps1` and `build_agent_context` in Rust `agents.rs`**

Add context markdown generation containing project name, path, tech stack, git branch, and README preview.

- [ ] **Step 4: Verify all tests pass**

Run: `powershell -NoProfile -Command "Invoke-Pester cli/tests/agent.tests.ps1"; cargo test --manifest-path tui/Cargo.toml`
Expected: ALL PASS

- [ ] **Step 5: Commit Task 4**

```bash
git add cli/src/commands/agent.ps1 tui/src/data/agents.rs cli/tests/agent.tests.ps1
git commit -m "feat(agent): implement AgentOrchestrator transient context injection and process runner"
```

---

## Self-Review

1. **Spec Coverage**: All 4 deepening opportunities from `CONTEXT.md` are covered in Tasks 1–4.
2. **Placeholder Scan**: Zero TBDs or unformatted placeholders.
3. **Type Consistency**: `Project`, `TabController`, `MaintenanceTaskRegistry`, and `AgentOrchestrator` maintain identical names across tasks.
