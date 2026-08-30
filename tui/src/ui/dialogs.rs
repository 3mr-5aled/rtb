use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    PauseProject(String),
    ResumeProject(String),
    DeployProject(String, bool), // (name, is_production)
    ArchiveProject(String),
    PruneDependencies,
    GitCommitAndPush(String, PathBuf),
    GitPush(String, PathBuf),
    GitPull(String, PathBuf),
    DeleteGitBranch(String, PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub action: ConfirmAction,
}

#[derive(Debug, Clone)]
pub struct CommitDialog {
    pub repo_name: String,
    pub repo_path: PathBuf,
    pub message: String,
    pub amend: bool,
}

impl CommitDialog {
    pub fn new(repo_name: String, repo_path: PathBuf) -> Self {
        CommitDialog {
            repo_name,
            repo_path,
            message: String::new(),
            amend: false,
        }
    }
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

pub fn draw_commit_dialog(f: &mut Frame, dialog: &CommitDialog, area: Rect) {
    let popup_area = centered_rect(60, 35, area);

    f.render_widget(Clear, popup_area);

    let amend_checkbox = if dialog.amend {
        "[x] Amend previous commit (--amend)"
    } else {
        "[ ] Amend previous commit (--amend)"
    };
    let amend_style = if dialog.amend {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Commit Message: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  > ", Style::default().fg(Color::Yellow)),
            Span::styled(
                if dialog.message.is_empty() {
                    "Type commit message...".to_string()
                } else {
                    format!("{}_", dialog.message)
                },
                if dialog.message.is_empty() {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                },
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(amend_checkbox, amend_style),
            Span::styled(" (Toggle: Alt+A)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("Commit   ", Style::default().fg(Color::White)),
            Span::styled("  [Alt+A] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Toggle Amend   ", Style::default().fg(Color::White)),
            Span::styled("  [Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("Cancel", Style::default().fg(Color::White)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" 📝 Git Commit — {} ", dialog.repo_name))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_dialog_initial_and_amend_toggle() {
        let mut dialog = CommitDialog::new("test-repo".into(), PathBuf::from("D:\\test"));
        assert!(!dialog.amend);
        assert!(dialog.message.is_empty());

        dialog.amend = true;
        dialog.message = "fix: test amend".into();
        assert!(dialog.amend);
        assert_eq!(dialog.message, "fix: test amend");
    }
}

