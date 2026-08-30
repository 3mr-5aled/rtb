use crate::data::deps::format_bytes;
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DiskCategory {
    pub name: String,
    pub path_str: String,
    pub size_bytes: u64,
    pub percentage: f64,
}

#[allow(dead_code)]
impl DiskCategory {
    pub fn size_str(&self) -> String {
        format_bytes(self.size_bytes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DiskStats {
    pub total_d_drive_bytes: u64,
    pub free_d_drive_bytes: u64,
    pub used_d_drive_bytes: u64,
    pub categories: Vec<DiskCategory>,
}

pub fn calculate_disk_stats() -> DiskStats {
    let base_root = if let Ok(cfg) = crate::config::DevConfig::load() {
        if let Some(parent) = Path::new(&cfg.project_roots.active).parent().and_then(|p| p.parent()) {
            parent.to_path_buf()
        } else {
            Path::new(".").to_path_buf()
        }
    } else {
        Path::new(".").to_path_buf()
    };

    let category_defs = [
        "01-SandBox",
        "02-Projects",
        "03-Career",
        "04-Docs",
        "05-Config",
        "06-Tools",
        "07-Resources",
        "08-Backup",
    ];

    let mut categories = Vec::new();
    let mut total_categorized: u64 = 0;

    for name in &category_defs {
        let path = base_root.join(name);
        let path_str = path.to_string_lossy().to_string();
        let size = if path.exists() {
            calculate_shallow_dir_size(&path)
        } else {
            0
        };

        total_categorized += size;

        categories.push(DiskCategory {
            name: name.to_string(),
            path_str,
            size_bytes: size,
            percentage: 0.0,
        });
    }

    // Update percentages
    if total_categorized > 0 {
        for cat in &mut categories {
            cat.percentage = (cat.size_bytes as f64 / total_categorized as f64) * 100.0;
        }
    }

    // Sort categories by size descending
    categories.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    DiskStats {
        total_d_drive_bytes: 103_488_700_416, // ~96.38 GB (D drive capacity)
        free_d_drive_bytes: 49_394_000_000,
        used_d_drive_bytes: total_categorized,
        categories,
    }
}

fn calculate_shallow_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .max_depth(4)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !matches!(name.as_ref(), ".git" | "$RECYCLE.BIN" | "System Volume Information")
        })
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}
