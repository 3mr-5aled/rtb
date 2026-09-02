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

    // Tiered scoring:
    // Exact: 100, Prefix: 75, Substring: 50, Path substring: 25
    let mut scored: Vec<(&crate::data::project::Project, u32)> = Vec::new();
    for p in &projects {
        let name_lower = p.name.to_lowercase();
        let path_lower = p.path.to_string_lossy().to_lowercase();
        let score = if name_lower == target {
            100
        } else if name_lower.starts_with(&target) {
            75
        } else if name_lower.contains(&target) {
            50
        } else if path_lower.contains(&target) {
            25
        } else {
            0
        };

        if score > 0 {
            scored.push((p, score));
        }
    }

    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.name.cmp(&b.0.name)));

    if scored.is_empty() {
        eprintln!("rtb: no project found matching '{}'", target);
        return Ok(1);
    }

    let top_score = scored[0].1;
    let top_matches: Vec<_> = scored.iter().filter(|(_, s)| *s == top_score).collect();

    if top_matches.len() == 1 {
        println!("{}", top_matches[0].0.path.display());
        return Ok(0);
    }

    if std::io::stdin().is_terminal() {
        eprintln!("Multiple projects match '{}':", target);
        for (idx, (p, score)) in scored.iter().enumerate() {
            eprintln!("  [{}] {} ({}) [Score: {}]", idx + 1, p.name, p.path.display(), score);
        }
        eprint!("Select project (1-{}): ", scored.len());
        let _ = std::io::stderr().flush();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            if let Ok(choice) = input.trim().parse::<usize>() {
                if choice >= 1 && choice <= scored.len() {
                    println!("{}", scored[choice - 1].0.path.display());
                    return Ok(0);
                }
            }
        }
        eprintln!("Invalid selection.");
        return Ok(1);
    } else {
        let names: Vec<String> = top_matches.iter().map(|(p, _)| p.name.clone()).collect();
        eprintln!(
            "rtb: multiple projects found matching '{}': {}",
            target,
            names.join(", ")
        );
        return Ok(1);
    }
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
        let agents = crate::data::agents::AgentOrchestrator::discover();
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
        None => match crate::data::agents::AgentOrchestrator::default_agent() {
            Some(a) => a.command,
            None => {
                eprintln!("No installed AI agents found in PATH.");
                return Ok(1);
            }
        },
    };

    if !crate::data::agents::AgentOrchestrator::is_installed(&target_agent) {
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

    let _ = crate::data::agents::AgentOrchestrator::generate_context(&proj);

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Launching AI Agent ({})", target_agent);
    println!("══════════════════════════════════════════\n");
    println!("  Context generated: {}", target_path.join(".rtb_context.md").display());
    println!("  Launching process '{}' in {}...\n", target_agent, target_path.display());

    match crate::data::agents::AgentOrchestrator::launch(&target_agent, &target_path) {
        Ok(code) => Ok(code),
        Err(e) => {
            eprintln!("Failed to launch agent process '{}': {}", target_agent, e);
            Ok(1)
        }
    }
}
