use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    pub is_revealed: bool,
}

#[derive(Debug, Clone)]
pub struct EnvVaultModal {
    pub project_name: String,
    pub file_name: String,
    pub file_path: std::path::PathBuf,
    pub vars: Vec<EnvVar>,
    pub selected_index: usize,
}

impl EnvVaultModal {
    pub fn load(project_name: String, project_path: &std::path::Path) -> Option<Self> {
        let env_names = [".env", ".env.local", ".env.development", ".env.production"];
        for name in &env_names {
            let path = project_path.join(name);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let mut vars = Vec::new();
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() || trimmed.starts_with('#') {
                            continue;
                        }
                        if let Some(eq_idx) = trimmed.find('=') {
                            let key = trimmed[..eq_idx].trim().to_string();
                            let val = trimmed[eq_idx + 1..].trim().to_string();
                            vars.push(EnvVar {
                                key,
                                value: val,
                                is_revealed: false,
                            });
                        }
                    }

                    return Some(EnvVaultModal {
                        project_name,
                        file_name: name.to_string(),
                        file_path: path,
                        vars,
                        selected_index: 0,
                    });
                }
            }
        }
        None
    }
}

pub fn draw(f: &mut Frame, vault: &EnvVaultModal, area: Rect) {
    let popup_area = centered_rect(75, 75, area);

    f.render_widget(Clear, popup_area);

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  File: ", Style::default().fg(Color::DarkGray)),
        Span::styled(vault.file_path.to_string_lossy().to_string(), Style::default().fg(Color::Cyan)),
        Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{} variables loaded", vault.vars.len()), Style::default().fg(Color::Green)),
    ]));
    lines.push(Line::from(""));

    if vault.vars.is_empty() {
        lines.push(Line::from("  No environment variables found in file."));
    } else {
        for (i, v) in vault.vars.iter().enumerate() {
            let is_selected = i == vault.selected_index;
            let cursor = if is_selected { "▶ " } else { "  " };

            let displayed_val = if v.is_revealed {
                v.value.clone()
            } else {
                "•".repeat(v.value.len().min(16).max(8))
            };

            let key_color = if is_selected { Color::Yellow } else { Color::White };
            let val_color = if v.is_revealed { Color::Green } else { Color::DarkGray };

            lines.push(Line::from(vec![
                Span::styled(cursor, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:<28}", v.key),
                    Style::default().fg(key_color).add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
                ),
                Span::styled(" = ", Style::default().fg(Color::DarkGray)),
                Span::styled(displayed_val, Style::default().fg(val_color)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [Space] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Reveal/Mask Key   ", Style::default().fg(Color::White)),
        Span::styled("[a] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Reveal All   ", Style::default().fg(Color::White)),
        Span::styled("[b] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("Backup .env   ", Style::default().fg(Color::White)),
        Span::styled("[Esc/E] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled("Close", Style::default().fg(Color::White)),
    ]));

    let title = format!(" 🔐 Environment Secrets Vault — {} ({}) ", vault.project_name, vault.file_name);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .title_alignment(Alignment::Center);

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
