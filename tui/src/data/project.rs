use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Paused,
    Production,
    Staging,
    Vibe,
    Sandbox,
    Planning,
    Testing,
    Abandoned,
}

#[allow(dead_code)]
impl ProjectStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            ProjectStatus::Active => "🟢",
            ProjectStatus::Paused => "⏸ ",
            ProjectStatus::Production => "🚀",
            ProjectStatus::Staging => "🧪",
            ProjectStatus::Vibe => "⚡",
            ProjectStatus::Sandbox => "🔬",
            ProjectStatus::Planning => "📝",
            ProjectStatus::Testing => "🧪",
            ProjectStatus::Abandoned => "❌",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ProjectStatus::Active => "Active",
            ProjectStatus::Paused => "Paused",
            ProjectStatus::Production => "Production",
            ProjectStatus::Staging => "Staging",
            ProjectStatus::Vibe => "Vibe",
            ProjectStatus::Sandbox => "Sandbox",
            ProjectStatus::Planning => "Planning",
            ProjectStatus::Testing => "Testing",
            ProjectStatus::Abandoned => "Abandoned",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub status: ProjectStatus,
    pub stack: Vec<String>,
    pub last_modified: Option<chrono::DateTime<chrono::Local>>,
    pub total_size_bytes: u64,
    pub dep_size_bytes: u64,
    pub git: Option<GitInfo>,
    pub readme_preview: Option<String>,
    pub is_monorepo: bool,
    pub ci_cd: Option<String>,
    pub runtime_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub branch: String,
    pub uncommitted: u32,
    pub unpushed: u32,
    pub last_commit_msg: Option<String>,
    pub last_commit_relative: Option<String>,
    pub has_remote: bool,
}

impl Project {
    pub fn last_modified_str(&self) -> String {
        match &self.last_modified {
            Some(dt) => dt.format("%Y-%m-%d").to_string(),
            None => "-".into(),
        }
    }

    pub fn total_size_str(&self) -> String {
        format_bytes(self.total_size_bytes)
    }

    pub fn dep_size_str(&self) -> String {
        format_bytes(self.dep_size_bytes)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "-".into();
    }
    if bytes < 1_024 {
        format!("{} B", bytes)
    } else if bytes < 1_024 * 1_024 {
        format!("{:.1} KB", bytes as f64 / 1_024.0)
    } else if bytes < 1_024 * 1_024 * 1_024 {
        format!("{:.1} MB", bytes as f64 / (1_024.0 * 1_024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1_024.0 * 1_024.0 * 1_024.0))
    }
}
