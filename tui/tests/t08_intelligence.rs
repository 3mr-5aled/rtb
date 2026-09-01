use rtb::engine::RtbEngine;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_rtb_intelligence_and_agent_commands() {
    let temp = tempdir().expect("tempdir");
    let base_path = temp.path();

    let active_dir = base_path.join("01-Active");
    let proj1 = active_dir.join("alpha-service");
    fs::create_dir_all(&proj1).expect("create proj1");

    File::create(proj1.join("package.json"))
        .expect("create pkg")
        .write_all(b"{\"name\": \"alpha-service\"}")
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

    // 1. _goto-resolve existing project
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "_goto-resolve", "alpha-service"])
        .expect("dispatch _goto-resolve");
    assert_eq!(exit_code, 0, "_goto-resolve should exit 0 for matching project");

    // 2. _goto-resolve non-existent project
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "_goto-resolve", "non-existent-proj-xyz"])
        .expect("dispatch _goto-resolve");
    assert_eq!(exit_code, 1, "_goto-resolve should exit 1 for non-existent project");

    // 3. agent --list
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "agent", "--list"])
        .expect("dispatch agent --list");
    assert_eq!(exit_code, 0, "agent --list should exit 0");

    // 4. agent unknown agent
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "agent", "non_existent_agent_999"])
        .expect("dispatch agent unknown");
    assert_eq!(exit_code, 1, "agent unknown should exit 1");

    // 5. agent --clean
    let ctx_file = proj1.join(".rtb_context.md");
    fs::write(&ctx_file, "dummy context").expect("write context");
    assert!(ctx_file.exists());
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "--config", config_str, "agent", "--clean", "--project", "alpha-service"])
        .expect("dispatch agent --clean");
    assert_eq!(exit_code, 0, "agent --clean should exit 0");
    assert!(!ctx_file.exists(), ".rtb_context.md should be removed by --clean");

    // 6. shell-init for all shells
    for shell in &["pwsh", "powershell", "bash", "zsh", "fish"] {
        let exit_code = RtbEngine::dispatch_args(vec!["rtb", "shell-init", shell])
            .expect("dispatch shell-init");
        assert_eq!(exit_code, 0, "shell-init {} should exit 0", shell);
    }
}
