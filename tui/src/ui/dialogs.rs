use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    PauseProject(String),
    ResumeProject(String),
    DeployProject(String, bool), // (name, is_production)
    ArchiveProject(String),
    PruneDependencies,
    GitCommitAndPush(String, PathBuf),
    GitPush(String, PathBuf),
    GitPull(String, PathBuf),
}

#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub action: ConfirmAction,
}

pub fn draw(f: &mut Frame, dialog: &ConfirmDialog, area: Rect) {
    let popup_area = centered_rect(55, 25, area);

    f.render_widget(Clear, popup_area);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(&dialog.message, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [y] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("Confirm     ", Style::default().fg(Color::White)),
            Span::styled("  [n / Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("Cancel", Style::default().fg(Color::White)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", dialog.title))
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
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
