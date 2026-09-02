use crate::data::project::Project;
use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentInfo {
    pub name: String,
    pub command: String,
    pub installed: bool,
}

pub fn is_command_installed(cmd: &str) -> bool {
    if let Ok(path_var) = env::var("PATH") {
        let paths = env::split_paths(&path_var);
        for mut path in paths {
            path.push(cmd);
            if path.is_file() {
                return true;
            }
            #[cfg(target_os = "windows")]
            {
                for ext in &[".exe", ".cmd", ".bat", ".ps1"] {
                    let mut path_ext = path.clone();
                    path_ext.set_extension(&ext[1..]);
                    if path_ext.is_file() {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn get_installed_agents() -> Vec<AgentInfo> {
    let agents = vec![
        ("Google Antigravity", "agy"),
        ("Claude Code", "claude"),
        ("Gemini CLI", "gemini"),
        ("Codex CLI", "codex"),
        ("Cursor", "cursor"),
        ("Windsurf", "windsurf"),
        ("Aider", "aider"),
        ("OpenHands", "openhands"),
    ];

    agents
        .into_iter()
        .map(|(name, cmd)| AgentInfo {
            name: name.to_string(),
            command: cmd.to_string(),
            installed: is_command_installed(cmd),
        })
        .collect()
}

pub fn get_default_agent() -> Option<AgentInfo> {
    let all = get_installed_agents();
    if let Some(agy) = all.iter().find(|a| a.command == "agy" && a.installed) {
        return Some(agy.clone());
    }
    all.into_iter().find(|a| a.installed)
}

pub struct AgentOrchestrator;

impl AgentOrchestrator {
    pub fn discover() -> Vec<AgentInfo> {
        get_installed_agents()
    }

    pub fn is_installed(agent_cmd: &str) -> bool {
        is_command_installed(agent_cmd)
    }

    pub fn default_agent() -> Option<AgentInfo> {
        get_default_agent()
    }

    pub fn generate_context(project: &Project) -> Option<std::path::PathBuf> {
        create_agent_context_file(project)
    }

    pub fn launch(agent_cmd: &str, project_path: &std::path::Path) -> std::io::Result<i32> {
        let cmd_to_run = if cfg!(windows) {
            match agent_cmd {
                "npm" => "npm.cmd",
                "npx" => "npx.cmd",
                "pnpm" => "pnpm.cmd",
                "yarn" => "yarn.cmd",
                _ => agent_cmd,
            }
        } else {
            agent_cmd
        };

        let status = Command::new(cmd_to_run)
            .current_dir(project_path)
            .status()?;

        Ok(status.code().unwrap_or(1))
    }
}

pub fn create_agent_context_file(project: &Project) -> Option<std::path::PathBuf> {
    let context_path = project.path.join(".rtb_context.md");
    let stack_str = if project.stack.is_empty() || (project.stack.len() == 1 && project.stack[0] == "-") {
        "Unknown".into()
    } else {
        project.stack.join(", ")
    };
    let branch_str = project
        .git
        .as_ref()
        .map(|g| g.branch.as_str())
        .filter(|b| !b.is_empty() && *b != "-")
        .unwrap_or("unknown");
    let readme_str = project
        .readme_preview
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("(no README)");

    let path_str = project.path.to_string_lossy();
    let is_git_repo = project.path.join(".git").exists();

    let (git_log_indented, git_diff_stat) = if is_git_repo {
        let git_log_output = std::process::Command::new("git")
            .args(["-C", &path_str, "log", "--oneline", "-10"])
            .output()
            .ok()
            .and_then(|o| if o.status.success() { String::from_utf8(o.stdout).ok() } else { None });

        let log_lines = match git_log_output {
            Some(ref s) if !s.trim().is_empty() => s
                .lines()
                .map(|l| format!("  {}", l))
                .collect::<Vec<_>>()
                .join("\n"),
            Some(_) => "  (no commits)".into(),
            None => "  (no commits)".into(),
        };

        let git_diff_output = std::process::Command::new("git")
            .args(["-C", &path_str, "diff", "--stat", "HEAD"])
            .output()
            .ok()
            .and_then(|o| if o.status.success() { String::from_utf8(o.stdout).ok() } else { None });

        let diff_lines = match git_diff_output {
            Some(ref s) if !s.trim().is_empty() => s
                .lines()
                .map(|l| format!("  {}", l))
                .collect::<Vec<_>>()
                .join("\n"),
            Some(_) => "  (working tree clean)".into(),
            None => "  (working tree clean)".into(),
        };

        (log_lines, diff_lines)
    } else {
        (
            "  (not a git repository)".into(),
            "  (not a git repository)".into(),
        )
    };

    let mut deps_section = String::new();

    // 1. package.json
    let pkg_path = project.path.join("package.json");
    if pkg_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                    let names: Vec<&str> = deps.keys().take(20).map(|k| k.as_str()).collect();
                    if !names.is_empty() {
                        deps_section.push_str(&format!("**package.json deps:** {}\n", names.join(", ")));
                    }
                }
                if let Some(dev_deps) = json.get("devDependencies").and_then(|d| d.as_object()) {
                    let dev_names: Vec<&str> = dev_deps.keys().take(10).map(|k| k.as_str()).collect();
                    if !dev_names.is_empty() {
                        deps_section.push_str(&format!("**devDependencies:** {}\n", dev_names.join(", ")));
                    }
                }
            } else {
                deps_section.push_str("(could not parse package.json)\n");
            }
        }
    }

    // 2. Cargo.toml
    let cargo_path = project.path.join("Cargo.toml");
    if cargo_path.exists() {
        if let Ok(cargo_content) = std::fs::read_to_string(&cargo_path) {
            let mut crates = Vec::new();
            for line in cargo_content.lines() {
                let trimmed = line.trim();
                if let Some(eq_pos) = trimmed.find('=') {
                    let key = trimmed[..eq_pos].trim();
                    if !key.is_empty()
                        && key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                        && key.chars().next().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false)
                    {
                        crates.push(key.to_string());
                        if crates.len() >= 20 {
                            break;
                        }
                    }
                }
            }
            if !crates.is_empty() {
                deps_section.push_str(&format!("**Cargo.toml crates:** {}\n", crates.join(", ")));
            }
        }
    }

    // 3. requirements.txt
    let req_path = project.path.join("requirements.txt");
    if req_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&req_path) {
            let reqs: Vec<&str> = content
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .take(20)
                .collect();
            if !reqs.is_empty() {
                deps_section.push_str(&format!("**requirements.txt:** {}\n", reqs.join(", ")));
            }
        }
    }

    // 4. go.mod
    let gomod_path = project.path.join("go.mod");
    if gomod_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&gomod_path) {
            let go_mods: Vec<&str> = content
                .lines()
                .filter(|l| l.starts_with("    ") || l.starts_with('\t'))
                .map(|l| l.trim())
                .filter(|l| l.contains(' ') && l.contains('v'))
                .take(20)
                .collect();
            if !go_mods.is_empty() {
                deps_section.push_str(&format!("**go.mod requires:** {}\n", go_mods.join(", ")));
            }
        }
    }

    if deps_section.is_empty() {
        deps_section.push_str("(no recognised dependency manifest found)\n");
    }

    let generated_at = chrono::Local::now().to_rfc3339();

    let content = format!(
        "# RTB Agent Workspace Context: {name}\n\n\
         ## Project Info\n\
         - **Project Path**: {path}\n\
         - **Status**: {status}\n\
         - **Detected Stack**: {stack}\n\
         - **Git Branch**: {branch}\n\
         - **Generated At**: {generated_at}\n\n\
         ## README Preview\n{readme}\n\n\
         ## Git Context\n\n### Last 10 Commits\n{log}\n\n### Current Diff (--stat HEAD)\n{diff}\n\n\
         ## Dependencies\n{deps}",
        name = project.name,
        path = project.path.display(),
        status = project.status.label(),
        stack = stack_str,
        branch = branch_str,
        generated_at = generated_at,
        readme = readme_str,
        log = git_log_indented,
        diff = git_diff_stat,
        deps = deps_section
    );

    if std::fs::write(&context_path, content).is_ok() {
        Some(context_path)
    } else {
        None
    }
}

pub fn launch_agent(project: &Project, agent_cmd: Option<&str>) -> bool {
    let _ = create_agent_context_file(project);

    let cmd_to_run = match agent_cmd {
        Some(cmd) => cmd.to_string(),
        None => match get_default_agent() {
            Some(agent) => agent.command,
            None => return false,
        },
    };

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("cmd")
            .args(["/C", "start", &cmd_to_run])
            .current_dir(&project.path)
            .spawn();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new(&cmd_to_run)
            .current_dir(&project.path)
            .spawn();
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_installed_agents_returns_eight_agents() {
        let agents = get_installed_agents();
        assert_eq!(agents.len(), 8);
        assert_eq!(agents[0].command, "agy");
        assert_eq!(agents[1].command, "claude");
        assert_eq!(agents[2].command, "gemini");
        assert_eq!(agents[3].command, "codex");
        assert_eq!(agents[4].command, "cursor");
        assert_eq!(agents[5].command, "windsurf");
        assert_eq!(agents[6].command, "aider");
        assert_eq!(agents[7].command, "openhands");
    }

    #[test]
    fn test_is_command_installed_non_existent() {
        assert!(!is_command_installed("non_existent_command_12345"));
    }

    #[test]
    fn test_create_agent_context_file_basic() {
        use crate::data::project::ProjectStatus;
        let temp_dir = std::env::temp_dir().join("rtb_agent_context_rust_test_basic");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let proj = Project {
            name: "test_proj".into(),
            path: temp_dir.clone(),
            status: ProjectStatus::Active,
            stack: vec!["Rust".into(), "Ratatui".into()],
            last_modified: None,
            total_size_bytes: 0,
            dep_size_bytes: 0,
            git: None,
            readme_preview: Some("Test README Header".into()),
            is_monorepo: false,
            ci_cd: None,
            runtime_version: None,
            dev_command: None,
        };

        let context_file = create_agent_context_file(&proj);
        assert!(context_file.is_some());
        let path = context_file.unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("# RTB Agent Workspace Context: test_proj"));
        assert!(content.contains("## Project Info"));
        assert!(content.contains("- **Status**: Active"));
        assert!(content.contains("- **Detected Stack**: Rust, Ratatui"));
        assert!(content.contains("- **Git Branch**: unknown"));
        assert!(content.contains("- **Generated At**:"));
        assert!(content.contains("## README Preview"));
        assert!(content.contains("Test README Header"));
        assert!(content.contains("## Git Context"));
        assert!(content.contains("### Last 10 Commits"));
        assert!(content.contains("  (not a git repository)"));
        assert!(content.contains("### Current Diff (--stat HEAD)"));
        assert!(content.contains("## Dependencies"));
        assert!(content.contains("(no recognised dependency manifest found)"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_create_agent_context_file_with_deps() {
        use crate::data::project::ProjectStatus;
        let temp_dir = std::env::temp_dir().join("rtb_agent_context_rust_test_deps");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Write package.json
        let pkg_content = r#"{"dependencies": {"react": "^18.0.0"}, "devDependencies": {"typescript": "^5.0.0"}}"#;
        std::fs::write(temp_dir.join("package.json"), pkg_content).unwrap();

        // Write Cargo.toml
        let cargo_content = "ratatui = \"0.29\"\ncrossterm = \"0.28\"\n";
        std::fs::write(temp_dir.join("Cargo.toml"), cargo_content).unwrap();

        // Write requirements.txt
        let req_content = "flask>=2.0.0\n# comment\nrequests==2.28.1\n";
        std::fs::write(temp_dir.join("requirements.txt"), req_content).unwrap();

        // Write go.mod
        let gomod_content = "module example.com/app\n\ngo 1.20\n\nrequire (\n\tgithub.com/gin-gonic/gin v1.9.1\n)\n";
        std::fs::write(temp_dir.join("go.mod"), gomod_content).unwrap();

        let proj = Project {
            name: "multi_dep_proj".into(),
            path: temp_dir.clone(),
            status: ProjectStatus::Paused,
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
        };

        let context_file = create_agent_context_file(&proj);
        assert!(context_file.is_some());
        let path = context_file.unwrap();
        let content = std::fs::read_to_string(path).unwrap();

        assert!(content.contains("- **Status**: Paused"));
        assert!(content.contains("- **Detected Stack**: Unknown"));
        assert!(content.contains("(no README)"));
        assert!(content.contains("**package.json deps:** react"));
        assert!(content.contains("**devDependencies:** typescript"));
        assert!(content.contains("**Cargo.toml crates:** ratatui, crossterm"));
        assert!(content.contains("**requirements.txt:** flask>=2.0.0, requests==2.28.1"));
        assert!(content.contains("**go.mod requires:** github.com/gin-gonic/gin v1.9.1"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_create_agent_context_file_zero_commits() {
        use crate::data::project::ProjectStatus;
        let temp_dir = std::env::temp_dir().join("rtb_agent_context_rust_test_zero_commits");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Initialize a clean 0-commit git repository
        let _ = std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&temp_dir)
            .output();

        let proj = Project {
            name: "zero_commit_proj".into(),
            path: temp_dir.clone(),
            status: ProjectStatus::Active,
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
        };

        let context_file = create_agent_context_file(&proj);
        assert!(context_file.is_some());
        let path = context_file.unwrap();
        let content = std::fs::read_to_string(path).unwrap();

        assert!(content.contains("### Last 10 Commits"));
        assert!(content.contains("  (no commits)"));
        assert!(content.contains("### Current Diff (--stat HEAD)"));
        assert!(content.contains("  (working tree clean)"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
