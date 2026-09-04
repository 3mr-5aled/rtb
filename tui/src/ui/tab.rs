use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::Frame;
use crate::app::{GitFilter, Tab};
use crate::data::project::ProjectStatus;
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
    Confirm(String),
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
    OpenEditor(PathBuf),
    OpenExplorer(PathBuf),
    ExecuteCommand(String, Vec<String>),
    Quit,
}

#[allow(dead_code)]
pub trait TabController {
    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> AppAction;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ProjectsTab {
    pub selected_index: usize,
    pub filter: Option<ProjectStatus>,
}

impl TabController for ProjectsTab {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> AppAction {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_index = self.selected_index.saturating_add(1);
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
                AppAction::Handled
            }
            KeyCode::Char('N') => AppAction::OpenModal(ModalKind::Scaffold),
            KeyCode::Char('?') => AppAction::OpenModal(ModalKind::Help),
            KeyCode::Char('p') => AppAction::OpenModal(ModalKind::CommandPalette),
            KeyCode::Char('q') => AppAction::Quit,
            _ => AppAction::None,
        }
    }

    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct GitHealthTab {
    pub selected_index: usize,
    pub filter: GitFilter,
}

impl Default for GitHealthTab {
    fn default() -> Self {
        Self {
            selected_index: 0,
            filter: GitFilter::All,
        }
    }
}

impl TabController for GitHealthTab {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> AppAction {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_index = self.selected_index.saturating_add(1);
                AppAction::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
                AppAction::Handled
            }
            KeyCode::Char('f') => {
                self.filter = self.filter.next();
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub struct DepCleanerTab {
    pub selected_index: usize,
    pub threshold_mb: u64,
}

impl TabController for DepCleanerTab {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> AppAction {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_index = self.selected_index.saturating_add(1);
                AppAction::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub struct MaintenanceTab {
    pub selected_index: usize,
}

impl TabController for MaintenanceTab {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> AppAction {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_index = self.selected_index.saturating_add(1);
                AppAction::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub struct DevPortsTab {
    pub selected_index: usize,
}

impl TabController for DevPortsTab {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> AppAction {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_index = self.selected_index.saturating_add(1);
                AppAction::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub struct DashboardTab {
    pub selected_index: usize,
}

impl TabController for DashboardTab {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> AppAction {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_index = self.selected_index.saturating_add(1);
                AppAction::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
                AppAction::Handled
            }
            KeyCode::Char('p') => AppAction::OpenModal(ModalKind::CommandPalette),
            KeyCode::Char('?') => AppAction::OpenModal(ModalKind::Help),
            KeyCode::Char('q') => AppAction::Quit,
            _ => AppAction::None,
        }
    }

    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}
}

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
    }
}

