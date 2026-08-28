use crate::app::App;
use crate::data::maintenance::TaskStatus;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(8),    // Main content
            Constraint::Length(2), // Action bar
        ])
        .split(area);

    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[0]);

    // Left pane: Split between task list (top) and task description (bottom)
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(middle_chunks[0]);

    draw_tasks_list(f, app, left_chunks[0]);
    draw_task_details(f, app, left_chunks[1]);
    draw_logs_panel(f, app, middle_chunks[1]);
    draw_action_bar(f, app, chunks[1]);
}

fn draw_tasks_list(f: &mut Frame, app: &App, area: Rect) {
    let mut items: Vec<ListItem> = Vec::new();

    for (i, task) in app.maintenance_state.tasks.iter().enumerate() {
        let (icon, color) = match task.status {
            TaskStatus::Pending => ("⬚ ", Color::DarkGray),
            TaskStatus::Running => ("🔄", Color::Yellow),
            TaskStatus::Success => ("✅", Color::Green),
            TaskStatus::Warning => ("⚠ ", Color::Yellow),
            TaskStatus::Failed => ("❌", Color::Red),
        };

        let duration_str = if task.duration_ms > 0 {
            format!(" ({:.1}s)", task.duration_ms as f64 / 1000.0)
        } else {
            "".into()
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(color)),
            Span::styled(
                task.name,
                Style::default().fg(if i == app.maintenance_state.selected_task {
                    Color::White
                } else {
                    Color::Gray
                }),
            ),
            Span::styled(duration_str, Style::default().fg(Color::DarkGray)),
        ])));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Maintenance Tasks ")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(app.maintenance_state.selected_task));
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_task_details(f: &mut Frame, app: &App, area: Rect) {
    let selected_idx = app.maintenance_state.selected_task;
    let task = app.maintenance_state.tasks.get(selected_idx);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Task Description & Scope ")
        .title_style(Style::default().fg(Color::Yellow));

    if let Some(t) = task {
        let (status_str, status_color) = match t.status {
            TaskStatus::Pending => ("Pending execution", Color::DarkGray),
            TaskStatus::Running => ("Running in background...", Color::Yellow),
            TaskStatus::Success => ("Last run succeeded", Color::Green),
            TaskStatus::Warning => ("Completed with warnings", Color::Yellow),
            TaskStatus::Failed => ("Last run failed", Color::Red),
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(t.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(" — ", Style::default().fg(Color::DarkGray)),
                Span::styled(status_str, Style::default().fg(status_color)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Purpose: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!("  {}", t.description), Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Target:  ", Style::default().fg(Color::Cyan)),
                Span::styled(t.target_info, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  Script:  ", Style::default().fg(Color::Cyan)),
                Span::styled(t.script_path, Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled("  Flags:   ", Style::default().fg(Color::Cyan)),
                Span::styled(t.flags_info, Style::default().fg(Color::Yellow)),
            ]),
        ];

        let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
        f.render_widget(para, area);
    } else {
        let para = Paragraph::new(vec![Line::from("  No task selected")])
            .block(block);
        f.render_widget(para, area);
    }
}

fn draw_logs_panel(f: &mut Frame, app: &App, area: Rect) {
    let visible_lines: Vec<Line> = app
        .maintenance_state
        .logs
        .iter()
        .rev()
        .take(30)
        .rev()
        .map(|l| {
            let color = if l.starts_with("▶") {
                Color::Cyan
            } else if l.starts_with("✅") {
                Color::Green
            } else if l.starts_with("❌") {
                Color::Red
            } else if l.starts_with("⚠") {
                Color::Yellow
            } else if l.starts_with("===") || l.starts_with("---") {
                Color::Blue
            } else {
                Color::Gray
            };
            Line::from(Span::styled(format!(" {}", l), Style::default().fg(color)))
        })
        .collect();

    let title = if app.maintenance_state.is_running {
        " Live Output (Running...) "
    } else {
        " Execution Output Log "
    };

    let title_color = if app.maintenance_state.is_running {
        Color::Yellow
    } else {
        Color::Cyan
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(title_color));

    let para = Paragraph::new(visible_lines).block(block);
    f.render_widget(para, area);
}

fn draw_action_bar(f: &mut Frame, app: &App, area: Rect) {
    let running_indicator = if app.maintenance_state.is_running {
        Span::styled("  [🔄 TASK IN PROGRESS...]  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("", Style::default())
    };

    let line = Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("Run All Tasks   ", Style::default().fg(Color::White)),
        Span::styled("[s] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Run Selected Task   ", Style::default().fg(Color::White)),
        Span::styled("[c] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Clear Logs", Style::default().fg(Color::White)),
        running_indicator,
    ]);

    let para = Paragraph::new(line);
    f.render_widget(para, area);
}
