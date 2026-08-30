use crate::app::App;
use crate::data::actions;
use crate::data::agents;
use crate::data::project::ProjectStatus;
use crate::ui::dialogs::{ConfirmAction, ConfirmDialog};
use crate::ui::env_vault::EnvVaultModal;
use crate::ui::git_diff::GitDiffModal;
use crate::ui::scaffold::ScaffoldModal;
use crossterm::event::KeyCode;

impl App {
    pub fn handle_projects_key(&mut self, key: KeyCode) {
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
            KeyCode::Char('x') => {
                if let Some(project) = self.selected_project() {
                    let cmd = project.get_dev_command();
                    actions::run_live_program(project);
                    self.status_message = Some(format!("Launched live runner for {}: {}", project.name, cmd));
                }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Tab;
    use crate::data::project::Project;
    use std::path::PathBuf;

    fn make_test_project(name: &str, status: ProjectStatus) -> Project {
        Project {
            name: name.to_string(),
            path: PathBuf::from(format!("D:\\projects\\{}", name)),
            status,
            stack: vec!["Rust".into()],
            last_modified: None,
            total_size_bytes: 1024,
            dep_size_bytes: 512,
            git: None,
            readme_preview: None,
            is_monorepo: false,
            ci_cd: None,
            runtime_version: None,
            dev_command: None,
        }
    }

    #[test]
    fn test_projects_navigation_empty_list_no_panic() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::Projects;
        app.projects.clear();
        app.selected_index = 0;

        let test_keys = [
            KeyCode::Down, KeyCode::Up, KeyCode::Char('j'), KeyCode::Char('k'),
            KeyCode::Char('o'), KeyCode::Enter, KeyCode::Char('v'), KeyCode::Char('N'),
            KeyCode::Char('x'), KeyCode::Char('E'), KeyCode::Char('d'), KeyCode::Char('e'),
            KeyCode::Char('p'), KeyCode::Char('r'), KeyCode::Char('D'), KeyCode::Char('a'),
            KeyCode::Char('A'), KeyCode::Char('f'),
        ];

        for key in test_keys {
            app.handle_projects_key(key);
        }
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_projects_navigation_wrap_around() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::Projects;
        app.project_filter = None;
        app.search_query.clear();
        app.projects = vec![
            make_test_project("p1", ProjectStatus::Active),
            make_test_project("p2", ProjectStatus::Active),
            make_test_project("p3", ProjectStatus::Active),
        ];
        app.selected_index = 0;

        // Down/j moves forward
        app.handle_projects_key(KeyCode::Down);
        assert_eq!(app.selected_index, 1);
        app.handle_projects_key(KeyCode::Char('j'));
        assert_eq!(app.selected_index, 2);
        // Wraps to 0
        app.handle_projects_key(KeyCode::Down);
        assert_eq!(app.selected_index, 0);

        // Up/k wraps backward to 2
        app.handle_projects_key(KeyCode::Up);
        assert_eq!(app.selected_index, 2);
        app.handle_projects_key(KeyCode::Char('k'));
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn test_projects_filter_cycle_resets_selected_index() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::Projects;
        app.projects = vec![
            make_test_project("p1", ProjectStatus::Active),
            make_test_project("p2", ProjectStatus::Paused),
        ];
        app.selected_index = 1;
        app.project_filter = None;

        app.handle_projects_key(KeyCode::Char('f'));
        assert_eq!(app.project_filter, Some(ProjectStatus::Active));
        assert_eq!(app.selected_index, 0);

        app.selected_index = 1;
        app.handle_projects_key(KeyCode::Char('f'));
        assert_eq!(app.project_filter, Some(ProjectStatus::Paused));
        assert_eq!(app.selected_index, 0);

        app.handle_projects_key(KeyCode::Char('f'));
        assert_eq!(app.project_filter, Some(ProjectStatus::Production));
        assert_eq!(app.selected_index, 0);

        app.handle_projects_key(KeyCode::Char('f'));
        assert_eq!(app.project_filter, Some(ProjectStatus::Vibe));
        assert_eq!(app.selected_index, 0);

        app.handle_projects_key(KeyCode::Char('f'));
        assert_eq!(app.project_filter, Some(ProjectStatus::Sandbox));
        assert_eq!(app.selected_index, 0);

        app.handle_projects_key(KeyCode::Char('f'));
        assert_eq!(app.project_filter, None);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_projects_dialog_triggers_and_guards() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::Projects;
        app.project_filter = None;
        app.projects = vec![
            make_test_project("active-proj", ProjectStatus::Active),
            make_test_project("paused-proj", ProjectStatus::Paused),
        ];

        // 1. Selected is active-proj
        app.selected_index = 0;
        app.confirm_dialog = None;

        // 'p' (pause) on active project opens dialog
        app.handle_projects_key(KeyCode::Char('p'));
        assert!(app.confirm_dialog.is_some());
        if let Some(ref d) = app.confirm_dialog {
            assert_eq!(d.action, ConfirmAction::PauseProject("active-proj".into()));
        }

        // 'r' (resume) on active project does not open resume dialog
        app.confirm_dialog = None;
        app.handle_projects_key(KeyCode::Char('r'));
        assert!(app.confirm_dialog.is_none());

        // 'D' (deploy) on active project opens deploy dialog
        app.handle_projects_key(KeyCode::Char('D'));
        assert!(app.confirm_dialog.is_some());
        if let Some(ref d) = app.confirm_dialog {
            assert_eq!(d.action, ConfirmAction::DeployProject("active-proj".into(), true));
        }

        // 'A' (archive) opens archive dialog
        app.confirm_dialog = None;
        app.handle_projects_key(KeyCode::Char('A'));
        assert!(app.confirm_dialog.is_some());
        if let Some(ref d) = app.confirm_dialog {
            assert_eq!(d.action, ConfirmAction::ArchiveProject("active-proj".into()));
        }

        // 2. Selected is paused-proj
        app.selected_index = 1;
        app.confirm_dialog = None;

        // 'p' on paused project should reject with status message
        app.handle_projects_key(KeyCode::Char('p'));
        assert!(app.confirm_dialog.is_none());
        assert_eq!(app.status_message.as_deref(), Some("Only active projects can be paused"));

        // 'r' on paused project opens resume dialog
        app.handle_projects_key(KeyCode::Char('r'));
        assert!(app.confirm_dialog.is_some());
        if let Some(ref d) = app.confirm_dialog {
            assert_eq!(d.action, ConfirmAction::ResumeProject("paused-proj".into()));
        }
    }

    #[test]
    fn test_projects_scaffold_and_readme_modals() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::Projects;
        app.project_filter = None;
        app.projects = vec![make_test_project("demo", ProjectStatus::Active)];
        app.selected_index = 0;

        // 'N' opens Scaffold modal
        app.scaffold_modal = None;
        app.handle_projects_key(KeyCode::Char('N'));
        assert!(app.scaffold_modal.is_some());

        // 'v' opens README modal
        app.readme_modal = None;
        app.handle_projects_key(KeyCode::Char('v'));
        assert!(app.readme_modal.is_some());
        let (name, content, scroll) = app.readme_modal.as_ref().unwrap();
        assert_eq!(name, "demo");
        assert!(content.contains("demo"));
        assert_eq!(*scroll, 0);
    }
}

