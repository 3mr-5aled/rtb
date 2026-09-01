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
    },
    Pause {
        project: Option<String>,
        #[arg(long)]
        prune_deps: bool,
    },
    Resume {
        project: Option<String>,
    },
    Deploy {
        project: Option<String>,
        #[arg(long, short = 't')]
        to: Option<String>,
    },
    Archive {
        project: Option<String>,
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
}
