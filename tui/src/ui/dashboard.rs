use crate::app::App;
use crate::data::deps::format_bytes;
use crate::data::project::ProjectStatus;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

const ASCII_LOGO: &[&str] = &[
    r"  &&&&&&&&&&&&&&&         &&&&&&&&&&&&&&&&&&X    X&&&&&&&&&&&&&&  ",
    r"  &&&&&&&&&&&&&&&&&&     &&&&&&&&&&&&&&&&&&&&    &&&&&&&&&&&&&&&&&& ",
    r"  &&&&&        &&&&&&&          &&&&&&           &&&&&&        &&&&&&",
    r"  &&&&&         &&&&&&          &&&&&&           &&&&&&        &&&&&&",
    r"  &&&&&         &&&&&&          &&&&&&           &&&&&&&&&&&&&&&&&&& ",
    r"  &&&&&&&&&&&&&&&&&&&&          &&&&&&           &&&&&&&&&&&&&&&&&&  ",
    r"  &&&&&&&&&&&&&&&&              &&&&&&           &&&&&&         &&&&&",
    r"  &&&&&     &&&&&&&             &&&&&&           &&&&&&        &&&&&&",
    r"  &&&&&        &&&&&&           &&&&&&           &&&&&&&&&&&&&&&&&&& ",
    r"  &&&&&         &&&&&&          X&&&&X           X&&&&&&&&&&&&&&&    ",
];

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(11), // Centered Logo Banner (No border)
            Constraint::Length(6),  // Workspace Pulse
            Constraint::Min(10),    // Quick Jump + Action Items
            Constraint::Length(5),  // Tech Stack Ecosystem Bar
        ])
        .split(area);

    draw_logo_banner(f, chunks[0]);
    draw_workspace_pulse(f, app, chunks[1]);
    draw_middle_row(f, app, chunks[2]);
    draw_tech_stack_ecosystem(f, app, chunks[3]);
}

fn draw_logo_banner(f: &mut Frame, area: Rect) {
    let mut lines = Vec::new();
    for line in ASCII_LOGO {
        lines.push(Line::from(Span::styled(
            *line,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
    }

    let block = Block::default(); // No border
    let para = Paragraph::new(lines).block(block).alignment(Alignment::Center);
    f.render_widget(para, area);
}

fn draw_workspace_pulse(f: &mut Frame, app: &App, area: Rect) {
    let total = app.projects.len();
    let active = app.active_count();
    let paused = app.paused_count();
    let deployed = app.deployed_count();
    let vibe = app.vibe_count();

    let reclaimable_bytes: u64 = app.dep_folders
        .iter()
        .filter(|d| matches!(d.project_status, ProjectStatus::Paused | ProjectStatus::Abandoned))
        .map(|d| d.size_bytes)
        .sum();

    let port_count = app.active_ports.len();
    let ports_summary = if port_count > 0 {
        let ports_list: Vec<String> = app.active_ports.iter().map(|p| format!(":{}", p.port)).collect();
        format!("{} active ({})", port_count, ports_list.join(", "))
    } else {
        "0 active".to_string()
    };

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  📊 Projects: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} total", total), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("🟢 {} active", active), Style::default().fg(Color::Green)),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("⏸  {} paused", paused), Style::default().fg(Color::Yellow)),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("🚀 {} deployed", deployed), Style::default().fg(Color::Blue)),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("⚡ {} vibe", vibe), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  🔌 Dev Servers: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(ports_summary, Style::default().fg(if port_count > 0 { Color::Green } else { Color::DarkGray })),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled("🧹 Reclaimable: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} in paused projects", format_bytes(reclaimable_bytes)), Style::default().fg(Color::White)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Workspace Pulse ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let para = Paragraph::new(text).block(block);
    f.render_widget(para, area);
}

fn draw_middle_row(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(area);

    draw_quick_jump(f, app, chunks[0]);
    draw_action_items(f, app, chunks[1]);
}

fn draw_quick_jump(f: &mut Frame, app: &App, area: Rect) {
    let recent = app.recent_projects();
    let mut lines = vec![Line::from("")];

    if recent.is_empty() {
        lines.push(Line::from("  No projects found in workspace."));
    } else {
        for (i, project) in recent.iter().enumerate() {
            let is_selected = i == app.dashboard_selected_index;
            let cursor = if is_selected { "▶ " } else { "  " };

            let (icon, color) = match project.status {
                ProjectStatus::Active => ("🟢", Color::Green),
                ProjectStatus::Paused => ("⏸ ", Color::Yellow),
                ProjectStatus::Production => ("🚀", Color::Blue),
                ProjectStatus::Staging => ("🧪", Color::Cyan),
                ProjectStatus::Vibe => ("⚡", Color::Magenta),
                _ => ("• ", Color::Gray),
            };

            let stack_badge = if !project.stack.is_empty() {
                format!("({})", project.stack.join(", "))
            } else {
                "-".to_string()
            };

            let name_color = if is_selected { Color::Yellow } else { Color::White };

            lines.push(Line::from(vec![
                Span::styled(cursor, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", icon), Style::default().fg(color)),
                Span::styled(
                    format!("{:<20}", &project.name),
                    Style::default().fg(name_color).add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
                ),
                Span::styled(format!("{:<18}", stack_badge), Style::default().fg(Color::Cyan)),
                Span::styled(
                    project.last_modified_str(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [↑/↓] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Select  ", Style::default().fg(Color::White)),
        Span::styled("[Enter/o] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("Code  ", Style::default().fg(Color::White)),
        Span::styled("[e] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Explorer  ", Style::default().fg(Color::White)),
        Span::styled("[v] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("README  ", Style::default().fg(Color::White)),
        Span::styled("[d] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Diff", Style::default().fg(Color::White)),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ⚡ Quick Jump (Top Recent Projects) ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, area);
}

fn draw_action_items(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![Line::from("")];

    // 1. Projects with high unpushed commits (Red)
    let mut unpushed: Vec<_> = app.projects.iter()
        .filter(|p| p.git.as_ref().map(|g| g.unpushed > 0).unwrap_or(false))
        .collect();
    unpushed.sort_by(|a, b| {
        let a_count = a.git.as_ref().map(|g| g.unpushed).unwrap_or(0);
        let b_count = b.git.as_ref().map(|g| g.unpushed).unwrap_or(0);
        b_count.cmp(&a_count)
    });

    for p in unpushed.iter().take(2) {
        let count = p.git.as_ref().map(|g| g.unpushed).unwrap_or(0);
        lines.push(Line::from(vec![
            Span::styled("  🔴 ", Style::default().fg(Color::Red)),
            Span::styled(format!("{}: ", p.name), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} unpushed commits (backup!)", count), Style::default().fg(Color::Red)),
        ]));
    }

    // 2. Stale dirty projects (Yellow)
    let uncommitted: Vec<_> = app.projects.iter()
        .filter(|p| p.git.as_ref().map(|g| g.uncommitted > 0).unwrap_or(false))
        .collect();

    for p in uncommitted.iter().take(2) {
        let count = p.git.as_ref().map(|g| g.uncommitted).unwrap_or(0);
        lines.push(Line::from(vec![
            Span::styled("  🟡 ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("{}: ", p.name), Style::default().fg(Color::White)),
            Span::styled(format!("{} uncommitted changes", count), Style::default().fg(Color::Yellow)),
        ]));
    }

    // 3. Local clean repositories
    let local_clean_count = app.projects.iter()
        .filter(|p| p.git.as_ref().map(|g| !g.has_remote && g.uncommitted == 0 && g.unpushed == 0).unwrap_or(false))
        .count();

    if local_clean_count > 0 {
        lines.push(Line::from(vec![
            Span::styled("  🏠 ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{} local clean repositories (offline)", local_clean_count), Style::default().fg(Color::Cyan)),
        ]));
    }

    if unpushed.is_empty() && uncommitted.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  ✅ ", Style::default().fg(Color::Green)),
            Span::styled("All repositories synchronized & clean!", Style::default().fg(Color::Green)),
        ]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ⚠️ Action Items & Workspace Health ")
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, area);
}

fn draw_tech_stack_ecosystem(f: &mut Frame, app: &App, area: Rect) {
    let mut stack_counts: HashMap<String, usize> = HashMap::new();
    let mut total_tags = 0;

    for project in &app.projects {
        for tag in &project.stack {
            *stack_counts.entry(tag.clone()).or_insert(0) += 1;
            total_tags += 1;
        }
    }

    let mut sorted_stacks: Vec<(String, usize)> = stack_counts.into_iter().collect();
    sorted_stacks.sort_by(|a, b| b.1.cmp(&a.1));

    let bar_width = 40;
    let colors = [
        Color::Blue,
        Color::Cyan,
        Color::Yellow,
        Color::Green,
        Color::Magenta,
        Color::Red,
    ];

    let mut bar_spans = vec![Span::styled("  ", Style::default())];
    let mut legend_spans = vec![Span::styled("  ", Style::default())];

    if total_tags > 0 {
        for (i, (name, count)) in sorted_stacks.iter().take(6).enumerate() {
            let pct = (*count as f64 / total_tags as f64) * 100.0;
            let width = ((pct / 100.0) * bar_width as f64).round() as usize;
            let color = colors[i % colors.len()];

            if width > 0 {
                bar_spans.push(Span::styled("█".repeat(width), Style::default().fg(color)));
            }

            legend_spans.push(Span::styled(format!("■ {} ", name), Style::default().fg(color)));
            legend_spans.push(Span::styled(format!("({:.0}%)  ", pct), Style::default().fg(Color::White)));
        }
    }

    let lines = vec![
        Line::from(""),
        Line::from(bar_spans),
        Line::from(legend_spans),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 📊 Tech Stack Ecosystem ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, area);
}
