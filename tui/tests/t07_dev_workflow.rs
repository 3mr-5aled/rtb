use rtb::engine::RtbEngine;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_rtb_dev_workflow_commands() {
    let temp = tempdir().expect("tempdir");
    let base_path = temp.path();

    let active_dir = base_path.join("01-Active");
    let proj1 = active_dir.join("dev-app-one");
    fs::create_dir_all(&proj1).expect("create proj1");

    // Add package.json with dev, build, test scripts & dependencies
    let pkg_json = r#"{
        "name": "dev-app-one",
        "workspaces": ["packages/*"],
        "scripts": {
            "dev": "echo dev-running",
            "build": "echo build-running",
            "test": "echo test-running"
        },
        "dependencies": {
            "express": "^4.18.0"
        },
        "devDependencies": {
            "jest": "^29.0.0"
        }
    }"#;
    File::create(proj1.join("package.json"))
        .expect("create pkg")
        .write_all(pkg_json.as_bytes())
        .expect("write pkg");

    let config_content = format!(
        r#"{{
            "version": "1.0.0",
            "projectRoots": {{
                "active": "{}"
            }},
            "backupRoot": "",
            "configRoot": "",
            "templateDir": "",
            "cleanDeps": {{ "daysInactive": 0, "targets": ["node_modules"] }},
            "staleThresholdDays": 0,
            "gitHealth": {{ "scanRoots": [] }}
        }}"#,
        active_dir.display().to_string().replace('\\', "/")
    );

    let config_path = base_path.join("rtb.config.json");
    fs::write(&config_path, config_content).expect("write config");
    let config_str = config_path.to_str().unwrap();

    // 1. rtb deps plain & json
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "deps", "dev-app-one"])
        .expect("dispatch deps");
    assert_eq!(exit_code, 0, "rtb deps should exit 0");

    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "deps", "dev-app-one", "--json"])
        .expect("dispatch deps --json");
    assert_eq!(exit_code, 0, "rtb deps --json should exit 0");

    // 2. rtb workspace plain & json
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "workspace", "dev-app-one"])
        .expect("dispatch workspace");
    assert_eq!(exit_code, 0, "rtb workspace should exit 0");

    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "workspace", "dev-app-one", "--json"])
        .expect("dispatch workspace --json");
    assert_eq!(exit_code, 0, "rtb workspace --json should exit 0");

    // 3. rtb clean --dry-run
    let dep_folder = proj1.join("node_modules");
    fs::create_dir_all(&dep_folder).expect("create node_modules");
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "clean", "--dry-run", "--days", "0"])
        .expect("dispatch clean --dry-run");
    assert_eq!(exit_code, 0, "rtb clean --dry-run should exit 0");
    assert!(dep_folder.exists(), "node_modules should remain in dry-run mode");

    // 4. rtb clean --commit
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "clean", "--commit", "--days", "0"])
        .expect("dispatch clean --commit");
    assert_eq!(exit_code, 0, "rtb clean --commit should exit 0");
    assert!(!dep_folder.exists(), "node_modules should be deleted after --commit clean");

    // 5. rtb run
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "run", "dev-app-one"])
        .expect("dispatch run");
    assert_eq!(exit_code, 0, "rtb run should exit 0");

    // 6. rtb build
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "build", "dev-app-one"])
        .expect("dispatch build");
    assert_eq!(exit_code, 0, "rtb build should exit 0");

    // 7. rtb test
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "test", "dev-app-one"])
        .expect("dispatch test");
    assert_eq!(exit_code, 0, "rtb test should exit 0");
}
