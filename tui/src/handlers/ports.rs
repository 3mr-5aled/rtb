use crate::app::App;
use crate::data::ports::{kill_port_process, scan_dev_ports};
use crossterm::event::KeyCode;
use std::process::Command;

impl App {
    pub fn handle_ports_key(&mut self, key: KeyCode) {
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
            KeyCode::Char('R') => {
                self.active_ports = scan_dev_ports();
                self.status_message = Some("Dev ports rescanned".into());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Tab;
    use crate::data::ports::DevPort;

    fn make_test_port(port: u16, pid: u32) -> DevPort {
        DevPort {
            port,
            pid,
            process_name: "node.exe".into(),
            memory_str: "15 MB".into(),
            project_name: Some("test-app".into()),
        }
    }

    #[test]
    fn test_ports_empty_list_no_panic() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::DevPorts;
        app.active_ports.clear();
        app.ports_selected_index = 0;

        let test_keys = [
            KeyCode::Down, KeyCode::Up, KeyCode::Char('j'), KeyCode::Char('k'),
            KeyCode::Char('o'), KeyCode::Char('R'),
        ];

        for key in test_keys {
            app.handle_ports_key(key);
        }
        assert_eq!(app.ports_selected_index, 0);
    }

    #[test]
    fn test_ports_navigation_bounds() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::DevPorts;
        app.active_ports = vec![
            make_test_port(3000, 1111),
            make_test_port(8080, 2222),
        ];
        app.ports_selected_index = 0;

        // Up at 0 clamps to 0
        app.handle_ports_key(KeyCode::Up);
        assert_eq!(app.ports_selected_index, 0);

        // Down moves to 1
        app.handle_ports_key(KeyCode::Down);
        assert_eq!(app.ports_selected_index, 1);

        // Down at bottom clamps to 1
        app.handle_ports_key(KeyCode::Char('j'));
        assert_eq!(app.ports_selected_index, 1);
    }

    #[test]
    fn test_ports_rescan_status_message() {
        let mut app = App::new().unwrap();
        app.current_tab = Tab::DevPorts;

        app.handle_ports_key(KeyCode::Char('R'));
        assert_eq!(app.status_message.as_deref(), Some("Dev ports rescanned"));
    }
}

