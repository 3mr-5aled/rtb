use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GitDiffModal {
    pub repo_name: String,
    pub diff_content: String,
    pub scroll_offset: usize,
}

impl GitDiffModal {
    pub fn load(repo_name: String, repo_path: &Path) -> Option<Self> {
        let diff = Command::new("git")
            .args(["diff", "HEAD"])
            .current_dir(repo_path)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        let staged_diff = Command::new("git")
            .args(["diff", "--cached"])
            .current_dir(repo_path)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        let full_diff = if !diff.is_empty() && !staged_diff.is_empty() {
            format!("=== STAGED CHANGES ===\n{}\n\n=== UNSTAGED CHANGES ===\n{}", staged_diff, diff)
        } else if !staged_diff.is_empty() {
            format!("=== STAGED CHANGES ===\n{}", staged_diff)
        } else if !diff.is_empty() {
            diff
        } else {
            "No uncommitted changes in this repository.".to_string()
        };

        Some(GitDiffModal {
            repo_name,
            diff_content: full_diff,
            scroll_offset: 0,
        })
    }
}

pub fn draw(f: &mut Frame, modal: &GitDiffModal, area: Rect) {
    let popup_area = centered_rect(88, 88, area);

    f.render_widget(Clear, popup_area);

    let mut formatted_lines: Vec<Line> = Vec::new();

    for line in modal.diff_content.lines() {
        let color = if line.starts_with('+') && !line.starts_with("+++") {
            Color::Green
        } else if line.starts_with('-') && !line.starts_with("---") {
            Color::Red
        } else if line.starts_with("@@") {
            Color::Cyan
        } else if line.starts_with("diff --git") || line.starts_with("===") {
            Color::Yellow
        } else if line.starts_with("---") || line.starts_with("+++") || line.starts_with("index") {
            Color::DarkGray
        } else {
            Color::White
        };

        formatted_lines.push(Line::from(Span::styled(
            format!(" {}", line),
            Style::default().fg(color),
        )));
    }

    let max_scroll = formatted_lines.len().saturating_sub(1);
    let current_scroll = modal.scroll_offset.min(max_scroll);

    let title = format!(
        " 🌿 Git Diff — {} (Line {}/{} │ [↑/↓/j/k] Scroll │ [Esc/d] Close) ",
        modal.repo_name,
        current_scroll + 1,
        formatted_lines.len().max(1)
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .title_alignment(Alignment::Center);

    let para = Paragraph::new(formatted_lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((current_scroll as u16, 0));

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
