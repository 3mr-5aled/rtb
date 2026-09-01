use rtb::engine::RtbEngine;
use std::env;

#[test]
fn test_version_output() {
    let args = vec!["rtb", "--version"];
    let exit_code = RtbEngine::dispatch_args(args).expect("dispatch failed");
    assert_eq!(exit_code, 0);

    let version_str = RtbEngine::version_string();
    assert!(version_str.starts_with("rtb "));
    assert!(version_str.contains('('));
    assert!(version_str.contains(')'));
}

#[test]
fn test_config_gate_exempt_commands() {
    // These commands must bypass the Config Gate even when RTB_NON_INTERACTIVE is set and no config exists.
    env::set_var("RTB_NON_INTERACTIVE", "1");
    env::set_var("RTB_MOCK_DO_NOT_REMOVE_EXE", "1");
    env::set_var("RTB_TEST_SKIP_PATH_REMOVAL", "1");

    let exempt = vec![
        vec!["rtb", "init"],
        vec!["rtb", "config"],
        vec!["rtb", "doctor"],
        vec!["rtb", "uninstall"],
        vec!["rtb", "upgrade"],
        vec!["rtb", "shell-init", "powershell"],
        vec!["rtb", "completions", "powershell"],
    ];

    for args in exempt {
        let cmd_name = args[1];
        let exit_code = RtbEngine::dispatch_args(args).expect("dispatch failed");
        if cmd_name == "completions" {
            assert_eq!(exit_code, 1, "{} should exit 1 (unimplemented)", cmd_name);
        } else if cmd_name == "doctor" {
            assert!(exit_code == 0 || exit_code == 1, "doctor should run and exit 0 or 1");
        } else {
            assert_eq!(exit_code, 0, "{} should exit 0", cmd_name);
        }
    }

    env::remove_var("RTB_MOCK_DO_NOT_REMOVE_EXE");
    env::remove_var("RTB_TEST_SKIP_PATH_REMOVAL");
}

#[test]
fn test_config_gate_non_interactive_blocks_data_commands() {
    env::set_var("RTB_NON_INTERACTIVE", "1");
    // Ensure custom config path points to a non-existent file
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let non_existent_config = temp_dir.path().join("non_existent_rtb.config.json");

    let data_commands = vec![
        vec!["rtb", "--config", non_existent_config.to_str().unwrap(), "list"],
        vec!["rtb", "--config", non_existent_config.to_str().unwrap(), "status"],
        vec!["rtb", "--config", non_existent_config.to_str().unwrap(), "run"],
    ];

    for args in data_commands {
        let exit_code = RtbEngine::dispatch_args(args).expect("dispatch failed");
        assert_eq!(exit_code, 1, "Data command without config should exit 1");
    }
}

#[test]
fn test_unimplemented_commands_exit_1() {
    env::set_var("RTB_NON_INTERACTIVE", "1");

    let unimplemented = vec![
        vec!["rtb", "completions", "powershell"],
    ];

    for args in unimplemented {
        let exit_code = RtbEngine::dispatch_args(args).expect("dispatch failed");
        assert_eq!(exit_code, 1);
    }
}

