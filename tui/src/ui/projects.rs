use crate::app::App;
use crate::data::project::ProjectStatus;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Split pane: List + Detail
            Constraint::Length(2), // Action bar
        ])
        .split(area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(vertical_chunks[0]);

    draw_project_table(f, app, chunks[0]);
    draw_project_detail(f, app, chunks[1]);
    draw_action_bar(f, vertical_chunks[1]);
}

fn draw_project_table(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_projects();

    let rows: Vec<Row> = filtered
        .iter()
        .map(|project| {
            let (icon, color) = status_style(&project.status);
            let date_str = project.last_modified_str();

            let (git_icon, git_color) = if let Some(git) = &project.git {
                if git.uncommitted > 0 || git.unpushed > 0 {
                    ("⚠ ", Color::Yellow)
                } else if !git.has_remote {
                    ("🏠", Color::Cyan)
                } else {
                    ("✅", Color::Green)
                }
            } else {
                ("- ", Color::DarkGray)
            };

            Row::new(vec![
                Cell::from(Span::styled(format!(" {}", icon), Style::default().fg(color))),
                Cell::from(Span::styled(
                    project.name.clone(),
                    Style::default().fg(Color::White),
                )),
                Cell::from(Span::styled(git_icon, Style::default().fg(git_color))),
                Cell::from(Span::styled(
                    date_str,
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let header = Row::new(vec![
        Cell::from(Span::styled("Stat", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Project Name", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Git", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Last Modified", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
    ])
    .style(Style::default().bg(Color::DarkGray));

    let filter_label = match app.project_filter {
        None => "ALL",
        Some(ProjectStatus::Active) => "Active",
        Some(ProjectStatus::Paused) => "Paused",
        Some(ProjectStatus::Production) => "Production",
        Some(ProjectStatus::Staging) => "Staging",
        Some(ProjectStatus::Vibe) => "Vibe",
        Some(ProjectStatus::Sandbox) => "Sandbox",
        _ => "Filtered",
    };

    let title = if app.search_active {
        format!(" Projects — /{} ", app.search_query)
    } else {
        format!(" Projects ({}) [Filter: {}] ", filtered.len(), filter_label)
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),        // Status icon
            Constraint::Percentage(45),   // Project Name
            Constraint::Length(6),        // Git status icon
            Constraint::Percentage(35),   // Last Modified date
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
    if !filtered.is_empty() {
        state.select(Some(app.selected_index.min(filtered.len().saturating_sub(1))));
    }
    f.render_stateful_widget(table, area, &mut state);
}

fn draw_project_detail(f: &mut Frame, app: &App, area: Rect) {
    let project = app.selected_project();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Detail & Overview ")
        .title_style(Style::default().fg(Color::Cyan));

    if let Some(p) = project {
        let (icon, color) = status_style(&p.status);

        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&p.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Status:   ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{} {}", icon, p.status.label()), Style::default().fg(color)),
            ]),
            Line::from(vec![
                Span::styled("  Stack:    ", Style::default().fg(Color::Gray)),
                Span::styled(p.stack.join(", "), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  Modified: ", Style::default().fg(Color::Gray)),
                Span::styled(p.last_modified_str(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  Location: ", Style::default().fg(Color::Gray)),
                Span::styled(p.path.to_string_lossy().to_string(), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(""),
        ];

        // Git section
        if let Some(git) = &p.git {
            lines.push(Line::from(vec![
                Span::styled("  ── Git Status ", Style::default().fg(Color::DarkGray)),
                Span::styled("─────────────────", Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Branch: ", Style::default().fg(Color::Gray)),
                Span::styled(&git.branch, Style::default().fg(Color::Cyan)),
            ]));

            if git.uncommitted > 0 {
                lines.push(Line::from(vec![
                    Span::styled("  ⚠ ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{} uncommitted files", git.uncommitted),
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
            }

            if git.unpushed > 0 {
                lines.push(Line::from(vec![
                    Span::styled("  ⚠ ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{} unpushed commits", git.unpushed),
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
            }

            if git.uncommitted == 0 && git.unpushed == 0 {
                if git.has_remote {
                    lines.push(Line::from(vec![
                        Span::styled("  ✅ ", Style::default().fg(Color::Green)),
                        Span::styled("Working tree clean & synced", Style::default().fg(Color::Green)),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled("  🏠 ", Style::default().fg(Color::Cyan)),
                        Span::styled("Local repository (clean)", Style::default().fg(Color::Cyan)),
                    ]));
                }
            }

            if let Some(msg) = &git.last_commit_msg {
                lines.push(Line::from(vec![
                    Span::styled("  Last:   ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{} ({})", msg, git.last_commit_relative.as_deref().unwrap_or("")),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }

            lines.push(Line::from(""));
        }

        // README Markdown Preview Section
        if let Some(readme) = &p.readme_preview {
            lines.push(Line::from(vec![
                Span::styled("  ── README.md Preview ", Style::default().fg(Color::DarkGray)),
                Span::styled(" (press [v] to expand) ", Style::default().fg(Color::Yellow)),
                Span::styled("──", Style::default().fg(Color::DarkGray)),
            ]));

            for line in readme.lines().take(5) {
                let trimmed = line.trim();
                if trimmed.starts_with("#") {
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled(trimmed, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    ]));
                } else if trimmed.starts_with("-") || trimmed.starts_with("*") {
                    lines.push(Line::from(vec![
                        Span::styled("    • ", Style::default().fg(Color::Yellow)),
                        Span::styled(trimmed[1..].trim_start().to_string(), Style::default().fg(Color::White)),
                    ]));
                } else if !trimmed.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled(trimmed, Style::default().fg(Color::DarkGray)),
                    ]));
                }
            }
        }

        let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
        f.render_widget(para, area);
    } else {
        let para = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Select a project to see details", Style::default().fg(Color::DarkGray)),
            ]),
        ])
        .block(block);
        f.render_widget(para, area);
    }
}

fn draw_action_bar(f: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(" ✨ [N] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("New  ", Style::default().fg(Color::White)),
        Span::styled("🤖 [a] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Agent  ", Style::default().fg(Color::White)),
        Span::styled("🔐 [E] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(".env  ", Style::default().fg(Color::White)),
        Span::styled("🌿 [d] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Diff  ", Style::default().fg(Color::White)),
        Span::styled("📖 [v] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("README  ", Style::default().fg(Color::White)),
        Span::styled("💻 [o] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Code  ", Style::default().fg(Color::White)),
        Span::styled("📂 [e] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Explorer  ", Style::default().fg(Color::White)),
        Span::styled("⏸ [p] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Pause  ", Style::default().fg(Color::White)),
        Span::styled("🟢 [r] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("Resume  ", Style::default().fg(Color::White)),
        Span::styled("🏷️ [f] ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled("Filter", Style::default().fg(Color::White)),
    ]);

    let para = Paragraph::new(line);
    f.render_widget(para, area);
}

fn status_style(status: &ProjectStatus) -> (&'static str, Color) {
    match status {
        ProjectStatus::Active => ("🟢", Color::Green),
        ProjectStatus::Paused => ("⏸ ", Color::Yellow),
        ProjectStatus::Production => ("🚀", Color::Blue),
        ProjectStatus::Staging => ("🧪", Color::Cyan),
        ProjectStatus::Vibe => ("⚡", Color::Magenta),
        ProjectStatus::Sandbox => ("🔬", Color::Gray),
        ProjectStatus::Planning => ("📝", Color::Gray),
        ProjectStatus::Testing => ("🧪", Color::Gray),
        ProjectStatus::Abandoned => ("❌", Color::Red),
    }
}
