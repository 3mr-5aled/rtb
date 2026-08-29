# Task 4 Brief: Git Operations Extensions (Commit Amend, Branch Creation & Deletion)

## Requirements
1. **Branch Picker Modal Extensions (`tui/src/ui/branch_picker.rs`):**
   - Add `c` shortcut inside `BranchPickerModal` to create a new branch (`git checkout -b <name>`). Opens text input prompt for branch name.
   - Add `d` shortcut inside `BranchPickerModal` to delete selected branch (`git branch -d <name>`). Opens safety confirmation dialog before deletion.

2. **Commit Dialog Amend Option (`tui/src/ui/dialogs.rs` & `tui/src/app.rs`):**
   - Add `amend: bool` flag to commit dialog. When enabled (`Alt+A` or checkbox in dialog), executes `git commit --amend -m <message>`.

3. **Update Roadmap Checklist (`todo.md`):**
   - Check off (`- [x]`) completed Phase 2 items in `todo.md`.

4. **Testing & Verification:**
   - Verify Rust TUI compilation (`cargo check`).
   - Run full Pester test suite (`pwsh -NoProfile -Command "rtb test"`).
   - Write full task report to `D:\02-Projects\01-Development\01-Active\dev-tools\.superpowers\sdd\rtb_phase2_plan\task-4-report.md`.
