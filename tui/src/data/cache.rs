use crate::data::disk::DiskStats;
use crate::data::project::Project;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CACHE_PATH: &str = "D:\\05-Config\\dev.cache.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkspaceCache {
    pub timestamp: i64,
    pub projects: Vec<Project>,
    pub disk_stats: DiskStats,
}

pub fn load_cache() -> Option<(Vec<Project>, DiskStats)> {
    let path = Path::new(CACHE_PATH);
    if !path.exists() {
        return None;
    }

    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(cache) = serde_json::from_str::<WorkspaceCache>(&content) {
            return Some((cache.projects, cache.disk_stats));
        }
    }
    None
}

pub fn save_cache(projects: &[Project], disk_stats: &DiskStats) -> Result<()> {
    let cache = WorkspaceCache {
        timestamp: chrono::Utc::now().timestamp(),
        projects: projects.to_vec(),
        disk_stats: disk_stats.clone(),
    };

    if let Ok(json) = serde_json::to_string_pretty(&cache) {
        let parent = Path::new(CACHE_PATH).parent().unwrap_or_else(|| Path::new("D:\\05-Config"));
        let _ = fs::create_dir_all(parent);
        let _ = fs::write(CACHE_PATH, json);
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SessionState {
    pub active_tab: usize,
    pub selected_project_name: Option<String>,
}

impl SessionState {
    pub fn new(active_tab: usize, selected_project_name: Option<String>) -> Self {
        Self {
            active_tab,
            selected_project_name,
        }
    }

    pub fn session_state_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            let rtb_dir = config_dir.join("rtb");
            let _ = fs::create_dir_all(&rtb_dir);
            rtb_dir.join("state.json")
        } else {
            let fallback = PathBuf::from("config/.rtb_state.json");
            if let Some(parent) = fallback.parent() {
                let _ = fs::create_dir_all(parent);
            }
            fallback
        }
    }

    pub fn load() -> Option<Self> {
        let path = Self::session_state_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str::<SessionState>(&content) {
                    return Some(state);
                }
            }
        }
        None
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::session_state_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_serialization() {
        let state = SessionState {
            active_tab: 2,
            selected_project_name: Some("rtb-command-tool".into()),
        };

        let json = serde_json::to_string(&state).unwrap();
        let restored: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, restored);
    }

    #[test]
    fn test_session_state_save_load() {
        let state = SessionState {
            active_tab: 3,
            selected_project_name: Some("test-project".into()),
        };

        assert!(state.save().is_ok());

        let loaded = SessionState::load();
        assert!(loaded.is_some());
        let loaded_state = loaded.unwrap();
        assert_eq!(loaded_state.active_tab, 3);
        assert_eq!(loaded_state.selected_project_name.as_deref(), Some("test-project"));
    }
}
