use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteAction {
    Dashboard,
    Projects,
    GitHealth,
    DepCleaner,
    Maintenance,
    DevPorts,
    Scaffold,
    Search,
    LaunchAgent,
    ReadmeViewer,
    Refresh,
    Help,
}

impl PaletteAction {
    pub fn name(&self) -> &'static str {
        match self {
            PaletteAction::Dashboard => "Dashboard",
            PaletteAction::Projects => "Projects",
            PaletteAction::GitHealth => "Git Health",
            PaletteAction::DepCleaner => "Dep Cleaner",
            PaletteAction::Maintenance => "Maintenance",
            PaletteAction::DevPorts => "Dev Ports",
            PaletteAction::Scaffold => "Scaffold",
            PaletteAction::Search => "Search",
            PaletteAction::LaunchAgent => "Launch Agent",
            PaletteAction::ReadmeViewer => "Readme Viewer",
            PaletteAction::Refresh => "Refresh",
            PaletteAction::Help => "Help",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PaletteAction::Dashboard => "Jump to Dashboard tab",
            PaletteAction::Projects => "Jump to Projects list tab",
            PaletteAction::GitHealth => "Jump to Git Health tab",
            PaletteAction::DepCleaner => "Jump to Dependency Pruner tab",
            PaletteAction::Maintenance => "Jump to Maintenance tab",
            PaletteAction::DevPorts => "Jump to Ports tab",
            PaletteAction::Scaffold => "Open Scaffold Project modal",
            PaletteAction::Search => "Open Global Search modal",
            PaletteAction::LaunchAgent => "Open AI Agent for selected project",
            PaletteAction::ReadmeViewer => "Open Markdown Readme viewer",
            PaletteAction::Refresh => "Reload projects cache",
            PaletteAction::Help => "Open Help overlay",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            PaletteAction::Dashboard => "1",
            PaletteAction::Projects => "2",
            PaletteAction::GitHealth => "3",
            PaletteAction::DepCleaner => "4",
            PaletteAction::Maintenance => "5",
            PaletteAction::DevPorts => "6",
            PaletteAction::Scaffold => "N",
            PaletteAction::Search => "/",
            PaletteAction::LaunchAgent => "a",
            PaletteAction::ReadmeViewer => "v",
            PaletteAction::Refresh => "R",
            PaletteAction::Help => "?",
        }
    }

    pub fn all() -> Vec<PaletteAction> {
        vec![
            PaletteAction::Dashboard,
            PaletteAction::Projects,
            PaletteAction::GitHealth,
            PaletteAction::DepCleaner,
            PaletteAction::Maintenance,
            PaletteAction::DevPorts,
            PaletteAction::Scaffold,
            PaletteAction::Search,
            PaletteAction::LaunchAgent,
            PaletteAction::ReadmeViewer,
            PaletteAction::Refresh,
            PaletteAction::Help,
        ]
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommandPalette {
    pub query: String,
    pub selected_index: usize,
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            selected_index: 0,
        }
    }

    pub fn filtered_actions(&self) -> Vec<PaletteAction> {
        let all = PaletteAction::all();
        if self.query.trim().is_empty() {
            return all;
        }

        use fuzzy_matcher::skim::SkimMatcherV2;
        use fuzzy_matcher::FuzzyMatcher;

        let matcher = SkimMatcherV2::default();
        let q = self.query.trim();

        let mut scored: Vec<(i64, PaletteAction)> = all
            .into_iter()
            .filter_map(|action| {
                let target = format!("{} {}", action.name(), action.description());
                if let Some(score) = matcher.fuzzy_match(&target, q) {
                    Some((score, action))
                } else if target.to_lowercase().contains(&q.to_lowercase()) {
                    Some((0, action))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, action)| action).collect()
    }
}

pub fn draw(f: &mut Frame, palette: &CommandPalette, area: Rect) {
    let popup_area = centered_rect(65, 60, area);

    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search Input box
            Constraint::Min(0),    // Filtered actions list
            Constraint::Length(2), // Help footer
        ])
        .split(popup_area);

    // 1. Search Input
    let input_lines = vec![Line::from(vec![
        Span::styled(" > ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(&palette.query, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("█", Style::default().fg(Color::Cyan)),
    ])];

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(" 🚀 Command Palette (Ctrl+P / Ctrl+K) ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .title_alignment(Alignment::Center);

    let input_para = Paragraph::new(input_lines).block(input_block);
    f.render_widget(input_para, chunks[0]);

    // 2. Actions List
    let actions = palette.filtered_actions();
    let mut list_lines = Vec::new();

    if actions.is_empty() {
        list_lines.push(Line::from(""));
        list_lines.push(Line::from(vec![
            Span::styled("   No matching actions found", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        for (i, action) in actions.iter().enumerate() {
            let is_selected = i == palette.selected_index;
            let cursor = if is_selected { "▶ " } else { "  " };

            let name_style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            list_lines.push(Line::from(vec![
                Span::styled(cursor, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:<15}", action.name()), name_style),
                Span::styled(format!(" {:<30}", action.description()), Style::default().fg(Color::Gray)),
                Span::styled(format!(" [{}]", action.shortcut()), Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Actions ({}) ", actions.len()))
        .title_style(Style::default().fg(Color::DarkGray));

    let list_para = Paragraph::new(list_lines).block(list_block);
    f.render_widget(list_para, chunks[1]);

    // 3. Footer
    let footer_lines = vec![Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("Execute  ", Style::default().fg(Color::White)),
        Span::styled("[Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel  ", Style::default().fg(Color::White)),
        Span::styled("[↑/↓] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Navigate", Style::default().fg(Color::White)),
    ])];

    let footer_para = Paragraph::new(footer_lines).alignment(Alignment::Center);
    f.render_widget(footer_para, chunks[2]);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_palette_all_actions() {
        let palette = CommandPalette::new();
        let actions = palette.filtered_actions();
        assert_eq!(actions.len(), 12);
        assert!(actions.contains(&PaletteAction::Dashboard));
        assert!(actions.contains(&PaletteAction::Projects));
        assert!(actions.contains(&PaletteAction::GitHealth));
    }

    #[test]
    fn test_command_palette_fuzzy_filter() {
        let mut palette = CommandPalette::new();
        palette.query = "dash".into();
        let actions = palette.filtered_actions();
        assert!(!actions.is_empty());
        assert_eq!(actions[0], PaletteAction::Dashboard);

        palette.query = "scaff".into();
        let actions = palette.filtered_actions();
        assert!(!actions.is_empty());
        assert_eq!(actions[0], PaletteAction::Scaffold);
    }
}
