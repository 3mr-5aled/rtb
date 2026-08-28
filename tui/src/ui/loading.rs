use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

const ASCII_LOGO: &[&str] = &[
    r"   ____  _______     _______ _   _ ___ ",
    r"  |  _ \| ____\ \   / /_   _| | | |_ _|",
    r"  | | | |  _|  \ \ / /  | | | | | || | ",
    r"  | |_| | |___  \ V /   | | | |_| || | ",
    r"  |____/|_____|  \_/    |_|  \___/|___|",
];

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn draw(f: &mut Frame, tick_count: u64, message: &str, area: Rect) {
    let popup_area = centered_rect(58, 38, area);

    f.render_widget(Clear, popup_area);

    let frame_idx = (tick_count as usize) % SPINNER_FRAMES.len();
    let spinner_char = SPINNER_FRAMES[frame_idx];

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    // ASCII Art Logo
    for line in ASCII_LOGO {
        lines.push(Line::from(Span::styled(
            *line,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(format!("    {} ", spinner_char), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(message, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("    Interactive Developer Project Manager for D: Drive", Style::default().fg(Color::DarkGray)),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" devtui ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .title_alignment(Alignment::Center);

    let para = Paragraph::new(lines).block(block).alignment(Alignment::Center);
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
