use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, project_name: &str, content: &str, scroll_offset: usize, area: Rect) {
    let popup_area = centered_rect(85, 85, area);

    // Clear background
    f.render_widget(Clear, popup_area);

    let raw_lines: Vec<&str> = content.lines().collect();
    let total_lines = raw_lines.len();

    let mut in_code_block = false;
    let mut formatted_lines: Vec<Line> = Vec::new();

    for line in raw_lines {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            formatted_lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(Color::DarkGray),
            )));
            continue;
        }

        if in_code_block {
            formatted_lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(line, Style::default().fg(Color::Green)),
            ]));
            continue;
        }

        if trimmed.starts_with("# ") {
            formatted_lines.push(Line::from(""));
            formatted_lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}", trimmed),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else if trimmed.starts_with("## ") {
            formatted_lines.push(Line::from(""));
            formatted_lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}", trimmed),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else if trimmed.starts_with("### ") {
            formatted_lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}", trimmed),
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let bullet_text = &trimmed[2..];
            formatted_lines.push(Line::from(vec![
                Span::styled("    • ", Style::default().fg(Color::Cyan)),
                Span::styled(bullet_text, Style::default().fg(Color::White)),
            ]));
        } else if trimmed.starts_with(">") {
            formatted_lines.push(Line::from(vec![
                Span::styled("    │ ", Style::default().fg(Color::DarkGray)),
                Span::styled(trimmed[1..].trim_start().to_string(), Style::default().fg(Color::Gray)),
            ]));
        } else if trimmed.is_empty() {
            formatted_lines.push(Line::from(""));
        } else {
            formatted_lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(line, Style::default().fg(Color::White)),
            ]));
        }
    }

    let max_scroll = formatted_lines.len().saturating_sub(1);
    let current_scroll = scroll_offset.min(max_scroll);

    let title = format!(
        " 📖 README.md Preview — {} (Line {}/{} │ [↑/↓/j/k] Scroll │ [Esc/v] Close) ",
        project_name,
        current_scroll + 1,
        total_lines.max(1)
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
