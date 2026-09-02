use anyhow::Result;
use std::io::Write;
use std::io::IsTerminal;
use crate::config::DevConfig;
use crate::data::scanner::scan_all_projects;
use crate::engine::Cli;
use crate::engine::helpers::{get_cmd, resolve_project_or_cwd};

pub fn execute_goto_resolve(query: Option<String>, cli: &Cli) -> Result<i32> {
    let config = match DevConfig::load_from(&cli.config) {
        Ok(c) => c,
        Err(_) => return Ok(1),
    };
    let projects = scan_all_projects(&config);
    let target = query.as_deref().unwrap_or("").trim().to_lowercase();
    if target.is_empty() {
        return Ok(1);
    }

    // 1. Exact match
    if let Some(p) = projects.iter().find(|p| p.name.to_lowercase() == target) {
        println!("{}", p.path.display());
        return Ok(0);
    }

    // 2. Substring matches
    let matches: Vec<_> = projects
        .iter()
        .filter(|p| p.name.to_lowercase().contains(&target))
        .collect();

    if matches.len() == 1 {
        println!("{}", matches[0].path.display());
        return Ok(0);
    } else if matches.len() > 1 {
        if std::io::stdin().is_terminal() {
            eprintln!("Multiple projects match '{}':", target);
            for (idx, p) in matches.iter().enumerate() {
                eprintln!("  [{}] {} ({})", idx + 1, p.name, p.path.display());
            }
            eprint!("Select project (1-{}): ", matches.len());
            let _ = std::io::stderr().flush();

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() {
                if let Ok(choice) = input.trim().parse::<usize>() {
                    if choice >= 1 && choice <= matches.len() {
                        println!("{}", matches[choice - 1].path.display());
                        return Ok(0);
                    }
                }
            }
            eprintln!("Invalid selection.");
            return Ok(1);
        } else {
            let names: Vec<String> = matches.iter().map(|p| p.name.clone()).collect();
            eprintln!(
                "rtb: multiple projects found matching '{}': {}",
                target,
                names.join(", ")
            );
            return Ok(1);
        }
    }

    eprintln!("rtb: no project found matching '{}'", target);
    Ok(1)
}

pub fn execute_agent(
    name_or_proj: Option<String>,
    project_arg: Option<String>,
    list: bool,
    clean: bool,
    cli: &Cli,
) -> Result<i32> {
    let (agent_name, proj_name) = match (name_or_proj, project_arg) {
        (Some(ref a), Some(ref p)) => (Some(a.clone()), Some(p.clone())),
        (Some(ref val), None) => {
            let is_known = matches!(
                val.to_lowercase().as_str(),
                "agy" | "claude" | "gemini" | "codex" | "cursor" | "windsurf" | "aider" | "openhands"
            ) || crate::data::agents::is_command_installed(val);
            if is_known {
                (Some(val.clone()), None)
            } else {
                (None, Some(val.clone()))
            }
        }
        (None, p) => (None, p),
    };

    let target_path = match resolve_project_or_cwd(proj_name.as_deref(), cli) {
        Ok(p) => p,
        Err(_) => return Ok(1),
    };

    if clean {
        let ctx_file = target_path.join(".rtb_context.md");
        if ctx_file.exists() {
            let _ = std::fs::remove_file(ctx_file);
            println!("  Removed .rtb_context.md from {}", target_path.display());
        }
        return Ok(0);
    }

    if list {
        let agents = crate::data::agents::get_installed_agents();
        println!("══════════════════════════════════════════");
        println!("  rtb (رتّب) » Installed AI Agents");
        println!("══════════════════════════════════════════\n");
        for a in agents {
            let status = if a.installed { "[Installed]" } else { "[Not Installed]" };
            println!("  {:<25} {:<12} {}", a.name, a.command, status);
        }
        println!();
        return Ok(0);
    }

    let target_agent = match agent_name {
        Some(n) => n,
        None => match crate::data::agents::get_default_agent() {
            Some(a) => a.command,
            None => {
                eprintln!("No installed AI agents found in PATH.");
                return Ok(1);
            }
        },
    };

    if !crate::data::agents::is_command_installed(&target_agent) {
        eprintln!("Agent '{}' not found in PATH.", target_agent);
        return Ok(1);
    }

    // Build Project object for context generation
    let config = DevConfig::load_from(&cli.config)?;
    let projects = scan_all_projects(&config);
    let proj = projects.into_iter().find(|p| p.path == target_path).unwrap_or_else(|| {
        let folder_name = target_path.file_name().unwrap_or(target_path.as_os_str()).to_string_lossy().to_string();
        crate::data::project::Project {
            name: folder_name,
            path: target_path.clone(),
            status: crate::data::project::ProjectStatus::Active,
            stack: vec![],
            last_modified: None,
            total_size_bytes: 0,
            dep_size_bytes: 0,
            git: None,
            readme_preview: None,
            is_monorepo: false,
            ci_cd: None,
            runtime_version: None,
            dev_command: None,
        }
    });

    let _ = crate::data::agents::create_agent_context_file(&proj);

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Launching AI Agent ({})", target_agent);
    println!("══════════════════════════════════════════\n");
    println!("  Context generated: {}", target_path.join(".rtb_context.md").display());
    println!("  Launching process '{}' in {}...\n", target_agent, target_path.display());

    let cmd_to_run = get_cmd(&target_agent);
    let status = std::process::Command::new(cmd_to_run)
        .current_dir(&target_path)
        .status();

    match status {
        Ok(s) => Ok(s.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("Failed to launch agent process '{}': {}", target_agent, e);
            Ok(1)
        }
    }
}
