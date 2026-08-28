use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum ScaffoldStep {
    NameInput,
    CategorySelect,
    TemplateSelect,
    Creating,
}

#[derive(Debug, Clone)]
pub struct ScaffoldModal {
    pub step: ScaffoldStep,
    pub project_name: String,
    pub category_index: usize,
    pub template_index: usize,
    pub status_message: Option<String>,
}

impl ScaffoldModal {
    pub fn new() -> Self {
        ScaffoldModal {
            step: ScaffoldStep::NameInput,
            project_name: String::new(),
            category_index: 0,
            template_index: 0,
            status_message: None,
        }
    }

    pub fn categories() -> &'static [(&'static str, &'static str)] {
        &[
            ("01-Active", "D:\\02-Projects\\01-Development\\01-Active"),
            ("03-Vibe", "D:\\02-Projects\\03-Vibe-Coding"),
            ("01-SandBox", "D:\\01-SandBox\\01-Quick-Tests"),
            ("02-Planning", "D:\\02-Projects\\01-Development\\02-Planning"),
        ]
    }

    pub fn templates() -> &'static [(&'static str, &'static str)] {
        &[
            ("⚡ Next.js 15", "Next.js App Router, Tailwind CSS, TypeScript"),
            ("⚛️ React 19 + Vite", "React + Vite, Tailwind CSS, TypeScript"),
            ("🦀 Rust CLI Application", "Rust binary project with Clap & Ratatui"),
            ("🐍 Python FastAPI Backend", "FastAPI, Uvicorn, Pydantic"),
            ("📦 Clean Empty Starter", "Standardized with PROJECT.md, .gitignore, Git repo"),
        ]
    }

    pub fn execute_scaffold(&self) -> Result<PathBuf, String> {
        let clean_name = self.project_name.trim().to_lowercase().replace(' ', "-");
        if clean_name.is_empty() {
            return Err("Project name cannot be empty".into());
        }

        let cat_path = Self::categories()[self.category_index].1;
        let target_dir = PathBuf::from(cat_path).join(&clean_name);

        if target_dir.exists() {
            return Err(format!("Directory already exists: {:?}", target_dir));
        }

        if let Err(e) = fs::create_dir_all(&target_dir) {
            return Err(format!("Failed to create directory: {}", e));
        }

        // Write PROJECT.md
        let project_md = format!(
            "# Project Metadata: {}\n\n- **Status**: Active\n- **Created Date**: {}\n- **Tech Stack**: {}\n- **Repository**: Local (D Drive)\n\n## Description\nAuto-scaffolded project via devtui.\n",
            clean_name,
            chrono::Local::now().format("%Y-%m-%d"),
            Self::templates()[self.template_index].0
        );
        let _ = fs::write(target_dir.join("PROJECT.md"), project_md);

        // Write .gitignore
        let gitignore = "node_modules/\n.next/\ndist/\nbuild/\ntarget/\n.venv/\n__pycache__/\n*.log\n.env\n.env.local\n";
        let _ = fs::write(target_dir.join(".gitignore"), gitignore);

        // Initialize Git
        let _ = Command::new("git").arg("init").current_dir(&target_dir).output();

        // Launch in VS Code
        let _ = Command::new("cmd").args(["/C", "code", "."]).current_dir(&target_dir).spawn();

        Ok(target_dir)
    }
}

pub fn draw(f: &mut Frame, modal: &ScaffoldModal, area: Rect) {
    let popup_area = centered_rect(65, 60, area);

    f.render_widget(Clear, popup_area);

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    match modal.step {
        ScaffoldStep::NameInput => {
            lines.push(Line::from(vec![
                Span::styled("  Step 1 of 3: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Enter Project Name (kebab-case)", Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Name: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{}_", &modal.project_name),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("Next Step   ", Style::default().fg(Color::White)),
                Span::styled("[Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled("Cancel", Style::default().fg(Color::White)),
            ]));
        }
        ScaffoldStep::CategorySelect => {
            lines.push(Line::from(vec![
                Span::styled("  Step 2 of 3: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Select Target Destination", Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(""));

            for (i, (name, path)) in ScaffoldModal::categories().iter().enumerate() {
                let is_sel = i == modal.category_index;
                let cursor = if is_sel { "▶ " } else { "  " };
                lines.push(Line::from(vec![
                    Span::styled(cursor, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:<15}", name), Style::default().fg(if is_sel { Color::Yellow } else { Color::White })),
                    Span::styled(format!(" ({})", path), Style::default().fg(Color::DarkGray)),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  [↑/↓] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled("Choose   ", Style::default().fg(Color::White)),
                Span::styled("[Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("Next Step   ", Style::default().fg(Color::White)),
                Span::styled("[Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled("Back", Style::default().fg(Color::White)),
            ]));
        }
        ScaffoldStep::TemplateSelect => {
            lines.push(Line::from(vec![
                Span::styled("  Step 3 of 3: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Select Stack Template Preset", Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(""));

            for (i, (name, desc)) in ScaffoldModal::templates().iter().enumerate() {
                let is_sel = i == modal.template_index;
                let cursor = if is_sel { "▶ " } else { "  " };
                lines.push(Line::from(vec![
                    Span::styled(cursor, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:<28}", name), Style::default().fg(if is_sel { Color::Yellow } else { Color::White })),
                    Span::styled(format!(" — {}", desc), Style::default().fg(Color::DarkGray)),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("Create Project & Launch VS Code   ", Style::default().fg(Color::White)),
                Span::styled("[Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled("Back", Style::default().fg(Color::White)),
            ]));
        }
        ScaffoldStep::Creating => {
            lines.push(Line::from(""));
            lines.push(Line::from("  🚀 Scaffolding new project & opening VS Code..."));
        }
    }

    if let Some(msg) = &modal.status_message {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(format!("  ⚠ {}", msg), Style::default().fg(Color::Red))));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ✨ New Project Scaffolding Wizard ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .title_alignment(Alignment::Center);

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, popup_area);
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
