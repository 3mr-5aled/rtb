use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::Frame;
use crate::app::{GitFilter, Tab};
use crate::data::project::ProjectStatus;
use crate::ui::dialogs::ConfirmAction;
use crate::ui::toast::ToastLevel;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ModalKind {
    Readme(String, String),
    Scaffold,
    GitDiff(PathBuf),
    EnvVault(PathBuf),
    BranchPicker(PathBuf),
    Help,
    CommandPalette,
    Confirm(ConfirmAction),
    Commit(String, PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AppAction {
    None,
    Handled,
    SwitchTab(Tab),
    OpenModal(ModalKind),
    CloseModal,
    ShowToast(String, ToastLevel),
    StartScan(&'static str),
    OpenEditor(Option<usize>),
    OpenExplorer(Option<usize>),
    ExecuteCommand(String, Vec<String>),
    RunProgram(Option<usize>),
    OpenEnvVault(Option<usize>),
    OpenGitDiff(Option<usize>),
    OpenBranchPicker(Option<usize>),
    OpenCommitDialog(Option<usize>),
    GitPush(Option<usize>),
    GitPull(Option<usize>),
    GitSync(Option<usize>),
    StageUntracked(Option<usize>),
    OpenReadme(Option<usize>),
    ToggleDepSelection(usize),
    SelectAllDeps,
    UnselectAllDeps,
    PruneSelectedDeps,
    StartMaintenance(Option<usize>),
    ClearMaintenanceLogs,
    KillPort(usize),
    OpenPortUrl(usize),
    Quit,
}

#[allow(dead_code)]
pub trait TabController {
    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> AppAction;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

// ---------------------------------------------------------------------------
// 1. ProjectsTab
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectsTab {
    pub selected_index: usize,
    pub filter: Option<ProjectStatus>,
    pub search_query: String,
    pub search_active: bool,
    pub item_count: usize,
}

impl Default for ProjectsTab {
    fn default() -> Self {
        Self {
            selected_index: 0,
            filter: None,
            search_query: String::new(),
            search_active: false,
            item_count: 0,
        }
    }
}

impl TabController for ProjectsTab {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> AppAction {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                if self.item_count > 0 {
                    self.selected_index = (self.selected_index + 1).min(self.item_count - 1);
                } else {
                    self.selected_index = self.selected_index.saturating_add(1);
                }
                AppAction::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
                AppAction::Handled
            }
            KeyCode::Char('f') => {
                self.filter = match self.filter {
                    None => Some(ProjectStatus::Active),
                    Some(ProjectStatus::Active) => Some(ProjectStatus::Paused),
                    Some(ProjectStatus::Paused) => Some(ProjectStatus::Production),
                    Some(ProjectStatus::Production) => Some(ProjectStatus::Vibe),
                    Some(ProjectStatus::Vibe) => Some(ProjectStatus::Sandbox),
                    Some(ProjectStatus::Sandbox) => None,
                    _ => None,
                };
                self.selected_index = 0;
                AppAction::Handled
            }
            KeyCode::Char('/') => {
                self.search_active = true;
                AppAction::Handled
            }
            KeyCode::Char('o') | KeyCode::Enter => AppAction::OpenEditor(Some(self.selected_index)),
            KeyCode::Char('e') => AppAction::OpenExplorer(Some(self.selected_index)),
            KeyCode::Char('v') => AppAction::OpenReadme(Some(self.selected_index)),
            KeyCode::Char('x') => AppAction::RunProgram(Some(self.selected_index)),
            KeyCode::Char('d') => AppAction::OpenGitDiff(Some(self.selected_index)),
            KeyCode::Char('E') => AppAction::OpenEnvVault(Some(self.selected_index)),
            KeyCode::Char('N') => AppAction::OpenModal(ModalKind::Scaffold),
            KeyCode::Char('?') => AppAction::OpenModal(ModalKind::Help),
            KeyCode::Char('p') => AppAction::OpenModal(ModalKind::CommandPalette),
            KeyCode::Char('q') => AppAction::Quit,
            _ => AppAction::None,
        }
    }

    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}
}

// ---------------------------------------------------------------------------
// 2. GitHealthTab
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHealthTab {
    pub selected_index: usize,
    pub filter: GitFilter,
    pub item_count: usize,
}

impl Default for GitHealthTab {
    fn default() -> Self {
        Self {
            selected_index: 0,
            filter: GitFilter::All,
            item_count: 0,
        }
    }
}

impl TabController for GitHealthTab {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> AppAction {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                if self.item_count > 0 {
                    self.selected_index = (self.selected_index + 1).min(self.item_count - 1);
                } else {
                    self.selected_index = self.selected_index.saturating_add(1);
                }
                AppAction::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
                AppAction::Handled
            }
            KeyCode::Char('f') => {
                self.filter = self.filter.next();
                self.selected_index = 0;
                AppAction::Handled
            }
            KeyCode::Char('o') | KeyCode::Enter => AppAction::OpenEditor(Some(self.selected_index)),
            KeyCode::Char('e') => AppAction::OpenExplorer(Some(self.selected_index)),
            KeyCode::Char('d') => AppAction::OpenGitDiff(Some(self.selected_index)),
            KeyCode::Char('b') => AppAction::OpenBranchPicker(Some(self.selected_index)),
            KeyCode::Char('c') | KeyCode::Char('C') => AppAction::OpenCommitDialog(Some(self.selected_index)),
            KeyCode::Char('P') => AppAction::GitPush(Some(self.selected_index)),
            KeyCode::Char('l') => AppAction::GitPull(Some(self.selected_index)),
            KeyCode::Char('s') => AppAction::GitSync(Some(self.selected_index)),
            KeyCode::Char('u') => AppAction::StageUntracked(Some(self.selected_index)),
            KeyCode::Char('?') => AppAction::OpenModal(ModalKind::Help),
            KeyCode::Char('p') => AppAction::OpenModal(ModalKind::CommandPalette),
            KeyCode::Char('q') => AppAction::Quit,
            _ => AppAction::None,
        }
    }

    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}
}

// ---------------------------------------------------------------------------
// 3. DepCleanerTab
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepCleanerTab {
    pub selected_index: usize,
    pub threshold_days: u64,
    pub category_filter: Option<ProjectStatus>,
    pub item_count: usize,
}

impl Default for DepCleanerTab {
    fn default() -> Self {
        Self {
            selected_index: 0,
            threshold_days: 30,
            category_filter: None,
            item_count: 0,
        }
    }
}

impl TabController for DepCleanerTab {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> AppAction {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                if self.item_count > 0 {
                    self.selected_index = (self.selected_index + 1).min(self.item_count - 1);
                } else {
                    self.selected_index = self.selected_index.saturating_add(1);
                }
                AppAction::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
                AppAction::Handled
            }
            KeyCode::Char(' ') => AppAction::ToggleDepSelection(self.selected_index),
            KeyCode::Char('c') => {
                self.category_filter = match self.category_filter {
                    None => Some(ProjectStatus::Paused),
                    Some(ProjectStatus::Paused) => Some(ProjectStatus::Abandoned),
                    Some(ProjectStatus::Abandoned) => Some(ProjectStatus::Active),
                    Some(ProjectStatus::Active) => Some(ProjectStatus::Vibe),
                    Some(ProjectStatus::Vibe) => Some(ProjectStatus::Sandbox),
                    Some(ProjectStatus::Sandbox) => None,
                    _ => None,
                };
                self.selected_index = 0;
                AppAction::Handled
            }
            KeyCode::Char('a') => AppAction::SelectAllDeps,
            KeyCode::Char('n') => AppAction::UnselectAllDeps,
            KeyCode::Char('d') => AppAction::OpenModal(ModalKind::Confirm(ConfirmAction::PruneDependencies)),
            KeyCode::Char('t') => {
                self.threshold_days = match self.threshold_days {
                    30 => 60,
                    60 => 90,
                    90 => 180,
                    180 => 0,
                    _ => 30,
                };
                AppAction::Handled
            }
            KeyCode::Char('?') => AppAction::OpenModal(ModalKind::Help),
            KeyCode::Char('p') => AppAction::OpenModal(ModalKind::CommandPalette),
            KeyCode::Char('q') => AppAction::Quit,
            _ => AppAction::None,
        }
    }

    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}
}

// ---------------------------------------------------------------------------
// 4. MaintenanceTab
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaintenanceTab {
    pub selected_index: usize,
    pub item_count: usize,
}

impl TabController for MaintenanceTab {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> AppAction {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                if self.item_count > 0 {
                    self.selected_index = (self.selected_index + 1).min(self.item_count - 1);
                } else {
                    self.selected_index = self.selected_index.saturating_add(1);
                }
                AppAction::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
                AppAction::Handled
            }
            KeyCode::Enter => AppAction::StartMaintenance(None),
            KeyCode::Char('s') => AppAction::StartMaintenance(Some(self.selected_index)),
            KeyCode::Char('c') => AppAction::ClearMaintenanceLogs,
            KeyCode::Char('?') => AppAction::OpenModal(ModalKind::Help),
            KeyCode::Char('p') => AppAction::OpenModal(ModalKind::CommandPalette),
            KeyCode::Char('q') => AppAction::Quit,
            _ => AppAction::None,
        }
    }

    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}
}

// ---------------------------------------------------------------------------
// 5. DevPortsTab
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DevPortsTab {
    pub selected_index: usize,
    pub item_count: usize,
}

impl TabController for DevPortsTab {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> AppAction {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                if self.item_count > 0 {
                    self.selected_index = (self.selected_index + 1).min(self.item_count - 1);
                } else {
                    self.selected_index = self.selected_index.saturating_add(1);
                }
                AppAction::Handled
            }
            KeyCode::Up => {
                self.selected_index = self.selected_index.saturating_sub(1);
                AppAction::Handled
            }
            KeyCode::Char('k') => AppAction::KillPort(self.selected_index),
            KeyCode::Char('o') => AppAction::OpenPortUrl(self.selected_index),
            KeyCode::Char('R') => AppAction::StartScan("ports"),
            KeyCode::Char('?') => AppAction::OpenModal(ModalKind::Help),
            KeyCode::Char('p') => AppAction::OpenModal(ModalKind::CommandPalette),
            KeyCode::Char('q') => AppAction::Quit,
            _ => AppAction::None,
        }
    }

    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}
}

// ---------------------------------------------------------------------------
// 6. DashboardTab
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DashboardTab {
    pub selected_index: usize,
    pub item_count: usize,
}

impl TabController for DashboardTab {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> AppAction {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                if self.item_count > 0 {
                    self.selected_index = (self.selected_index + 1).min(self.item_count - 1);
                } else {
                    self.selected_index = self.selected_index.saturating_add(1);
                }
                AppAction::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
                AppAction::Handled
            }
            KeyCode::Char('o') | KeyCode::Enter => AppAction::OpenEditor(Some(self.selected_index)),
            KeyCode::Char('e') => AppAction::OpenExplorer(Some(self.selected_index)),
            KeyCode::Char('v') => AppAction::OpenReadme(Some(self.selected_index)),
            KeyCode::Char('x') => AppAction::RunProgram(Some(self.selected_index)),
            KeyCode::Char('d') => AppAction::OpenGitDiff(Some(self.selected_index)),
            KeyCode::Char('p') => AppAction::OpenModal(ModalKind::CommandPalette),
            KeyCode::Char('?') => AppAction::OpenModal(ModalKind::Help),
            KeyCode::Char('q') => AppAction::Quit,
            _ => AppAction::None,
        }
    }

    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projects_tab_key_handling_and_state() {
        let mut tab = ProjectsTab::default();
        assert_eq!(tab.selected_index, 0);
        assert_eq!(tab.filter, None);

        let action = tab.handle_key(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(action, AppAction::Handled);
        assert_eq!(tab.selected_index, 1);

        let action = tab.handle_key(KeyCode::Char('f'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::Handled);
        assert_eq!(tab.filter, Some(ProjectStatus::Active));

        let action = tab.handle_key(KeyCode::Char('N'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::OpenModal(ModalKind::Scaffold));

        let action = tab.handle_key(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::Quit);
    }

    #[test]
    fn test_git_health_tab_cycling() {
        let mut tab = GitHealthTab::default();
        assert_eq!(tab.filter, GitFilter::All);

        let action = tab.handle_key(KeyCode::Char('f'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::Handled);
        assert_eq!(tab.filter, GitFilter::NeedsAttention);

        let action = tab.handle_key(KeyCode::Char('p'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::OpenModal(ModalKind::CommandPalette));

        let action = tab.handle_key(KeyCode::Char('P'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::GitPush(Some(0)));
    }

    #[test]
    fn test_dep_cleaner_tab_actions() {
        let mut tab = DepCleanerTab::default();
        assert_eq!(tab.threshold_days, 30);
        assert_eq!(tab.category_filter, None);

        let action = tab.handle_key(KeyCode::Char(' '), KeyModifiers::NONE);
        assert_eq!(action, AppAction::ToggleDepSelection(0));

        let action = tab.handle_key(KeyCode::Char('t'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::Handled);
        assert_eq!(tab.threshold_days, 60);

        let action = tab.handle_key(KeyCode::Char('c'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::Handled);
        assert_eq!(tab.category_filter, Some(ProjectStatus::Paused));

        let action = tab.handle_key(KeyCode::Char('d'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::OpenModal(ModalKind::Confirm(ConfirmAction::PruneDependencies)));
    }

    #[test]
    fn test_maintenance_tab_triggers() {
        let mut tab = MaintenanceTab::default();
        let action = tab.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(action, AppAction::StartMaintenance(None));

        let action = tab.handle_key(KeyCode::Char('s'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::StartMaintenance(Some(0)));

        let action = tab.handle_key(KeyCode::Char('c'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::ClearMaintenanceLogs);
    }

    #[test]
    fn test_ports_tab_actions() {
        let mut tab = DevPortsTab::default();
        let action = tab.handle_key(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::KillPort(0));

        let action = tab.handle_key(KeyCode::Char('o'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::OpenPortUrl(0));

        let action = tab.handle_key(KeyCode::Char('R'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::StartScan("ports"));
    }

    #[test]
    fn test_dashboard_tab_actions() {
        let mut tab = DashboardTab::default();
        let action = tab.handle_key(KeyCode::Char('o'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::OpenEditor(Some(0)));

        let action = tab.handle_key(KeyCode::Char('v'), KeyModifiers::NONE);
        assert_eq!(action, AppAction::OpenReadme(Some(0)));
    }
}
