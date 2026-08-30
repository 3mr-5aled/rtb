use crate::app::App;
use crate::data::maintenance::MaintenanceState;
use crossterm::event::KeyCode;
use std::sync::mpsc::channel;

impl App {
    pub fn handle_maintenance_key(&mut self, key: KeyCode) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Tab;

    #[test]
    fn test_maintenance_empty_list_no_panic() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::Maintenance;
        app.maintenance_state.tasks.clear();
        app.maintenance_state.selected_task = 0;

        let test_keys = [
            KeyCode::Down, KeyCode::Up, KeyCode::Char('j'), KeyCode::Char('k'),
            KeyCode::Enter, KeyCode::Char('s'), KeyCode::Char('c'),
        ];

        for key in test_keys {
            app.handle_maintenance_key(key);
        }
        assert_eq!(app.maintenance_state.selected_task, 0);
    }

    #[test]
    fn test_maintenance_navigation_bounds() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::Maintenance;
        let task_count = app.maintenance_state.tasks.len();
        assert!(task_count > 1);

        app.maintenance_state.selected_task = 0;

        // Up at 0 clamps to 0
        app.handle_maintenance_key(KeyCode::Up);
        assert_eq!(app.maintenance_state.selected_task, 0);
        app.handle_maintenance_key(KeyCode::Char('k'));
        assert_eq!(app.maintenance_state.selected_task, 0);

        // Move to bottom
        for _ in 0..task_count + 5 {
            app.handle_maintenance_key(KeyCode::Down);
        }
        assert_eq!(app.maintenance_state.selected_task, task_count - 1);
    }

    #[test]
    fn test_maintenance_running_guard() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::Maintenance;
        app.maintenance_state.is_running = true;

        // Enter when running sets message and doesn't crash
        app.handle_maintenance_key(KeyCode::Enter);
        assert_eq!(
            app.status_message.as_deref(),
            Some("Maintenance tasks are already running in background")
        );

        // 's' when running sets message
        app.handle_maintenance_key(KeyCode::Char('s'));
        assert_eq!(
            app.status_message.as_deref(),
            Some("Maintenance task already running in background")
        );
    }
}

