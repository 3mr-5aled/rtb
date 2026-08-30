use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(68, 88, area);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" RTB (ﺐﺘّﺭ) Help & Keybindings ([Esc] or [?] to close) ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .title_alignment(Alignment::Center);

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let logo_lines = super::get_logo_lines();
    let logo_height = logo_lines.len() as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(logo_height),
            Constraint::Min(10),
        ])
        .split(inner_area);

    super::render_logo_block(f, chunks[0], &logo_lines);

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("  GLOBAL KEYS", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  1-5", Style::default().fg(Color::Cyan)),
        Span::styled("          Switch tabs (Dashboard/Projects/Git/Clean/Maint)", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Tab", Style::default().fg(Color::Cyan)),
        Span::styled("          Cycle to next view", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  /", Style::default().fg(Color::Cyan)),
        Span::styled("            Global fuzzy search modal", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  R", Style::default().fg(Color::Cyan)),
        Span::styled("            Multi-threaded refresh & re-scan", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ?", Style::default().fg(Color::Cyan)),
        Span::styled("            Toggle this help modal", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  q", Style::default().fg(Color::Cyan)),
        Span::styled("            Quit RTB", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("  PROJECT BROWSER (Tab 2)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ↑/↓, j/k", Style::default().fg(Color::Cyan)),
        Span::styled("     Navigate project list with 1:1 precision", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  v", Style::default().fg(Color::Cyan)),
        Span::styled("            Open interactive terminal Markdown README preview", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  o / Enter", Style::default().fg(Color::Cyan)),
        Span::styled("    Open project in VS Code (`code .`)", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  e", Style::default().fg(Color::Cyan)),
        Span::styled("            Open in Windows File Explorer", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  p / r", Style::default().fg(Color::Cyan)),
        Span::styled("        Pause active project / Resume paused project", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  a / A", Style::default().fg(Color::Cyan)),
        Span::styled("        Launch default AI Agent / Archive project to .tar.gz", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  f", Style::default().fg(Color::Cyan)),
        Span::styled("            Cycle category filter (Active, Paused, Prod, etc.)", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("  GIT HEALTH (Tab 3)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ↑/↓, j/k", Style::default().fg(Color::Cyan)),
        Span::styled("     Navigate git repositories", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  c / P / p", Style::default().fg(Color::Cyan)),
        Span::styled("    Commit all changes & Push / Git Push / Git Pull", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("  DEP CLEANER (Tab 4)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Space", Style::default().fg(Color::Cyan)),
        Span::styled("        Toggle folder selection checkbox", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  c", Style::default().fg(Color::Cyan)),
        Span::styled("            Cycle category filter (Paused Only, Active, etc.)", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  a / n", Style::default().fg(Color::Cyan)),
        Span::styled("        Select All / Deselect All", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Enter", Style::default().fg(Color::Cyan)),
        Span::styled("        Prune selected dependency folders", Style::default().fg(Color::White)),
    ]));

    let para = Paragraph::new(lines);
    f.render_widget(para, chunks[1]);
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
