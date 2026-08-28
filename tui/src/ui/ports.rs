use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4), // Summary bar
            Constraint::Min(5),    // Table of active ports
            Constraint::Length(2), // Action bar
        ])
        .split(area);

    draw_summary(f, app, chunks[0]);
    draw_ports_table(f, app, chunks[1]);
    draw_action_bar(f, chunks[2]);
}

fn draw_summary(f: &mut Frame, app: &App, area: Rect) {
    let count = app.active_ports.len();

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  🔌 Active Dev Ports: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{} listening servers", count), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Monitored: ", Style::default().fg(Color::Yellow)),
            Span::styled(":3000, :3001, :5173, :8000, :8080, :4321, :5000", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Active Dev Servers & Port Monitor ")
        .title_style(Style::default().fg(Color::Cyan));

    let para = Paragraph::new(text).block(block);
    f.render_widget(para, area);
}

fn draw_ports_table(f: &mut Frame, app: &App, area: Rect) {
    let rows: Vec<Row> = app
        .active_ports
        .iter()
        .map(|port| {
            let port_str = format!(":{}", port.port);

            Row::new(vec![
                Cell::from(Span::styled("🟢", Style::default().fg(Color::Green))),
                Cell::from(Span::styled(
                    port_str,
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    port.process_name.clone(),
                    Style::default().fg(Color::White),
                )),
                Cell::from(Span::styled(
                    format!("{}", port.pid),
                    Style::default().fg(Color::Cyan),
                )),
                Cell::from(Span::styled(
                    port.memory_str.clone(),
                    Style::default().fg(Color::DarkGray),
                )),
                Cell::from(Span::styled(
                    format!("http://localhost:{}", port.port),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let header = Row::new(vec![
        Cell::from(Span::styled("Stat", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Port", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Process Name", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("PID", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Memory", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Local URL", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
    ])
    .style(Style::default().bg(Color::DarkGray));

    let title = if app.active_ports.is_empty() {
        " Active Ports (No development servers running) ".to_string()
    } else {
        format!(" Active Ports ({}) ", app.active_ports.len())
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(Style::default().fg(Color::Cyan)),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    if !app.active_ports.is_empty() {
        state.select(Some(app.ports_selected_index.min(app.active_ports.len().saturating_sub(1))));
    }
    f.render_stateful_widget(table, area, &mut state);
}

fn draw_action_bar(f: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(" 🚫 [k] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled("Kill Process (taskkill /F)   ", Style::default().fg(Color::White)),
        Span::styled("🌐 [o] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Open in Browser   ", Style::default().fg(Color::White)),
        Span::styled("🔄 [r] ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::styled("Re-scan Ports", Style::default().fg(Color::White)),
    ]);

    let para = Paragraph::new(line);
    f.render_widget(para, area);
}
