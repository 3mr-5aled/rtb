use anyhow::Result;
use std::path::PathBuf;
use crate::config::DevConfig;
use crate::data::project::ProjectStatus;
use crate::data::scanner::scan_all_projects;
use crate::engine::Cli;
use crate::engine::helpers::resolve_project_or_cwd;

pub fn execute_info(project_name: String, cmd_json: bool, cli: &Cli) -> Result<i32> {
    let config = DevConfig::load_from(&cli.config)?;
    let projects = scan_all_projects(&config);

    let proj = match projects.iter().find(|p| p.name.eq_ignore_ascii_case(&project_name)) {
        Some(p) => p,
        None => {
            eprintln!("Project '{}' not found.", project_name);
            return Ok(1);
        }
    };

    let is_json = cli.json || cmd_json;
    if is_json {
        println!("{}", serde_json::to_string_pretty(proj)?);
        return Ok(0);
    }

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Project Info: {}", proj.name);
    println!("══════════════════════════════════════════\n");
    println!("  Name:            {}", proj.name);
    println!("  Status:          {}", proj.status.label());
    println!("  Path:            {}", proj.path.display());
    println!("  Stack:           {}", proj.stack.join(", "));
    println!("  Monorepo:        {}", if proj.is_monorepo { "Yes" } else { "No" });
    println!("  CI/CD:           {}", proj.ci_cd.as_deref().unwrap_or("None"));
    println!("  Runtime Version: {}", proj.runtime_version.as_deref().unwrap_or("N/A"));

    if let Some(ref git) = proj.git {
        println!();
        println!("  Git Info:");
        println!("    Branch:        {}", git.branch);
        println!("    Uncommitted:   {}", git.uncommitted);
        println!("    Unpushed:      {}", git.unpushed);
        println!("    Has Remote:    {}", git.has_remote);
        if let Some(ref msg) = git.last_commit_msg {
            let rel = git.last_commit_relative.as_deref().unwrap_or("");
            println!("    Last Commit:   {} ({})", msg, rel);
        }
    }

    if let Some(ref readme) = proj.readme_preview {
        println!();
        println!("  README Preview:");
        for line in readme.lines() {
            println!("    {}", line);
        }
    }
    println!();
    Ok(0)
}

pub fn execute_list(
    active: bool,
    paused: bool,
    deployed: bool,
    vibe: bool,
    _all: bool,
    cmd_json: bool,
    cli: &Cli,
) -> Result<i32> {
    let config = DevConfig::load_from(&cli.config)?;
    let mut projects = scan_all_projects(&config);

    if active || paused || deployed || vibe {
        projects.retain(|p| {
            if active && p.status == ProjectStatus::Active { return true; }
            if paused && p.status == ProjectStatus::Paused { return true; }
            if deployed && (p.status == ProjectStatus::Production || p.status == ProjectStatus::Staging) { return true; }
            if vibe && p.status == ProjectStatus::Vibe { return true; }
            false
        });
    }

    let is_json = cli.json || cmd_json;
    if is_json {
        println!("{}", serde_json::to_string_pretty(&projects)?);
        return Ok(0);
    }

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Project List");
    println!("══════════════════════════════════════════\n");

    let categories = [
        ("Active", ProjectStatus::Active, "📁"),
        ("Paused", ProjectStatus::Paused, "⏸️"),
        ("Production", ProjectStatus::Production, "🚀"),
        ("Staging", ProjectStatus::Staging, "🚀"),
        ("Vibe", ProjectStatus::Vibe, "✨"),
        ("Sandbox", ProjectStatus::Sandbox, "🔬"),
        ("Planning", ProjectStatus::Planning, "📝"),
        ("Testing", ProjectStatus::Testing, "🧪"),
        ("Abandoned", ProjectStatus::Abandoned, "❌"),
    ];

    let mut total = 0;
    for (cat_name, status, default_emoji) in &categories {
        let cat_projs: Vec<&crate::data::project::Project> =
            projects.iter().filter(|p| &p.status == status).collect();
        if cat_projs.is_empty() {
            continue;
        }

        println!("  {} {} ({})", default_emoji, cat_name, cat_projs.len());
        for p in &cat_projs {
            total += 1;
            let last_mod = p.last_modified_str();
            println!("    {}  ({})", p.name, last_mod);
        }
        println!();
    }

    println!("  Total: {} projects", total);
    Ok(0)
}

pub fn execute_status(cmd_json: bool, cli: &Cli) -> Result<i32> {
    let is_json = cli.json || cmd_json;
    let cwd = std::env::current_dir()?;
    let config = DevConfig::load_from(&cli.config).ok();

    let mut project_name: Option<String> = None;
    let mut project_status: Option<String> = None;
    let mut project_root_path: Option<PathBuf> = None;

    if let Some(ref cfg) = config {
        let roots = vec![
            ("Active", &cfg.project_roots.active),
            ("Paused", &cfg.project_roots.paused),
            ("Production", &cfg.project_roots.production),
            ("Staging", &cfg.project_roots.staging),
            ("Vibe", &cfg.project_roots.vibe),
            ("Sandbox", &cfg.project_roots.sandbox),
            ("Planning", &cfg.project_roots.planning),
            ("Testing", &cfg.project_roots.testing),
            ("Abandoned", &cfg.project_roots.abandoned),
        ];

        for (status_label, root_path_str) in roots {
            if root_path_str.is_empty() { continue; }
            let root_path = PathBuf::from(root_path_str);
            if let Ok(rel) = cwd.strip_prefix(&root_path) {
                if let Some(first_comp) = rel.components().next() {
                    let name = first_comp.as_os_str().to_string_lossy().to_string();
                    if !name.is_empty() {
                        project_name = Some(name.clone());
                        project_status = Some(status_label.to_string());
                        project_root_path = Some(root_path.join(name));
                        break;
                    }
                }
            }
        }
    }

    let mut branch = String::new();
    let mut uncommitted: u32 = 0;
    let mut git_root: Option<PathBuf> = None;

    let mut check = Some(cwd.as_path());
    while let Some(path) = check {
        if path.join(".git").exists() {
            git_root = Some(path.to_path_buf());
            if let Some(b) = crate::data::scanner::run_git(path, &["branch", "--show-current"]) {
                let b_trim = b.trim();
                if !b_trim.is_empty() {
                    branch = b_trim.to_string();
                } else if let Some(head) = crate::data::scanner::run_git(path, &["rev-parse", "--short", "HEAD"]) {
                    if !head.trim().is_empty() {
                        branch = format!("HEAD@{}", head.trim());
                    }
                }
            }
            if let Some(porcelain) = crate::data::scanner::run_git(path, &["status", "--porcelain"]) {
                uncommitted = porcelain.lines().filter(|l| !l.trim().is_empty()).count() as u32;
            }
            break;
        }
        check = path.parent();
    }

    let mut search_paths = vec![cwd.clone()];
    if let Some(ref prp) = project_root_path {
        if prp.exists() && !search_paths.contains(prp) {
            search_paths.push(prp.clone());
        }
    }
    if let Some(ref gr) = git_root {
        if gr.exists() && !search_paths.contains(gr) {
            search_paths.push(gr.clone());
        }
    }

    let mut stack: Vec<String> = Vec::new();
    for p in search_paths {
        if !p.exists() { continue; }
        if p.join("package.json").exists() && !stack.contains(&"Node.js".to_string()) {
            stack.push("Node.js".into());
        }
        if (p.join("Cargo.toml").exists() || p.join("tui/Cargo.toml").exists()) && !stack.contains(&"Rust".to_string()) {
            stack.push("Rust".into());
        }
        if p.join("go.mod").exists() && !stack.contains(&"Go".to_string()) {
            stack.push("Go".into());
        }
        if (p.join("pyproject.toml").exists() || p.join("requirements.txt").exists() || p.join("uv.lock").exists() || p.join("poetry.lock").exists()) && !stack.contains(&"Python".to_string()) {
            stack.push("Python".into());
        }
        if (p.join("rtb.psm1").exists() || p.join("rtb.psd1").exists() || p.join("cli/rtb.psm1").exists() || p.join("dev.psm1").exists()) && !stack.contains(&"PowerShell".to_string()) {
            stack.push("PowerShell".into());
        }
        let has_dotnet = std::fs::read_dir(&p).ok().map(|entries| {
            entries.flatten().any(|e| {
                let n = e.file_name().to_string_lossy().to_lowercase();
                n.ends_with(".csproj") || n.ends_with(".sln")
            })
        }).unwrap_or(false);
        if has_dotnet && !stack.contains(&".NET".to_string()) {
            stack.push(".NET".into());
        }
    }

    let display_name = project_name
        .or_else(|| cwd.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| cwd.to_string_lossy().to_string());

    if is_json {
        #[derive(serde::Serialize)]
        struct StatusJson {
            project: String,
            status: Option<String>,
            branch: String,
            uncommitted: u32,
            stack: Vec<String>,
            cwd: String,
        }

        let sj = StatusJson {
            project: display_name,
            status: project_status,
            branch,
            uncommitted,
            stack,
            cwd: cwd.to_string_lossy().to_string(),
        };
        println!("{}", serde_json::to_string_pretty(&sj)?);
        return Ok(0);
    }

    let status_part = match project_status {
        Some(ref s) => format!(" ({})", s),
        None => "".to_string(),
    };
    let git_part = if !branch.is_empty() {
        let un_str = if uncommitted > 0 { format!(" ±{}", uncommitted) } else { "".to_string() };
        format!(" [{}{}]", branch, un_str)
    } else {
        "".to_string()
    };
    let stack_part = if !stack.is_empty() {
        format!(" {}", stack.join(","))
    } else {
        "".to_string()
    };

    println!("rtb » {}{}{}{}", display_name, status_part, git_part, stack_part);
    Ok(0)
}

pub fn execute_deps(project: Option<String>, cmd_json: bool, cli: &Cli) -> Result<i32> {
    let target_path = match resolve_project_or_cwd(project.as_deref(), cli) {
        Ok(p) => p,
        Err(_) => return Ok(1),
    };
    let is_json = cli.json || cmd_json;

    #[derive(serde::Serialize)]
    struct DepInfo {
        package: String,
        spec: String,
        dep_type: String,
        status: String,
    }

    let mut deps: Vec<DepInfo> = Vec::new();

    // 1. package.json
    let pkg_path = target_path.join("package.json");
    if pkg_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = json["dependencies"].as_object() {
                    for (k, v) in obj {
                        deps.push(DepInfo {
                            package: k.clone(),
                            spec: v.as_str().unwrap_or("").to_string(),
                            dep_type: "npm/pnpm/yarn".to_string(),
                            status: "Declared".to_string(),
                        });
                    }
                }
                if let Some(obj) = json["devDependencies"].as_object() {
                    for (k, v) in obj {
                        deps.push(DepInfo {
                            package: k.clone(),
                            spec: v.as_str().unwrap_or("").to_string(),
                            dep_type: "npm/pnpm (dev)".to_string(),
                            status: "Declared".to_string(),
                        });
                    }
                }
            }
        }
    }

    // 2. Cargo.toml
    let cargo_path = target_path.join("Cargo.toml");
    if cargo_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            let mut in_deps = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("[dependencies]") || trimmed.starts_with("[dev-dependencies]") {
                    in_deps = true;
                    continue;
                }
                if trimmed.starts_with('[') {
                    in_deps = false;
                    continue;
                }
                if in_deps && !trimmed.is_empty() && !trimmed.starts_with('#') {
                    let parts: Vec<&str> = trimmed.splitn(2, '=').map(|s| s.trim()).collect();
                    if parts.len() == 2 {
                        let name = parts[0];
                        let spec = parts[1].trim_matches('"');
                        deps.push(DepInfo {
                            package: name.to_string(),
                            spec: spec.to_string(),
                            dep_type: "Cargo (Rust)".to_string(),
                            status: "Declared".to_string(),
                        });
                    }
                }
            }
        }
    }

    // 3. pyproject.toml / requirements.txt
    let pyproject_path = target_path.join("pyproject.toml");
    if pyproject_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&pyproject_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if (trimmed.starts_with('"') || trimmed.starts_with('\''))
                    && (trimmed.contains("==") || trimmed.contains(">=") || trimmed.contains("~="))
                {
                    let clean = trimmed.trim_matches(|c| c == '"' || c == '\'' || c == ',');
                    deps.push(DepInfo {
                        package: clean.to_string(),
                        spec: "latest".to_string(),
                        dep_type: "Python (pyproject)".to_string(),
                        status: "Declared".to_string(),
                    });
                }
            }
        }
    } else {
        let req_path = target_path.join("requirements.txt");
        if req_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&req_path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        deps.push(DepInfo {
                            package: trimmed.to_string(),
                            spec: "latest".to_string(),
                            dep_type: "Python (requirements)".to_string(),
                            status: "Declared".to_string(),
                        });
                    }
                }
            }
        }
    }

    if is_json {
        println!("{}", serde_json::to_string_pretty(&deps)?);
        return Ok(0);
    }

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Dependency Inspector ({})", target_path.display());
    println!("══════════════════════════════════════════\n");

    if deps.is_empty() {
        println!("  No dependencies found in {}", target_path.display());
        return Ok(0);
    }

    println!("  Found {} declared dependencies:\n", deps.len());
    println!("  {:<30} {:<20} {:<20}", "PACKAGE", "SPEC", "TYPE");
    println!("  {}", "─".repeat(72));
    for d in &deps {
        println!("  {:<30} {:<20} {:<20}", d.package, d.spec, d.dep_type);
    }
    println!();
    Ok(0)
}

pub fn execute_workspace(project: Option<String>, cmd_json: bool, cli: &Cli) -> Result<i32> {
    let target_path = match resolve_project_or_cwd(project.as_deref(), cli) {
        Ok(p) => p,
        Err(_) => return Ok(1),
    };
    let is_json = cli.json || cmd_json;

    #[derive(serde::Serialize)]
    struct WorkspacePackageInfo {
        package_pattern: String,
        package_type: String,
    }

    #[derive(serde::Serialize)]
    struct WorkspaceInfo {
        project_path: String,
        workspace_type: String,
        is_monorepo: bool,
        packages: Vec<WorkspacePackageInfo>,
    }

    let mut workspace_packages: Vec<WorkspacePackageInfo> = Vec::new();
    let mut workspace_type = "Single Package / Standard Repository".to_string();

    // 1. pnpm-workspace.yaml
    let pnpm_ws = target_path.join("pnpm-workspace.yaml");
    if pnpm_ws.is_file() {
        workspace_type = "pnpm Workspaces".to_string();
        if let Ok(content) = std::fs::read_to_string(&pnpm_ws) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("- ") || trimmed.starts_with("'- ") {
                    let pat = trimmed.trim_start_matches('-').trim().trim_matches(|c| c == '\'' || c == '"');
                    if !pat.is_empty() && pat != "packages:" {
                        workspace_packages.push(WorkspacePackageInfo {
                            package_pattern: pat.to_string(),
                            package_type: "pnpm".to_string(),
                        });
                    }
                }
            }
        }
    }

    // 2. package.json workspaces
    let pkg_path = target_path.join("package.json");
    if pkg_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(ws_arr) = json["workspaces"].as_array() {
                    workspace_type = "npm/yarn Workspaces".to_string();
                    for item in ws_arr {
                        if let Some(s) = item.as_str() {
                            workspace_packages.push(WorkspacePackageInfo {
                                package_pattern: s.to_string(),
                                package_type: "npm/yarn".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    // 3. Cargo.toml workspace
    let cargo_path = target_path.join("Cargo.toml");
    if cargo_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            let mut in_ws = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("[workspace]") {
                    in_ws = true;
                    workspace_type = "Cargo Workspace (Rust)".to_string();
                    continue;
                }
                if trimmed.starts_with('[') {
                    in_ws = false;
                    continue;
                }
                if in_ws && (trimmed.starts_with('"') || trimmed.starts_with('\'')) {
                    let pat = trimmed.trim_matches(|c| c == '"' || c == '\'' || c == ',');
                    workspace_packages.push(WorkspacePackageInfo {
                        package_pattern: pat.to_string(),
                        package_type: "Cargo".to_string(),
                    });
                }
            }
        }
    }

    let is_monorepo = !workspace_packages.is_empty();
    let info = WorkspaceInfo {
        project_path: target_path.to_string_lossy().to_string(),
        workspace_type: workspace_type.clone(),
        is_monorepo,
        packages: workspace_packages,
    };

    if is_json {
        println!("{}", serde_json::to_string_pretty(&info)?);
        return Ok(0);
    }

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Monorepo Workspace Inspector ({})", target_path.display());
    println!("══════════════════════════════════════════\n");
    println!("  Monorepo Type: {}", workspace_type);

    if !info.packages.is_empty() {
        println!("  Declared Workspace Patterns:");
        for p in &info.packages {
            println!("    - {} ({})", p.package_pattern, p.package_type);
        }
    } else {
        println!("  No active monorepo workspace configurations detected.");
    }
    println!();
    Ok(0)
}

pub fn execute_health(cli: &Cli) -> Result<i32> {
    let config = DevConfig::load_from(&cli.config).ok();
    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Git Repository Health");
    println!("══════════════════════════════════════════\n");

    let scan_roots = if let Some(ref cfg) = config {
        if !cfg.git_health.scan_roots.is_empty() {
            cfg.git_health.scan_roots.clone()
        } else {
            vec![
                cfg.project_roots.active.clone(),
                cfg.project_roots.paused.clone(),
            ]
        }
    } else {
        vec![".".to_string()]
    };

    let stale_threshold = config.as_ref().map(|c| c.stale_threshold_days).unwrap_or(90);

    let mut scanned = 0;
    let mut issues = 0;

    fn walk_git_repos(dir: &std::path::Path, repos: &mut Vec<PathBuf>) {
        if !dir.is_dir() { return; }
        if dir.join(".git").exists() {
            repos.push(dir.to_path_buf());
            return;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name == "node_modules" || file_name == ".venv" || file_name == "target" || file_name == ".git" {
                        continue;
                    }
                    walk_git_repos(&p, repos);
                }
            }
        }
    }

    let mut repos = Vec::new();
    for root_str in scan_roots {
        if root_str.is_empty() { continue; }
        let root_path = PathBuf::from(root_str);
        if root_path.exists() {
            walk_git_repos(&root_path, &mut repos);
        }
    }

    let now = chrono::Local::now();

    for repo in repos {
        scanned += 1;
        let mut repo_issues = Vec::new();

        let porcelain = crate::data::scanner::run_git(&repo, &["status", "--porcelain"]);
        if let Some(ref p) = porcelain {
            let count = p.lines().filter(|l| !l.trim().is_empty()).count();
            if count > 0 {
                repo_issues.push(format!("UNCOMMITTED ({} files)", count));
            }
        }

        let unpushed = crate::data::scanner::run_git(&repo, &["log", "--branches", "--not", "--remotes", "--oneline"]);
        if let Some(ref u) = unpushed {
            let count = u.lines().filter(|l| !l.trim().is_empty()).count();
            if count > 0 {
                repo_issues.push(format!("UNPUSHED ({})", count));
            }
        }

        let last_rel = crate::data::scanner::run_git(&repo, &["log", "-1", "--format=%cr"])
            .unwrap_or_default()
            .trim()
            .to_string();

        let last_date_str = crate::data::scanner::run_git(&repo, &["log", "-1", "--format=%ai"]);
        if let Some(d_str) = last_date_str {
            let trimmed = d_str.trim();
            if let Ok(parsed) = chrono::DateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S %z") {
                let days = (now.signed_duration_since(parsed.with_timezone(&chrono::Local))).num_days();
                if days > stale_threshold as i64 {
                    repo_issues.push(format!("STALE ({} days)", days));
                }
            }
        }

        let remote = crate::data::scanner::run_git(&repo, &["remote"]);
        if remote.as_deref().unwrap_or("").trim().is_empty() {
            repo_issues.push("NO REMOTE".to_string());
        }

        if !repo_issues.is_empty() {
            issues += 1;
            println!("\n  {}", repo.display());
            if !last_rel.is_empty() {
                println!("    Last commit: {}", last_rel);
            }
            for issue in repo_issues {
                println!("    ⚠ {}", issue);
            }
        }
    }

    println!("\n  Scanned: {} repos | Issues: {}", scanned, issues);
    Ok(0)
}

pub fn execute_index(cli: &Cli) -> Result<i32> {
    let config = DevConfig::load_from(&cli.config)?;
    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Project Index Generator");
    println!("══════════════════════════════════════════\n");

    let projects = scan_all_projects(&config);
    let date_str = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();

    let mut output = format!(
        "# Project Index\n\n> Generated {}\n\n| Project | Status | Stack | Last Modified |\n|:---|:---|:---|:---|\n",
        date_str
    );

    let total = projects.len();
    for p in &projects {
        let stack_str = if p.stack.is_empty() {
            "-".to_string()
        } else {
            p.stack.join(", ")
        };
        let last_mod = p.last_modified_str();
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            p.name,
            p.status.label(),
            stack_str,
            last_mod
        ));
    }

    output.push_str(&format!("\n---\n*Total: {} projects*\n", total));

    let active_path = PathBuf::from(&config.project_roots.active);
    let out_path = if active_path.exists() {
        active_path.parent().unwrap_or(&active_path).join("PROJECT-INDEX.md")
    } else {
        PathBuf::from("PROJECT-INDEX.md")
    };

    std::fs::write(&out_path, output)?;
    println!("  Generated index: {} projects → {}", total, out_path.display());
    Ok(0)
}
