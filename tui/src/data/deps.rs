use crate::config::DevConfig;
use crate::data::project::ProjectStatus;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct DepFolder {
    pub project_name: String,
    pub project_status: ProjectStatus,
    pub path: PathBuf,
    pub rel_path: String,
    pub size_bytes: u64,
    pub last_modified: Option<chrono::DateTime<chrono::Local>>,
    pub is_selected: bool,
}

#[allow(dead_code)]
impl DepFolder {
    pub fn size_str(&self) -> String {
        format_bytes(self.size_bytes)
    }

    pub fn last_modified_str(&self) -> String {
        match &self.last_modified {
            Some(dt) => dt.format("%Y-%m-%d").to_string(),
            None => "-".into(),
        }
    }

    pub fn days_idle(&self) -> i64 {
        match self.last_modified {
            Some(dt) => (chrono::Local::now() - dt).num_days(),
            None => 0,
        }
    }
}

pub fn scan_dependencies(config: &DevConfig, _threshold_days: u64) -> Vec<DepFolder> {
    let scan_roots = [
        (&config.project_roots.active, ProjectStatus::Active),
        (&config.project_roots.paused, ProjectStatus::Paused),
        (&config.project_roots.production, ProjectStatus::Production),
        (&config.project_roots.staging, ProjectStatus::Staging),
        (&config.project_roots.vibe, ProjectStatus::Vibe),
        (&config.project_roots.sandbox, ProjectStatus::Sandbox),
        (&config.project_roots.abandoned, ProjectStatus::Abandoned),
    ];

    let target_names = &config.clean_deps.targets;
    let mut results = Vec::new();

    for (root_str, status) in &scan_roots {
        let root = Path::new(root_str);
        if !root.exists() {
            continue;
        }

        if let Ok(project_entries) = std::fs::read_dir(root) {
            for proj_entry in project_entries.flatten() {
                if !proj_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }

                let proj_path = proj_entry.path();
                let proj_name = proj_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                for target in target_names {
                    let dep_path = proj_path.join(target);
                    if dep_path.exists() && dep_path.is_dir() {
                        let size = calculate_dir_size(&dep_path);
                        if size > 0 {
                            let last_mod = get_dir_last_modified(&dep_path);
                            let rel_path = format!("{}/{}", proj_name, target);

                            // SAFE DEFAULT: Only pre-select Paused, Abandoned, or Sandbox projects
                            // Active and Production projects are UNCHECKED by default
                            let is_selected_by_default = matches!(
                                status,
                                ProjectStatus::Paused | ProjectStatus::Abandoned | ProjectStatus::Sandbox
                            );

                            results.push(DepFolder {
                                project_name: proj_name.clone(),
                                project_status: status.clone(),
                                path: dep_path,
                                rel_path,
                                size_bytes: size,
                                last_modified: last_mod,
                                is_selected: is_selected_by_default,
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort by size descending
    results.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    results
}

pub fn prune_selected_folders(folders: &mut [DepFolder]) -> (usize, u64) {
    let mut count = 0;
    let mut bytes_freed = 0;

    for folder in folders.iter_mut() {
        if folder.is_selected && folder.path.exists() {
            if std::fs::remove_dir_all(&folder.path).is_ok() {
                count += 1;
                bytes_freed += folder.size_bytes;
                folder.size_bytes = 0;
                folder.is_selected = false;
            }
        }
    }

    (count, bytes_freed)
}

fn calculate_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .max_depth(3)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

fn get_dir_last_modified(path: &Path) -> Option<chrono::DateTime<chrono::Local>> {
    let mut latest: Option<std::time::SystemTime> = None;
    for entry in WalkDir::new(path).max_depth(2).into_iter().flatten() {
        if let Ok(meta) = entry.metadata() {
            if let Ok(m) = meta.modified() {
                match &latest {
                    None => latest = Some(m),
                    Some(prev) if m > *prev => latest = Some(m),
                    _ => {}
                }
            }
        }
    }
    latest.map(|t| chrono::DateTime::<chrono::Local>::from(t))
}

pub fn format_bytes(bytes: u64) -> String {
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
