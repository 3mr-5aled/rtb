use crate::config::DevConfig;
use crate::data::project::Project;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn pause_project(project: &Project, config: &DevConfig) -> Result<PathBuf> {
    let target_dir = Path::new(&config.project_roots.paused);
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir)?;
    }

    let target_path = target_dir.join(&project.name);
    if target_path.exists() {
        anyhow::bail!("Target already exists in Paused: {}", target_path.display());
    }

    std::fs::rename(&project.path, &target_path)
        .with_context(|| format!("Failed to move {} to Paused", project.name))?;

    Ok(target_path)
}

pub fn resume_project(project: &Project, config: &DevConfig) -> Result<PathBuf> {
    let target_dir = Path::new(&config.project_roots.active);
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir)?;
    }

    let target_path = target_dir.join(&project.name);
    if target_path.exists() {
        anyhow::bail!("Target already exists in Active: {}", target_path.display());
    }

    std::fs::rename(&project.path, &target_path)
        .with_context(|| format!("Failed to move {} to Active", project.name))?;

    Ok(target_path)
}

pub fn deploy_project(project: &Project, config: &DevConfig, production: bool) -> Result<PathBuf> {
    let root_str = if production {
        &config.project_roots.production
    } else {
        &config.project_roots.staging
    };

    let target_dir = Path::new(root_str);
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir)?;
    }

    let target_path = target_dir.join(&project.name);
    if target_path.exists() {
        anyhow::bail!("Target already exists in Deployed: {}", target_path.display());
    }

    std::fs::rename(&project.path, &target_path)
        .with_context(|| format!("Failed to deploy {}", project.name))?;

    Ok(target_path)
}

pub fn archive_project(project: &Project, config: &DevConfig) -> Result<PathBuf> {
    let backup_dir = Path::new(&config.backup_root).join("project-snapshots");
    if !backup_dir.exists() {
        std::fs::create_dir_all(&backup_dir)?;
    }

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let archive_name = format!("{}-{}.tar.gz", project.name, today);
    let archive_path = backup_dir.join(&archive_name);

    // Create tar.gz using native Windows tar
    let parent_dir = project.path.parent().unwrap_or(Path::new("D:\\"));
    let status = Command::new("tar.exe")
        .args([
            "-czf",
            &archive_path.to_string_lossy(),
            &project.name,
        ])
        .current_dir(parent_dir)
        .status()
        .with_context(|| "Failed to execute tar.exe")?;

    if !status.success() {
        anyhow::bail!("tar.exe failed with exit code: {:?}", status.code());
    }

    // Delete the original project folder
    std::fs::remove_dir_all(&project.path)
        .with_context(|| format!("Failed to delete original project folder {}", project.path.display()))?;

    Ok(archive_path)
}

pub fn open_in_editor(project: &Project) {
    let _ = Command::new("cmd")
        .args(["/C", "code", &project.path.to_string_lossy()])
        .spawn();
}

pub fn open_in_explorer(project: &Project) {
    let _ = Command::new("explorer")
        .arg(&project.path)
        .spawn();
}

pub fn run_live_program(project: &Project) {
    let dev_cmd = project.get_dev_command();
    let ps_cmd = format!("cd /d '{}'; {}; Read-Host 'Press Enter to close window...'", project.path.to_string_lossy(), dev_cmd);
    let _ = Command::new("cmd")
        .args(["/C", "start", "powershell", "-NoExit", "-Command", &ps_cmd])
        .spawn();
}
