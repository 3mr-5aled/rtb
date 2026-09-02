use rtb::engine::RtbEngine;
use rtb::uninstall::{is_rtb_profile_line, is_rtb_path_line};
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_t12_uninstall_helper_predicates() {
    assert!(is_rtb_profile_line("Invoke-Expression (& rtb shell-init pwsh)"));
    assert!(is_rtb_profile_line("eval \"$(rtb shell-init bash)\""));
    assert!(is_rtb_profile_line("rtb _goto-resolve myproject"));
    assert!(is_rtb_profile_line("# RTB Autoload"));
    assert!(is_rtb_profile_line("Import-Module \"C:\\AppData\\rtb\\module\\rtb.psd1\""));
    assert!(is_rtb_profile_line("Import-Module rtb"));

    assert!(!is_rtb_profile_line("alias ll='ls -la'"));
    assert!(!is_rtb_profile_line("Import-Module PSReadLine"));
    assert!(!is_rtb_profile_line("export PATH=\"$HOME/bin:$PATH\""));

    assert!(is_rtb_path_line("export PATH=\"$HOME/.config/rtb/bin:$PATH\""));
    assert!(is_rtb_path_line("fish_add_path /home/user/.config/rtb/bin"));
    assert!(!is_rtb_path_line("export PATH=\"$HOME/bin:$PATH\""));
}

#[test]
fn test_t12_uninstall_non_interactive_full_cleanup() {
    let _guard = TEST_MUTEX.lock().unwrap();

    std::env::set_var("RTB_NON_INTERACTIVE", "1");
    std::env::set_var("RTB_MOCK_DO_NOT_REMOVE_EXE", "1");
    std::env::set_var("RTB_TEST_SKIP_PATH_REMOVAL", "1");

    let temp = tempdir().expect("tempdir");

    // 1. Setup mock binary directory
    let mock_bin_dir = temp.path().join("bin");
    fs::create_dir_all(&mock_bin_dir).expect("create mock_bin_dir");
    let dummy_exe = mock_bin_dir.join("rtb.exe");
    fs::write(&dummy_exe, b"dummy rtb binary").expect("write dummy exe");
    std::env::set_var("RTB_TEST_BIN_DIR", mock_bin_dir.to_str().unwrap());

    // 2. Setup mock config directory and config file inside an 'rtb' directory
    let mock_config_dir = temp.path().join("rtb");
    fs::create_dir_all(&mock_config_dir).expect("create mock_config_dir");
    let dummy_config = mock_config_dir.join("rtb.config.json");
    fs::write(&dummy_config, r#"{"version": "1.0.0"}"#).expect("write dummy config");
    std::env::set_var("RTB_TEST_CONFIG_PATH", dummy_config.to_str().unwrap());

    // 3. Setup mock shell profiles (one PowerShell, one Bash with both Phase 3 and Phase 2 lines)
    let profile_ps1 = temp.path().join("Microsoft.PowerShell_profile.ps1");
    let ps1_content = vec![
        "Import-Module PSReadLine",
        "# RTB Autoload",
        "Invoke-Expression (& rtb shell-init pwsh)",
        "Import-Module 'C:\\Users\\user\\AppData\\Roaming\\rtb\\module\\rtb.psd1'",
        "Set-Alias g git",
    ].join("\n");
    fs::write(&profile_ps1, &ps1_content).expect("write profile ps1");

    let profile_bash = temp.path().join(".bashrc");
    let bash_content = vec![
        "export EDITOR=nano",
        "eval \"$(rtb shell-init bash)\"",
        "export PATH=\"$HOME/.config/rtb/bin:$PATH\"",
        "alias gs='git status'",
    ].join("\n");
    fs::write(&profile_bash, &bash_content).expect("write profile bash");

    let profile_paths_var = format!("{};{}", profile_ps1.to_str().unwrap(), profile_bash.to_str().unwrap());
    std::env::set_var("RTB_TEST_PROFILE_PATHS", &profile_paths_var);

    // Act: Run rtb uninstall via RtbEngine::dispatch_args
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "uninstall"])
        .expect("dispatch uninstall");
    assert_eq!(exit_code, 0, "rtb uninstall should exit 0");

    // Assert: Binary removed
    assert!(!dummy_exe.exists(), "rtb binary should be deleted");

    // Assert: Config file and dir removed
    assert!(!dummy_config.exists(), "rtb.config.json should be deleted");

    // Assert: Profiles stripped of RTB lines, non-RTB lines preserved
    let updated_ps1 = fs::read_to_string(&profile_ps1).expect("read updated ps1");
    assert!(updated_ps1.contains("Import-Module PSReadLine"));
    assert!(updated_ps1.contains("Set-Alias g git"));
    assert!(!updated_ps1.contains("rtb shell-init"));
    assert!(!updated_ps1.contains("rtb.psd1"));
    assert!(!updated_ps1.contains("# RTB Autoload"));

    let updated_bash = fs::read_to_string(&profile_bash).expect("read updated bash");
    assert!(updated_bash.contains("export EDITOR=nano"));
    assert!(updated_bash.contains("alias gs='git status'"));
    assert!(!updated_bash.contains("rtb shell-init"));
    assert!(!updated_bash.contains("rtb/bin"));

    // Cleanup env vars
    std::env::remove_var("RTB_TEST_BIN_DIR");
    std::env::remove_var("RTB_TEST_CONFIG_PATH");
    std::env::remove_var("RTB_TEST_PROFILE_PATHS");
    std::env::remove_var("RTB_MOCK_DO_NOT_REMOVE_EXE");
    std::env::remove_var("RTB_TEST_SKIP_PATH_REMOVAL");
}

#[test]
fn test_t12_uninstall_yes_flag() {
    let _guard = TEST_MUTEX.lock().unwrap();

    std::env::set_var("RTB_MOCK_DO_NOT_REMOVE_EXE", "1");
    std::env::set_var("RTB_TEST_SKIP_PATH_REMOVAL", "1");

    let temp = tempdir().expect("tempdir");
    let mock_config_dir = temp.path().join("rtb");
    fs::create_dir_all(&mock_config_dir).unwrap();
    let dummy_config = mock_config_dir.join("rtb.config.json");
    fs::write(&dummy_config, r#"{}"#).unwrap();
    std::env::set_var("RTB_TEST_CONFIG_PATH", dummy_config.to_str().unwrap());

    // Dispatch rtb uninstall --yes (or -y)
    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "uninstall", "--yes"])
        .expect("dispatch uninstall --yes");
    assert_eq!(exit_code, 0);

    assert!(!dummy_config.exists(), "config should be deleted when --yes flag is specified");

    std::env::remove_var("RTB_TEST_CONFIG_PATH");
    std::env::remove_var("RTB_MOCK_DO_NOT_REMOVE_EXE");
    std::env::remove_var("RTB_TEST_SKIP_PATH_REMOVAL");
}

#[test]
fn test_t12_uninstall_partial_install_graceful() {
    let _guard = TEST_MUTEX.lock().unwrap();

    std::env::set_var("RTB_NON_INTERACTIVE", "1");
    std::env::set_var("RTB_MOCK_DO_NOT_REMOVE_EXE", "1");
    std::env::set_var("RTB_TEST_SKIP_PATH_REMOVAL", "1");

    let temp = tempdir().expect("tempdir");
    let missing_bin = temp.path().join("nonexistent_bin");
    let missing_config = temp.path().join("nonexistent_config").join("rtb.config.json");
    let missing_profile = temp.path().join("nonexistent_profile.ps1");

    std::env::set_var("RTB_TEST_BIN_DIR", missing_bin.to_str().unwrap());
    std::env::set_var("RTB_TEST_CONFIG_PATH", missing_config.to_str().unwrap());
    std::env::set_var("RTB_TEST_PROFILE_PATHS", missing_profile.to_str().unwrap());

    let exit_code = RtbEngine::dispatch_args(vec!["rtb", "uninstall"])
        .expect("dispatch uninstall partial");
    assert_eq!(exit_code, 0, "Partial or missing install components should exit 0 without crashing");

    std::env::remove_var("RTB_TEST_BIN_DIR");
    std::env::remove_var("RTB_TEST_CONFIG_PATH");
    std::env::remove_var("RTB_TEST_PROFILE_PATHS");
    std::env::remove_var("RTB_MOCK_DO_NOT_REMOVE_EXE");
    std::env::remove_var("RTB_TEST_SKIP_PATH_REMOVAL");
}
