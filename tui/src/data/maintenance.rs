use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TaskStatus {
    Pending,
    Running,
    Success,
    Warning,
    Failed,
}

#[allow(dead_code)]
impl TaskStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "⬚ ",
            TaskStatus::Running => "🔄",
            TaskStatus::Success => "✅",
            TaskStatus::Warning => "⚠ ",
            TaskStatus::Failed => "❌",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MaintenanceTask {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub target_info: &'static str,
    pub flags_info: &'static str,
    pub script_path: &'static str,
    pub args: Vec<&'static str>,
    pub status: TaskStatus,
    pub duration_ms: u128,
}

pub enum MaintenanceMessage {
    TaskStarted(usize),
    LogLine(String),
    TaskCompleted(usize, TaskStatus, u128),
    AllCompleted,
}

pub struct MaintenanceState {
    pub tasks: Vec<MaintenanceTask>,
    pub selected_task: usize,
    pub is_running: bool,
    pub logs: Vec<String>,
}

impl MaintenanceState {
    pub fn new() -> Self {
        let tasks = vec![
            MaintenanceTask {
                id: "guard",
                name: "1. Root Guardrail Audit",
                description: "Scans workspace root directory to detect and report unauthorized ghost folders, temporary test directories, or non-standard root clutter. Ensures the strict 8-folder architecture is maintained.",
                target_info: "Workspace Root",
                flags_info: "-ReportOnly",
                script_path: "scripts/guard-d-drive.ps1",
                args: vec!["-ReportOnly"],
                status: TaskStatus::Pending,
                duration_ms: 0,
            },
            MaintenanceTask {
                id: "clean",
                name: "2. Dependency Pruning Scan (Dry Run)",
                description: "Scans active and inactive development projects for stale build artifacts (node_modules, .venv, .next, dist, build, target, __pycache__) older than 60 days. Calculates reclaimable disk space without deleting files.",
                target_info: "Active Projects, SandBox",
                flags_info: "-DryRun -DaysInactive 60",
                script_path: "scripts/clean-deps.ps1",
                args: vec!["-DryRun", "-DaysInactive", "60"],
                status: TaskStatus::Pending,
                duration_ms: 0,
            },
            MaintenanceTask {
                id: "git",
                name: "3. Git Repository Health",
                description: "Inspects all Git repositories across the workspace. Checks for dirty working trees, uncommitted changes, unpushed commits, missing remotes, and stale branches older than 90 days.",
                target_info: "All Configured Scan Roots",
                flags_info: "Full Health Diagnostics",
                script_path: "scripts/git-health.ps1",
                args: vec![],
                status: TaskStatus::Pending,
                duration_ms: 0,
            },
            MaintenanceTask {
                id: "index",
                name: "4. Project Index Generator",
                description: "Auto-generates a master markdown index (PROJECT-INDEX.md) cataloging all projects with status badges, detected tech stack frameworks, and last modified timestamps.",
                target_info: "PROJECT-INDEX.md",
                flags_info: "UTF-8 Markdown Table",
                script_path: "scripts/project-index.ps1",
                args: vec![],
                status: TaskStatus::Pending,
                duration_ms: 0,
            },
            MaintenanceTask {
                id: "env",
                name: "5. Environment Secrets Backup",
                description: "Safely searches for gitignored .env and .env.local secret files containing API keys and database credentials, creating a date-stamped backup.",
                target_info: "Backup/env-secrets/",
                flags_info: "Safe Recursive Scans",
                script_path: "scripts/backup-env-files.ps1",
                args: vec![],
                status: TaskStatus::Pending,
                duration_ms: 0,
            },
            MaintenanceTask {
                id: "backup",
                name: "6. Configuration Snapshot Backup",
                description: "Creates a complete system backup: dotfiles (.gitconfig, .ssh), VS Code settings and extension lists, Windows Terminal JSON, winget installed app list, and WSL Ubuntu distribution archives.",
                target_info: "Backup/config-backups/",
                flags_info: "Dotfiles, Terminal, Winget, WSL",
                script_path: "scripts/backup-configs.ps1",
                args: vec![],
                status: TaskStatus::Pending,
                duration_ms: 0,
            },
        ];

        MaintenanceState {
            tasks,
            selected_task: 0,
            is_running: false,
            logs: vec![
                "=== D Drive Maintenance Runner ===".into(),
                "Navigate tasks on the left to view full descriptions.".into(),
                "Press [Enter] to run all tasks, or [s] to run highlighted task.".into(),
                "".into(),
            ],
        }
    }

    pub fn start_background_tasks(tasks_to_run: Vec<MaintenanceTask>, indices: Vec<usize>, tx: Sender<MaintenanceMessage>) {
        thread::spawn(move || {
            for (i, task) in tasks_to_run.into_iter().enumerate() {
                let task_idx = indices[i];
                let _ = tx.send(MaintenanceMessage::TaskStarted(task_idx));
                let _ = tx.send(MaintenanceMessage::LogLine(format!("\n▶ Running: {}", task.name)));

                let start = Instant::now();
                let actual_script_path = resolve_script_path(task.script_path);

                let mut cmd = Command::new("pwsh");
                cmd.arg("-NoProfile")
                    .arg("-File")
                    .arg(&actual_script_path);

                for arg in &task.args {
                    cmd.arg(arg);
                }

                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                match cmd.spawn() {
                    Ok(mut child) => {
                        if let Some(stdout) = child.stdout.take() {
                            let reader = BufReader::new(stdout);
                            for line in reader.lines().flatten() {
                                let trimmed = line.trim();
                                if !trimmed.is_empty() && !trimmed.starts_with("OS :") && !trimmed.starts_with("Computer :") {
                                    let _ = tx.send(MaintenanceMessage::LogLine(format!("  {}", trimmed)));
                                }
                            }
                        }

                        let status = child.wait();
                        let elapsed = start.elapsed().as_millis();

                        match status {
                            Ok(exit_status) if exit_status.success() => {
                                let _ = tx.send(MaintenanceMessage::LogLine(format!("✅ Completed in {:.1}s", elapsed as f64 / 1000.0)));
                                let _ = tx.send(MaintenanceMessage::TaskCompleted(task_idx, TaskStatus::Success, elapsed));
                            }
                            Ok(exit_status) => {
                                let _ = tx.send(MaintenanceMessage::LogLine(format!("❌ Failed with exit code: {:?}", exit_status.code())));
                                let _ = tx.send(MaintenanceMessage::TaskCompleted(task_idx, TaskStatus::Failed, elapsed));
                            }
                            Err(e) => {
                                let _ = tx.send(MaintenanceMessage::LogLine(format!("❌ Process error: {}", e)));
                                let _ = tx.send(MaintenanceMessage::TaskCompleted(task_idx, TaskStatus::Failed, elapsed));
                            }
                        }
                    }
                    Err(e) => {
                        let elapsed = start.elapsed().as_millis();
                        let _ = tx.send(MaintenanceMessage::LogLine(format!("❌ Failed to spawn process: {}", e)));
                        let _ = tx.send(MaintenanceMessage::TaskCompleted(task_idx, TaskStatus::Failed, elapsed));
                    }
                }
            }

            let _ = tx.send(MaintenanceMessage::LogLine("\n=== All Selected Tasks Finished ===".into()));
            let _ = tx.send(MaintenanceMessage::AllCompleted);
        });
    }
}

fn resolve_script_path(configured_path: &str) -> std::path::PathBuf {
    let p = std::path::Path::new(configured_path);
    if p.exists() {
        return p.to_path_buf();
    }

    if let Some(filename) = p.file_name() {
        let repo_path = std::path::Path::new("cli").join("scripts").join(filename);
        if repo_path.exists() {
            return repo_path;
        }

        let root_repo_path = std::path::Path::new("scripts").join(filename);
        if root_repo_path.exists() {
            return root_repo_path;
        }
    }

    p.to_path_buf()
}
