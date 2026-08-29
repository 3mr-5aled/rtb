use crate::config::DevConfig;
use crate::data::actions;
use crate::data::agents;
use crate::data::cache::{load_cache, save_cache, SessionState};
use crate::data::deps::{format_bytes, prune_selected_folders, scan_dependencies, DepFolder};
use crate::data::disk::{calculate_disk_stats, DiskStats};
use crate::data::maintenance::{MaintenanceMessage, MaintenanceState, TaskStatus};
use crate::data::ports::{kill_port_process, scan_dev_ports, DevPort};
use crate::data::project::{Project, ProjectStatus};
use crate::data::scanner::scan_all_projects;
use crate::ui::branch_picker::BranchPickerModal;
use crate::ui::command_palette::{CommandPalette, PaletteAction};
use crate::ui::dialogs::{CommitDialog, ConfirmAction, ConfirmDialog};
use crate::ui::env_vault::EnvVaultModal;
use crate::ui::git_diff::GitDiffModal;
use crate::ui::scaffold::{ScaffoldModal, ScaffoldStep};
use crate::ui::toast::{ToastLevel, ToastQueue};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard = 0,
    Projects = 1,
    GitHealth = 2,
    DepCleaner = 3,
    Maintenance = 4,
    DevPorts = 5,
}

impl Tab {
    pub fn next(self) -> Self {
        match self {
            Tab::Dashboard => Tab::Projects,
            Tab::Projects => Tab::GitHealth,
            Tab::GitHealth => Tab::DepCleaner,
            Tab::DepCleaner => Tab::Maintenance,
            Tab::Maintenance => Tab::DevPorts,
            Tab::DevPorts => Tab::Dashboard,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Tab::Dashboard => Tab::DevPorts,
            Tab::Projects => Tab::Dashboard,
            Tab::GitHealth => Tab::Projects,
            Tab::DepCleaner => Tab::GitHealth,
            Tab::Maintenance => Tab::DepCleaner,
            Tab::DevPorts => Tab::Maintenance,
        }
    }
}

type ScanResult = (Vec<Project>, Vec<DepFolder>, DiskStats);

pub struct App {
    pub config: DevConfig,
    pub current_tab: Tab,
    pub projects: Vec<Project>,
    pub selected_index: usize,
    pub search_query: String,
    pub search_active: bool,
    pub should_quit: bool,
    pub show_help: bool,
    pub status_message: Option<String>,
    pub tick_count: u64,

    // Loading indicator & background scanning channel
    pub is_loading: bool,
    pub loading_message: &'static str,
    scan_receiver: Option<Receiver<ScanResult>>,

    // Maintenance background channel
    maintenance_receiver: Option<Receiver<MaintenanceMessage>>,

    // Interactive Modals
    pub command_palette: Option<CommandPalette>,
    pub toast_queue: ToastQueue,
    pub readme_modal: Option<(String, String, usize)>,
    pub env_vault_modal: Option<EnvVaultModal>,
    pub git_diff_modal: Option<GitDiffModal>,
    pub branch_picker_modal: Option<BranchPickerModal>,
    pub scaffold_modal: Option<ScaffoldModal>,

    // Dashboard state
    pub dashboard_selected_index: usize,

    // Port Manager state
    pub active_ports: Vec<DevPort>,
    pub ports_selected_index: usize,

    // Dep Cleaner state
    pub dep_folders: Vec<DepFolder>,
    pub cleaner_selected_index: usize,
    pub cleaner_threshold_days: u64,
    pub cleaner_category_filter: Option<ProjectStatus>,

    // Git Health state
    pub git_selected_index: usize,

    // Disk & Maintenance state
    pub disk_stats: DiskStats,
    pub maintenance_state: MaintenanceState,
    pub confirm_dialog: Option<ConfirmDialog>,
    pub commit_dialog: Option<CommitDialog>,
    pub project_filter: Option<ProjectStatus>,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = DevConfig::load()?;
        let maintenance_state = MaintenanceState::new();

        // 1. Fast Cache Loading (0ms Startup)
        let (cached_projects, cached_disk, had_cache) = if let Some((p, d)) = load_cache() {
            (p, d, true)
        } else {
            (
                Vec::new(),
                DiskStats {
                    total_d_drive_bytes: 103_488_700_416,
                    free_d_drive_bytes: 49_394_000_000,
                    used_d_drive_bytes: 0,
                    categories: Vec::new(),
                },
                false,
            )
        };

        let initial_ports = scan_dev_ports();

        // 2. Load Session State Memory
        let session = SessionState::load();
        let initial_tab = session.as_ref().map(|s| match s.active_tab {
            0 => Tab::Dashboard,
            1 => Tab::Projects,
            2 => Tab::GitHealth,
            3 => Tab::DepCleaner,
            4 => Tab::Maintenance,
            5 => Tab::DevPorts,
            _ => Tab::Dashboard,
        }).unwrap_or(Tab::Dashboard);

        let mut initial_selected_index = 0;
        if let Some(ref s) = session {
            if let Some(ref name) = s.selected_project_name {
                if let Some(pos) = cached_projects.iter().position(|p| &p.name == name) {
                    initial_selected_index = pos;
                }
            }
        }

        let mut app = App {
            config: config.clone(),
            current_tab: initial_tab,
            dashboard_selected_index: 0,
            projects: cached_projects,
            selected_index: initial_selected_index,
            search_query: String::new(),
            search_active: false,
            should_quit: false,
            show_help: false,
            status_message: None,
            tick_count: 0,
            is_loading: !had_cache,
            loading_message: "Scanning D: Drive Projects & Git Health...",
            scan_receiver: None,
            maintenance_receiver: None,
            command_palette: None,
            toast_queue: ToastQueue::new(),
            readme_modal: None,
            env_vault_modal: None,
            git_diff_modal: None,
            branch_picker_modal: None,
            scaffold_modal: None,
            active_ports: initial_ports,
            ports_selected_index: 0,
            dep_folders: Vec::new(),
            cleaner_selected_index: 0,
            cleaner_threshold_days: 60,
            cleaner_category_filter: None,
            git_selected_index: 0,
            disk_stats: cached_disk,
            maintenance_state,
            confirm_dialog: None,
            commit_dialog: None,
            project_filter: None,
        };

        // Start background scanner to verify state silently
        app.start_background_scan("Scanning D: Drive Projects & Git Health...");

        Ok(app)
    }

    pub fn show_toast(&mut self, message: impl Into<String>, level: ToastLevel) {
        self.toast_queue.push(message, level, std::time::Duration::from_secs(3));
    }

    pub fn save_session_state(&self) {
        let selected_project_name = self.selected_project().map(|p| p.name.clone());
        let state = SessionState::new(self.current_tab as usize, selected_project_name);
        let _ = state.save();
    }

    pub fn start_background_scan(&mut self, msg: &'static str) {
        if self.projects.is_empty() {
            self.is_loading = true;
            self.loading_message = msg;
        }

        let (tx, rx): (Sender<ScanResult>, Receiver<ScanResult>) = channel();
        self.scan_receiver = Some(rx);

        let config_clone = self.config.clone();
        let threshold = self.cleaner_threshold_days;

        thread::spawn(move || {
            let projects = scan_all_projects(&config_clone);
            let dep_folders = scan_dependencies(&config_clone, threshold);
            let disk_stats = calculate_disk_stats();
            let _ = tx.send((projects, dep_folders, disk_stats));
        });
    }

    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        // While initial loading on startup (if no cache), allow Esc/q to quit
        if self.is_loading && self.projects.is_empty() {
            if matches!(key, KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc) {
                self.should_quit = true;
                self.save_session_state();
                return true;
            }
            return false;
        }

        // Global Command Palette Trigger (Ctrl+P or Ctrl+K)
        let is_ctrl_p = modifiers.contains(KeyModifiers::CONTROL) && matches!(key, KeyCode::Char('p') | KeyCode::Char('P'));
        let is_ctrl_k = modifiers.contains(KeyModifiers::CONTROL) && matches!(key, KeyCode::Char('k') | KeyCode::Char('K'));
        if is_ctrl_p || is_ctrl_k {
            self.command_palette = Some(CommandPalette::new());
            return false;
        }

        // Command Palette modal handling
        if self.command_palette.is_some() {
            return self.handle_command_palette_key(key);
        }

        // Scaffold modal handling
        if self.scaffold_modal.is_some() {
            return self.handle_scaffold_key(key);
        }

        // Env Vault modal handling
        if self.env_vault_modal.is_some() {
            return self.handle_env_vault_key(key);
        }

        // Git Diff modal handling
        if self.git_diff_modal.is_some() {
            return self.handle_git_diff_key(key);
        }

        // Branch Picker modal handling
        if self.branch_picker_modal.is_some() {
            return self.handle_branch_picker_key(key);
        }

        // README viewer modal handling
        if self.readme_modal.is_some() {
            return self.handle_readme_modal_key(key);
        }

        // Commit dialog modal handling
        if self.commit_dialog.is_some() {
            return self.handle_commit_dialog_key(key, modifiers);
        }

        // Confirmation dialog modal
        if self.confirm_dialog.is_some() {
            return self.handle_dialog_key(key);
        }

        // Search mode overlay
        if self.search_active {
            return self.handle_search_key(key);
        }

        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT {
                    self.should_quit = true;
                    self.save_session_state();
                    return true;
                }
            }
            KeyCode::Char('?') => self.show_help = !self.show_help,
            KeyCode::Esc => {
                self.show_help = false;
                self.search_active = false;
                self.search_query.clear();
            }
            KeyCode::Tab => {
                self.current_tab = self.current_tab.next();
                self.save_session_state();
            }
            KeyCode::BackTab => {
                self.current_tab = self.current_tab.prev();
                self.save_session_state();
            }
            KeyCode::Char('1') => {
                self.current_tab = Tab::Dashboard;
                self.save_session_state();
            }
            KeyCode::Char('2') => {
                self.current_tab = Tab::Projects;
                self.save_session_state();
            }
            KeyCode::Char('3') => {
                self.current_tab = Tab::GitHealth;
                self.save_session_state();
            }
            KeyCode::Char('4') => {
                self.current_tab = Tab::DepCleaner;
                self.save_session_state();
            }
            KeyCode::Char('5') => {
                self.current_tab = Tab::Maintenance;
                self.save_session_state();
            }
            KeyCode::Char('6') => {
                self.current_tab = Tab::DevPorts;
                self.save_session_state();
            }
            KeyCode::Char('/') => {
                self.search_active = true;
                self.current_tab = Tab::Projects;
                self.save_session_state();
            }
            KeyCode::Char('r') => {
                self.active_ports = scan_dev_ports();
                self.start_background_scan("Refreshing workspace...");
            }

            // Route view-specific keys
            _ => match self.current_tab {
                Tab::Projects => self.handle_projects_key(key),
                Tab::GitHealth => self.handle_git_health_key(key),
                Tab::DepCleaner => self.handle_cleaner_key(key),
                Tab::Maintenance => self.handle_maintenance_key(key),
                Tab::DevPorts => self.handle_ports_key(key),
                Tab::Dashboard => {
                    let recent = self.recent_projects();
                    let count = recent.len();
                    if matches!(key, KeyCode::Down | KeyCode::Char('j')) {
                        if count > 0 {
                            self.dashboard_selected_index = (self.dashboard_selected_index + 1).min(count - 1);
                        }
                    } else if matches!(key, KeyCode::Up | KeyCode::Char('k')) {
                        self.dashboard_selected_index = self.dashboard_selected_index.saturating_sub(1);
                    } else if matches!(key, KeyCode::Char('p')) {
                        self.command_palette = Some(CommandPalette::new());
                    } else if matches!(key, KeyCode::Enter | KeyCode::Char('o')) {
                        if let Some(project) = recent.get(self.dashboard_selected_index).copied() {
                            actions::open_in_editor(project);
                            self.status_message = Some(format!("Opened {} in VS Code", project.name));
                        }
                    } else if matches!(key, KeyCode::Char('e')) {
                        if let Some(project) = recent.get(self.dashboard_selected_index).copied() {
                            actions::open_in_explorer(project);
                            self.status_message = Some(format!("Opened {} in Explorer", project.name));
                        }
                    } else if matches!(key, KeyCode::Char('v')) {
                        if let Some(project) = recent.get(self.dashboard_selected_index).copied() {
                            let readme_path = project.path.join("README.md");
                            let content = if readme_path.exists() {
                                std::fs::read_to_string(&readme_path)
                                    .unwrap_or_else(|_| "Error: Unable to read README.md".into())
                            } else {
                                format!("# {}\n\nNo README.md file found in this project.", project.name)
                            };
                            self.readme_modal = Some((project.name.clone(), content, 0));
                        }
                    } else if matches!(key, KeyCode::Char('d')) {
                        if let Some(project) = recent.get(self.dashboard_selected_index).copied() {
                            if let Some(diff) = GitDiffModal::load(project.name.clone(), &project.path) {
                                self.git_diff_modal = Some(diff);
                            }
                        }
                    }
                }
            },
        }
        false
    }

    fn handle_command_palette_key(&mut self, key: KeyCode) -> bool {
        if let Some(ref mut palette) = self.command_palette {
            match key {
                KeyCode::Esc => {
                    self.command_palette = None;
                }
                KeyCode::Up => {
                    palette.selected_index = palette.selected_index.saturating_sub(1);
                }
                KeyCode::Down => {
                    let count = palette.filtered_actions().len();
                    if count > 0 {
                        palette.selected_index = (palette.selected_index + 1).min(count - 1);
                    }
                }
                KeyCode::Backspace => {
                    palette.query.pop();
                    palette.selected_index = 0;
                }
                KeyCode::Enter => {
                    let actions = palette.filtered_actions();
                    if let Some(&action) = actions.get(palette.selected_index) {
                        self.command_palette = None;
                        self.execute_palette_action(action);
                    } else {
                        self.command_palette = None;
                    }
                }
                KeyCode::Char(c) => {
                    palette.query.push(c);
                    palette.selected_index = 0;
                }
                _ => {}
            }
        }
        false
    }

    pub fn execute_palette_action(&mut self, action: PaletteAction) {
        match action {
            PaletteAction::Dashboard => {
                self.current_tab = Tab::Dashboard;
                self.save_session_state();
            }
            PaletteAction::Projects => {
                self.current_tab = Tab::Projects;
                self.save_session_state();
            }
            PaletteAction::GitHealth => {
                self.current_tab = Tab::GitHealth;
                self.save_session_state();
            }
            PaletteAction::DepCleaner => {
                self.current_tab = Tab::DepCleaner;
                self.save_session_state();
            }
            PaletteAction::Maintenance => {
                self.current_tab = Tab::Maintenance;
                self.save_session_state();
            }
            PaletteAction::DevPorts => {
                self.current_tab = Tab::DevPorts;
                self.save_session_state();
            }
            PaletteAction::Scaffold => {
                self.scaffold_modal = Some(ScaffoldModal::new());
            }
            PaletteAction::Search => {
                self.search_active = true;
                self.current_tab = Tab::Projects;
                self.save_session_state();
            }
            PaletteAction::LaunchAgent => {
                if let Some(project) = self.selected_project().cloned() {
                    if let Some(agent) = agents::get_default_agent() {
                        let agent_name = agent.name.clone();
                        let proj_name = project.name.clone();
                        if agents::launch_agent(&project, None) {
                            self.show_toast(format!("Launched {} for {}", agent_name, proj_name), ToastLevel::Success);
                        } else {
                            self.show_toast("Failed to launch AI Agent", ToastLevel::Error);
                        }
                    } else {
                        self.show_toast("No installed AI Agent found in PATH", ToastLevel::Warning);
                    }
                } else {
                    self.show_toast("No project selected", ToastLevel::Warning);
                }
            }
            PaletteAction::ReadmeViewer => {
                if let Some(project) = self.selected_project().cloned() {
                    let readme_path = project.path.join("README.md");
                    let content = if readme_path.exists() {
                        std::fs::read_to_string(&readme_path)
                            .unwrap_or_else(|_| "Error: Unable to read README.md".into())
                    } else {
                        format!("# {}\n\nNo README.md file found in this project.", project.name)
                    };
                    self.readme_modal = Some((project.name.clone(), content, 0));
                } else {
                    self.show_toast("No project selected", ToastLevel::Warning);
                }
            }
            PaletteAction::Refresh => {
                self.active_ports = scan_dev_ports();
                self.start_background_scan("Refreshing workspace...");
                self.show_toast("Refreshing workspace cache...", ToastLevel::Info);
            }
            PaletteAction::Help => {
                self.show_help = true;
            }
        }
    }

    fn handle_scaffold_key(&mut self, key: KeyCode) -> bool {
        if let Some(ref mut modal) = self.scaffold_modal {
            match modal.step {
                ScaffoldStep::NameInput => match key {
                    KeyCode::Esc => self.scaffold_modal = None,
                    KeyCode::Enter => {
                        if !modal.project_name.trim().is_empty() {
                            modal.step = ScaffoldStep::CategorySelect;
                        }
                    }
                    KeyCode::Backspace => {
                        modal.project_name.pop();
                    }
                    KeyCode::Char(c) => {
                        if c.is_alphanumeric() || c == '-' || c == '_' {
                            modal.project_name.push(c);
                        }
                    }
                    _ => {}
                },
                ScaffoldStep::CategorySelect => match key {
                    KeyCode::Esc => modal.step = ScaffoldStep::NameInput,
                    KeyCode::Up | KeyCode::Char('k') => {
                        modal.category_index = modal.category_index.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if modal.category_index + 1 < ScaffoldModal::categories().len() {
                            modal.category_index += 1;
                        }
                    }
                    KeyCode::Enter => {
                        modal.step = ScaffoldStep::TemplateSelect;
                    }
                    _ => {}
                },
                ScaffoldStep::TemplateSelect => match key {
                    KeyCode::Esc => modal.step = ScaffoldStep::CategorySelect,
                    KeyCode::Up | KeyCode::Char('k') => {
                        modal.template_index = modal.template_index.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if modal.template_index + 1 < ScaffoldModal::templates().len() {
                            modal.template_index += 1;
                        }
                    }
                    KeyCode::Enter => {
                        modal.step = ScaffoldStep::Creating;
                        match modal.execute_scaffold() {
                            Ok(dir) => {
                                self.status_message = Some(format!("Scaffolded: {:?} — Opening VS Code!", dir));
                                self.scaffold_modal = None;
                                self.start_background_scan("Updating project list...");
                            }
                            Err(e) => {
                                modal.status_message = Some(e);
                                modal.step = ScaffoldStep::NameInput;
                            }
                        }
                    }
                    _ => {}
                },
                ScaffoldStep::Creating => {}
            }
        }
        false
    }

    fn handle_env_vault_key(&mut self, key: KeyCode) -> bool {
        if let Some(ref mut vault) = self.env_vault_modal {
            match key {
                KeyCode::Esc | KeyCode::Char('E') | KeyCode::Char('q') => {
                    self.env_vault_modal = None;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !vault.vars.is_empty() {
                        vault.selected_index = (vault.selected_index + 1).min(vault.vars.len() - 1);
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    vault.selected_index = vault.selected_index.saturating_sub(1);
                }
                KeyCode::Char(' ') => {
                    if let Some(v) = vault.vars.get_mut(vault.selected_index) {
                        v.is_revealed = !v.is_revealed;
                    }
                }
                KeyCode::Char('a') => {
                    let any_masked = vault.vars.iter().any(|v| !v.is_revealed);
                    for v in &mut vault.vars {
                        v.is_revealed = any_masked;
                    }
                }
                KeyCode::Char('b') => {
                    let backup_dir = std::path::Path::new("D:\\08-Backup\\env-secrets");
                    let _ = std::fs::create_dir_all(backup_dir);
                    let target_name = format!("{}-{}", vault.project_name, vault.file_name);
                    let _ = std::fs::copy(&vault.file_path, backup_dir.join(target_name));
                    self.status_message = Some("Backed up .env to D:\\08-Backup\\env-secrets\\".into());
                }
                _ => {}
            }
        }
        false
    }

    fn handle_git_diff_key(&mut self, key: KeyCode) -> bool {
        if let Some(ref mut diff) = self.git_diff_modal {
            match key {
                KeyCode::Esc | KeyCode::Char('d') | KeyCode::Char('q') => {
                    self.git_diff_modal = None;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    diff.scroll_offset += 1;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    diff.scroll_offset = diff.scroll_offset.saturating_sub(1);
                }
                KeyCode::PageDown => {
                    diff.scroll_offset += 15;
                }
                KeyCode::PageUp => {
                    diff.scroll_offset = diff.scroll_offset.saturating_sub(15);
                }
                _ => {}
            }
        }
        false
    }

    fn handle_branch_picker_key(&mut self, key: KeyCode) -> bool {
        if let Some(ref mut picker) = self.branch_picker_modal {
            if picker.creating_branch {
                match key {
                    KeyCode::Esc => {
                        picker.creating_branch = false;
                        picker.new_branch_name.clear();
                    }
                    KeyCode::Enter => {
                        let new_branch = picker.new_branch_name.trim().to_string();
                        if !new_branch.is_empty() {
                            let path = picker.repo_path.clone();
                            let output = Command::new("git")
                                .args(["checkout", "-b", &new_branch])
                                .current_dir(&path)
                                .output();
                            match output {
                                Ok(out) if out.status.success() => {
                                    self.status_message = Some(format!("Created and switched to branch '{}'", new_branch));
                                    self.branch_picker_modal = None;
                                    self.start_background_scan("Refreshing Git state...");
                                }
                                Ok(out) => {
                                    let err = String::from_utf8_lossy(&out.stderr);
                                    self.status_message = Some(format!("Failed to create branch '{}': {}", new_branch, err.trim()));
                                    picker.creating_branch = false;
                                    picker.new_branch_name.clear();
                                }
                                Err(e) => {
                                    self.status_message = Some(format!("Error creating branch: {}", e));
                                    picker.creating_branch = false;
                                    picker.new_branch_name.clear();
                                }
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        picker.new_branch_name.pop();
                    }
                    KeyCode::Char(c) => {
                        picker.new_branch_name.push(c);
                    }
                    _ => {}
                }
                return false;
            }

            match key {
                KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('q') => {
                    self.branch_picker_modal = None;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !picker.branches.is_empty() {
                        picker.selected_index = (picker.selected_index + 1).min(picker.branches.len() - 1);
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    picker.selected_index = picker.selected_index.saturating_sub(1);
                }
                KeyCode::Char('c') => {
                    picker.creating_branch = true;
                    picker.new_branch_name.clear();
                }
                KeyCode::Char('d') => {
                    if let Some(target_branch) = picker.branches.get(picker.selected_index).cloned() {
                        let repo_name = picker.repo_name.clone();
                        let repo_path = picker.repo_path.clone();
                        self.branch_picker_modal = None;
                        self.confirm_dialog = Some(ConfirmDialog {
                            title: "Delete Branch".into(),
                            message: format!("Delete branch '{}' in '{}'?", target_branch, repo_name),
                            action: ConfirmAction::DeleteGitBranch(target_branch, repo_path),
                        });
                    }
                }
                KeyCode::Enter => {
                    if let Some(target_branch) = picker.branches.get(picker.selected_index).cloned() {
                        let path = picker.repo_path.clone();
                        let _ = Command::new("git")
                            .args(["checkout", &target_branch])
                            .current_dir(&path)
                            .output();
                        self.status_message = Some(format!("Switched to branch '{}'", target_branch));
                        self.branch_picker_modal = None;
                        self.start_background_scan("Refreshing Git state...");
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn handle_commit_dialog_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        if let Some(ref mut dialog) = self.commit_dialog {
            if modifiers.contains(KeyModifiers::ALT) && matches!(key, KeyCode::Char('a') | KeyCode::Char('A')) {
                dialog.amend = !dialog.amend;
                return false;
            }

            match key {
                KeyCode::Esc => {
                    self.commit_dialog = None;
                }
                KeyCode::Enter => {
                    let repo_path = dialog.repo_path.clone();
                    let repo_name = dialog.repo_name.clone();
                    let msg = dialog.message.trim().to_string();
                    let amend = dialog.amend;
                    self.commit_dialog = None;

                    self.status_message = Some(format!("Committing changes to {}...", repo_name));
                    thread::spawn(move || {
                        let _ = Command::new("git").args(["add", "."]).current_dir(&repo_path).output();
                        let output = if amend {
                            if msg.is_empty() {
                                Command::new("git").args(["commit", "--amend", "--no-edit"]).current_dir(&repo_path).output()
                            } else {
                                Command::new("git").args(["commit", "--amend", "-m", &msg]).current_dir(&repo_path).output()
                            }
                        } else {
                            let commit_msg = if msg.is_empty() { "update: sync workspace changes" } else { &msg };
                            Command::new("git").args(["commit", "-m", commit_msg]).current_dir(&repo_path).output()
                        };
                        let _ = output;
                    });
                    self.start_background_scan("Refreshing Git state...");
                }
                KeyCode::Backspace => {
                    dialog.message.pop();
                }
                KeyCode::Char(c) => {
                    dialog.message.push(c);
                }
                _ => {}
            }
        }
        false
    }

    fn handle_ports_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.active_ports.is_empty() {
                    self.ports_selected_index = (self.ports_selected_index + 1).min(self.active_ports.len() - 1);
                }
            }
            KeyCode::Up => {
                self.ports_selected_index = self.ports_selected_index.saturating_sub(1);
            }
            KeyCode::Char('k') => {
                if let Some(port) = self.active_ports.get(self.ports_selected_index) {
                    let pid = port.pid;
                    if kill_port_process(pid) {
                        self.status_message = Some(format!("Terminated process PID {} on port :{}", pid, port.port));
                    } else {
                        self.status_message = Some(format!("Failed to kill PID {}", pid));
                    }
                    self.active_ports = scan_dev_ports();
                }
            }
            KeyCode::Char('o') => {
                if let Some(port) = self.active_ports.get(self.ports_selected_index) {
                    let url = format!("http://localhost:{}", port.port);
                    let _ = Command::new("cmd").args(["/C", "start", &url]).spawn();
                    self.status_message = Some(format!("Opening {}", url));
                }
            }
            KeyCode::Char('r') => {
                self.active_ports = scan_dev_ports();
                self.status_message = Some("Dev ports rescanned".into());
            }
            _ => {}
        }
    }

    fn handle_readme_modal_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') => {
                self.readme_modal = None;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some((_, _, ref mut scroll)) = self.readme_modal {
                    *scroll += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some((_, _, ref mut scroll)) = self.readme_modal {
                    *scroll = scroll.saturating_sub(1);
                }
            }
            KeyCode::PageDown => {
                if let Some((_, _, ref mut scroll)) = self.readme_modal {
                    *scroll += 10;
                }
            }
            KeyCode::PageUp => {
                if let Some((_, _, ref mut scroll)) = self.readme_modal {
                    *scroll = scroll.saturating_sub(10);
                }
            }
            _ => {}
        }
        false
    }

    fn handle_dialog_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                if let Some(dialog) = self.confirm_dialog.take() {
                    self.execute_confirmed_action(dialog.action);
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.confirm_dialog = None;
                self.status_message = Some("Action cancelled".into());
            }
            _ => {}
        }
        false
    }

    fn handle_projects_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Char('o') | KeyCode::Enter => {
                if let Some(project) = self.selected_project() {
                    actions::open_in_editor(project);
                    self.status_message = Some(format!("Opened {} in VS Code", project.name));
                }
            }
            KeyCode::Char('v') => {
                if let Some(project) = self.selected_project() {
                    let readme_path = project.path.join("README.md");
                    let content = if readme_path.exists() {
                        std::fs::read_to_string(&readme_path)
                            .unwrap_or_else(|_| "Error: Unable to read README.md".into())
                    } else {
                        format!("# {}\n\nNo README.md file found in this project.", project.name)
                    };
                    self.readme_modal = Some((project.name.clone(), content, 0));
                }
            }
            KeyCode::Char('N') => {
                self.scaffold_modal = Some(ScaffoldModal::new());
            }
            KeyCode::Char('E') => {
                if let Some(project) = self.selected_project() {
                    if let Some(vault) = EnvVaultModal::load(project.name.clone(), &project.path) {
                        self.env_vault_modal = Some(vault);
                    } else {
                        self.status_message = Some(format!("No .env file found in {}", project.name));
                    }
                }
            }
            KeyCode::Char('d') => {
                if let Some(project) = self.selected_project() {
                    if let Some(diff) = GitDiffModal::load(project.name.clone(), &project.path) {
                        self.git_diff_modal = Some(diff);
                    }
                }
            }
            KeyCode::Char('e') => {
                if let Some(project) = self.selected_project() {
                    actions::open_in_explorer(project);
                    self.status_message = Some(format!("Opened {} in Explorer", project.name));
                }
            }
            KeyCode::Char('p') => {
                if let Some(project) = self.selected_project() {
                    if project.status == ProjectStatus::Active {
                        self.confirm_dialog = Some(ConfirmDialog {
                            title: "Pause Project".into(),
                            message: format!(
                                "Move '{}' to 04-Paused and prune dependencies?",
                                project.name
                            ),
                            action: ConfirmAction::PauseProject(project.name.clone()),
                        });
                    } else {
                        self.status_message = Some("Only active projects can be paused".into());
                    }
                }
            }
            KeyCode::Char('r') => {
                if let Some(project) = self.selected_project() {
                    if project.status == ProjectStatus::Paused {
                        self.confirm_dialog = Some(ConfirmDialog {
                            title: "Resume Project".into(),
                            message: format!(
                                "Move '{}' back to 01-Active?",
                                project.name
                            ),
                            action: ConfirmAction::ResumeProject(project.name.clone()),
                        });
                    }
                }
            }
            KeyCode::Char('D') => {
                if let Some(project) = self.selected_project() {
                    if project.status == ProjectStatus::Active {
                        self.confirm_dialog = Some(ConfirmDialog {
                            title: "Deploy Project".into(),
                            message: format!(
                                "Move '{}' to 02-Deployed/01-Production?",
                                project.name
                            ),
                            action: ConfirmAction::DeployProject(project.name.clone(), true),
                        });
                    }
                }
            }
            KeyCode::Char('a') => {
                if let Some(project) = self.selected_project() {
                    if let Some(agent) = agents::get_default_agent() {
                        let agent_name = agent.name.clone();
                        let proj_name = project.name.clone();
                        if agents::launch_agent(project, None) {
                            self.status_message = Some(format!("Launched {} for {}", agent_name, proj_name));
                        } else {
                            self.status_message = Some("Failed to launch AI Agent".into());
                        }
                    } else {
                        self.status_message = Some("No installed AI Agent found in PATH (agy, claude, gemini, codex)".into());
                    }
                }
            }
            KeyCode::Char('A') => {
                if let Some(project) = self.selected_project() {
                    self.confirm_dialog = Some(ConfirmDialog {
                        title: "Archive Project".into(),
                        message: format!(
                            "Compress '{}' to 08-Backup/project-archives/?",
                            project.name
                        ),
                        action: ConfirmAction::ArchiveProject(project.name.clone()),
                    });
                }
            }
            KeyCode::Char('f') => {
                self.project_filter = match self.project_filter {
                    None => Some(ProjectStatus::Active),
                    Some(ProjectStatus::Active) => Some(ProjectStatus::Paused),
                    Some(ProjectStatus::Paused) => Some(ProjectStatus::Production),
                    Some(ProjectStatus::Production) => Some(ProjectStatus::Vibe),
                    Some(ProjectStatus::Vibe) => Some(ProjectStatus::Sandbox),
                    Some(ProjectStatus::Sandbox) => None,
                    _ => None,
                };
                self.selected_index = 0;
            }
            _ => {}
        }
    }

    fn handle_git_health_key(&mut self, key: KeyCode) {
        let git_projects: Vec<&Project> = self.projects.iter().filter(|p| p.git.is_some()).collect();
        match key {
            KeyCode::Down | KeyCode::Char('j') => {
                if !git_projects.is_empty() {
                    self.git_selected_index = (self.git_selected_index + 1).min(git_projects.len() - 1);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.git_selected_index = self.git_selected_index.saturating_sub(1);
            }
            KeyCode::Char('o') | KeyCode::Enter => {
                if let Some(project) = git_projects.get(self.git_selected_index) {
                    actions::open_in_editor(project);
                    self.status_message = Some(format!("Opened {} in VS Code", project.name));
                }
            }
            KeyCode::Char('d') => {
                if let Some(project) = git_projects.get(self.git_selected_index) {
                    if let Some(diff) = GitDiffModal::load(project.name.clone(), &project.path) {
                        self.git_diff_modal = Some(diff);
                    }
                }
            }
            KeyCode::Char('b') => {
                if let Some(project) = git_projects.get(self.git_selected_index) {
                    if let Some(picker) = BranchPickerModal::load(project.name.clone(), &project.path) {
                        self.branch_picker_modal = Some(picker);
                    }
                }
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if let Some(project) = git_projects.get(self.git_selected_index) {
                    self.commit_dialog = Some(CommitDialog::new(project.name.clone(), project.path.clone()));
                }
            }
            KeyCode::Char('P') => {
                if let Some(project) = git_projects.get(self.git_selected_index) {
                    let path = project.path.clone();
                    let name = project.name.clone();
                    self.status_message = Some(format!("Pushing {}...", name));
                    thread::spawn(move || {
                        let _ = Command::new("git").args(["push"]).current_dir(&path).output();
                    });
                }
            }
            KeyCode::Char('p') => {
                if let Some(project) = git_projects.get(self.git_selected_index) {
                    let path = project.path.clone();
                    let name = project.name.clone();
                    self.status_message = Some(format!("Pulling {}...", name));
                    thread::spawn(move || {
                        let _ = Command::new("git").args(["pull"]).current_dir(&path).output();
                    });
                }
            }
            KeyCode::Char('r') => {
                self.start_background_scan("Re-scanning Git health...");
            }
            _ => {}
        }
    }

    fn handle_cleaner_key(&mut self, key: KeyCode) {
        let count = self.filtered_dep_folders().len();
        match key {
            KeyCode::Down | KeyCode::Char('j') => {
                if count > 0 {
                    self.cleaner_selected_index = (self.cleaner_selected_index + 1).min(count - 1);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.cleaner_selected_index = self.cleaner_selected_index.saturating_sub(1);
            }
            KeyCode::Char(' ') => {
                let filtered_indices: Vec<usize> = self.dep_folders
                    .iter()
                    .enumerate()
                    .filter(|(_, d)| {
                        if let Some(cat) = &self.cleaner_category_filter {
                            &d.project_status == cat
                        } else {
                            true
                        }
                    })
                    .map(|(i, _)| i)
                    .collect();

                if let Some(&actual_idx) = filtered_indices.get(self.cleaner_selected_index) {
                    if let Some(folder) = self.dep_folders.get_mut(actual_idx) {
                        folder.is_selected = !folder.is_selected;
                    }
                }
            }
            KeyCode::Char('c') => {
                self.cleaner_category_filter = match self.cleaner_category_filter {
                    None => Some(ProjectStatus::Paused),
                    Some(ProjectStatus::Paused) => Some(ProjectStatus::Abandoned),
                    Some(ProjectStatus::Abandoned) => Some(ProjectStatus::Active),
                    Some(ProjectStatus::Active) => Some(ProjectStatus::Vibe),
                    Some(ProjectStatus::Vibe) => Some(ProjectStatus::Sandbox),
                    Some(ProjectStatus::Sandbox) => None,
                    _ => None,
                };
                self.cleaner_selected_index = 0;
            }
            KeyCode::Char('a') => {
                for folder in &mut self.dep_folders {
                    if let Some(cat) = &self.cleaner_category_filter {
                        if &folder.project_status == cat {
                            folder.is_selected = true;
                        }
                    } else {
                        folder.is_selected = true;
                    }
                }
            }
            KeyCode::Char('n') => {
                for folder in &mut self.dep_folders {
                    folder.is_selected = false;
                }
            }
            KeyCode::Enter => {
                let selected_count = self.dep_folders.iter().filter(|f| f.is_selected).count();
                let selected_bytes: u64 = self.dep_folders.iter().filter(|f| f.is_selected).map(|f| f.size_bytes).sum();
                if selected_count > 0 {
                    self.confirm_dialog = Some(ConfirmDialog {
                        title: "Prune Selected Dependencies".into(),
                        message: format!(
                            "Permanently delete {} dependency folder(s) reclaiming {}?",
                            selected_count,
                            format_bytes(selected_bytes)
                        ),
                        action: ConfirmAction::PruneDependencies,
                    });
                } else {
                    self.status_message = Some("No folders selected for pruning".into());
                }
            }
            KeyCode::Char('r') => {
                self.start_background_scan("Re-scanning dependencies...");
            }
            _ => {}
        }
    }

    fn execute_confirmed_action(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::PauseProject(name) => {
                if let Some(project) = self.projects.iter().find(|p| p.name == name).cloned() {
                    match actions::pause_project(&project, &self.config) {
                        Ok(target) => {
                            self.status_message = Some(format!("Paused {} -> {:?}", name, target));
                            self.start_background_scan("Refreshing workspace after pause...");
                        }
                        Err(e) => self.status_message = Some(format!("Error: {}", e)),
                    }
                }
            }
            ConfirmAction::ResumeProject(name) => {
                if let Some(project) = self.projects.iter().find(|p| p.name == name).cloned() {
                    match actions::resume_project(&project, &self.config) {
                        Ok(target) => {
                            self.status_message = Some(format!("Resumed {} -> {:?}", name, target));
                            self.start_background_scan("Refreshing workspace after resume...");
                        }
                        Err(e) => self.status_message = Some(format!("Error: {}", e)),
                    }
                }
            }
            ConfirmAction::DeployProject(name, is_prod) => {
                if let Some(project) = self.projects.iter().find(|p| p.name == name).cloned() {
                    match actions::deploy_project(&project, &self.config, is_prod) {
                        Ok(target) => {
                            self.status_message = Some(format!("Deployed {} -> {:?}", name, target));
                            self.start_background_scan("Refreshing workspace after deploy...");
                        }
                        Err(e) => self.status_message = Some(format!("Error: {}", e)),
                    }
                }
            }
            ConfirmAction::ArchiveProject(name) => {
                if let Some(project) = self.projects.iter().find(|p| p.name == name).cloned() {
                    match actions::archive_project(&project, &self.config) {
                        Ok(target) => {
                            self.status_message = Some(format!("Archived {} -> {:?}", name, target));
                            self.start_background_scan("Refreshing workspace after archive...");
                        }
                        Err(e) => self.status_message = Some(format!("Error: {}", e)),
                    }
                }
            }
            ConfirmAction::PruneDependencies => {
                let (freed, count) = prune_selected_folders(&mut self.dep_folders);
                self.status_message = Some(format!("Pruned {} folders, freed {}", count, format_bytes(freed as u64)));
                self.start_background_scan("Updating dependency list...");
            }
            ConfirmAction::GitCommitAndPush(name, path) => {
                self.status_message = Some(format!("Syncing {} in background...", name));
                thread::spawn(move || {
                    let _ = Command::new("git").args(["add", "."]).current_dir(&path).output();
                    let _ = Command::new("git").args(["commit", "-m", "update: sync workspace changes"]).current_dir(&path).output();
                    let _ = Command::new("git").args(["push"]).current_dir(&path).output();
                });
            }
            ConfirmAction::GitPush(name, path) => {
                self.status_message = Some(format!("Pushing {}...", name));
                thread::spawn(move || {
                    let _ = Command::new("git").args(["push"]).current_dir(&path).output();
                });
            }
            ConfirmAction::GitPull(name, path) => {
                self.status_message = Some(format!("Pulling {}...", name));
                thread::spawn(move || {
                    let _ = Command::new("git").args(["pull"]).current_dir(&path).output();
                });
            }
            ConfirmAction::DeleteGitBranch(branch, path) => {
                let output = Command::new("git")
                    .args(["branch", "-d", &branch])
                    .current_dir(&path)
                    .output();
                match output {
                    Ok(out) if out.status.success() => {
                        self.status_message = Some(format!("Deleted branch '{}'", branch));
                        self.start_background_scan("Refreshing Git state...");
                    }
                    Ok(out) => {
                        let err_msg = String::from_utf8_lossy(&out.stderr);
                        self.status_message = Some(format!("Failed to delete branch '{}': {}", branch, err_msg.trim()));
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error deleting branch: {}", e));
                    }
                }
            }
        }
    }

    fn handle_maintenance_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.maintenance_state.tasks.is_empty() {
                    self.maintenance_state.selected_task = (self.maintenance_state.selected_task + 1).min(self.maintenance_state.tasks.len() - 1);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.maintenance_state.selected_task = self.maintenance_state.selected_task.saturating_sub(1);
            }
            KeyCode::Enter => {
                if self.maintenance_state.is_running {
                    self.status_message = Some("Maintenance tasks are already running in background".into());
                    return;
                }
                self.maintenance_state.is_running = true;
                self.maintenance_state.logs.clear();

                let (tx, rx) = channel();
                self.maintenance_receiver = Some(rx);

                let all_tasks = self.maintenance_state.tasks.clone();
                let indices: Vec<usize> = (0..all_tasks.len()).collect();
                MaintenanceState::start_background_tasks(all_tasks, indices, tx);
                self.status_message = Some("Running all maintenance tasks in background...".into());
            }
            KeyCode::Char('s') => {
                if self.maintenance_state.is_running {
                    self.status_message = Some("Maintenance task already running in background".into());
                    return;
                }
                let idx = self.maintenance_state.selected_task;
                if let Some(task) = self.maintenance_state.tasks.get(idx).cloned() {
                    self.maintenance_state.is_running = true;
                    let (tx, rx) = channel();
                    self.maintenance_receiver = Some(rx);

                    MaintenanceState::start_background_tasks(vec![task], vec![idx], tx);
                    self.status_message = Some(format!("Running task #{} in background...", idx + 1));
                }
            }
            KeyCode::Char('c') => {
                self.maintenance_state.logs.clear();
            }
            _ => {}
        }
    }

    fn handle_search_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Esc => {
                self.search_active = false;
                self.search_query.clear();
            }
            KeyCode::Enter => {
                self.search_active = false;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.selected_index = 0;
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.selected_index = 0;
            }
            _ => {}
        }
        false
    }

    fn move_selection(&mut self, delta: i32) {
        let filtered = self.filtered_projects();
        if filtered.is_empty() {
            return;
        }
        let len = filtered.len() as i32;
        let new = (self.selected_index as i32 + delta).rem_euclid(len);
        self.selected_index = new as usize;
    }

    pub fn filtered_projects(&self) -> Vec<&Project> {
        let mut list: Vec<&Project> = self.projects.iter().collect();

        // Apply status filter
        if let Some(status) = &self.project_filter {
            list.retain(|p| &p.status == status);
        }

        // Apply search query
        if !self.search_query.is_empty() {
            let q = self.search_query.to_lowercase();
            list.retain(|p| p.name.to_lowercase().contains(&q));
        }

        list
    }

    pub fn filtered_dep_folders(&self) -> Vec<&DepFolder> {
        let mut list: Vec<&DepFolder> = self.dep_folders.iter().collect();
        if let Some(status) = &self.cleaner_category_filter {
            list.retain(|d| &d.project_status == status);
        }
        list
    }

    pub fn selected_project(&self) -> Option<&Project> {
        let filtered = self.filtered_projects();
        filtered.get(self.selected_index).copied()
    }

    pub fn on_tick(&mut self) {
        self.tick_count += 1;

        // Cleanup expired toast notifications
        self.toast_queue.cleanup_expired();

        // Check if background worker finished workspace scanning
        if let Some(rx) = &self.scan_receiver {
            if let Ok((projects, dep_folders, disk_stats)) = rx.try_recv() {
                self.projects = projects;
                self.dep_folders = dep_folders;
                self.disk_stats = disk_stats;
                self.is_loading = false;
                self.scan_receiver = None;

                // Save updated workspace cache to disk for instant cold start next time
                let _ = save_cache(&self.projects, &self.disk_stats);
            }
        }

        // Check for streaming maintenance messages
        if let Some(rx) = &self.maintenance_receiver {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    MaintenanceMessage::TaskStarted(idx) => {
                        if let Some(task) = self.maintenance_state.tasks.get_mut(idx) {
                            task.status = TaskStatus::Running;
                        }
                    }
                    MaintenanceMessage::LogLine(line) => {
                        self.maintenance_state.logs.push(line);
                    }
                    MaintenanceMessage::TaskCompleted(idx, status, duration) => {
                        if let Some(task) = self.maintenance_state.tasks.get_mut(idx) {
                            task.status = status;
                            task.duration_ms = duration;
                        }
                    }
                    MaintenanceMessage::AllCompleted => {
                        self.maintenance_state.is_running = false;
                        self.maintenance_receiver = None;
                        self.start_background_scan("Refreshing data after maintenance...");
                        break;
                    }
                }
            }
        }

        if self.tick_count % 30 == 0 {
            self.status_message = None;
        }
    }

    pub fn active_count(&self) -> usize {
        self.projects.iter().filter(|p| p.status == ProjectStatus::Active).count()
    }

    pub fn paused_count(&self) -> usize {
        self.projects.iter().filter(|p| p.status == ProjectStatus::Paused).count()
    }

    pub fn deployed_count(&self) -> usize {
        self.projects.iter().filter(|p| {
            matches!(p.status, ProjectStatus::Production | ProjectStatus::Staging)
        }).count()
    }

    pub fn vibe_count(&self) -> usize {
        self.projects.iter().filter(|p| p.status == ProjectStatus::Vibe).count()
    }

    pub fn recent_projects(&self) -> Vec<&Project> {
        let mut sorted: Vec<&Project> = self.projects.iter().collect();
        sorted.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        sorted.truncate(4);
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_app_toast_system() {
        let mut app = App::new().unwrap();
        assert!(app.toast_queue.is_empty());

        app.show_toast("Test Toast", ToastLevel::Info);
        assert_eq!(app.toast_queue.len(), 1);

        app.toast_queue.toasts[0].duration = Duration::from_millis(10);
        std::thread::sleep(Duration::from_millis(20));

        app.on_tick();
        assert!(app.toast_queue.is_empty());
    }

    #[test]
    fn test_app_palette_action_execution() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::Dashboard;

        app.execute_palette_action(PaletteAction::GitHealth);
        assert_eq!(app.current_tab, Tab::GitHealth);

        app.execute_palette_action(PaletteAction::DevPorts);
        assert_eq!(app.current_tab, Tab::DevPorts);
    }

    #[test]
    fn test_commit_dialog_key_handling() {
        let mut app = App::new().unwrap();
        app.commit_dialog = Some(CommitDialog::new("test-repo".into(), std::path::PathBuf::from("D:\\test")));

        // Type characters
        app.handle_key(KeyCode::Char('f'), KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('i'), KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('x'), KeyModifiers::NONE);
        assert_eq!(app.commit_dialog.as_ref().unwrap().message, "fix");

        // Toggle amend with Alt+A
        app.handle_key(KeyCode::Char('a'), KeyModifiers::ALT);
        assert!(app.commit_dialog.as_ref().unwrap().amend);

        // Cancel with Esc
        app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        assert!(app.commit_dialog.is_none());
    }

    #[test]
    fn test_branch_picker_key_handling() {
        let mut app = App::new().unwrap();
        app.branch_picker_modal = Some(BranchPickerModal {
            repo_name: "test-repo".into(),
            repo_path: std::path::PathBuf::from("D:\\test"),
            branches: vec!["main".into(), "feature".into()],
            current_branch: "main".into(),
            selected_index: 0,
            creating_branch: false,
            new_branch_name: String::new(),
        });

        // Press 'c' to enter create branch mode
        app.handle_key(KeyCode::Char('c'), KeyModifiers::NONE);
        assert!(app.branch_picker_modal.as_ref().unwrap().creating_branch);

        // Type branch name
        app.handle_key(KeyCode::Char('n'), KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('e'), KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('w'), KeyModifiers::NONE);
        assert_eq!(app.branch_picker_modal.as_ref().unwrap().new_branch_name, "new");

        // Cancel branch creation with Esc
        app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        assert!(!app.branch_picker_modal.as_ref().unwrap().creating_branch);
        assert!(app.branch_picker_modal.as_ref().unwrap().new_branch_name.is_empty());

        // Press 'd' to open delete confirm dialog
        app.handle_key(KeyCode::Char('d'), KeyModifiers::NONE);
        assert!(app.branch_picker_modal.is_none());
        assert!(app.confirm_dialog.is_some());
        if let Some(ref dialog) = app.confirm_dialog {
            if let ConfirmAction::DeleteGitBranch(ref branch, _) = dialog.action {
                assert_eq!(branch, "main");
            } else {
                panic!("Expected DeleteGitBranch action");
            }
        }
    }
}
