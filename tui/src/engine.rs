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
    Deps,
    Workspace,

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
    Run,
    Build,
    Test,
    Clean {
        #[arg(long)]
        dry_run: bool,
    },
    Commit {
        #[arg(short)]
        message: Option<String>,
    },
    Open {
        project: Option<String>,
    },

    // --- System ---
    #[command(next_help_heading = "System")]
    Doctor,
    Init,
    Config,
    Upgrade {
        #[arg(long)]
        check: bool,
    },
    Uninstall {
        #[arg(long)]
        yes: bool,
    },
    Maintenance(MaintenanceArgs),

    // --- Agent ---
    #[command(next_help_heading = "Agent")]
    Agent {
        name: Option<String>,
        project: Option<String>,
        #[arg(long)]
        list: bool,
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
    #[command(hide = true)]
    GotoResolve {
        query: String,
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
                    eprintln!("rtb: command 'init' not yet implemented");
                    eprintln!("Run `rtb {}` again to continue.", Self::command_name(&cmd));
                    return Ok(1);
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
            Commands::Init
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
        if let Some(p) = custom_path {
            return p.is_file();
        }
        DevConfig::candidate_paths().iter().any(|p| p.is_file())
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
            Commands::Deps => "deps",
            Commands::Workspace => "workspace",
            Commands::New { .. } => "new",
            Commands::Pause { .. } => "pause",
            Commands::Resume { .. } => "resume",
            Commands::Deploy { .. } => "deploy",
            Commands::Archive { .. } => "archive",
            Commands::Unarchive { .. } => "unarchive",
            Commands::Run => "run",
            Commands::Build => "build",
            Commands::Test => "test",
            Commands::Clean { .. } => "clean",
            Commands::Commit { .. } => "commit",
            Commands::Open { .. } => "open",
            Commands::Doctor => "doctor",
            Commands::Init => "init",
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
            ShellChoice::Bash | ShellChoice::Zsh => {
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
                println!(
                    r#"# rtb shell integration — generated by rtb shell-init powershell
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
# Add to your $PROFILE: Invoke-Expression (& rtb shell-init powershell)"#
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
