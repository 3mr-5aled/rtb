use anyhow::Result;
use std::env;
use std::fs;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;

/// Helper to determine if a line in a shell profile file is related to RTB.
pub fn is_rtb_profile_line(line: &str) -> bool {
    let l = line.trim();
    if l.contains("rtb shell-init")
        || l.contains("rtb.exe shell-init")
        || l.contains("rtb _goto-resolve")
    {
        return true;
    }
    if l.starts_with("# RTB") || l.starts_with("# rtb") {
        return true;
    }
    let lower = l.to_lowercase();
    if lower.contains("import-module") {
        if lower.contains("rtb")
            || lower.contains("dev-tools")
            || lower.contains("dev-cli")
            || lower.contains("rtb-command-tool")
        {
            return true;
        }
    }
    false
}

/// Helper to determine if a line in a shell profile sets PATH for RTB.
pub fn is_rtb_path_line(line: &str) -> bool {
    let lower = line.trim().to_lowercase();
    (lower.contains("path") || lower.contains("fish_add_path"))
        && (lower.contains("rtb/bin") || lower.contains("rtb\\bin") || lower.contains(".config/rtb"))
}

/// Resolves standard shell profile file paths across operating systems.
pub fn get_profile_paths() -> Vec<PathBuf> {
    if let Ok(override_paths) = env::var("RTB_TEST_PROFILE_PATHS") {
        return override_paths
            .split(';')
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect();
    }

    let mut paths = Vec::new();

    if let Ok(prof) = env::var("PROFILE") {
        if !prof.is_empty() {
            paths.push(PathBuf::from(prof));
        }
    }

    if let Some(home) = dirs::home_dir() {
        // Windows PowerShell profile paths
        #[cfg(target_os = "windows")]
        {
            if let Some(docs) = dirs::document_dir() {
                paths.push(docs.join("PowerShell").join("Microsoft.PowerShell_profile.ps1"));
                paths.push(docs.join("WindowsPowerShell").join("Microsoft.PowerShell_profile.ps1"));
                paths.push(docs.join("PowerShell").join("Profile.ps1"));
            }
            paths.push(home.join("Documents").join("PowerShell").join("Microsoft.PowerShell_profile.ps1"));
            paths.push(home.join("Documents").join("WindowsPowerShell").join("Microsoft.PowerShell_profile.ps1"));
        }

        // Unix shell RC paths
        paths.push(home.join(".bashrc"));
        paths.push(home.join(".bash_profile"));
        paths.push(home.join(".zshrc"));
        paths.push(home.join(".profile"));
        paths.push(home.join(".config").join("fish").join("config.fish"));
    }

    paths.sort();
    paths.dedup();
    paths
}

/// Strip RTB shell init lines, legacy Phase 2 imports, and PATH lines from shell profiles.
pub fn clean_shell_profiles() -> Result<()> {
    for profile_path in get_profile_paths() {
        if !profile_path.is_file() {
            continue;
        }

        let content = match fs::read_to_string(&profile_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let lines: Vec<&str> = content.lines().collect();
        let filtered: Vec<&str> = lines
            .into_iter()
            .filter(|l| !is_rtb_profile_line(l) && !is_rtb_path_line(l))
            .collect();

        let new_content = if filtered.is_empty() {
            String::new()
        } else {
            let mut joined = filtered.join("\n");
            if content.ends_with('\n') || content.ends_with("\r\n") {
                joined.push('\n');
            }
            joined
        };

        if new_content != content {
            let _ = fs::write(&profile_path, new_content);
        }
    }
    Ok(())
}

/// Remove rtb binary from system bin directory.
pub fn remove_binary() -> Result<()> {
    let mut candidate_bins: Vec<PathBuf> = Vec::new();

    if let Ok(test_bin_dir) = env::var("RTB_TEST_BIN_DIR") {
        let p = PathBuf::from(test_bin_dir);
        candidate_bins.push(p.join("rtb.exe"));
        candidate_bins.push(p.join("rtb"));
    } else if let Ok(bin_dir) = env::var("RTB_BIN_DIR") {
        let p = PathBuf::from(bin_dir);
        candidate_bins.push(p.join("rtb.exe"));
        candidate_bins.push(p.join("rtb"));
    } else {
        if let Some(config_dir) = dirs::config_dir() {
            candidate_bins.push(config_dir.join("rtb").join("bin").join("rtb.exe"));
            candidate_bins.push(config_dir.join("rtb").join("bin").join("rtb"));
        }
        if let Some(home) = dirs::home_dir() {
            candidate_bins.push(home.join(".config").join("rtb").join("bin").join("rtb.exe"));
            candidate_bins.push(home.join(".config").join("rtb").join("bin").join("rtb"));
            candidate_bins.push(home.join(".local").join("bin").join("rtb.exe"));
            candidate_bins.push(home.join(".local").join("bin").join("rtb"));
        }
    }

    for bin_path in &candidate_bins {
        if bin_path.is_file() {
            let _ = fs::remove_file(bin_path);
            if let Some(parent) = bin_path.parent() {
                if parent.file_name().map_or(false, |n| n == "bin") {
                    let _ = fs::remove_dir(parent);
                }
            }
        }
    }

    if env::var("RTB_MOCK_DO_NOT_REMOVE_EXE").is_err() {
        if let Ok(exe_path) = env::current_exe() {
            if let Some(stem) = exe_path.file_stem() {
                if stem.to_string_lossy().eq_ignore_ascii_case("rtb") {
                    let _ = fs::remove_file(&exe_path);
                }
            }
        }
    }

    Ok(())
}

/// Remove user configuration file and directory.
pub fn remove_config() -> Result<()> {
    if let Ok(test_config_path) = env::var("RTB_TEST_CONFIG_PATH") {
        let p = PathBuf::from(test_config_path);
        if p.is_file() {
            let _ = fs::remove_file(&p);
        }
        if let Some(parent) = p.parent() {
            if parent.exists() && parent.file_name().map_or(false, |n| n == "rtb") {
                let _ = fs::remove_dir_all(parent);
            }
        }
        return Ok(());
    }

    if let Some(config_dir) = dirs::config_dir() {
        let rtb_dir = config_dir.join("rtb");
        if rtb_dir.exists() {
            let _ = fs::remove_dir_all(&rtb_dir);
        }
    }

    if let Some(home) = dirs::home_dir() {
        let rtb_dir = home.join(".config").join("rtb");
        if rtb_dir.exists() {
            let _ = fs::remove_dir_all(&rtb_dir);
        }
    }

    Ok(())
}

/// Remove rtb bin path entry from environment / registry.
pub fn clean_path_entry() -> Result<()> {
    if env::var("RTB_TEST_SKIP_PATH_REMOVAL").is_ok() {
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let script = r#"
            $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
            if ($userPath) {
                $newPath = ($userPath -split ';' | Where-Object { $_ -and $_ -notlike '*\rtb\bin*' -and $_ -notlike '*\.config\rtb\bin*' }) -join ';'
                [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
            }
        "#;
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .output();
    }

    Ok(())
}

/// Main execution function for `rtb uninstall`.
pub fn execute_uninstall(yes: bool) -> Result<i32> {
    let is_non_interactive = yes
        || env::var("RTB_NON_INTERACTIVE").is_ok()
        || env::var("CI").is_ok()
        || env::var("GITHUB_ACTIONS").is_ok()
        || !std::io::stdin().is_terminal();

    if !is_non_interactive {
        print!("Are you sure you want to uninstall RTB from your system? (y/N) ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_lowercase();
        if trimmed != "y" && trimmed != "yes" {
            println!("Uninstallation canceled.");
            return Ok(0);
        }
    }

    remove_binary()?;
    remove_config()?;
    clean_shell_profiles()?;
    clean_path_entry()?;

    println!("Uninstallation Complete! RTB has been removed from your system.");
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_rtb_profile_line() {
        assert!(is_rtb_profile_line("Invoke-Expression (& rtb shell-init pwsh)"));
        assert!(is_rtb_profile_line("eval \"$(rtb shell-init bash)\""));
        assert!(is_rtb_profile_line("rtb _goto-resolve myproject"));
        assert!(is_rtb_profile_line("# RTB Autoload"));
        assert!(is_rtb_profile_line("Import-Module \"C:\\Users\\dev\\rtb\\module\\rtb.psd1\""));
        assert!(is_rtb_profile_line("Import-Module rtb"));

        assert!(!is_rtb_profile_line("alias ll='ls -la'"));
        assert!(!is_rtb_profile_line("Import-Module PSReadLine"));
        assert!(!is_rtb_profile_line("export PATH=$PATH:/usr/local/bin"));
    }

    #[test]
    fn test_is_rtb_path_line() {
        assert!(is_rtb_path_line("export PATH=\"$HOME/.config/rtb/bin:$PATH\""));
        assert!(is_rtb_path_line("fish_add_path /home/user/.config/rtb/bin"));
        assert!(!is_rtb_path_line("export PATH=\"$HOME/bin:$PATH\""));
    }
}
