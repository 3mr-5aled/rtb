use crate::data::disk::DiskStats;
use crate::data::project::Project;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
