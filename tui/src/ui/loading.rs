use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn draw(f: &mut Frame, tick_count: u64, message: &str, area: Rect) {
    let popup_area = centered_rect(80, 85, area);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" RTB — ﺐﺘّﺭ (Repository & Tooling Base) ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .title_alignment(Alignment::Center);

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let logo_lines = super::get_logo_lines();
    let logo_height = logo_lines.len() as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top margin
            Constraint::Length(logo_height), // Logo Block
            Constraint::Min(4),   // Spinner & Status
        ])
        .split(inner_area);

    super::render_logo_block(f, chunks[1], &logo_lines);

    let frame_idx = (tick_count as usize) % SPINNER_FRAMES.len();
    let spinner_char = SPINNER_FRAMES[frame_idx];

    let status_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("    {} ", spinner_char), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(message, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    Interactive Developer Project Operations Engine", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let status_para = Paragraph::new(status_lines).alignment(Alignment::Center);
    f.render_widget(status_para, chunks[2]);
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
