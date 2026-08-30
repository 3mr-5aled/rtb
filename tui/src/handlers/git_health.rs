use crate::app::App;
use crate::data::actions;
use crate::ui::branch_picker::BranchPickerModal;
use crate::ui::dialogs::CommitDialog;
use crate::ui::git_diff::GitDiffModal;
use crossterm::event::KeyCode;
use std::process::Command;
use std::thread;

impl App {
    pub fn handle_git_health_key(&mut self, key: KeyCode) {
        let git_projects = self.filtered_git_projects();
        match key {
            KeyCode::Down | KeyCode::Char('j') => {
                if !git_projects.is_empty() {
                    self.git_selected_index = (self.git_selected_index + 1).min(git_projects.len() - 1);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.git_selected_index = self.git_selected_index.saturating_sub(1);
            }
            KeyCode::Char('f') => {
                self.git_filter = self.git_filter.next();
                self.git_selected_index = 0;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{GitFilter, Tab};
    use crate::data::project::{GitInfo, Project, ProjectStatus};
    use std::path::PathBuf;

    fn make_test_git_project(name: &str, uncommitted: u32, unpushed: u32, has_remote: bool) -> Project {
        Project {
            name: name.to_string(),
            path: PathBuf::from(format!("D:\\projects\\{}", name)),
            status: ProjectStatus::Active,
            stack: vec!["Rust".into()],
            last_modified: None,
            total_size_bytes: 1024,
            dep_size_bytes: 512,
            git: Some(GitInfo {
                branch: "main".into(),
                uncommitted,
                unpushed,
                last_commit_msg: Some("test commit".into()),
                last_commit_relative: Some("1 hour ago".into()),
                has_remote,
            }),
            readme_preview: None,
            is_monorepo: false,
            ci_cd: None,
            runtime_version: None,
            dev_command: None,
        }
    }

    #[test]
    fn test_git_health_empty_list_no_panic() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::GitHealth;
        app.projects.clear();
        app.git_selected_index = 0;

        let test_keys = [
            KeyCode::Down, KeyCode::Up, KeyCode::Char('j'), KeyCode::Char('k'),
            KeyCode::Char('f'), KeyCode::Char('o'), KeyCode::Enter, KeyCode::Char('d'),
            KeyCode::Char('b'), KeyCode::Char('c'), KeyCode::Char('C'), KeyCode::Char('P'),
            KeyCode::Char('p'), KeyCode::Char('r'),
        ];

        for key in test_keys {
            app.handle_git_health_key(key);
        }
        assert_eq!(app.git_selected_index, 0);
    }

    #[test]
    fn test_git_health_navigation_bounds() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::GitHealth;
        app.git_filter = GitFilter::All;
        app.projects = vec![
            make_test_git_project("g1", 0, 0, true),
            make_test_git_project("g2", 1, 0, true),
            make_test_git_project("g3", 0, 1, false),
        ];
        app.git_selected_index = 0;

        // Up at 0 clamps to 0 (saturating_sub)
        app.handle_git_health_key(KeyCode::Up);
        assert_eq!(app.git_selected_index, 0);
        app.handle_git_health_key(KeyCode::Char('k'));
        assert_eq!(app.git_selected_index, 0);

        // Down moves to 1 and 2
        app.handle_git_health_key(KeyCode::Down);
        assert_eq!(app.git_selected_index, 1);
        app.handle_git_health_key(KeyCode::Char('j'));
        assert_eq!(app.git_selected_index, 2);

        // Down at bottom clamps to 2
        app.handle_git_health_key(KeyCode::Down);
        assert_eq!(app.git_selected_index, 2);
    }

    #[test]
    fn test_git_health_commit_dialog_and_push_pull() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::GitHealth;
        app.git_filter = GitFilter::All;
        app.projects = vec![make_test_git_project("repo-a", 2, 0, true)];
        app.git_selected_index = 0;

        // 'c' opens CommitDialog
        app.commit_dialog = None;
        app.handle_git_health_key(KeyCode::Char('c'));
        assert!(app.commit_dialog.is_some());
        assert_eq!(app.commit_dialog.as_ref().unwrap().repo_name, "repo-a");

        // 'P' sets status message for push
        app.handle_git_health_key(KeyCode::Char('P'));
        assert_eq!(app.status_message.as_deref(), Some("Pushing repo-a..."));

        // 'p' sets status message for pull
        app.handle_git_health_key(KeyCode::Char('p'));
        assert_eq!(app.status_message.as_deref(), Some("Pulling repo-a..."));
    }
}

