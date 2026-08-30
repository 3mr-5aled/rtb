use crate::app::App;
use crate::data::deps::format_bytes;
use crate::data::project::ProjectStatus;
use crate::ui::dialogs::{ConfirmAction, ConfirmDialog};
use crossterm::event::KeyCode;

impl App {
    pub fn handle_cleaner_key(&mut self, key: KeyCode) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Tab;
    use crate::data::deps::DepFolder;
    use std::path::PathBuf;

    fn make_test_dep(name: &str, status: ProjectStatus, size: u64) -> DepFolder {
        DepFolder {
            project_name: name.to_string(),
            project_status: status,
            path: PathBuf::from(format!("D:\\projects\\{}\\{}", name, "node_modules")),
            rel_path: "node_modules".into(),
            size_bytes: size,
            last_modified: None,
            is_selected: false,
        }
    }

    #[test]
    fn test_cleaner_empty_list_no_panic() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::DepCleaner;
        app.dep_folders.clear();
        app.cleaner_selected_index = 0;

        let test_keys = [
            KeyCode::Down, KeyCode::Up, KeyCode::Char('j'), KeyCode::Char('k'),
            KeyCode::Char(' '), KeyCode::Char('c'), KeyCode::Char('a'), KeyCode::Char('n'),
            KeyCode::Enter, KeyCode::Char('r'),
        ];

        for key in test_keys {
            app.handle_cleaner_key(key);
        }
        assert_eq!(app.cleaner_selected_index, 0);
    }

    #[test]
    fn test_cleaner_navigation_bounds() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::DepCleaner;
        app.cleaner_category_filter = None;
        app.dep_folders = vec![
            make_test_dep("d1", ProjectStatus::Active, 100),
            make_test_dep("d2", ProjectStatus::Paused, 200),
        ];
        app.cleaner_selected_index = 0;

        // Up at 0 clamps to 0
        app.handle_cleaner_key(KeyCode::Up);
        assert_eq!(app.cleaner_selected_index, 0);
        app.handle_cleaner_key(KeyCode::Char('k'));
        assert_eq!(app.cleaner_selected_index, 0);

        // Down moves to 1
        app.handle_cleaner_key(KeyCode::Down);
        assert_eq!(app.cleaner_selected_index, 1);

        // Down at bottom clamps to 1
        app.handle_cleaner_key(KeyCode::Down);
        assert_eq!(app.cleaner_selected_index, 1);
    }

    #[test]
    fn test_cleaner_category_filter_and_selective_select() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::DepCleaner;
        app.dep_folders = vec![
            make_test_dep("p_paused", ProjectStatus::Paused, 500),
            make_test_dep("p_active", ProjectStatus::Active, 300),
        ];
        app.cleaner_category_filter = None;

        // Filter cycle: None -> Paused
        app.handle_cleaner_key(KeyCode::Char('c'));
        assert_eq!(app.cleaner_category_filter, Some(ProjectStatus::Paused));

        // 'a' while Paused filter active only selects Paused items
        app.handle_cleaner_key(KeyCode::Char('a'));
        assert!(app.dep_folders[0].is_selected);
        assert!(!app.dep_folders[1].is_selected);

        // 'n' deselects all
        app.handle_cleaner_key(KeyCode::Char('n'));
        assert!(!app.dep_folders[0].is_selected);
        assert!(!app.dep_folders[1].is_selected);
    }

    #[test]
    fn test_cleaner_enter_confirm_dialog_or_status() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::DepCleaner;
        app.dep_folders = vec![make_test_dep("p1", ProjectStatus::Active, 1024)];
        app.confirm_dialog = None;

        // When none selected, Enter sets status message
        app.handle_cleaner_key(KeyCode::Enter);
        assert!(app.confirm_dialog.is_none());
        assert_eq!(app.status_message.as_deref(), Some("No folders selected for pruning"));

        // When selected, Enter opens ConfirmDialog
        app.dep_folders[0].is_selected = true;
        app.handle_cleaner_key(KeyCode::Enter);
        assert!(app.confirm_dialog.is_some());
        if let Some(ref d) = app.confirm_dialog {
            assert_eq!(d.action, ConfirmAction::PruneDependencies);
        }
    }
}

