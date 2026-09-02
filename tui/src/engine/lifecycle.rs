use anyhow::Result;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use crate::config::DevConfig;
use crate::data::scanner::scan_all_projects;
use crate::engine::Cli;
use crate::engine::helpers::{is_git_clean, move_dir_all, to_kebab_case};

pub fn execute_new(
    name: String,
    root: Option<String>,
    stack: Option<String>,
    cli: &Cli,
) -> Result<i32> {
    let config = DevConfig::load_from(&cli.config)?;
    let kebab_name = to_kebab_case(&name);
    if kebab_name.is_empty() {
        eprintln!("  Invalid project name: '{}'", name);
        return Ok(1);
    }

    let active_root = PathBuf::from(&config.project_roots.active);
    let target_dir = if let Some(r) = root {
        PathBuf::from(r).join(&kebab_name)
    } else {
        if config.project_roots.active.is_empty() {
            eprintln!("  Active project root is not configured in rtb.config.json!");
            return Ok(1);
        }
        active_root.join(&kebab_name)
    };

    if target_dir.exists() {
        eprintln!("  Project '{}' already exists!", kebab_name);
        return Ok(1);
    }

    std::fs::create_dir_all(&target_dir)?;

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Creating project: {}", kebab_name);
    println!("══════════════════════════════════════════\n");

    let stack_str = stack.as_deref().unwrap_or("generic");
    let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    let month_year = chrono::Local::now().format("%B %Y").to_string();

    let template_dir = PathBuf::from(&config.template_dir);
    let template_path = template_dir.join("PROJECT.md");
    if template_path.is_file() {
        if let Ok(meta) = std::fs::read_to_string(&template_path) {
            let meta = meta
                .replace("[Project Name]", &name)
                .replace("YYYY-MM-DD", &date_str)
                .replace("[e.g. react|nextjs|node|python|generic]", stack_str);
            std::fs::write(target_dir.join("PROJECT.md"), meta)?;
            println!("  Created PROJECT.md");
        }
    } else {
        let meta = format!(
            "# {}\n\n## Status\nPhase: Discovery\nCreated: {}\nStack: {}\n",
            name, date_str, stack_str
        );
        std::fs::write(target_dir.join("PROJECT.md"), meta)?;
        println!("  Created PROJECT.md");
    }

    let gitignore = "node_modules/\n.next/\n.venv/\n__pycache__/\ndist/\nbuild/\n.env\n.env.local\n*.log\n";
    std::fs::write(target_dir.join(".gitignore"), gitignore)?;
    println!("  Created .gitignore");

    let readme = format!(
        "# {}\n\nNew development project ({} stack).\n\nCreated: {}\n",
        name, stack_str, month_year
    );
    std::fs::write(target_dir.join("README.md"), readme)?;
    println!("  Created README.md");

    println!("\n  Project '{}' created in 01-Active!", kebab_name);
    println!("  Run: rtb goto {}", kebab_name);
    Ok(0)
}

pub fn execute_pause(
    project: Option<String>,
    prune_deps: bool,
    force: bool,
    cli: &Cli,
) -> Result<i32> {
    let config = DevConfig::load_from(&cli.config)?;
    let name = match project {
        Some(n) => n,
        None => {
            eprintln!("Usage: rtb pause <project-name> [--prune-deps] [--force]");
            return Ok(1);
        }
    };

    let kebab_name = to_kebab_case(&name);
    let active_root = PathBuf::from(&config.project_roots.active);
    let paused_root = PathBuf::from(&config.project_roots.paused);
    let active_path = active_root.join(&kebab_name);
    let paused_path = paused_root.join(&kebab_name);

    if !active_path.exists() {
        eprintln!("  Project '{}' not found in Active!", kebab_name);
        return Ok(1);
    }

    if !is_git_clean(&active_path) {
        eprintln!("  ⚠ WARNING: This project has uncommitted git changes!");
        eprintln!("  Commit or stash first, or pass --force to override.");
        if !force {
            eprintln!("  Aborting.");
            return Ok(1);
        }
    }

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Pausing: {}", kebab_name);
    println!("══════════════════════════════════════════\n");

    if prune_deps {
        println!("  Pruning dependencies...");
        let targets = if config.clean_deps.targets.is_empty() {
            vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".venv".to_string(),
                "dist".to_string(),
                "build".to_string(),
            ]
        } else {
            config.clean_deps.targets.clone()
        };

        for t in targets {
            let dep_path = active_path.join(&t);
            if dep_path.exists() {
                let _ = std::fs::remove_dir_all(&dep_path);
                println!("    Removed {}", t);
            }
        }
    }

    if let Some(parent) = paused_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    move_dir_all(&active_path, &paused_path)?;
    println!("  '{}' moved to Paused ({})", kebab_name, paused_path.display());
    Ok(0)
}

pub fn execute_resume(project: Option<String>, install: bool, cli: &Cli) -> Result<i32> {
    let config = DevConfig::load_from(&cli.config)?;
    let name = match project {
        Some(n) => n,
        None => {
            eprintln!("Usage: rtb resume <project-name> [--install]");
            return Ok(1);
        }
    };

    let kebab_name = to_kebab_case(&name);
    let active_root = PathBuf::from(&config.project_roots.active);
    let paused_root = PathBuf::from(&config.project_roots.paused);
    let paused_path = paused_root.join(&kebab_name);
    let active_path = active_root.join(&kebab_name);

    if !paused_path.exists() {
        eprintln!("  Project '{}' not found in Paused!", kebab_name);
        return Ok(1);
    }

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Resuming: {}", kebab_name);
    println!("══════════════════════════════════════════\n");

    if let Some(parent) = active_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    move_dir_all(&paused_path, &active_path)?;
    println!("  '{}' moved to 01-Active", kebab_name);

    if install {
        if active_path.join("package.json").exists() {
            println!("  Running npm install...");
            let _ = std::process::Command::new("npm")
                .arg("install")
                .current_dir(&active_path)
                .status();
            println!("  npm install complete!");
        } else if active_path.join("requirements.txt").exists() {
            println!("  Running pip install...");
            let _ = std::process::Command::new("pip")
                .args(&["install", "-r", "requirements.txt"])
                .current_dir(&active_path)
                .status();
            println!("  pip install complete!");
        }
    }

    println!("  Run: rtb goto {}", kebab_name);
    Ok(0)
}

pub fn execute_deploy(
    project: Option<String>,
    to: Option<String>,
    _prod: bool,
    staging: bool,
    cli: &Cli,
) -> Result<i32> {
    let config = DevConfig::load_from(&cli.config)?;
    let name = match project {
        Some(n) => n,
        None => {
            eprintln!("Usage: rtb deploy <project-name> [--to production|staging]");
            return Ok(1);
        }
    };

    let target_env = if staging || to.as_deref() == Some("staging") {
        "staging"
    } else {
        "production"
    };

    let kebab_name = to_kebab_case(&name);
    let active_root = PathBuf::from(&config.project_roots.active);
    let active_path = active_root.join(&kebab_name);
    let deploy_root_str = if target_env == "staging" {
        &config.project_roots.staging
    } else {
        &config.project_roots.production
    };
    let deploy_root = PathBuf::from(deploy_root_str);
    let deploy_path = deploy_root.join(&kebab_name);

    if !active_path.exists() {
        eprintln!("  Project '{}' not found in Active!", kebab_name);
        return Ok(1);
    }

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Deploying: {} → {}", kebab_name, target_env);
    println!("══════════════════════════════════════════\n");

    if let Some(parent) = deploy_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    move_dir_all(&active_path, &deploy_path)?;
    println!("  '{}' deployed to {}!", kebab_name, target_env);
    Ok(0)
}

pub fn execute_archive(project: Option<String>, force: bool, cli: &Cli) -> Result<i32> {
    let config = DevConfig::load_from(&cli.config)?;
    let name = match project {
        Some(n) => n,
        None => {
            eprintln!("Usage: rtb archive <project-name> [--force]");
            return Ok(1);
        }
    };

    let projects = scan_all_projects(&config);
    let proj = match projects.iter().find(|p| p.name.eq_ignore_ascii_case(&name)) {
        Some(p) => p,
        None => {
            eprintln!("  Project '{}' not found!", name);
            return Ok(1);
        }
    };

    let proj_path = &proj.path;
    if !is_git_clean(proj_path) {
        eprintln!("  ⚠ WARNING: This project has uncommitted git changes!");
        eprintln!("  Commit or stash your changes first, or pass --force to override.");
        if !force {
            eprintln!("  Aborting.");
            return Ok(1);
        }
    }

    let is_non_interactive = std::env::var("RTB_NON_INTERACTIVE").is_ok()
        || std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || !std::io::stdin().is_terminal();

    let backup_root = if config.backup_root.is_empty() {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".rtb")
            .join("backups")
    } else {
        PathBuf::from(&config.backup_root)
    };

    if !force {
        if is_non_interactive {
            eprintln!("  Archiving in non-interactive mode requires --force.");
            return Ok(1);
        }
        eprintln!("\n  This will:");
        eprintln!("    1. Prune dep folders (node_modules, target, .venv, etc.)");
        eprintln!("    2. Create a .tar.gz in {}", backup_root.display());
        eprintln!("    3. PERMANENTLY DELETE: {}", proj_path.display());
        eprintln!();
        eprint!("  Archive and delete '{}'? (y/N) ", proj.name);
        std::io::stderr().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_lowercase();
        if trimmed != "y" && trimmed != "yes" {
            println!("  Aborted.");
            return Ok(0);
        }
    }

    let snapshot_dir = backup_root.join("project-snapshots");
    std::fs::create_dir_all(&snapshot_dir)?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d").to_string();
    let archive_name = format!("{}-{}.tar.gz", proj.name, timestamp);
    let archive_path = snapshot_dir.join(&archive_name);

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Archiving: {}", proj.name);
    println!("══════════════════════════════════════════\n");

    println!("  Pruning dependencies before archiving...");
    let targets = if config.clean_deps.targets.is_empty() {
        vec![
            "node_modules".to_string(),
            "target".to_string(),
            ".venv".to_string(),
            "dist".to_string(),
            "build".to_string(),
        ]
    } else {
        config.clean_deps.targets.clone()
    };

    for t in targets {
        let dep_path = proj_path.join(&t);
        if dep_path.exists() {
            let _ = std::fs::remove_dir_all(&dep_path);
            println!("    Removed {}", t);
        }
    }

    let parent_dir = proj_path.parent().unwrap_or(proj_path);
    let folder_name = proj_path.file_name().unwrap_or(proj_path.as_os_str());

    println!("  Compressing...");
    let tar_status = std::process::Command::new("tar")
        .arg("-czf")
        .arg(&archive_path)
        .arg(folder_name)
        .current_dir(parent_dir)
        .status();

    let tar_success = match tar_status {
        Ok(s) => s.success(),
        Err(_) => false,
    };

    if tar_success && archive_path.exists() && archive_path.metadata()?.len() > 0 {
        let size_mb = archive_path.metadata()?.len() as f64 / 1_048_576.0;
        std::fs::remove_dir_all(proj_path)?;
        println!("  Archived: {} ({:.2} MB)", archive_name, size_mb);
        println!("  Location: {}", archive_path.display());
        println!("  Original folder removed.");
        println!("\n  To restore: rtb unarchive {}", archive_name);
        Ok(0)
    } else {
        eprintln!("  Archive creation FAILED — source folder was NOT deleted.");
        if archive_path.exists() {
            let _ = std::fs::remove_file(&archive_path);
        }
        Ok(1)
    }
}

pub fn execute_unarchive(project: Option<String>, cli: &Cli) -> Result<i32> {
    let config = DevConfig::load_from(&cli.config)?;
    let archive_input = match project {
        Some(a) => a,
        None => {
            eprintln!("Usage: rtb unarchive <archive-name.tar.gz>");
            return Ok(1);
        }
    };

    let backup_root = if config.backup_root.is_empty() {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".rtb")
            .join("backups")
    } else {
        PathBuf::from(&config.backup_root)
    };

    let snapshot_dir = backup_root.join("project-snapshots");
    let mut archive_path = snapshot_dir.join(&archive_input);
    let mut display_name = archive_input.clone();

    if !archive_path.exists() {
        if let Ok(entries) = std::fs::read_dir(&snapshot_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.contains(&archive_input) {
                    archive_path = entry.path();
                    display_name = file_name;
                    break;
                }
            }
        }
    }

    if !archive_path.exists() {
        eprintln!("  Archive '{}' not found in {}", archive_input, snapshot_dir.display());
        return Ok(1);
    }

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Unarchiving: {}", display_name);
    println!("══════════════════════════════════════════\n");

    let active_dir = PathBuf::from(&config.project_roots.active);
    if config.project_roots.active.is_empty() {
        eprintln!("  Active project root is not configured in rtb.config.json!");
        return Ok(1);
    }
    std::fs::create_dir_all(&active_dir)?;

    let tar_status = std::process::Command::new("tar")
        .arg("-xzf")
        .arg(&archive_path)
        .current_dir(&active_dir)
        .status();

    if let Ok(s) = tar_status {
        if s.success() {
            println!("  Extracted to: {}", active_dir.display());
            println!("  Run: rtb list --active");
            return Ok(0);
        }
    }

    eprintln!("  Unarchive FAILED.");
    Ok(1)
}
