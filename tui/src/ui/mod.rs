pub mod branch_picker;
pub mod command_palette;
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
pub mod tab;
pub mod toast;

const EMBEDDED_LOGO: &str = include_str!("../../../logo.txt");

fn get_logo() -> String {
    // 1. Next to the deployed binary (highest priority — updated by install.ps1)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            if let Ok(content) = std::fs::read_to_string(exe_dir.join("logo.txt")) {
                if !content.trim().is_empty() {
                    return content;
                }
            }
        }
    }
    // 2. User config dir (%APPDATA%\rtb\logo.txt or ~/.config/rtb/logo.txt)
    if let Some(config_dir) = dirs::config_dir() {
        if let Ok(content) = std::fs::read_to_string(config_dir.join("rtb").join("logo.txt")) {
            if !content.trim().is_empty() {
                return content;
            }
        }
    }
    // 3. Relative repo path for OSS contributors running from source
    if let Ok(content) = std::fs::read_to_string("logo.txt") {
        if !content.trim().is_empty() {
            return content;
        }
    }
    // 4. Compile-time embedded fallback
    EMBEDDED_LOGO.to_string()
}

/// Parse a single line that may contain ANSI SGR escape sequences into Ratatui Spans.
///
/// Handled sequences:
///   `\x1b[0m`          — reset to default style
///   `\x1b[1m`          — bold
///   `\x1b[38;5;Nm`     — 256-colour foreground  → Color::Indexed(N)
///   `\x1b[38;2;R;G;Bm` — truecolour foreground  → Color::Rgb(R,G,B)
///
/// All other sequences are silently skipped (the escape is consumed, the text
/// after it continues with the last known style).
fn parse_ansi_line_to_spans(line: &str) -> Vec<ratatui::text::Span<'static>> {
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::Span;

    // Normalize escape representations: \033, \e, \x1b, or raw [38;5;...m
    let normalized = line
        .replace("\\033", "\x1b")
        .replace("\\e", "\x1b")
        .replace("\\x1b", "\x1b");

    let mut chars_vec: Vec<char> = Vec::new();
    let chars: Vec<char> = normalized.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '[' && (i == 0 || chars[i - 1] != '\x1b') {
            let rest: String = chars[i..].iter().take(10).collect();
            if rest.starts_with("[38;") || rest.starts_with("[0m") || rest.starts_with("[1m") || rest.starts_with("[39m") {
                chars_vec.push('\x1b');
            }
        }
        chars_vec.push(chars[i]);
        i += 1;
    }

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut buf = String::new();
    let mut style = Style::default();
    let mut iter = chars_vec.into_iter().peekable();

    while let Some(ch) = iter.next() {
        if ch != '\x1b' {
            buf.push(ch);
            continue;
        }
        if iter.peek() != Some(&'[') {
            buf.push(ch);
            continue;
        }
        iter.next(); // consume '['

        if !buf.is_empty() {
            spans.push(Span::styled(buf.clone(), style));
            buf.clear();
        }

        let mut params = String::new();
        let terminator = loop {
            match iter.next() {
                Some(c) if c.is_ascii_alphabetic() => break c,
                Some(c) => params.push(c),
                None => break 'm',
            }
        };

        if terminator != 'm' {
            continue;
        }

        let parts: Vec<&str> = params.split(';').collect();
        match parts.as_slice() {
            ["0"] | [""] => style = Style::default(),
            ["1"] => style = style.add_modifier(Modifier::BOLD),
            ["38", "5", n] => {
                if let Ok(idx) = n.parse::<u8>() {
                    style = Style::default().fg(Color::Indexed(idx));
                }
            }
            ["38", "2", r, g, b] => {
                if let (Ok(rv), Ok(gv), Ok(bv)) =
                    (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>())
                {
                    style = Style::default().fg(Color::Rgb(rv, gv, bv));
                }
            }
            _ => {}
        }
    }

    if !buf.is_empty() {
        spans.push(Span::styled(buf, style));
    }
    if spans.is_empty() {
        spans.push(Span::raw(""));
    }
    spans
}

/// Strip ANSI escape sequences from a string to get raw unstyled text for display width calculations.
pub fn strip_ansi(s: &str) -> String {
    let normalized = s
        .replace("\\033", "\x1b")
        .replace("\\e", "\x1b")
        .replace("\\x1b", "\x1b");

    let mut out = String::new();
    let mut chars = normalized.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
        }
        out.push(ch);
    }
    out
}

/// Calculate exact terminal display-cell width of a string (excluding ANSI escape sequences).
pub fn display_width(s: &str) -> usize {
    use unicode_width::UnicodeWidthChar;
    let clean = strip_ansi(s);
    clean
        .chars()
        .map(|c| {
            if ('\u{2800}'..='\u{28FF}').contains(&c) {
                1
            } else {
                UnicodeWidthChar::width(c).unwrap_or(1)
            }
        })
        .sum()
}

/// Render a multi-line logo as a unified block centered within `area`.
/// Ensures EVERY logo row starts at the EXACT SAME intended X coordinate
/// so horizontal geometry and relative row alignments are NEVER distorted.
pub fn render_logo_block<'a>(f: &mut Frame, area: Rect, lines: &[ratatui::text::Line<'a>]) {
    use ratatui::layout::Alignment;
    use ratatui::widgets::Paragraph;

    if lines.is_empty() || area.width == 0 || area.height == 0 {
        return;
    }

    let max_width = lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| display_width(&span.content))
                .sum::<usize>()
        })
        .max()
        .unwrap_or(0) as u16;

    if max_width == 0 {
        return;
    }

    let block_width = max_width.min(area.width);
    let block_x = area.x + (area.width.saturating_sub(block_width)) / 2;

    let logo_rect = Rect {
        x: block_x,
        y: area.y,
        width: block_width,
        height: (lines.len() as u16).min(area.height),
    };

    let para = Paragraph::new(lines.to_vec()).alignment(Alignment::Left);
    f.render_widget(para, logo_rect);
}

/// Read logo.txt from disk (or fall back to embedded), parse any ANSI colour
/// escape sequences, and return ready-to-render Ratatui Lines.
pub fn get_logo_lines() -> Vec<ratatui::text::Line<'static>> {
    let logo = get_logo();
    // Strip UTF-8 BOM (U+FEFF) that some editors prepend — it shifts line 1 right.
    let content = logo.trim_start_matches('\u{FEFF}');
    content
        .lines()
        .map(|l| {
            // Trim trailing ASCII spaces; they inflate max_width and shift the block left.
            let trimmed = l.trim_end_matches(' ');
            ratatui::text::Line::from(parse_ansi_line_to_spans(trimmed))
        })
        .collect()
}

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
    if let Some(cmd) = &app.command_palette {
        command_palette::draw(f, cmd, size);
    } else if let Some(scaffold) = &app.scaffold_modal {
        scaffold::draw(f, scaffold, size);
    } else if let Some(vault) = &app.env_vault_modal {
        env_vault::draw(f, vault, size);
    } else if let Some(diff) = &app.git_diff_modal {
        git_diff::draw(f, diff, size);
    } else if let Some(picker) = &app.branch_picker_modal {
        branch_picker::draw(f, picker, size);
    } else if let Some((name, content, scroll)) = &app.readme_modal {
        readme_viewer::draw(f, name, content, *scroll, size);
    } else if let Some(commit) = &app.commit_dialog {
        dialogs::draw_commit_dialog(f, commit, size);
    } else if let Some(dialog) = &app.confirm_dialog {
        dialogs::draw(f, dialog, size);
    } else if app.search_active {
        search::draw(f, &app.search_query, app.filtered_projects().len(), size);
    } else if app.show_help {
        help::draw(f, size);
    }

    // Always render toasts on top of everything
    toast::draw(f, &app.toast_queue, size);
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
                .title(" RTB (rtbtui) v2 ")
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
