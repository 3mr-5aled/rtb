use anyhow::Result;
use std::io::Write;
use std::io::IsTerminal;
use std::path::PathBuf;
use crate::config::DevConfig;
use crate::engine::{Cli, MaintenanceArgs, MaintenanceCommands, ShellChoice};

pub fn execute_config(cli: &Cli) -> Result<i32> {
    let config_path = crate::engine::RtbEngine::resolve_config_path(&cli.config);
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

pub fn execute_init(force: bool, cli: &Cli) -> Result<i32> {
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
    let prod_path = chosen_root.join("02-Projects").join("02-Deployed").join("01-Production");
    let staging_path = chosen_root.join("02-Projects").join("02-Deployed").join("02-Staging");
    let vibe_path = chosen_root.join("02-Projects").join("04-Sandbox").join("03-VibeCoding");
    let sandbox_path = chosen_root.join("02-Projects").join("04-Sandbox").join("01-Exploration");
    let abandoned_path = chosen_root.join("02-Projects").join("03-Archived").join("01-Abandoned");

    std::fs::create_dir_all(&active_path)?;
    std::fs::create_dir_all(&paused_path)?;
    std::fs::create_dir_all(&prod_path)?;
    std::fs::create_dir_all(&staging_path)?;

    let backup_dir = chosen_root.join("05-Backups");
    let config_dir = chosen_root.join("06-Configuration");
    let template_dir = chosen_root.join("01-Templates");
    std::fs::create_dir_all(&backup_dir)?;
    std::fs::create_dir_all(&config_dir)?;
    std::fs::create_dir_all(&template_dir)?;

    let config_json = serde_json::json!({
        "version": "1.0.0",
        "projectRoots": {
            "active": { "path": active_path.to_string_lossy(), "label": "Active", "emoji": "⚡" },
            "paused": { "path": paused_path.to_string_lossy(), "label": "Paused", "emoji": "⏸️" },
            "planning": { "path": planning_path.to_string_lossy(), "label": "Planning", "emoji": "📋" },
            "testing": { "path": testing_path.to_string_lossy(), "label": "Testing", "emoji": "🧪" },
            "production": { "path": prod_path.to_string_lossy(), "label": "Production", "emoji": "🚀" },
            "staging": { "path": staging_path.to_string_lossy(), "label": "Staging", "emoji": "🚀" },
            "vibe": { "path": vibe_path.to_string_lossy(), "label": "Vibe", "emoji": "✨" },
            "sandbox": { "path": sandbox_path.to_string_lossy(), "label": "Sandbox", "emoji": "📦" },
            "abandoned": { "path": abandoned_path.to_string_lossy(), "label": "Abandoned", "emoji": "🪦" }
        },
        "backupRoot": backup_dir.to_string_lossy(),
        "configRoot": config_dir.to_string_lossy(),
        "templateDir": template_dir.to_string_lossy(),
        "staleThresholdDays": 90,
        "cleanDeps": {
            "daysInactive": 14,
            "targets": ["node_modules", "target", ".venv", "dist", "build"]
        },
        "gitHealth": {
            "scanRoots": [active_path.to_string_lossy(), paused_path.to_string_lossy()]
        }
    });

    let formatted = serde_json::to_string_pretty(&config_json)?;
    std::fs::write(&user_config_file, formatted)?;

    println!("\n  ✅ Configuration created successfully at:");
    println!("     {}", user_config_file.display());
    println!("\n  Workspace Folders Scaffolded:");
    println!("    📁 Active:     {}", active_path.display());
    println!("    ⏸️ Paused:     {}", paused_path.display());
    println!("    🚀 Production: {}", prod_path.display());
    println!("    💾 Backups:    {}", backup_dir.display());
    println!("\n  Run 'rtb' to open the interactive TUI dashboard!");
    Ok(0)
}

pub fn execute_doctor(cli: &Cli) -> Result<i32> {
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
    let resolved_config_path = crate::engine::RtbEngine::resolve_config_path(&cli.config);
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

    Ok(0)
}

pub struct MaintenanceTaskRegistry;

impl MaintenanceTaskRegistry {
    pub fn resolve_script(script_name: &str, custom_config: &Option<PathBuf>) -> Option<PathBuf> {
        let cfg = DevConfig::load_from(custom_config).ok();
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
            config_scripts_dir.as_ref().map(|d| d.join(script_name)),
            exe_dir.as_ref().map(|d| d.join("scripts").join(script_name)),
            exe_dir.as_ref().map(|d| d.join("cli").join("scripts").join(script_name)),
            Some(cwd.join("scripts").join(script_name)),
            Some(cwd.join("cli").join("scripts").join(script_name)),
            dirs::config_dir().map(|d| d.join("rtb").join("bin").join(script_name)),
            dirs::config_dir().map(|d| d.join("rtb").join("scripts").join(script_name)),
        ];

        candidate_paths.into_iter().flatten().find(|p| p.is_file())
    }

    pub fn execute(script_name: &str, custom_config: &Option<PathBuf>) -> Result<i32> {
        let script_path = match Self::resolve_script(script_name, custom_config) {
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
}

pub fn execute_maintenance(m_args: MaintenanceArgs, cli: &Cli) -> Result<i32> {
    let cmd = m_args.command.unwrap_or(MaintenanceCommands::Run {
        script: "maintenance.ps1".to_string(),
    });

    let script_name = match cmd {
        MaintenanceCommands::Backup => "backup.ps1".to_string(),
        MaintenanceCommands::Env => "env-sync.ps1".to_string(),
        MaintenanceCommands::Guard => "guard.ps1".to_string(),
        MaintenanceCommands::Run { script } => {
            if script.ends_with(".ps1") {
                script
            } else {
                format!("{}.ps1", script)
            }
        }
    };

    println!("══════════════════════════════════════════");
    println!("  rtb (رتّب) » Maintenance: {}", script_name);
    println!("══════════════════════════════════════════\n");

    MaintenanceTaskRegistry::execute(&script_name, &cli.config)
}

pub fn print_shell_init(shell: ShellChoice) {
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
