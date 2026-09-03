use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Table of git repos
            Constraint::Length(2), // Action bar
        ])
        .split(area);

    let git_projects = app.filtered_git_projects();

    let all_git_repos: Vec<_> = app.projects.iter().filter(|p| p.git.is_some()).collect();
    let dirty_count = all_git_repos
        .iter()
        .filter(|p| {
            p.git.as_ref().map(|g| g.uncommitted > 0 || g.unpushed > 0).unwrap_or(false)
        })
        .count();
    let local_clean_count = all_git_repos
        .iter()
        .filter(|p| {
            p.git.as_ref().map(|g| !g.has_remote && g.uncommitted == 0 && g.unpushed == 0).unwrap_or(false)
        })
        .count();
    let synced_count = all_git_repos.len().saturating_sub(dirty_count + local_clean_count);

    let loading_suffix = if app.is_loading { " ⏳ [Scanning...]" } else { "" };
    let title = format!(
        " Git Health ({}) [Filter: {}]{} — ⚠ {} needs attention │ 🏠 {} local clean │ ✅ {} synced ",
        git_projects.len(), app.git_filter.label(), loading_suffix, dirty_count, local_clean_count, synced_count
    );

    let rows: Vec<Row> = git_projects
        .iter()
        .map(|project| {
            if let Some(git) = &project.git {
                let is_dirty = git.uncommitted > 0 || git.unpushed > 0;
                let is_local_clean = !git.has_remote && git.uncommitted == 0 && git.unpushed == 0;

                let (status_icon, status_color) = if is_dirty {
                    ("⚠ ", Color::Yellow)
                } else if is_local_clean {
                    ("🏠", Color::Cyan)
                } else {
                    ("✅", Color::Green)
                };

                let uncommitted_str = if git.uncommitted > 0 {
                    format!("{} files", git.uncommitted)
                } else {
                    "-".into()
                };

                let unpushed_str = if git.unpushed > 0 {
                    format!("{} commits", git.unpushed)
                } else {
                    "-".into()
                };

                let remote_cell = if git.has_remote {
                    Span::styled("✅ synced", Style::default().fg(Color::Green))
                } else if is_local_clean {
                    Span::styled("🏠 local", Style::default().fg(Color::Cyan))
                } else {
                    Span::styled("❌ no remote", Style::default().fg(Color::Red))
                };

                let last_commit = git
                    .last_commit_relative
                    .as_deref()
                    .unwrap_or("-")
                    .to_string();

                Row::new(vec![
                    Cell::from(Span::styled(status_icon, Style::default().fg(status_color))),
                    Cell::from(Span::styled(
                        project.name.clone(),
                        Style::default().fg(if is_dirty {
                            Color::Yellow
                        } else if is_local_clean {
                            Color::Cyan
                        } else {
                            Color::White
                        }),
                    )),
                    Cell::from(Span::styled(git.branch.clone(), Style::default().fg(Color::Cyan))),
                    Cell::from(Span::styled(
                        uncommitted_str,
                        Style::default().fg(if git.uncommitted > 0 { Color::Red } else { Color::Gray }),
                    )),
                    Cell::from(Span::styled(
                        unpushed_str,
                        Style::default().fg(if git.unpushed > 0 { Color::Yellow } else { Color::Gray }),
                    )),
                    Cell::from(Span::styled(last_commit, Style::default().fg(Color::DarkGray))),
                    Cell::from(remote_cell),
                ])
            } else {
                Row::new(vec![
                    Cell::from(Span::styled("📁", Style::default().fg(Color::Gray))),
                    Cell::from(Span::styled(project.name.clone(), Style::default().fg(Color::Gray))),
                    Cell::from(Span::styled("-", Style::default().fg(Color::DarkGray))),
                    Cell::from(Span::styled("No Git Repo", Style::default().fg(Color::DarkGray))),
                    Cell::from(Span::styled("-", Style::default().fg(Color::DarkGray))),
                    Cell::from(Span::styled("-", Style::default().fg(Color::DarkGray))),
                    Cell::from(Span::styled("Non-Git", Style::default().fg(Color::DarkGray))),
                ])
            }
        })
        .collect();

    let header = Row::new(vec![
        Cell::from(Span::styled("", Style::default().add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Repository", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Branch", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Uncommitted", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Unpushed", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Last Commit", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Remote Status", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
    ])
    .style(Style::default().bg(Color::DarkGray));

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Percentage(25),
            Constraint::Percentage(12),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(18),
            Constraint::Percentage(12),
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
    if !git_projects.is_empty() {
        state.select(Some(app.git_selected_index.min(git_projects.len().saturating_sub(1))));
    }
    f.render_stateful_widget(table, vertical_chunks[0], &mut state);

    // Action bar
    let line = Line::from(vec![
        Span::styled(" 💻 [o] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Code  ", Style::default().fg(Color::White)),
        Span::styled("🌿 [d] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Diff  ", Style::default().fg(Color::White)),
        Span::styled("🎋 [b] ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled("Branch  ", Style::default().fg(Color::White)),
        Span::styled("📝 [c] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Commit & Push  ", Style::default().fg(Color::White)),
        Span::styled("⬆️ [P] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("Push  ", Style::default().fg(Color::White)),
        Span::styled("⬇️ [p] ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled("Pull  ", Style::default().fg(Color::White)),
        Span::styled("🏷️ [f] ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled("Filter  ", Style::default().fg(Color::White)),
        Span::styled("🔄 [R] ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::styled("Re-scan", Style::default().fg(Color::White)),
    ]);
    let para = Paragraph::new(line);
    f.render_widget(para, vertical_chunks[1]);
}
