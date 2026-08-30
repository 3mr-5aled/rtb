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

pub fn create_agent_context_file(project: &Project) -> Option<std::path::PathBuf> {
    let context_path = project.path.join(".rtb_context.md");
    let stack_str = if project.stack.is_empty() { "-".into() } else { project.stack.join(", ") };
    let branch_str = project.git.as_ref().map(|g| g.branch.as_str()).unwrap_or("-");
    let readme_str = project.readme_preview.as_ref().and_then(|r| r.lines().next()).unwrap_or("-");

    let content = format!(
        "# RTB Agent Workspace Context: {}\n\n- **Project Path**: {}\n- **Status**: {:?}\n- **Detected Stack**: {}\n- **Git Branch**: {}\n- **README**: {}\n",
        project.name,
        project.path.display(),
        project.status,
        stack_str,
        branch_str,
        readme_str
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
    fn test_create_agent_context_file() {
        use crate::data::project::ProjectStatus;
        let temp_dir = std::env::temp_dir().join("rtb_agent_context_rust_test");
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
        assert!(content.contains("Rust, Ratatui"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
