pub mod branch_picker;
pub mod dashboard;
pub mod dep_cleaner;
pub mod dialogs;
pub mod env_vault;
pub mod git_diff;
pub mod git_health;
pub mod help;
pub mod loading;
pub mod maintenance;
pub mod ports;
pub mod projects;
pub mod readme_viewer;
pub mod scaffold;
pub mod search;
pub mod status_bar;

use crate::app::{App, Tab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Tabs},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // If initial loading or refreshing and no cache available, show spinner overlay
    if app.is_loading {
        loading::draw(f, app.tick_count, app.loading_message, size);
        return;
    }

    // Main layout: tab bar + content + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(size);

    draw_tabs(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    status_bar::draw(f, app, chunks[2]);

    // Render modals on top in priority order
    if let Some(scaffold) = &app.scaffold_modal {
        scaffold::draw(f, scaffold, size);
    } else if let Some(vault) = &app.env_vault_modal {
        env_vault::draw(f, vault, size);
    } else if let Some(diff) = &app.git_diff_modal {
        git_diff::draw(f, diff, size);
    } else if let Some(picker) = &app.branch_picker_modal {
        branch_picker::draw(f, picker, size);
    } else if let Some((name, content, scroll)) = &app.readme_modal {
        readme_viewer::draw(f, name, content, *scroll, size);
    } else if let Some(dialog) = &app.confirm_dialog {
        dialogs::draw(f, dialog, size);
    } else if app.search_active {
        search::draw(f, &app.search_query, app.filtered_projects().len(), size);
    } else if app.show_help {
        help::draw(f, size);
    }
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let tabs = vec![
        Line::from(" 1 Dashboard "),
        Line::from(" 2 Projects "),
        Line::from(" 3 Git Health "),
        Line::from(" 4 Dep Cleaner "),
        Line::from(" 5 Maintenance "),
        Line::from(" 6 Dev Ports "),
    ];

    let selected = app.current_tab as usize;

    let tab_widget = Tabs::new(tabs)
        .select(selected)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" devtui v2 ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(Color::Gray));

    f.render_widget(tab_widget, area);
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    match app.current_tab {
        Tab::Dashboard => dashboard::draw(f, app, area),
        Tab::Projects => projects::draw(f, app, area),
        Tab::GitHealth => git_health::draw(f, app, area),
        Tab::DepCleaner => dep_cleaner::draw(f, app, area),
        Tab::Maintenance => maintenance::draw(f, app, area),
        Tab::DevPorts => ports::draw(f, app, area),
    }
}
