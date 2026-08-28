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

pub fn launch_agent(project: &Project, agent_cmd: Option<&str>) -> bool {
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
    fn test_get_installed_agents_returns_four_agents() {
        let agents = get_installed_agents();
        assert_eq!(agents.len(), 4);
        assert_eq!(agents[0].command, "agy");
        assert_eq!(agents[1].command, "claude");
        assert_eq!(agents[2].command, "gemini");
        assert_eq!(agents[3].command, "codex");
    }

    #[test]
    fn test_is_command_installed_non_existent() {
        assert!(!is_command_installed("non_existent_command_12345"));
    }
}
