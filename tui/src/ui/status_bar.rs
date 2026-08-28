use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let total = app.projects.len();
    let active = app.active_count();
    let paused = app.paused_count();

    // Build status bar spans
    let mut spans = vec![
        Span::styled(" 📊 ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{} projects", total),
            Style::default().fg(Color::White),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled("🟢 ", Style::default()),
        Span::styled(
            format!("{}", active),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" active", Style::default().fg(Color::Gray)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled("⏸  ", Style::default()),
        Span::styled(
            format!("{}", paused),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(" paused", Style::default().fg(Color::Gray)),
    ];

    if let Some(msg) = &app.status_message {
        spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled(msg.clone(), Style::default().fg(Color::Green)));
    }

    // Right side hint
    let hint = "[1-5] Switch tab  [/] Search  [r] Refresh  [?] Help  [q] Quit";
    spans.push(Span::styled(
        format!("  {}", hint),
        Style::default().fg(Color::DarkGray),
    ));

    let line = Line::from(spans);
    let para = Paragraph::new(line)
        .style(Style::default().bg(Color::Reset));
    f.render_widget(para, area);
}
