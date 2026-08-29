# Task 3 Brief: TUI Command Palette (`Ctrl+P`), Toast Notifications & Tab Memory

## Requirements
1. **Command Palette (`tui/src/ui/command_palette.rs`):**
   - Create `CommandPalette` struct and UI renderer.
   - Shortcut trigger: `Ctrl+P` or `Ctrl+K` (or `p` in Dashboard view).
   - List actions:
     - `Dashboard`: Jump to Dashboard tab
     - `Projects`: Jump to Projects list tab
     - `Git Health`: Jump to Git Health tab
     - `Dep Cleaner`: Jump to Dependency Pruner tab
     - `Maintenance`: Jump to Maintenance tab
     - `Dev Ports`: Jump to Ports tab
     - `Scaffold`: Open Scaffold Project modal
     - `Search`: Open Global Search modal
     - `Launch Agent`: Open AI Agent for selected project
     - `Readme Viewer`: Open Markdown Readme viewer
     - `Refresh`: Reload projects cache
     - `Help`: Open Help overlay
   - Text input field with fuzzy filtering of actions. Enter executes selected action, Esc cancels.

2. **Toast Notification System (`tui/src/ui/toast.rs`):**
   - Create `ToastMessage` struct (`message: String`, `level: ToastLevel` [Info, Success, Warning, Error], `created_at: Instant`, `duration: Duration`).
   - Add `ToastQueue` inside `App` state in `tui/src/app.rs`.
   - Implement `show_toast(&mut self, message, level)` method.
   - Render top-right or bottom-right styled popup overlay in TUI. Auto-dismiss expired toasts.

3. **Session State Memory (`tui/src/data/cache.rs`):**
   - Create `SessionState` struct (`active_tab: usize`, `selected_project_name: Option<String>`).
   - Save session state to `%APPDATA%\rtb\state.json` or `~/.config/rtb/state.json` (fallback `config/.rtb_state.json`) upon TUI exit or tab switch.
   - Restore session state on TUI startup.

4. **Testing & Report:**
   - Add unit tests in Rust for Command Palette fuzzy filtering, Toast queue expiration, and SessionState serialization.
   - Verify build with `cargo test --manifest-path tui/Cargo.toml`.
   - Write full task report to `D:\02-Projects\01-Development\01-Active\dev-tools\.superpowers\sdd\rtb_phase2_plan\task-3-report.md`.
