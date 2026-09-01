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
    List,
    #[command(visible_alias = "st")]
    Status,
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
        let matches = match Cli::try_parse_from(args) {
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
            Commands::List => "list",
            Commands::Status => "status",
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

    fn execute_command(cmd: Commands, _cli: &Cli) -> Result<i32> {
        match cmd {
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
