use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DevConfig {
    pub version: String,
    pub project_roots: ProjectRoots,
    pub backup_root: String,
    pub config_root: String,
    pub template_dir: String,
    pub clean_deps: CleanDepsConfig,
    pub stale_threshold_days: u64,
    pub git_health: GitHealthConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectRoots {
    pub active: String,
    pub paused: String,
    pub planning: String,
    pub testing: String,
    pub abandoned: String,
    pub production: String,
    pub staging: String,
    pub vibe: String,
    pub sandbox: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CleanDepsConfig {
    pub days_inactive: u64,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct GitHealthConfig {
    pub scan_roots: Vec<String>,
}

impl DevConfig {
    /// Returns the ordered list of config file paths to try, from highest to lowest priority.
    pub fn candidate_paths() -> Vec<std::path::PathBuf> {
        let mut paths: Vec<std::path::PathBuf> = Vec::new();

        // 1. User config dir: %APPDATA%\rtb\rtb.config.json (Windows)
        //                     ~/.config/rtb/rtb.config.json (Linux/macOS)
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("rtb").join("rtb.config.json"));
            paths.push(config_dir.join("rtb").join("dev.config.json"));
        }

        // 2. Next to the running binary (useful when installed via install.ps1)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                paths.push(exe_dir.join("rtb.config.json"));
            }
        }

        // 3. Relative repo fallback for OSS contributors running from source
        paths.push(PathBuf::from("config/rtb.config.json"));
        paths.push(PathBuf::from("config/dev.config.json"));
        paths.push(PathBuf::from("../config/rtb.config.json"));

        paths
    }

    pub fn load() -> Result<Self> {
        for path in Self::candidate_paths() {
            if path.is_file() {
                let content = std::fs::read_to_string(&path)
                    .with_context(|| format!("Cannot read config from {}", path.display()))?;
                return serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse config file {}", path.display()));
            }
        }

        anyhow::bail!(
            "No rtb.config.json found.\n\
             Run 'rtb init' to create your workspace configuration.\n\
             Searched:\n{}",
            Self::candidate_paths()
                .iter()
                .map(|p| format!("  - {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_candidate_paths_contain_no_hardcoded_personal_paths() {
        let paths = DevConfig::candidate_paths();
        assert!(!paths.is_empty(), "Candidate paths should not be empty");

        // The only path that may contain the current working repo path is the dynamic exe dir.
        // No path should contain legacy hardcoded 'dev-cli'.
        for p in &paths {
            let s = p.to_string_lossy();
            assert!(
                !s.contains("dev-cli"),
                "Hardcoded personal path found: {}",
                s
            );
        }

        // Relative fallback paths and config_dir paths must not hardcode personal absolute paths.
        let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
        for p in &paths {
            if let Some(ref dir) = exe_dir {
                if p.starts_with(dir) {
                    continue;
                }
            }
            let s = p.to_string_lossy();
            assert!(
                !s.contains("02-Projects"),
                "Hardcoded personal path found: {}",
                s
            );
        }

        // Relative paths must include standard fallback locations
        assert!(paths.contains(&PathBuf::from("config/rtb.config.json")));
        assert!(paths.contains(&PathBuf::from("config/dev.config.json")));
        assert!(paths.contains(&PathBuf::from("../config/rtb.config.json")));
    }

    #[test]
    fn config_deserialize_sample() {
        let sample_json = r#"{
            "version": "0.2.0-beta",
            "projectRoots": {
                "active": "D:/Projects/Active",
                "paused": "D:/Projects/Paused",
                "planning": "D:/Projects/Planning",
                "testing": "D:/Projects/Testing",
                "abandoned": "D:/Projects/Abandoned",
                "production": "D:/Projects/Production",
                "staging": "D:/Projects/Staging",
                "vibe": "D:/Projects/Vibe",
                "sandbox": "D:/Projects/Sandbox"
            },
            "backupRoot": "D:/Backups",
            "configRoot": "D:/Config",
            "templateDir": "D:/Templates",
            "cleanDeps": {
                "daysInactive": 30,
                "targets": ["node_modules", "target"]
            },
            "staleThresholdDays": 60,
            "gitHealth": {
                "scanRoots": ["D:/Projects"]
            }
        }"#;

        let config: Result<DevConfig, _> = serde_json::from_str(sample_json);
        assert!(config.is_ok(), "Sample config should parse correctly");
        let config = config.unwrap();
        assert_eq!(config.version, "0.2.0-beta");
        assert_eq!(config.clean_deps.days_inactive, 30);
    }
}

