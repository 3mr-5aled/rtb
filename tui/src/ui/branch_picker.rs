use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct BranchPickerModal {
    pub repo_name: String,
    pub repo_path: PathBuf,
    pub branches: Vec<String>,
    pub current_branch: String,
    pub selected_index: usize,
}

impl BranchPickerModal {
    pub fn load(repo_name: String, repo_path: &Path) -> Option<Self> {
        let output = Command::new("git")
            .args(["branch"])
            .current_dir(repo_path)
            .output()
            .ok()?;

        let text = String::from_utf8_lossy(&output.stdout);
        let mut branches = Vec::new();
        let mut current_branch = String::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with('*') {
                let name = trimmed[1..].trim().to_string();
                current_branch = name.clone();
                branches.push(name);
            } else {
                branches.push(trimmed.to_string());
            }
        }

        if branches.is_empty() {
            return None;
        }

        Some(BranchPickerModal {
            repo_name,
            repo_path: repo_path.to_path_buf(),
            branches,
            current_branch,
            selected_index: 0,
        })
    }
}

pub fn draw(f: &mut Frame, picker: &BranchPickerModal, area: Rect) {
    let popup_area = centered_rect(50, 50, area);

    f.render_widget(Clear, popup_area);

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Current Branch: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&picker.current_branch, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    for (i, branch) in picker.branches.iter().enumerate() {
        let is_selected = i == picker.selected_index;
        let is_current = branch == &picker.current_branch;

        let cursor = if is_selected { "▶ " } else { "  " };
        let active_badge = if is_current { " [current]" } else { "" };

        let branch_color = if is_selected {
            Color::Yellow
        } else if is_current {
            Color::Green
        } else {
            Color::White
        };

        lines.push(Line::from(vec![
            Span::styled(cursor, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(
                branch.clone(),
                Style::default().fg(branch_color).add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
            ),
            Span::styled(active_badge, Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("Checkout Branch   ", Style::default().fg(Color::White)),
        Span::styled("[Esc/b] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", Style::default().fg(Color::White)),
    ]));

    let title = format!(" 🌿 Switch Branch — {} ", picker.repo_name);

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
