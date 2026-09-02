use rtb::engine::RtbEngine;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_rtb_info_existing_project() {
    let temp = tempdir().expect("tempdir");
    let base_path = temp.path();

    let active_dir = base_path.join("01-Active");
    let proj1 = active_dir.join("my-awesome-app");
    fs::create_dir_all(&proj1).expect("create dir");

    File::create(proj1.join("package.json"))
        .expect("file")
        .write_all(b"{\"name\":\"my-awesome-app\"}")
        .expect("write");

    File::create(proj1.join("README.md"))
        .expect("file")
        .write_all(b"# My Awesome App\nThis is a test readme.")
        .expect("write");

    let config_content = format!(
        r#"{{
            "version": "1.0.0",
            "projectRoots": {{
                "active": "{}"
            }},
            "backupRoot": "",
            "configRoot": "",
            "templateDir": "",
            "cleanDeps": {{ "daysInactive": 30, "targets": [] }},
            "staleThresholdDays": 60,
            "gitHealth": {{ "scanRoots": [] }}
        }}"#,
        active_dir.display().to_string().replace('\\', "/")
    );

    let config_path = base_path.join("rtb.config.json");
    fs::write(&config_path, config_content).expect("write config");
    let config_str = config_path.to_str().unwrap();

    // Plain info
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "info", "my-awesome-app"])
        .expect("dispatch");
    assert_eq!(exit_code, 0, "rtb info should exit 0 for existing project");

    // JSON info
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "info", "my-awesome-app", "--json"])
        .expect("dispatch");
    assert_eq!(exit_code, 0, "rtb info --json should exit 0 for existing project");
}

#[test]
fn test_rtb_info_non_existent_project() {
    let temp = tempdir().expect("tempdir");
    let base_path = temp.path();

    let active_dir = base_path.join("01-Active");
    fs::create_dir_all(&active_dir).expect("create dir");

    let config_content = format!(
        r#"{{
            "version": "1.0.0",
            "projectRoots": {{
                "active": "{}"
            }},
            "backupRoot": "",
            "configRoot": "",
            "templateDir": "",
            "cleanDeps": {{ "daysInactive": 30, "targets": [] }},
            "staleThresholdDays": 60,
            "gitHealth": {{ "scanRoots": [] }}
        }}"#,
        active_dir.display().to_string().replace('\\', "/")
    );

    let config_path = base_path.join("rtb.config.json");
    fs::write(&config_path, config_content).expect("write config");
    let config_str = config_path.to_str().unwrap();

    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "info", "non-existent-proj"])
        .expect("dispatch");
    assert_eq!(exit_code, 1, "rtb info for non-existent project should exit 1");
}

#[test]
fn test_rtb_config_creates_file_and_opens() {
    std::env::set_var("RTB_NON_INTERACTIVE", "1");
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("sub").join("rtb.config.json");
    let config_str = config_path.to_str().unwrap();

    assert!(!config_path.exists());

    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "config"])
        .expect("dispatch");
    assert_eq!(exit_code, 0, "rtb config should exit 0");
    assert!(config_path.exists(), "rtb config should create missing config file");
}
