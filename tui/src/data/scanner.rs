use crate::config::DevConfig;
use crate::data::project::{GitInfo, Project, ProjectStatus};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

struct RootDef {
    path: String,
    status: ProjectStatus,
}

pub fn scan_all_projects(config: &DevConfig) -> Vec<Project> {
    let roots = vec![
        RootDef { path: config.project_roots.active.clone(), status: ProjectStatus::Active },
        RootDef { path: config.project_roots.paused.clone(), status: ProjectStatus::Paused },
        RootDef { path: config.project_roots.production.clone(), status: ProjectStatus::Production },
        RootDef { path: config.project_roots.staging.clone(), status: ProjectStatus::Staging },
        RootDef { path: config.project_roots.vibe.clone(), status: ProjectStatus::Vibe },
        RootDef { path: config.project_roots.sandbox.clone(), status: ProjectStatus::Sandbox },
        RootDef { path: config.project_roots.planning.clone(), status: ProjectStatus::Planning },
        RootDef { path: config.project_roots.testing.clone(), status: ProjectStatus::Testing },
        RootDef { path: config.project_roots.abandoned.clone(), status: ProjectStatus::Abandoned },
    ];

    // Collect all candidate project directories
    let mut dir_entries: Vec<(PathBuf, ProjectStatus)> = Vec::new();

    for root_def in &roots {
        let root_path = Path::new(&root_def.path);
        if !root_path.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(root_path) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    dir_entries.push((entry.path(), root_def.status.clone()));
                }
            }
        }
    }

    // Parallel scan across all CPU cores via Rayon
    let mut projects: Vec<Project> = dir_entries
        .into_par_iter()
        .map(|(path, status)| scan_project(&path, status))
        .collect();

    // Deterministic sorting: Group by status priority, then by last_modified descending
    projects.sort_by(|a, b| {
        let status_order_a = status_priority(&a.status);
        let status_order_b = status_priority(&b.status);

        if status_order_a != status_order_b {
            status_order_a.cmp(&status_order_b)
        } else {
            b.last_modified.cmp(&a.last_modified)
        }
    });

    projects
}

fn status_priority(status: &ProjectStatus) -> usize {
    match status {
        ProjectStatus::Active => 0,
        ProjectStatus::Paused => 1,
        ProjectStatus::Production => 2,
        ProjectStatus::Staging => 3,
        ProjectStatus::Vibe => 4,
        ProjectStatus::Sandbox => 5,
        ProjectStatus::Planning => 6,
        ProjectStatus::Testing => 7,
        ProjectStatus::Abandoned => 8,
    }
}

fn scan_project(path: &Path, status: ProjectStatus) -> Project {
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let stack = detect_stack(path);
    let last_modified = get_last_modified(path);
    let (total_size_bytes, dep_size_bytes) = get_sizes(path);
    let git = get_git_info(path);
    let readme_preview = get_readme_preview(path);
    let is_monorepo = detect_monorepo(path);
    let ci_cd = detect_ci_cd(path);
    let runtime_version = detect_runtime_version(path);

    Project {
        name,
        path: path.to_path_buf(),
        status,
        stack,
        last_modified,
        total_size_bytes,
        dep_size_bytes,
        git,
        readme_preview,
        is_monorepo,
        ci_cd,
        runtime_version,
        dev_command: None,
    }
}

fn detect_stack(path: &Path) -> Vec<String> {
    let mut stack = Vec::new();

    // Package managers & Lockfiles
    if path.join("pnpm-lock.yaml").exists() {
        stack.push("pnpm".into());
    } else if path.join("yarn.lock").exists() {
        stack.push("yarn".into());
    } else if path.join("bun.lockb").exists() || path.join("bun.lock").exists() {
        stack.push("bun".into());
    } else if path.join("package-lock.json").exists() {
        stack.push("npm".into());
    }

    let pkg_path = path.join("package.json");
    if pkg_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                let deps = pkg.get("dependencies").and_then(|d| d.as_object());
                let dev_deps = pkg.get("devDependencies").and_then(|d| d.as_object());

                let all_keys: Vec<String> = [
                    deps.map(|d| d.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
                    dev_deps.map(|d| d.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
                ]
                .concat();

                if all_keys.iter().any(|k| k == "next") {
                    stack.push("Next.js".into());
                } else if all_keys.iter().any(|k| k == "react") {
                    stack.push("React".into());
                } else if all_keys.iter().any(|k| k == "vue") {
                    stack.push("Vue".into());
                } else if all_keys.iter().any(|k| k == "vite") {
                    stack.push("Vite".into());
                }

                if all_keys.iter().any(|k| k == "tailwindcss") {
                    stack.push("Tailwind".into());
                }
                if all_keys.iter().any(|k| k == "prisma" || k == "@prisma/client") {
                    stack.push("Prisma".into());
                }
                if all_keys.iter().any(|k| k == "typescript") {
                    stack.push("TypeScript".into());
                }
                if all_keys.iter().any(|k| k == "express") {
                    stack.push("Express".into());
                } else if all_keys.iter().any(|k| k == "fastify") {
                    stack.push("Fastify".into());
                }
            }
        }
        if !stack.iter().any(|s| s == "Next.js" || s == "React" || s == "Vue" || s == "Vite" || s == "Node.js") {
            stack.push("Node.js".into());
        }
    }

    // Python runtimes and package managers
    if path.join("uv.lock").exists() {
        stack.push("uv".into());
        stack.push("Python".into());
    } else if path.join("poetry.lock").exists() {
        stack.push("poetry".into());
        stack.push("Python".into());
    } else if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
        stack.push("Python".into());
    }

    if path.join("Cargo.toml").exists() {
        stack.push("Cargo".into());
        stack.push("Rust".into());
    }

    if path.join("go.mod").exists() {
        stack.push("Go".into());
    }

    if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
        stack.push("Java".into());
    }

    if path.join("Dockerfile").exists() {
        stack.push("Docker".into());
    }
    if path.join("docker-compose.yml").exists() || path.join("docker-compose.yaml").exists() {
        stack.push("Compose".into());
    }

    if path.join("rtb.psm1").exists() || path.join("rtb.psd1").exists() || path.join("dev.psm1").exists() {
        stack.push("PowerShell".into());
    }

    let has_dotnet = std::fs::read_dir(path)
        .ok()
        .map(|entries| {
            entries.flatten().any(|e| {
                let name = e.file_name().to_string_lossy().to_lowercase();
                name.ends_with(".csproj") || name.ends_with(".sln")
            })
        })
        .unwrap_or(false);

    if has_dotnet {
        stack.push(".NET".into());
    }

    if stack.is_empty() {
        stack.push("-".into());
    }

    stack
}

fn detect_monorepo(path: &Path) -> bool {
    if path.join("pnpm-workspace.yaml").exists()
        || path.join("lerna.json").exists()
        || path.join("nx.json").exists()
        || path.join("turbo.json").exists()
    {
        return true;
    }

    let pkg_path = path.join("package.json");
    if pkg_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                if pkg.get("workspaces").is_some() {
                    return true;
                }
            }
        }
    }

    false
}

fn detect_ci_cd(path: &Path) -> Option<String> {
    if path.join(".github").join("workflows").exists() {
        Some("GitHub Actions".into())
    } else if path.join(".gitlab-ci.yml").exists() {
        Some("GitLab CI".into())
    } else if path.join("azure-pipelines.yml").exists() {
        Some("Azure Pipelines".into())
    } else if path.join(".circleci").exists() {
        Some("CircleCI".into())
    } else {
        None
    }
}

fn detect_runtime_version(path: &Path) -> Option<String> {
    let nvmrc = path.join(".nvmrc");
    if nvmrc.exists() {
        if let Ok(content) = std::fs::read_to_string(&nvmrc) {
            let trimmed = content.lines().next().unwrap_or("").trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    let py_ver = path.join(".python-version");
    if py_ver.exists() {
        if let Ok(content) = std::fs::read_to_string(&py_ver) {
            let trimmed = content.lines().next().unwrap_or("").trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    let rust_tc = path.join("rust-toolchain.toml");
    if rust_tc.exists() {
        if let Ok(content) = std::fs::read_to_string(&rust_tc) {
            for line in content.lines() {
                let line_trim = line.trim();
                if line_trim.starts_with("channel") {
                    if let Some(val) = line_trim.split('=').nth(1) {
                        let clean = val.trim().trim_matches('"').trim_matches('\'').trim();
                        if !clean.is_empty() {
                            return Some(clean.to_string());
                        }
                    }
                }
            }
        }
    }

    let pkg_path = path.join("package.json");
    if pkg_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(engines) = pkg.get("engines").and_then(|e| e.as_object()) {
                    if let Some(node_ver) = engines.get("node").and_then(|v| v.as_str()) {
                        if !node_ver.is_empty() {
                            return Some(node_ver.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn test_detect_dotnet_stack() {
        let temp_dir = std::env::temp_dir().join("rtb_test_dotnet");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        File::create(temp_dir.join("TestApp.csproj")).unwrap();
        let stack = detect_stack(&temp_dir);
        assert!(stack.contains(&".NET".to_string()));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_monorepo() {
        let temp_dir = std::env::temp_dir().join("rtb_test_monorepo");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        assert!(!detect_monorepo(&temp_dir));

        File::create(temp_dir.join("pnpm-workspace.yaml")).unwrap();
        assert!(detect_monorepo(&temp_dir));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_ci_cd() {
        let temp_dir = std::env::temp_dir().join("rtb_test_cicd");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        assert_eq!(detect_ci_cd(&temp_dir), None);

        let workflows = temp_dir.join(".github").join("workflows");
        fs::create_dir_all(&workflows).unwrap();
        assert_eq!(detect_ci_cd(&temp_dir), Some("GitHub Actions".to_string()));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_runtime_version() {
        let temp_dir = std::env::temp_dir().join("rtb_test_runtime_ver");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        assert_eq!(detect_runtime_version(&temp_dir), None);

        let mut file = File::create(temp_dir.join(".nvmrc")).unwrap();
        writeln!(file, "v20.11.0").unwrap();
        assert_eq!(detect_runtime_version(&temp_dir), Some("v20.11.0".to_string()));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_project_inspector_detects_nextjs_and_monorepo() {
        let temp_dir = std::env::temp_dir().join("rtb_rust_inspector_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let pkg_json = r#"{"name":"my-app","dependencies":{"next":"14.0.0","tailwindcss":"3.0.0"}}"#;
        fs::write(temp_dir.join("package.json"), pkg_json).unwrap();
        fs::write(temp_dir.join("pnpm-workspace.yaml"), "packages: ['*']").unwrap();

        let project = scan_project(&temp_dir, ProjectStatus::Active);
        assert!(project.stack.contains(&"Next.js".to_string()));
        assert!(project.stack.contains(&"Tailwind".to_string()));
        assert!(project.is_monorepo);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}

fn get_last_modified(path: &Path) -> Option<chrono::DateTime<chrono::Local>> {
    let mut latest: Option<std::time::SystemTime> = None;

    let walker = WalkDir::new(path)
        .max_depth(3)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !matches!(
                name.as_ref(),
                "node_modules" | ".git" | "dist" | "build" | ".next" | "target" | "__pycache__" | ".venv"
            )
        });

    for entry in walker.flatten() {
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    match &latest {
                        None => latest = Some(modified),
                        Some(prev) if modified > *prev => latest = Some(modified),
                        _ => {}
                    }
                }
            }
        }
    }

    latest.map(|t| chrono::DateTime::<chrono::Local>::from(t))
}

fn get_sizes(path: &Path) -> (u64, u64) {
    let dep_dirs = ["node_modules", ".next", "dist", "build", "target", ".venv", "__pycache__"];
    let mut total: u64 = 0;
    let mut dep_total: u64 = 0;

    let walker = WalkDir::new(path).max_depth(3).into_iter().filter_entry(|e| {
        let depth = e.depth();
        if depth > 1 {
            let name = e.file_name().to_string_lossy();
            !dep_dirs.contains(&name.as_ref())
        } else {
            true
        }
    });

    for entry in walker.flatten() {
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }

    for dep_dir in &dep_dirs {
        let dep_path = path.join(dep_dir);
        if dep_path.exists() {
            let size = WalkDir::new(&dep_path)
                .max_depth(3)
                .into_iter()
                .flatten()
                .filter(|e| e.file_type().is_file())
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum::<u64>();
            dep_total += size;
            total += size;
        }
    }

    (total, dep_total)
}

fn get_git_info(path: &Path) -> Option<GitInfo> {
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        return None;
    }

    // Call 1: Get branch AND uncommitted files in one shot via status --porcelain=v1 -b
    let status_output = run_git(path, &["status", "--porcelain=v1", "-b"])?;
    let mut lines = status_output.lines();

    let header = lines.next().unwrap_or("");
    let branch = if header.starts_with("## ") {
        let branch_part = &header[3..];
        let branch_name = branch_part.split("...").next().unwrap_or("unknown");
        branch_name.trim().to_string()
    } else {
        "unknown".to_string()
    };

    let uncommitted = lines.count() as u32;

    // Call 2: Get commit msg AND relative date in one shot
    let log_output = run_git(path, &["log", "-1", "--format=%s§%cr"]);
    let (last_commit_msg, last_commit_relative) = match log_output {
        Some(ref s) if s.contains('§') => {
            let mut parts = s.split('§');
            (parts.next().map(|m| m.to_string()), parts.next().map(|d| d.to_string()))
        }
        Some(s) => (Some(s), None),
        None => (None, None),
    };

    // Call 3: Unpushed commits
    let unpushed = run_git(path, &["log", "@{u}..", "--oneline"])
        .map(|s| s.lines().filter(|l| !l.is_empty()).count() as u32)
        .unwrap_or(0);

    // Call 4: Remotes check
    let has_remote = run_git(path, &["remote"])
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    Some(GitInfo {
        branch,
        uncommitted,
        unpushed,
        last_commit_msg,
        last_commit_relative,
        has_remote,
    })
}

fn run_git(path: &Path, args: &[&str]) -> Option<String> {
    std::process::Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn get_readme_preview(path: &Path) -> Option<String> {
    for name in &["README.md", "readme.md", "README.txt"] {
        let readme_path = path.join(name);
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                let preview: String = content
                    .lines()
                    .take(6)
                    .collect::<Vec<_>>()
                    .join("\n");
                return Some(preview);
            }
        }
    }
    None
}
