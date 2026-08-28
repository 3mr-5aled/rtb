use crate::app::App;
use crate::data::deps::format_bytes;
use crate::data::project::ProjectStatus;
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
            Constraint::Min(5),    // Table of folders
            Constraint::Length(2), // Action bar
        ])
        .split(area);

    let filtered_folders = app.filtered_dep_folders();

    draw_summary_bar(f, app, &filtered_folders, chunks[0]);
    draw_folders_table(f, app, &filtered_folders, chunks[1]);
    draw_action_bar(f, chunks[2]);
}

fn draw_summary_bar(f: &mut Frame, app: &App, folders: &[&crate::data::deps::DepFolder], area: Rect) {
    let total_folders = folders.len();
    let selected_count = folders.iter().filter(|d| d.is_selected).count();
    let total_reclaimable: u64 = folders.iter().filter(|d| d.is_selected).map(|d| d.size_bytes).sum();
    let total_size: u64 = folders.iter().map(|d| d.size_bytes).sum();

    let cat_filter_label = match app.cleaner_category_filter {
        None => "ALL",
        Some(ProjectStatus::Paused) => "Paused Only (Safe)",
        Some(ProjectStatus::Active) => "Active Only",
        Some(ProjectStatus::Abandoned) => "Abandoned Only",
        Some(ProjectStatus::Sandbox) => "Sandbox Only",
        _ => "Custom",
    };

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  🧹 Reclaimable: ", Style::default().fg(Color::Yellow)),
            Span::styled(format_bytes(total_reclaimable), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" (out of {})", format_bytes(total_size)), Style::default().fg(Color::DarkGray)),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Selected: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{}/{} folders", selected_count, total_folders), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Filter: ", Style::default().fg(Color::Magenta)),
            Span::styled(cat_filter_label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Dependency Cleanup Summary ")
        .title_style(Style::default().fg(Color::Cyan));
    let para = Paragraph::new(text).block(block);
    f.render_widget(para, area);
}

fn draw_folders_table(f: &mut Frame, app: &App, folders: &[&crate::data::deps::DepFolder], area: Rect) {
    let rows: Vec<Row> = folders
        .iter()
        .map(|folder| {
            let checkbox = if folder.is_selected { "[x] " } else { "[ ] " };
            let check_color = if folder.is_selected { Color::Green } else { Color::DarkGray };

            let idle_days = folder.days_idle();
            let idle_str = if idle_days > 0 {
                format!("{}d ago", idle_days)
            } else {
                "recent".into()
            };

            let status_badge = match folder.project_status {
                ProjectStatus::Active => Span::styled("🟢 Active", Style::default().fg(Color::Green)),
                ProjectStatus::Paused => Span::styled("⏸  Paused", Style::default().fg(Color::Yellow)),
                ProjectStatus::Abandoned => Span::styled("❌ Abandon", Style::default().fg(Color::Red)),
                ProjectStatus::Production => Span::styled("🚀 Prod", Style::default().fg(Color::Blue)),
                ProjectStatus::Staging => Span::styled("🧪 Stage", Style::default().fg(Color::Cyan)),
                ProjectStatus::Vibe => Span::styled("⚡ Vibe", Style::default().fg(Color::Magenta)),
                ProjectStatus::Sandbox => Span::styled("🔬 Sand", Style::default().fg(Color::Gray)),
                _ => Span::styled("• Other", Style::default().fg(Color::Gray)),
            };

            Row::new(vec![
                Cell::from(Span::styled(checkbox, Style::default().fg(check_color).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(folder.project_name.clone(), Style::default().fg(Color::White))),
                Cell::from(status_badge),
                Cell::from(Span::styled(folder.rel_path.clone(), Style::default().fg(Color::Cyan))),
                Cell::from(Span::styled(folder.size_str(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(idle_str, Style::default().fg(Color::DarkGray))),
            ])
        })
        .collect();

    let header = Row::new(vec![
        Cell::from(Span::styled("Sel", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Project", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Status", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Target Path", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Size", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Last Active", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
    ])
    .style(Style::default().bg(Color::DarkGray));

    let title = format!(" Dependencies ({}) ", folders.len());

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(20),
            Constraint::Percentage(12),
            Constraint::Percentage(33),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
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
    if !folders.is_empty() {
        state.select(Some(app.cleaner_selected_index.min(folders.len().saturating_sub(1))));
    }
    f.render_stateful_widget(table, area, &mut state);
}

fn draw_action_bar(f: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(" ☑️ [Space] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Toggle  ", Style::default().fg(Color::White)),
        Span::styled("🏷️ [c] ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled("Filter Category  ", Style::default().fg(Color::White)),
        Span::styled("✅ [a] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("All  ", Style::default().fg(Color::White)),
        Span::styled("❎ [n] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("None  ", Style::default().fg(Color::White)),
        Span::styled("⚡ [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("PRUNE SELECTED  ", Style::default().fg(Color::Green)),
        Span::styled("🔄 [r] ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::styled("Rescan", Style::default().fg(Color::White)),
    ]);

    let para = Paragraph::new(line);
    f.render_widget(para, area);
}
