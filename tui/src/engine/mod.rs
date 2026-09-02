pub mod agent;
pub mod dev;
pub mod helpers;
pub mod inspect;
pub mod lifecycle;
pub mod system;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::io::{IsTerminal, Write};
use std::path::PathBuf;

use crate::config::DevConfig;

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
                    let init_res = system::execute_init(false, &matches)?;
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

    pub fn command_name(cmd: &Commands) -> &'static str {
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
                inspect::execute_list(active, paused, deployed, vibe, all, json, cli)
            }
            Commands::Status { json } => {
                inspect::execute_status(json, cli)
            }
            Commands::Info { project, json } => {
                inspect::execute_info(project, json, cli)
            }
            Commands::Config => {
                system::execute_config(cli)
            }
            Commands::New { name, root, stack } => {
                lifecycle::execute_new(name, root, stack, cli)
            }
            Commands::Pause { project, prune_deps, force } => {
                lifecycle::execute_pause(project, prune_deps, force, cli)
            }
            Commands::Resume { project, install } => {
                lifecycle::execute_resume(project, install, cli)
            }
            Commands::Deploy { project, to, prod, staging } => {
                lifecycle::execute_deploy(project, to, prod, staging, cli)
            }
            Commands::Archive { project, force } => {
                lifecycle::execute_archive(project, force, cli)
            }
            Commands::Unarchive { project } => {
                lifecycle::execute_unarchive(project, cli)
            }
            Commands::Run { project } => {
                dev::execute_run(project, cli)
            }
            Commands::Build { project } => {
                dev::execute_build(project, cli)
            }
            Commands::Test { project } => {
                dev::execute_test(project, cli)
            }
            Commands::Clean { dry_run, commit, days } => {
                dev::execute_clean(dry_run, commit, days, cli)
            }
            Commands::Doctor => system::execute_doctor(cli),
            Commands::Init { force } => system::execute_init(force, cli),
            Commands::Upgrade { check } => crate::upgrade::execute_upgrade(check),
            Commands::Uninstall { yes } => crate::uninstall::execute_uninstall(yes),
            Commands::Maintenance(m) => system::execute_maintenance(m, cli),
            Commands::Health => inspect::execute_health(cli),
            Commands::Index => inspect::execute_index(cli),
            Commands::Open { project } => dev::execute_open(project, cli),
            Commands::Commit { positional_message, message } => {
                dev::execute_commit(message.or(positional_message), cli)
            }
            Commands::Deps { project, json } => {
                inspect::execute_deps(project, json, cli)
            }
            Commands::Workspace { project, json } => {
                inspect::execute_workspace(project, json, cli)
            }
            Commands::Agent { name, project, list, clean } => {
                agent::execute_agent(name, project, list, clean, cli)
            }
            Commands::GotoResolve { query } => {
                agent::execute_goto_resolve(query, cli)
            }
            Commands::External(args) => {
                if !args.is_empty() {
                    let cmd_name = &args[0];
                    let remaining = if args.len() > 1 { Some(args[1].clone()) } else { None };
                    let lower = cmd_name.to_lowercase();
                    if lower == "backup" {
                        return system::execute_maintenance(MaintenanceArgs { command: Some(MaintenanceCommands::Backup) }, cli);
                    }
                    if lower == "env" {
                        return system::execute_maintenance(MaintenanceArgs { command: Some(MaintenanceCommands::Env) }, cli);
                    }
                    if lower == "guard" {
                        return system::execute_maintenance(MaintenanceArgs { command: Some(MaintenanceCommands::Guard) }, cli);
                    }
                    if matches!(
                        lower.as_str(),
                        "agy" | "claude" | "gemini" | "codex" | "cursor" | "windsurf" | "aider" | "openhands"
                    ) || crate::data::agents::is_command_installed(cmd_name) {
                        return agent::execute_agent(Some(cmd_name.clone()), remaining, false, false, cli);
                    }
                }
                eprintln!("rtb: command '{}' not recognized", args.get(0).cloned().unwrap_or_default());
                Ok(1)
            }
            Commands::ShellInit { shell } => {
                system::print_shell_init(shell);
                Ok(0)
            }
            Commands::Ui { .. } => {
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
}
