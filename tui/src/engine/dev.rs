use anyhow::Result;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use crate::config::DevConfig;
use crate::engine::Cli;
use crate::engine::helpers::{dir_size, get_cmd, resolve_project_or_cwd};

pub fn execute_run(project: Option<String>, cli: &Cli) -> Result<i32> {
    let target_path = match resolve_project_or_cwd(project.as_deref(), cli) {
        Ok(p) => p,
        Err(_) => return Ok(1),
    };
    let folder_name = target_path
        .file_name()
        .unwrap_or(target_path.as_os_str())
        .to_string_lossy();

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Run Project ({})", folder_name);
    println!("══════════════════════════════════════════\n");

    let pkg_path = target_path.join("package.json");
    if pkg_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if json["scripts"]["dev"].is_string() {
                    println!("Running 'npm run dev' in {}...", target_path.display());
                    let status = std::process::Command::new(get_cmd("npm"))
                        .args(&["run", "dev"])
                        .current_dir(&target_path)
                        .status();
                    return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
                } else if json["scripts"]["start"].is_string() {
                    println!("Running 'npm start' in {}...", target_path.display());
                    let status = std::process::Command::new(get_cmd("npm"))
                        .arg("start")
                        .current_dir(&target_path)
                        .status();
                    return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
                }
            }
        }
    }

    if target_path.join("Cargo.toml").exists() {
        println!("Running 'cargo run' in {}...", target_path.display());
        let status = std::process::Command::new(get_cmd("cargo"))
            .arg("run")
            .current_dir(&target_path)
            .status();
        return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
    }

    if target_path.join("go.mod").exists() {
        println!("Running 'go run .' in {}...", target_path.display());
        let status = std::process::Command::new(get_cmd("go"))
            .args(&["run", "."])
            .current_dir(&target_path)
            .status();
        return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
    }

    if target_path.join("main.py").exists() {
        println!("Running 'python main.py' in {}...", target_path.display());
        let status = std::process::Command::new(get_cmd("python"))
            .args(&["main.py"])
            .current_dir(&target_path)
            .status();
        return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
    }

    eprintln!(
        "No runnable script or main entrypoint detected in {}.",
        target_path.display()
    );
    Ok(1)
}

pub fn execute_build(project: Option<String>, cli: &Cli) -> Result<i32> {
    let target_path = match resolve_project_or_cwd(project.as_deref(), cli) {
        Ok(p) => p,
        Err(_) => return Ok(1),
    };
    let folder_name = target_path
        .file_name()
        .unwrap_or(target_path.as_os_str())
        .to_string_lossy();

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Build Project ({})", folder_name);
    println!("══════════════════════════════════════════\n");

    let pkg_path = target_path.join("package.json");
    if pkg_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if json["scripts"]["build"].is_string() {
                    println!("Running 'npm run build' in {}...", target_path.display());
                    let status = std::process::Command::new(get_cmd("npm"))
                        .args(&["run", "build"])
                        .current_dir(&target_path)
                        .status();
                    return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
                }
            }
        }
    }

    if target_path.join("Cargo.toml").exists() {
        println!("Running 'cargo build --release' in {}...", target_path.display());
        let status = std::process::Command::new(get_cmd("cargo"))
            .args(&["build", "--release"])
            .current_dir(&target_path)
            .status();
        return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
    }

    if target_path.join("go.mod").exists() {
        println!("Running 'go build' in {}...", target_path.display());
        let status = std::process::Command::new(get_cmd("go"))
            .arg("build")
            .current_dir(&target_path)
            .status();
        return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
    }

    eprintln!("No build configuration detected in {}.", target_path.display());
    Ok(1)
}

pub fn execute_test(project: Option<String>, cli: &Cli) -> Result<i32> {
    let target_path = match resolve_project_or_cwd(project.as_deref(), cli) {
        Ok(p) => p,
        Err(_) => return Ok(1),
    };
    let folder_name = target_path
        .file_name()
        .unwrap_or(target_path.as_os_str())
        .to_string_lossy();

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Test Project ({})", folder_name);
    println!("══════════════════════════════════════════\n");

    let pkg_path = target_path.join("package.json");
    if pkg_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if json["scripts"]["test"].is_string() {
                    println!("Running 'npm test' in {}...", target_path.display());
                    let status = std::process::Command::new(get_cmd("npm"))
                        .arg("test")
                        .current_dir(&target_path)
                        .status();
                    return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
                }
            }
        }
    }

    if target_path.join("Cargo.toml").exists() {
        println!("Running 'cargo test' in {}...", target_path.display());
        let status = std::process::Command::new(get_cmd("cargo"))
            .arg("test")
            .current_dir(&target_path)
            .status();
        return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
    }

    if target_path.join("pytest.ini").exists() || target_path.join("pyproject.toml").exists() {
        println!("Running 'pytest' in {}...", target_path.display());
        let status = std::process::Command::new(get_cmd("pytest"))
            .current_dir(&target_path)
            .status();
        return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
    }

    if target_path.join("cli/tests").exists() {
        println!("Running 'Invoke-Pester' in {}/cli/tests...", target_path.display());
        let status = std::process::Command::new(get_cmd("pwsh"))
            .args(&["-Command", "Invoke-Pester cli/tests/"])
            .current_dir(&target_path)
            .status();
        return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
    }

    if target_path.join("tests").exists() {
        println!("Running 'Invoke-Pester' in {}...", target_path.display());
        let status = std::process::Command::new(get_cmd("pwsh"))
            .args(&["-Command", "Invoke-Pester tests/"])
            .current_dir(&target_path)
            .status();
        return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
    }

    eprintln!("No test configuration detected in {}.", target_path.display());
    Ok(1)
}

pub fn execute_clean(
    dry_run: bool,
    commit: bool,
    days: Option<u64>,
    cli: &Cli,
) -> Result<i32> {
    let config = DevConfig::load_from(&cli.config)?;
    let resolved_days = days.unwrap_or(config.stale_threshold_days);
    let is_dry_run = dry_run || !commit;

    let search_paths = vec![
        PathBuf::from(&config.project_roots.active),
        PathBuf::from(&config.project_roots.paused),
        PathBuf::from(&config.project_roots.vibe),
        PathBuf::from(&config.project_roots.sandbox),
    ];

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

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Dependency Pruning ({}d threshold)", resolved_days);
    println!("══════════════════════════════════════════\n");

    if is_dry_run {
        println!("  [DRY RUN MODE] No files will be deleted. Use '--commit' to perform deletion.\n");
    }

    let cutoff_time = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(resolved_days * 86400))
        .unwrap_or(std::time::SystemTime::now());

    struct FlaggedItem {
        path: PathBuf,
    }

    let mut flagged: Vec<FlaggedItem> = Vec::new();
    let mut total_bytes: u64 = 0;

    for root in search_paths {
        if !root.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let proj_dir = entry.path();
                if !proj_dir.is_dir() { continue; }
                for target_name in &targets {
                    let dep_dir = proj_dir.join(target_name);
                    if dep_dir.is_dir() {
                        let modified = std::fs::metadata(&dep_dir)
                            .and_then(|m| m.modified())
                            .unwrap_or(std::time::SystemTime::now());
                        if modified < cutoff_time {
                            let size = dir_size(&dep_dir);
                            total_bytes += size;
                            let mb = size as f64 / 1_048_576.0;
                            println!("  {} ({:.1} MB)", dep_dir.display(), mb);
                            flagged.push(FlaggedItem {
                                path: dep_dir,
                            });
                        }
                    }
                }
            }
        }
    }

    let gb = total_bytes as f64 / 1_073_741_824.0;
    let suffix = if is_dry_run { "(dry run)" } else { "flagged" };
    println!("\n  Flagged: {} folders | Space: {:.2} GB {}", flagged.len(), gb, suffix);

    if !is_dry_run && !flagged.is_empty() {
        for item in &flagged {
            if std::fs::remove_dir_all(&item.path).is_ok() {
                println!("    -> DELETED: {}", item.path.display());
            }
        }
        println!("\n  Clean complete. Space recovered: {:.2} GB", gb);
    }

    Ok(0)
}

pub fn execute_open(project: Option<String>, cli: &Cli) -> Result<i32> {
    let target_path = match resolve_project_or_cwd(project.as_deref(), cli) {
        Ok(p) => p,
        Err(_) => return Ok(1),
    };

    let folder_name = target_path
        .file_name()
        .unwrap_or(target_path.as_os_str())
        .to_string_lossy();

    println!("Opening project '{}' in file explorer...", folder_name);
    println!("  Path: {}", target_path.display());

    let is_non_interactive = std::env::var("RTB_NON_INTERACTIVE").is_ok()
        || std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok();

    if is_non_interactive {
        return Ok(0);
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer.exe")
            .arg(&target_path)
            .status();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(&target_path)
            .status();
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&target_path)
            .status();
    }

    Ok(0)
}

pub fn execute_commit(message: Option<String>, _cli: &Cli) -> Result<i32> {
    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Git Commit & Push");
    println!("══════════════════════════════════════════\n");

    let cwd = std::env::current_dir()?;
    if !cwd.join(".git").exists() {
        eprintln!("  Error: Current directory is not a Git repository.");
        return Ok(1);
    }

    let status = match crate::data::scanner::run_git(&cwd, &["status", "--short"]) {
        Some(s) if !s.trim().is_empty() => s,
        _ => {
            println!("  Working tree is clean. Nothing to commit.");
            return Ok(0);
        }
    };

    println!("  Changed Files:");
    for line in status.lines() {
        if !line.trim().is_empty() {
            println!("    {}", line.trim());
        }
    }
    println!();

    let commit_msg = if let Some(m) = message {
        m
    } else {
        let is_non_interactive = std::env::var("RTB_NON_INTERACTIVE").is_ok()
            || std::env::var("CI").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || !std::io::stdin().is_terminal();

        if is_non_interactive {
            "update: sync workspace changes".to_string()
        } else {
            print!("  Enter Commit Message: ");
            let _ = std::io::stdout().flush();
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
            let trimmed = input.trim();
            if trimmed.is_empty() {
                "update: sync workspace changes".to_string()
            } else {
                trimmed.to_string()
            }
        }
    };

    println!("  Staging files (git add .)...");
    let _ = crate::data::scanner::run_git(&cwd, &["add", "."]);

    println!("  Running git commit...");
    let commit_res = std::process::Command::new(get_cmd("git"))
        .args(&["commit", "-m", &commit_msg])
        .current_dir(&cwd)
        .status();

    match commit_res {
        Ok(s) if s.success() => {
            println!("  Successfully committed with message: '{}'", commit_msg);
            Ok(0)
        }
        _ => {
            eprintln!("  Git commit failed.");
            Ok(1)
        }
    }
}
