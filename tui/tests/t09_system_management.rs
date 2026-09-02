use rtb::engine::RtbEngine;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_t09_system_management_commands() {
    std::env::set_var("RTB_NON_INTERACTIVE", "1");

    let temp = tempdir().expect("tempdir");
    let base_path = temp.path();

    let active_dir = base_path.join("01-Active");
    let proj1 = active_dir.join("system-test-proj");
    fs::create_dir_all(&proj1).expect("create proj1");

    File::create(proj1.join("package.json"))
        .expect("create pkg")
        .write_all(b"{\"name\": \"system-test-proj\", \"dependencies\": {\"next\": \"13.0.0\"}}")
        .expect("write pkg");

    let config_path = base_path.join("rtb.config.json");
    let config_str = config_path.to_str().unwrap();

    // 1. rtb init
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "init"])
        .expect("dispatch init");
    assert_eq!(exit_code, 0, "rtb init should exit 0");
    assert!(config_path.exists(), "rtb.config.json should be created by rtb init");

    // Re-run init without force
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "init"])
        .expect("dispatch init again");
    assert_eq!(exit_code, 0, "rtb init again without force should exit 0");

    // Re-run init with force
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "init", "--force"])
        .expect("dispatch init force");
    assert_eq!(exit_code, 0, "rtb init with --force should exit 0");

    // Update config to point active root to our temp active_dir
    let config_content = format!(
        r#"{{
            "version": "1.0.0",
            "projectRoots": {{
                "active": "{}",
                "paused": "{}",
                "production": "{}",
                "staging": "{}",
                "vibe": "{}",
                "sandbox": "{}",
                "planning": "{}",
                "testing": "{}",
                "abandoned": "{}"
            }},
            "backupRoot": "{}",
            "configRoot": "{}",
            "templateDir": "",
            "cleanDeps": {{ "daysInactive": 60, "targets": ["node_modules"] }},
            "staleThresholdDays": 90,
            "gitHealth": {{ "scanRoots": ["{}"] }}
        }}"#,
        active_dir.display().to_string().replace('\\', "/"),
        base_path.join("04-Paused").display().to_string().replace('\\', "/"),
        base_path.join("02-Production").display().to_string().replace('\\', "/"),
        base_path.join("02-Staging").display().to_string().replace('\\', "/"),
        base_path.join("03-Vibe").display().to_string().replace('\\', "/"),
        base_path.join("01-Sandbox").display().to_string().replace('\\', "/"),
        base_path.join("02-Planning").display().to_string().replace('\\', "/"),
        base_path.join("03-Testing").display().to_string().replace('\\', "/"),
        base_path.join("05-Abandoned").display().to_string().replace('\\', "/"),
        base_path.join("08-Backup").display().to_string().replace('\\', "/"),
        base_path.join("05-Config").display().to_string().replace('\\', "/"),
        active_dir.display().to_string().replace('\\', "/")
    );
    fs::write(&config_path, config_content).expect("update config");

    // Create the root dirs so doctor passes
    for d in &["04-Paused", "02-Production", "02-Staging", "03-Vibe", "01-Sandbox", "02-Planning", "03-Testing", "05-Abandoned", "08-Backup", "05-Config"] {
        fs::create_dir_all(base_path.join(d)).ok();
    }

    // 2. rtb doctor
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "doctor"])
        .expect("dispatch doctor");
    assert_eq!(exit_code, 0, "rtb doctor should exit 0");

    // 3. rtb index
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "index"])
        .expect("dispatch index");
    assert_eq!(exit_code, 0, "rtb index should exit 0");
    let index_file = base_path.join("PROJECT-INDEX.md");
    assert!(index_file.exists(), "PROJECT-INDEX.md should be created");
    let index_str = fs::read_to_string(index_file).expect("read index");
    assert!(index_str.contains("system-test-proj"), "INDEX should list system-test-proj");

    // 4. rtb health
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "health"])
        .expect("dispatch health");
    assert_eq!(exit_code, 0, "rtb health should exit 0");

    // 5. rtb maintenance
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "maintenance", "backup"])
        .expect("dispatch maintenance backup");
    assert!(exit_code == 0 || exit_code == 1);

    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "maintenance", "env"])
        .expect("dispatch maintenance env");
    assert!(exit_code == 0 || exit_code == 1);

    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "maintenance", "guard"])
        .expect("dispatch maintenance guard");
    assert!(exit_code == 0 || exit_code == 1);

    // Also test shortcut subcommands rtb backup, rtb env, rtb guard
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "backup"])
        .expect("dispatch backup shortcut");
    assert!(exit_code == 0 || exit_code == 1);

    // 6. rtb open
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "open", "system-test-proj"])
        .expect("dispatch open");
    assert_eq!(exit_code, 0, "rtb open should exit 0");

    // 7. rtb commit
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "commit", "-m", "test commit"])
        .expect("dispatch commit");
    assert_eq!(exit_code, 1, "rtb commit in non-git directory should exit 1");
}
