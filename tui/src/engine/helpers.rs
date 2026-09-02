use anyhow::Result;
use std::path::{Path, PathBuf};
use crate::config::DevConfig;
use crate::data::scanner::scan_all_projects;
use crate::engine::Cli;

pub fn dir_size(path: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

pub fn to_kebab_case(name: &str) -> String {
    let mut kebab = String::new();
    let mut last_was_dash = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            kebab.push(c.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            kebab.push('-');
            last_was_dash = true;
        }
    }
    kebab.trim_matches('-').to_string()
}

pub fn is_git_clean(path: &Path) -> bool {
    if let Some(porcelain) = crate::data::scanner::run_git(path, &["status", "--porcelain"]) {
        porcelain.lines().filter(|l| !l.trim().is_empty()).count() == 0
    } else {
        true
    }
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub fn move_dir_all(src: &Path, dst: &Path) -> Result<()> {
    if std::fs::rename(src, dst).is_ok() {
        return Ok(());
    }
    copy_dir_all(src, dst)?;
    std::fs::remove_dir_all(src)?;
    Ok(())
}

pub fn resolve_project_or_cwd(project_name: Option<&str>, cli: &Cli) -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    if let Some(name) = project_name {
        if let Ok(p) = std::fs::canonicalize(name) {
            if p.is_dir() {
                return Ok(p);
            }
        }
        if let Ok(config) = DevConfig::load_from(&cli.config) {
            let projects = scan_all_projects(&config);
            if let Some(p) = projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                return Ok(p.path.clone());
            }
        }
        let kebab = to_kebab_case(name);
        let candidate = cwd.join(&kebab);
        if candidate.is_dir() {
            return Ok(candidate);
        }
        eprintln!("Project '{}' not found.", name);
        anyhow::bail!("Project not found");
    }
    Ok(cwd)
}

pub fn get_cmd(cmd: &str) -> String {
    if cfg!(windows) {
        match cmd {
            "npm" => "npm.cmd".to_string(),
            "npx" => "npx.cmd".to_string(),
            "pnpm" => "pnpm.cmd".to_string(),
            "yarn" => "yarn.cmd".to_string(),
            _ => cmd.to_string(),
        }
    } else {
        cmd.to_string()
    }
}
