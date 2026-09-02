use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::io::{IsTerminal, Write};
use std::path::PathBuf;

use crate::config::DevConfig;
use crate::data::project::ProjectStatus;
use crate::data::scanner::scan_all_projects;

#[derive(Parser, Debug)]
#[command(
    name = "rtb",
    about = "Unified workspace & project manager",
    subcommand_required = false,
    arg_required_else_help = false,
    allow_external_subcommands = true,
    disable_version_flag = true,
    disable_help_flag = false
)]
pub struct Cli {
    #[arg(long, global = true, env = "RTB_CONFIG")]
    pub config: Option<PathBuf>,

    #[arg(long, global = true, env = "RTB_JSON")]
    pub json: bool,

    #[arg(long, global = true, env = "NO_COLOR")]
    pub no_color: bool,

    #[arg(long, short, global = true)]
    pub quiet: bool,

    #[arg(long, short = 'V', global = true)]
    pub version: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    // --- Inspect ---
    #[command(next_help_heading = "Inspect", visible_alias = "ls")]
    List {
        #[arg(long)]
        active: bool,
        #[arg(long)]
        paused: bool,
        #[arg(long)]
        deployed: bool,
        #[arg(long)]
        vibe: bool,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        json: bool,
    },
    #[command(visible_alias = "st")]
    Status {
        #[arg(long, short)]
        json: bool,
    },
    Info {
        project: String,
        #[arg(long, short)]
        json: bool,
    },
    Health,
    Index,
    Deps {
        project: Option<String>,
        #[arg(long, short)]
        json: bool,
    },
    Workspace {
        project: Option<String>,
        #[arg(long, short)]
        json: bool,
    },

    // --- Lifecycle ---
    #[command(next_help_heading = "Lifecycle")]
    New {
        name: String,
        #[arg(long)]
        root: Option<String>,
        #[arg(long)]
        stack: Option<String>,
    },
    Pause {
        project: Option<String>,
        #[arg(long, short = 'p', alias = "prune")]
        prune_deps: bool,
        #[arg(long, short = 'f')]
        force: bool,
    },
    Resume {
        project: Option<String>,
        #[arg(long)]
        install: bool,
    },
    Deploy {
        project: Option<String>,
        #[arg(long, short = 't')]
        to: Option<String>,
        #[arg(long)]
        prod: bool,
        #[arg(long)]
        staging: bool,
    },
    Archive {
        project: Option<String>,
        #[arg(long, short = 'f')]
        force: bool,
    },
    Unarchive {
        project: Option<String>,
    },

    // --- Dev ---
    #[command(next_help_heading = "Dev")]
    Run {
        project: Option<String>,
    },
    Build {
        project: Option<String>,
    },
    Test {
        project: Option<String>,
    },
    Clean {
        #[arg(long)]
        dry_run: bool,
        #[arg(long, short = 'c')]
        commit: bool,
        #[arg(long, short = 'd')]
        days: Option<u64>,
    },
    Commit {
        #[arg(value_name = "MESSAGE")]
        positional_message: Option<String>,

        #[arg(short = 'm', long = "message")]
        message: Option<String>,
    },
    Open {
        project: Option<String>,
    },

    // --- System ---
    #[command(next_help_heading = "System")]
    Doctor,
    Init {
        #[arg(long, short = 'f')]
        force: bool,
    },
    Config,
    Upgrade {
        #[arg(long)]
        check: bool,
    },
    Uninstall {
        #[arg(long, short = 'y')]
        yes: bool,
    },
    Maintenance(MaintenanceArgs),

    // --- Agent ---
    #[command(next_help_heading = "Agent")]
    Agent {
        name: Option<String>,
        #[arg(long, short)]
        project: Option<String>,
        #[arg(long, short)]
        list: bool,
        #[arg(long, short)]
        clean: bool,
    },

    // --- TUI ---
    #[command(next_help_heading = "TUI")]
    Ui {
        #[arg(long)]
        tab: Option<String>,
    },

    // --- Shell ---
    #[command(next_help_heading = "Shell")]
    ShellInit {
        shell: ShellChoice,
    },
    Completions {
        shell: ShellChoice,
    },

    // --- Hidden ---
    #[command(name = "_goto-resolve", hide = true)]
    GotoResolve {
        query: Option<String>,
    },

    // Catch-all for agent shorthands (rtb claude, rtb agy, ...)
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Parser, Debug, Clone)]
pub struct MaintenanceArgs {
    #[command(subcommand)]
    pub command: Option<MaintenanceCommands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum MaintenanceCommands {
    Backup,
    Env,
    Guard,
    Run { script: String },
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellChoice {
    Bash,
    Zsh,
    Fish,
    Powershell,
    Pwsh,
}

pub struct RtbEngine;

impl RtbEngine {
    pub fn version_string() -> String {
        let target = option_env!("TARGET").unwrap_or("unknown-target");
        format!("rtb {} ({})", env!("CARGO_PKG_VERSION"), target)
    }

    pub fn dispatch() -> Result<i32> {
        Self::dispatch_args(std::env::args_os())
    }

    pub fn dispatch_args<I, T>(args: I) -> Result<i32>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let normalized_args: Vec<std::ffi::OsString> = args
            .into_iter()
            .map(|arg| {
                let os_str: std::ffi::OsString = arg.into();
                if let Some(s) = os_str.to_str() {
                    if s.eq_ignore_ascii_case("-json") || s == "-J" {
                        return std::ffi::OsString::from("--json");
                    }
                    if s.eq_ignore_ascii_case("-active") { return std::ffi::OsString::from("--active"); }
                    if s.eq_ignore_ascii_case("-paused") { return std::ffi::OsString::from("--paused"); }
                    if s.eq_ignore_ascii_case("-deployed") { return std::ffi::OsString::from("--deployed"); }
                    if s.eq_ignore_ascii_case("-vibe") { return std::ffi::OsString::from("--vibe"); }
                    if s.eq_ignore_ascii_case("-all") { return std::ffi::OsString::from("--all"); }
                }
                os_str
            })
            .collect();

        let matches = match Cli::try_parse_from(normalized_args) {
            Ok(m) => m,
            Err(e) => {
                e.print()?;
                let exit_code = if e.use_stderr() { 1 } else { 0 };
                return Ok(exit_code);
            }
        };

        if matches.version {
            println!("{}", Self::version_string());
            return Ok(0);
        }

        let cmd = match &matches.command {
            Some(c) => c.clone(),
            None => {
                // Default to TUI if no subcommand provided
                Commands::Ui { tab: None }
            }
        };

        // Check Config Gate
        if Self::needs_config(&cmd) && !Self::config_exists(&matches.config) {
            let is_non_interactive = std::env::var("RTB_NON_INTERACTIVE").is_ok()
                || std::env::var("CI").is_ok()
                || std::env::var("GITHUB_ACTIONS").is_ok()
                || !std::io::stdin().is_terminal();

            if matches.json || matches.quiet || is_non_interactive {
                eprintln!("rtb: not configured. Run `rtb init` to set up your workspace.");
                return Ok(1);
            } else {
                let config_path = Self::resolve_config_path(&matches.config);
                eprintln!("⚠  rtb: not configured yet.");
                eprintln!("   Run 'rtb init' to set up your workspace,");
                eprintln!("   or edit {} directly.", config_path.display());
                eprintln!();
                eprint!("Would you like to configure now? (Y/n) ");
                std::io::stderr().flush()?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let trimmed = input.trim().to_lowercase();
                if trimmed.is_empty() || trimmed == "y" || trimmed == "yes" {
                    let init_res = Self::execute_init(false, &matches)?;
                    if init_res != 0 {
                        return Ok(init_res);
                    }
                    eprintln!("Run `rtb {}` again to continue.", Self::command_name(&cmd));
                    return Ok(0);
                } else {
                    return Ok(1);
                }
            }
        }

        // Execute command
        Self::execute_command(cmd, &matches)
    }

    pub fn needs_config(cmd: &Commands) -> bool {
        match cmd {
            Commands::Init { .. }
            | Commands::Config
            | Commands::Doctor
            | Commands::Uninstall { .. }
            | Commands::Upgrade { .. }
            | Commands::Completions { .. }
            | Commands::ShellInit { .. }
            | Commands::GotoResolve { .. }
            | Commands::Maintenance(_) => false,
            _ => true,
        }
    }

    pub fn config_exists(custom_path: &Option<PathBuf>) -> bool {
        match DevConfig::load_from(custom_path) {
            Ok(cfg) => cfg.is_configured(),
            Err(_) => false,
        }
    }

    pub fn resolve_config_path(custom_path: &Option<PathBuf>) -> PathBuf {
        if let Some(p) = custom_path {
            return p.clone();
        }
        DevConfig::candidate_paths()
            .into_iter()
            .next()
            .unwrap_or_else(|| PathBuf::from("rtb.config.json"))
    }

    fn command_name(cmd: &Commands) -> &'static str {
        match cmd {
            Commands::List { .. } => "list",
            Commands::Status { .. } => "status",
            Commands::Info { .. } => "info",
            Commands::Health => "health",
            Commands::Index => "index",
            Commands::Deps { .. } => "deps",
            Commands::Workspace { .. } => "workspace",
            Commands::New { .. } => "new",
            Commands::Pause { .. } => "pause",
            Commands::Resume { .. } => "resume",
            Commands::Deploy { .. } => "deploy",
            Commands::Archive { .. } => "archive",
            Commands::Unarchive { .. } => "unarchive",
            Commands::Run { .. } => "run",
            Commands::Build { .. } => "build",
            Commands::Test { .. } => "test",
            Commands::Clean { .. } => "clean",
            Commands::Commit { .. } => "commit",
            Commands::Open { .. } => "open",
            Commands::Doctor => "doctor",
            Commands::Init { .. } => "init",
            Commands::Config => "config",
            Commands::Upgrade { .. } => "upgrade",
            Commands::Uninstall { .. } => "uninstall",
            Commands::Maintenance(_) => "maintenance",
            Commands::Agent { .. } => "agent",
            Commands::Ui { .. } => "ui",
            Commands::ShellInit { .. } => "shell-init",
            Commands::Completions { .. } => "completions",
            Commands::GotoResolve { .. } => "_goto-resolve",
            Commands::External(_) => "external",
        }
    }

    fn execute_command(cmd: Commands, cli: &Cli) -> Result<i32> {
        match cmd {
            Commands::List { active, paused, deployed, vibe, all, json } => {
                Self::execute_list(active, paused, deployed, vibe, all, json, cli)
            }
            Commands::Status { json } => {
                Self::execute_status(json, cli)
            }
            Commands::Info { project, json } => {
                Self::execute_info(project, json, cli)
            }
            Commands::Config => {
                Self::execute_config(cli)
            }
            Commands::New { name, root, stack } => {
                Self::execute_new(name, root, stack, cli)
            }
            Commands::Pause { project, prune_deps, force } => {
                Self::execute_pause(project, prune_deps, force, cli)
            }
            Commands::Resume { project, install } => {
                Self::execute_resume(project, install, cli)
            }
            Commands::Deploy { project, to, prod, staging } => {
                Self::execute_deploy(project, to, prod, staging, cli)
            }
            Commands::Archive { project, force } => {
                Self::execute_archive(project, force, cli)
            }
            Commands::Unarchive { project } => {
                Self::execute_unarchive(project, cli)
            }
            Commands::Run { project } => {
                Self::execute_run(project, cli)
            }
            Commands::Build { project } => {
                Self::execute_build(project, cli)
            }
            Commands::Test { project } => {
                Self::execute_test(project, cli)
            }
            Commands::Clean { dry_run, commit, days } => {
                Self::execute_clean(dry_run, commit, days, cli)
            }
            Commands::Doctor => Self::execute_doctor(cli),
            Commands::Init { force } => Self::execute_init(force, cli),
            Commands::Upgrade { check } => crate::upgrade::execute_upgrade(check),
            Commands::Uninstall { yes } => crate::uninstall::execute_uninstall(yes),
            Commands::Maintenance(m) => Self::execute_maintenance(m, cli),
            Commands::Health => Self::execute_health(cli),
            Commands::Index => Self::execute_index(cli),
            Commands::Open { project } => Self::execute_open(project, cli),
            Commands::Commit { positional_message, message } => {
                Self::execute_commit(message.or(positional_message), cli)
            }
            Commands::Deps { project, json } => {
                Self::execute_deps(project, json, cli)
            }
            Commands::Workspace { project, json } => {
                Self::execute_workspace(project, json, cli)
            }
            Commands::Agent { name, project, list, clean } => {
                Self::execute_agent(name, project, list, clean, cli)
            }
            Commands::GotoResolve { query } => {
                Self::execute_goto_resolve(query, cli)
            }
            Commands::External(args) => {
                if !args.is_empty() {
                    let cmd_name = &args[0];
                    let remaining = if args.len() > 1 { Some(args[1].clone()) } else { None };
                    let lower = cmd_name.to_lowercase();
                    if lower == "backup" {
                        return Self::execute_maintenance(MaintenanceArgs { command: Some(MaintenanceCommands::Backup) }, cli);
                    }
                    if lower == "env" {
                        return Self::execute_maintenance(MaintenanceArgs { command: Some(MaintenanceCommands::Env) }, cli);
                    }
                    if lower == "guard" {
                        return Self::execute_maintenance(MaintenanceArgs { command: Some(MaintenanceCommands::Guard) }, cli);
                    }
                    if matches!(
                        lower.as_str(),
                        "agy" | "claude" | "gemini" | "codex" | "cursor" | "windsurf" | "aider" | "openhands"
                    ) || crate::data::agents::is_command_installed(cmd_name) {
                        return Self::execute_agent(Some(cmd_name.clone()), remaining, false, false, cli);
                    }
                }
                eprintln!("rtb: command '{}' not recognized", args.get(0).cloned().unwrap_or_default());
                Ok(1)
            }
            Commands::ShellInit { shell } => {
                Self::print_shell_init(shell);
                Ok(0)
            }
            Commands::Ui { .. } => {
                // If invoked via CLI without subcommand args in non-terminal mode or when TUI is requested:
                // For now, TUI launch stub or in-process launch.
                eprintln!("rtb: command 'ui' not yet implemented");
                Ok(1)
            }
            other => {
                let name = Self::command_name(&other);
                eprintln!("rtb: command '{}' not yet implemented", name);
                Ok(1)
            }
        }
    }

    fn execute_info(project_name: String, cmd_json: bool, cli: &Cli) -> Result<i32> {
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

    fn execute_config(cli: &Cli) -> Result<i32> {
        let config_path = Self::resolve_config_path(&cli.config);
        if !config_path.exists() {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let default_config = serde_json::json!({
                "version": "1.0.0",
                "projectRoots": {
                    "active": "",
                    "paused": "",
                    "production": ""
                }
            });
            let content = serde_json::to_string_pretty(&default_config)?;
            std::fs::write(&config_path, content)?;
        }

        println!("Opening RTB configuration...");
        println!("  Config file: {}", config_path.display());

        let is_non_interactive = std::env::var("RTB_NON_INTERACTIVE").is_ok()
            || std::env::var("CI").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || !std::io::stdin().is_terminal();

        if is_non_interactive {
            return Ok(0);
        }

        let editor = std::env::var("EDITOR").or_else(|_| std::env::var("VISUAL")).ok();
        if let Some(ed) = editor {
            let _ = std::process::Command::new(ed).arg(&config_path).status();
            return Ok(0);
        }

        #[cfg(target_os = "windows")]
        {
            let code_status = std::process::Command::new("code")
                .arg(&config_path)
                .status();
            if code_status.is_err() || !code_status.unwrap().success() {
                let _ = std::process::Command::new("notepad.exe")
                    .arg(&config_path)
                    .status();
            }
        }

        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(&config_path)
                .status();
        }

        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(&config_path)
                .status();
        }

        Ok(0)
    }

    fn execute_list(
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

    fn execute_status(cmd_json: bool, cli: &Cli) -> Result<i32> {
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

    fn print_shell_init(shell: ShellChoice) {
        match shell {
            ShellChoice::Bash => {
                println!(
                    r#"# rtb shell integration — generated by rtb shell-init bash
function rtb() {{
  if [[ "$1" == "goto" ]]; then
    shift
    local query="" agent=""
    for arg in "$@"; do
      if [[ "$arg" == --* ]]; then
        agent="${{arg#--}}"
      else
        query="$arg"
      fi
    done
    local path
    path=$(command rtb _goto-resolve "$query") || return $?
    cd "$path" || return 1
    [[ -n "$agent" ]] && command rtb agent "$agent" "$path"
  else
    command rtb "$@"
  fi
}}
# Add to your profile: eval "$(rtb shell-init bash)""#
                );
            }
            ShellChoice::Zsh => {
                println!(
                    r#"# rtb shell integration — generated by rtb shell-init zsh
function rtb() {{
  if [[ "$1" == "goto" ]]; then
    shift
    local query="" agent=""
    for arg in "$@"; do
      if [[ "$arg" == --* ]]; then
        agent="${{arg#--}}"
      else
        query="$arg"
      fi
    done
    local path
    path=$(command rtb _goto-resolve "$query") || return $?
    cd "$path" || return 1
    [[ -n "$agent" ]] && command rtb agent "$agent" "$path"
  else
    command rtb "$@"
  fi
}}
# Add to your profile: eval "$(rtb shell-init zsh)""#
                );
            }
            ShellChoice::Fish => {
                println!(
                    r#"# rtb shell integration — generated by rtb shell-init fish
function rtb
  if test (count $argv) -gt 0 -a "$argv[1]" = "goto"
    set -e argv[1]
    set -l query ""; set -l agent ""
    for arg in $argv
      if string match -qr -- "^--(.+)" $arg
        set agent (string replace -- "--" "" $arg)
      else
        set query $arg
      end
    end
    set -l path (command rtb _goto-resolve $query); or return
    cd $path; or return 1
    test -n "$agent"; and command rtb agent $agent $path
  else
    command rtb $argv
  end
end
# Add to your config.fish: rtb shell-init fish | source"#
                );
            }
            ShellChoice::Powershell | ShellChoice::Pwsh => {
                let shell_name = if shell == ShellChoice::Pwsh { "pwsh" } else { "powershell" };
                println!(
                    r#"# rtb shell integration — generated by rtb shell-init {}
$env:_RTB_BIN = (Get-Command rtb -CommandType Application -ErrorAction SilentlyContinue).Source
function global:rtb {{
  if ($args.Count -gt 0 -and $args[0] -eq 'goto') {{
    $query = $null; $agent = $null
    foreach ($a in $args[1..($args.Count - 1)]) {{
      if ($a -match '^--(.+)$') {{ $agent = $Matches[1] }} else {{ $query = $a }}
    }}
    $path = & $env:_RTB_BIN _goto-resolve $query
    if ($LASTEXITCODE -ne 0) {{ return }}
    Set-Location $path
    if ($agent) {{ & $env:_RTB_BIN agent $agent $path }}
  }} else {{
    & $env:_RTB_BIN @args
  }}
}}
# Add to your $PROFILE: Invoke-Expression (& rtb shell-init {})"#,
                    shell_name, shell_name
                );
            }
        }
    }

    fn execute_new(
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

    fn execute_pause(
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

    fn execute_resume(project: Option<String>, install: bool, cli: &Cli) -> Result<i32> {
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

    fn execute_deploy(
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

    fn execute_archive(project: Option<String>, force: bool, cli: &Cli) -> Result<i32> {
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

    fn execute_unarchive(project: Option<String>, cli: &Cli) -> Result<i32> {
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

    fn resolve_project_or_cwd(project_name: Option<&str>, cli: &Cli) -> Result<PathBuf> {
        let cwd = std::env::current_dir()?;
        if let Some(name) = project_name {
            if let Ok(p) = std::fs::canonicalize(name) {
                if p.is_dir() {
                    return Ok(p);
                }
            }
            if let Ok(config) = DevConfig::load_from(&cli.config) {
                let projects = scan_all_projects(&config);
                if let Some(p) = projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                    return Ok(p.path.clone());
                }
            }
            let kebab = to_kebab_case(name);
            let candidate = cwd.join(&kebab);
            if candidate.is_dir() {
                return Ok(candidate);
            }
            eprintln!("Project '{}' not found.", name);
            anyhow::bail!("Project not found");
        }
        Ok(cwd)
    }

    fn get_cmd(cmd: &str) -> String {
        if cfg!(windows) {
            match cmd {
                "npm" => "npm.cmd".to_string(),
                "npx" => "npx.cmd".to_string(),
                "pnpm" => "pnpm.cmd".to_string(),
                "yarn" => "yarn.cmd".to_string(),
                _ => cmd.to_string(),
            }
        } else {
            cmd.to_string()
        }
    }

    fn execute_run(project: Option<String>, cli: &Cli) -> Result<i32> {
        let target_path = match Self::resolve_project_or_cwd(project.as_deref(), cli) {
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
                        let status = std::process::Command::new(Self::get_cmd("npm"))
                            .args(&["run", "dev"])
                            .current_dir(&target_path)
                            .status();
                        return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
                    } else if json["scripts"]["start"].is_string() {
                        println!("Running 'npm start' in {}...", target_path.display());
                        let status = std::process::Command::new(Self::get_cmd("npm"))
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
            let status = std::process::Command::new(Self::get_cmd("cargo"))
                .arg("run")
                .current_dir(&target_path)
                .status();
            return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
        }

        if target_path.join("go.mod").exists() {
            println!("Running 'go run .' in {}...", target_path.display());
            let status = std::process::Command::new(Self::get_cmd("go"))
                .args(&["run", "."])
                .current_dir(&target_path)
                .status();
            return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
        }

        if target_path.join("main.py").exists() {
            println!("Running 'python main.py' in {}...", target_path.display());
            let status = std::process::Command::new(Self::get_cmd("python"))
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

    fn execute_build(project: Option<String>, cli: &Cli) -> Result<i32> {
        let target_path = match Self::resolve_project_or_cwd(project.as_deref(), cli) {
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
                        let status = std::process::Command::new(Self::get_cmd("npm"))
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
            let status = std::process::Command::new(Self::get_cmd("cargo"))
                .args(&["build", "--release"])
                .current_dir(&target_path)
                .status();
            return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
        }

        if target_path.join("go.mod").exists() {
            println!("Running 'go build' in {}...", target_path.display());
            let status = std::process::Command::new(Self::get_cmd("go"))
                .arg("build")
                .current_dir(&target_path)
                .status();
            return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
        }

        eprintln!("No build configuration detected in {}.", target_path.display());
        Ok(1)
    }

    fn execute_test(project: Option<String>, cli: &Cli) -> Result<i32> {
        let target_path = match Self::resolve_project_or_cwd(project.as_deref(), cli) {
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
                        let status = std::process::Command::new(Self::get_cmd("npm"))
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
            let status = std::process::Command::new(Self::get_cmd("cargo"))
                .arg("test")
                .current_dir(&target_path)
                .status();
            return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
        }

        if target_path.join("pytest.ini").exists() || target_path.join("pyproject.toml").exists() {
            println!("Running 'pytest' in {}...", target_path.display());
            let status = std::process::Command::new(Self::get_cmd("pytest"))
                .current_dir(&target_path)
                .status();
            return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
        }

        if target_path.join("cli/tests").exists() {
            println!("Running 'Invoke-Pester' in {}/cli/tests...", target_path.display());
            let status = std::process::Command::new(Self::get_cmd("pwsh"))
                .args(&["-Command", "Invoke-Pester cli/tests/"])
                .current_dir(&target_path)
                .status();
            return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
        }

        if target_path.join("tests").exists() {
            println!("Running 'Invoke-Pester' in {}...", target_path.display());
            let status = std::process::Command::new(Self::get_cmd("pwsh"))
                .args(&["-Command", "Invoke-Pester tests/"])
                .current_dir(&target_path)
                .status();
            return Ok(status.map(|s| s.code().unwrap_or(1)).unwrap_or(1));
        }

        eprintln!("No test configuration detected in {}.", target_path.display());
        Ok(1)
    }

    fn execute_deps(project: Option<String>, cmd_json: bool, cli: &Cli) -> Result<i32> {
        let target_path = match Self::resolve_project_or_cwd(project.as_deref(), cli) {
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

    fn execute_workspace(project: Option<String>, cmd_json: bool, cli: &Cli) -> Result<i32> {
        let target_path = match Self::resolve_project_or_cwd(project.as_deref(), cli) {
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

    fn execute_clean(
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
            _size_bytes: u64,
            _size_mb: f64,
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
                                    _size_bytes: size,
                                    _size_mb: mb,
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

    fn execute_doctor(cli: &Cli) -> Result<i32> {
        println!("══════════════════════════════════════════");
        println!("  rtb (رتّب) » System Doctor");
        println!("══════════════════════════════════════════\n");

        let mut all_good = true;

        let write_check = |pass: bool, label: &str, detail: &str| {
            if pass {
                println!("  ✅ {}", label);
            } else {
                println!("  ❌ {}", label);
                if !detail.is_empty() {
                    println!("     → {}", detail);
                }
            }
        };

        // 1. Config Check
        println!("  Config");
        let user_config_dir = dirs::config_dir()
            .map(|d| d.join("rtb"))
            .unwrap_or_else(|| PathBuf::from(".config/rtb"));
        let user_config_file = user_config_dir.join("rtb.config.json");

        let config_res = DevConfig::load_from(&cli.config);
        let config_passed = config_res.is_ok();
        let resolved_config_path = Self::resolve_config_path(&cli.config);
        let config_label = if config_passed {
            format!("rtb.config.json ({})", resolved_config_path.display())
        } else {
            "rtb.config.json found and parseable".to_string()
        };
        let detail_msg = format!("Run 'rtb init' to create your config at {}", user_config_file.display());
        write_check(config_passed, &config_label, if config_passed { "" } else { &detail_msg });
        if !config_passed {
            all_good = false;
        }

        // 2. Project Roots Check (9 roots)
        println!("\n  Project Roots");
        if let Ok(ref cfg) = config_res {
            let root_map = [
                ("active", &cfg.project_roots.active, "📁", "Active"),
                ("paused", &cfg.project_roots.paused, "⏸️", "Paused"),
                ("planning", &cfg.project_roots.planning, "📋", "Planning"),
                ("testing", &cfg.project_roots.testing, "🧪", "Testing"),
                ("production", &cfg.project_roots.production, "🚀", "Production"),
                ("staging", &cfg.project_roots.staging, "🚀", "Staging"),
                ("vibe", &cfg.project_roots.vibe, "✨", "Vibe"),
                ("sandbox", &cfg.project_roots.sandbox, "📦", "Sandbox"),
                ("abandoned", &cfg.project_roots.abandoned, "🪦", "Abandoned"),
            ];

            for (key, path_str, emoji, label_name) in root_map {
                let exists = !path_str.is_empty() && PathBuf::from(path_str).exists();
                let label = if !path_str.is_empty() {
                    format!("{} {} ({}) → {}", emoji, label_name, key, path_str)
                } else {
                    format!("{} → (not configured)", key)
                };
                let detail = format!("Directory does not exist. Create it or update projectRoots.{} in your config.", key);
                write_check(exists, &label, if exists { "" } else { &detail });
                if !exists {
                    all_good = false;
                }
            }
        } else {
            write_check(false, "Cannot check project roots (invalid or missing config)", "Fix rtb.config.json or run 'rtb init --force'");
            all_good = false;
        }

        // 3. Required Tools
        println!("\n  Required Tools");
        let git_found = crate::data::agents::is_command_installed("git");
        write_check(git_found, "git in PATH", if git_found { "" } else { "Install git and ensure it is on your PATH" });
        if !git_found {
            all_good = false;
        }

        // 4. Optional Tools
        println!("\n  Optional Tools");
        let optionals = [
            ("node", "Node.js (for JavaScript/TypeScript projects)"),
            ("cargo", "Cargo / Rust (for Rust projects and rtb build)"),
            ("python", "Python (for Python projects)"),
            ("tar", "tar (for rtb archive/unarchive)"),
        ];
        for (tool, desc) in optionals {
            let found = crate::data::agents::is_command_installed(tool);
            let icon = if found { "  ✅" } else { "  ⚠ " };
            println!("{} {}", icon, desc);
        }

        // 5. AI Agents
        println!("\n  AI Agents");
        let known_agents = ["agy", "claude", "gemini", "codex", "cursor", "windsurf", "aider", "openhands"];
        let found_agents: Vec<&str> = known_agents.iter().copied().filter(|a| crate::data::agents::is_command_installed(a)).collect();
        if !found_agents.is_empty() {
            println!("  ✅ Installed: {}", found_agents.join(", "));
        } else {
            println!("  ⚠  No AI agents found in PATH");
        }

        // 6. TUI Binary
        println!("\n  TUI Binary");
        let exe_found = std::env::current_exe().is_ok();
        let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("rtb"));
        write_check(exe_found, &format!("rtb binary installed ({})", exe_path.display()), "");

        // 7. Summary
        println!("\n══════════════════════════════════════════");
        if all_good {
            println!("  ✅ All checks passed — RTB is healthy!");
        } else {
            println!("  ❌ Some checks failed — see above for details.");
        }
        println!("══════════════════════════════════════════");

        Ok(if all_good { 0 } else { 1 })
    }

    fn execute_init(force: bool, cli: &Cli) -> Result<i32> {
        println!("══════════════════════════════════════════");
        println!("  rtb (رتّب) » Interactive Setup Wizard");
        println!("══════════════════════════════════════════\n");

        let user_home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let user_config_dir = dirs::config_dir()
            .map(|d| d.join("rtb"))
            .unwrap_or_else(|| user_home_dir.join(".config").join("rtb"));
        let user_config_file = if let Some(ref p) = cli.config {
            p.clone()
        } else {
            user_config_dir.join("rtb.config.json")
        };

        if user_config_file.exists() && !force {
            println!("  Configuration already exists at:");
            println!("    {}", user_config_file.display());
            println!("  Run 'rtb config' to open and edit your configuration in your default editor.");
            println!("  Use 'rtb init --force' to overwrite and re-run the setup wizard.");
            println!();
            return Ok(0);
        }

        if let Some(parent) = user_config_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let is_non_interactive = std::env::var("RTB_NON_INTERACTIVE").is_ok()
            || std::env::var("CI").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || !std::io::stdin().is_terminal();

        let chosen_root = if is_non_interactive {
            let d_drive = PathBuf::from("D:\\02-Projects");
            if d_drive.exists() { d_drive } else { user_home_dir.join("Projects") }
        } else {
            println!("  Step 1: Workspace Root Location");
            println!("  Where do you want to keep and manage your projects?");

            let default_root = user_home_dir.join("Projects");
            let candidate_list = [
                default_root.clone(),
                user_home_dir.join("dev"),
                user_home_dir.join("code"),
                user_home_dir.join("repos"),
                user_home_dir.join("workspace"),
                user_home_dir.join("src"),
                PathBuf::from("D:\\02-Projects"),
                PathBuf::from("D:\\Projects"),
                PathBuf::from("D:\\dev"),
            ];

            let existing_candidates: Vec<PathBuf> = candidate_list.iter().filter(|p| p.exists()).cloned().collect();
            let mut selected_root: Option<PathBuf> = None;

            if !existing_candidates.is_empty() {
                println!("\n  Detected existing project directories:");
                for (idx, candidate) in existing_candidates.iter().enumerate() {
                    println!("    [{}] {}", idx + 1, candidate.display());
                }
                println!("    [C] Enter custom path");
                print!("\n  Select an option [1-{} or C] (Default: 1): ", existing_candidates.len());
                let _ = std::io::stdout().flush();

                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_ok() {
                    let trimmed = input.trim();
                    if trimmed.is_empty() || trimmed == "1" {
                        selected_root = Some(existing_candidates[0].clone());
                    } else if let Ok(n) = trimmed.parse::<usize>() {
                        if n >= 1 && n <= existing_candidates.len() {
                            selected_root = Some(existing_candidates[n - 1].clone());
                        }
                    }
                }
            }

            if let Some(r) = selected_root {
                r
            } else {
                print!("  Enter your projects root path (Default: {}): ", default_root.display());
                let _ = std::io::stdout().flush();
                let mut input = String::new();
                let _ = std::io::stdin().read_line(&mut input);
                let trimmed = input.trim().trim_matches(|c| c == '"' || c == '\'');
                if trimmed.is_empty() {
                    default_root
                } else {
                    PathBuf::from(trimmed)
                }
            }
        };

        println!("  Selected root: {}", chosen_root.display());
        std::fs::create_dir_all(&chosen_root)?;

        // Build projectRoots
        let active_path = chosen_root.join("02-Projects").join("01-Development").join("01-Active");
        let paused_path = chosen_root.join("02-Projects").join("01-Development").join("04-Paused");
        let planning_path = chosen_root.join("02-Projects").join("01-Development").join("02-Planning");
        let testing_path = chosen_root.join("02-Projects").join("01-Development").join("03-Testing");
        let abandoned_path = chosen_root.join("02-Projects").join("01-Development").join("05-Abandoned");
        let prod_path = chosen_root.join("02-Projects").join("02-Deployed").join("01-Production");
        let staging_path = chosen_root.join("02-Projects").join("02-Deployed").join("02-Staging");
        let vibe_path = chosen_root.join("02-Projects").join("03-Vibe-Coding");
        let sandbox_path = chosen_root.join("01-SandBox");

        for p in &[&active_path, &paused_path, &planning_path, &testing_path, &abandoned_path, &prod_path, &staging_path, &vibe_path, &sandbox_path] {
            let _ = std::fs::create_dir_all(p);
        }

        let backup_root = chosen_root.join("08-Backup");
        let config_root = chosen_root.join("05-Config");
        let template_dir = chosen_root.join("05-Config").join("templates");

        let new_config = serde_json::json!({
            "version": "1.0.0",
            "projectRoots": {
                "active": { "path": active_path.to_string_lossy(), "label": "Active", "emoji": "📁" },
                "paused": { "path": paused_path.to_string_lossy(), "label": "Paused", "emoji": "⏸️" },
                "planning": { "path": planning_path.to_string_lossy(), "label": "Planning", "emoji": "📋" },
                "testing": { "path": testing_path.to_string_lossy(), "label": "Testing", "emoji": "🧪" },
                "abandoned": { "path": abandoned_path.to_string_lossy(), "label": "Abandoned", "emoji": "🪦" },
                "production": { "path": prod_path.to_string_lossy(), "label": "Production", "emoji": "🚀" },
                "staging": { "path": staging_path.to_string_lossy(), "label": "Staging", "emoji": "🚀" },
                "vibe": { "path": vibe_path.to_string_lossy(), "label": "Vibe Coding", "emoji": "✨" },
                "sandbox": { "path": sandbox_path.to_string_lossy(), "label": "Sandbox", "emoji": "📦" }
            },
            "backupRoot": backup_root.to_string_lossy(),
            "configRoot": config_root.to_string_lossy(),
            "templateDir": template_dir.to_string_lossy(),
            "cleanDeps": {
                "daysInactive": 60,
                "targets": ["node_modules", ".venv", ".next", "__pycache__", "dist", "build", "target"]
            },
            "staleThresholdDays": 90,
            "gitHealth": {
                "scanRoots": [
                    chosen_root.join("02-Projects").to_string_lossy(),
                    sandbox_path.to_string_lossy()
                ]
            }
        });

        let content = serde_json::to_string_pretty(&new_config)?;
        std::fs::write(&user_config_file, content)?;

        println!("\n  ✓ RTB configuration successfully initialized!");
        println!("    Configuration file: {}", user_config_file.display());
        println!("    Workspace root    : {}", chosen_root.display());
        println!("    💡 To customize emojis, labels, or paths, run 'rtb config' anytime.");
        println!("\n  Ready to build! Run 'rtb help' or launch the TUI with 'rtb ui'.");

        Ok(0)
    }

    fn execute_maintenance(m_args: MaintenanceArgs, _cli: &Cli) -> Result<i32> {
        let script_name = match m_args.command {
            Some(MaintenanceCommands::Backup) => "backup-configs.ps1".to_string(),
            Some(MaintenanceCommands::Env) => "backup-env-files.ps1".to_string(),
            Some(MaintenanceCommands::Guard) => "guard-d-drive.ps1".to_string(),
            Some(MaintenanceCommands::Run { script }) => {
                if script.ends_with(".ps1") { script } else { format!("{}.ps1", script) }
            }
            None => "weekly-maintenance.ps1".to_string(),
        };

        println!("══════════════════════════════════════════");
        println!("  rtb (رتّب) » Maintenance: {}", script_name);
        println!("══════════════════════════════════════════\n");

        let cfg = DevConfig::load_from(&cli.config).ok();
        let config_scripts_dir = cfg.as_ref().and_then(|c| {
            if c.config_root.trim().is_empty() {
                None
            } else {
                Some(PathBuf::from(&c.config_root).join("scripts"))
            }
        });

        let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let candidate_paths = [
            config_scripts_dir.as_ref().map(|d| d.join(&script_name)),
            exe_dir.as_ref().map(|d| d.join("scripts").join(&script_name)),
            exe_dir.as_ref().map(|d| d.join("cli").join("scripts").join(&script_name)),
            Some(cwd.join("scripts").join(&script_name)),
            Some(cwd.join("cli").join("scripts").join(&script_name)),
            dirs::config_dir().map(|d| d.join("rtb").join("bin").join(&script_name)),
            dirs::config_dir().map(|d| d.join("rtb").join("scripts").join(&script_name)),
        ];

        let mut resolved_script: Option<PathBuf> = None;
        for cand in candidate_paths.into_iter().flatten() {
            if cand.is_file() {
                resolved_script = Some(cand);
                break;
            }
        }

        let script_path = match resolved_script {
            Some(p) => p,
            None => {
                eprintln!("Maintenance script '{}' not found.", script_name);
                return Ok(1);
            }
        };

        let pwsh_bin = if crate::data::agents::is_command_installed("pwsh") {
            "pwsh"
        } else {
            "powershell"
        };

        let status = std::process::Command::new(pwsh_bin)
            .arg("-NoProfile")
            .arg("-File")
            .arg(&script_path)
            .status();

        match status {
            Ok(s) => Ok(s.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("Failed to execute script '{}': {}", script_path.display(), e);
                Ok(1)
            }
        }
    }

    fn execute_health(cli: &Cli) -> Result<i32> {
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

    fn execute_index(cli: &Cli) -> Result<i32> {
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

    fn execute_open(project: Option<String>, cli: &Cli) -> Result<i32> {
        let target_path = match Self::resolve_project_or_cwd(project.as_deref(), cli) {
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

    fn execute_commit(message: Option<String>, _cli: &Cli) -> Result<i32> {
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
        let commit_res = std::process::Command::new(Self::get_cmd("git"))
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

    fn execute_goto_resolve(query: Option<String>, cli: &Cli) -> Result<i32> {
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



    fn execute_agent(
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

        let target_path = match Self::resolve_project_or_cwd(proj_name.as_deref(), cli) {
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

        let cmd_to_run = Self::get_cmd(&target_agent);
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
}

fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

fn to_kebab_case(name: &str) -> String {
    let mut kebab = String::new();
    let mut last_was_dash = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            kebab.push(c.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            kebab.push('-');
            last_was_dash = true;
        }
    }
    kebab.trim_matches('-').to_string()
}

fn is_git_clean(path: &std::path::Path) -> bool {
    if let Some(porcelain) = crate::data::scanner::run_git(path, &["status", "--porcelain"]) {
        porcelain.lines().filter(|l| !l.trim().is_empty()).count() == 0
    } else {
        true
    }
}

pub fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub fn move_dir_all(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    if std::fs::rename(src, dst).is_ok() {
        return Ok(());
    }
    copy_dir_all(src, dst)?;
    std::fs::remove_dir_all(src)?;
    Ok(())
}
