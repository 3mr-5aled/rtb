use rtb::engine::RtbEngine;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_rtb_list_and_status() {
    let temp = tempdir().expect("tempdir");
    let base_path = temp.path();

    let active_dir = base_path.join("01-Active");
    let paused_dir = base_path.join("04-Paused");
    let proj1 = active_dir.join("alpha-project");
    let proj2 = active_dir.join("beta-project");
    let proj3 = paused_dir.join("gamma-project");

    fs::create_dir_all(&proj1).expect("create dir");
    fs::create_dir_all(&proj2).expect("create dir");
    fs::create_dir_all(&proj3).expect("create dir");

    // Add dummy files
    File::create(proj1.join("package.json"))
        .expect("file")
        .write_all(b"{\"name\":\"alpha-project\"}")
        .expect("write");
    File::create(proj2.join("Cargo.toml"))
        .expect("file")
        .write_all(b"[package]\nname=\"beta-project\"")
        .expect("write");

    let config_content = format!(
        r#"{{
            "version": "1.0.0",
            "projectRoots": {{
                "active": "{}",
                "paused": "{}"
            }},
            "backupRoot": "",
            "configRoot": "",
            "templateDir": "",
            "cleanDeps": {{ "daysInactive": 30, "targets": [] }},
            "staleThresholdDays": 60,
            "gitHealth": {{ "scanRoots": [] }}
        }}"#,
        active_dir.display().to_string().replace('\\', "/"),
        paused_dir.display().to_string().replace('\\', "/")
    );

    let config_path = base_path.join("rtb.config.json");
    fs::write(&config_path, config_content).expect("write config");

    let config_str = config_path.to_str().unwrap();

    // Test rtb list plain
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "list"]).expect("dispatch");
    assert_eq!(exit_code, 0, "rtb list should exit 0");

    // Test rtb list --json
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "list", "--json"]).expect("dispatch");
    assert_eq!(exit_code, 0, "rtb list --json should exit 0");

    // Test rtb list --active
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "list", "--active"]).expect("dispatch");
    assert_eq!(exit_code, 0, "rtb list --active should exit 0");

    // Test rtb status from inside proj1
    let original_dir = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(&proj1).expect("set cwd");

    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "status"]).expect("dispatch");
    assert_eq!(exit_code, 0, "rtb status should exit 0");

    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "status", "--json"]).expect("dispatch");
    assert_eq!(exit_code, 0, "rtb status --json should exit 0");

    std::env::set_current_dir(original_dir).expect("restore cwd");
}
