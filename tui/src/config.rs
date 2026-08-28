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
    pub fn load() -> Result<Self> {
        let mut candidate_paths = Vec::new();

        if let Some(config_dir) = dirs::config_dir() {
            candidate_paths.push(config_dir.join("rtb").join("rtb.config.json"));
        }
        candidate_paths.push(PathBuf::from("D:\\02-Projects\\01-Development\\01-Active\\dev-tools\\config\\rtb.config.json"));
        candidate_paths.push(PathBuf::from("..\\config\\rtb.config.json"));
        candidate_paths.push(PathBuf::from("config\\rtb.config.json"));
        candidate_paths.push(PathBuf::from("config\\dev.config.json"));

        for path in &candidate_paths {
            if path.exists() {
                let content = std::fs::read_to_string(path)
                    .with_context(|| format!("Cannot read config from {}", path.display()))?;
                return serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse config file {}", path.display()));
            }
        }

        anyhow::bail!("Cannot find rtb.config.json in any expected path")
    }
}
