use rtb::engine::RtbEngine;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_rtb_lifecycle_core_commands() {
    let temp = tempdir().expect("tempdir");
    let base_path = temp.path();

    let active_dir = base_path.join("01-Active");
    let paused_dir = base_path.join("04-Paused");
    let production_dir = base_path.join("02-Deployed").join("01-Production");
    let staging_dir = base_path.join("02-Deployed").join("02-Staging");
    let backup_dir = base_path.join("backups");

    fs::create_dir_all(&active_dir).expect("create active");
    fs::create_dir_all(&paused_dir).expect("create paused");
    fs::create_dir_all(&production_dir).expect("create prod");
    fs::create_dir_all(&staging_dir).expect("create staging");
    fs::create_dir_all(&backup_dir).expect("create backup");

    let config_content = format!(
        r#"{{
            "version": "1.0.0",
            "projectRoots": {{
                "active": "{}",
                "paused": "{}",
                "production": "{}",
                "staging": "{}"
            }},
            "backupRoot": "{}",
            "configRoot": "",
            "templateDir": "",
            "cleanDeps": {{ "daysInactive": 30, "targets": ["node_modules", "target"] }},
            "staleThresholdDays": 60,
            "gitHealth": {{ "scanRoots": [] }}
        }}"#,
        active_dir.display().to_string().replace('\\', "/"),
        paused_dir.display().to_string().replace('\\', "/"),
        production_dir.display().to_string().replace('\\', "/"),
        staging_dir.display().to_string().replace('\\', "/"),
        backup_dir.display().to_string().replace('\\', "/")
    );

    let config_path = base_path.join("rtb.config.json");
    fs::write(&config_path, config_content).expect("write config");
    let config_str = config_path.to_str().unwrap();

    // 1. rtb new
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "new", "test-project-x"])
        .expect("dispatch new");
    assert_eq!(exit_code, 0, "rtb new should exit 0");

    let created_proj = active_dir.join("test-project-x");
    assert!(created_proj.exists(), "test-project-x dir should exist in active");
    assert!(created_proj.join("PROJECT.md").exists(), "PROJECT.md should exist");
    assert!(created_proj.join(".gitignore").exists(), ".gitignore should exist");
    assert!(created_proj.join("README.md").exists(), "README.md should exist");

    // Create dummy dep folder for pause --prune-deps testing
    fs::create_dir_all(created_proj.join("node_modules")).expect("create dummy dep");

    // 2. rtb pause
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "pause", "test-project-x", "--prune-deps", "--force"])
        .expect("dispatch pause");
    assert_eq!(exit_code, 0, "rtb pause should exit 0");

    let paused_proj = paused_dir.join("test-project-x");
    assert!(!created_proj.exists(), "test-project-x should no longer be in active");
    assert!(paused_proj.exists(), "test-project-x should exist in paused");
    assert!(!paused_proj.join("node_modules").exists(), "node_modules should be pruned");

    // 3. rtb resume
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "resume", "test-project-x"])
        .expect("dispatch resume");
    assert_eq!(exit_code, 0, "rtb resume should exit 0");

    assert!(created_proj.exists(), "test-project-x should be back in active");
    assert!(!paused_proj.exists(), "test-project-x should no longer be in paused");

    // 4. rtb deploy --staging
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "deploy", "test-project-x", "--staging"])
        .expect("dispatch deploy");
    assert_eq!(exit_code, 0, "rtb deploy --staging should exit 0");

    let staging_proj = staging_dir.join("test-project-x");
    assert!(!created_proj.exists(), "test-project-x should no longer be in active");
    assert!(staging_proj.exists(), "test-project-x should exist in staging");

    // 5. rtb archive
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "archive", "test-project-x", "--force"])
        .expect("dispatch archive");
    assert_eq!(exit_code, 0, "rtb archive should exit 0");

    assert!(!staging_proj.exists(), "test-project-x source dir should be removed after archiving");

    let snapshots_dir = backup_dir.join("project-snapshots");
    assert!(snapshots_dir.exists(), "project-snapshots dir should exist");

    let archive_entry = fs::read_dir(&snapshots_dir)
        .expect("read snapshots")
        .flatten()
        .find(|e| e.file_name().to_string_lossy().contains("test-project-x"));
    assert!(archive_entry.is_some(), "archive tar.gz file should be created");
    let archive_file_name = archive_entry.unwrap().file_name().to_string_lossy().to_string();

    // 6. rtb unarchive
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "unarchive", &archive_file_name])
        .expect("dispatch unarchive");
    assert_eq!(exit_code, 0, "rtb unarchive should exit 0");

    assert!(created_proj.exists(), "test-project-x should be extracted into active dir");
}
