use anyhow::{bail, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct ReleaseInfo {
    tag_name: String,
    #[allow(dead_code)]
    html_url: Option<String>,
    #[serde(default)]
    assets: Vec<AssetInfo>,
}

#[derive(Debug, Deserialize)]
struct AssetInfo {
    name: String,
    browser_download_url: String,
}

pub fn target_asset_names() -> Result<(&'static str, &'static str)> {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Ok(("rtb-windows-amd64.exe", "rtb-windows-amd64.exe.sha256"))
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Ok(("rtb-linux-amd64", "rtb-linux-amd64.sha256"))
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Ok(("rtb-macos-amd64", "rtb-macos-amd64.sha256"))
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Ok(("rtb-macos-arm64", "rtb-macos-arm64.sha256"))
    } else {
        bail!("Unsupported target platform for self-upgrade");
    }
}

pub fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let clean = v.trim().trim_start_matches(['v', 'V']);
    let parts: Vec<&str> = clean.split('.').collect();
    if parts.len() >= 3 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].split('-').next()?.parse().ok()?;
        Some((major, minor, patch))
    } else {
        None
    }
}

pub fn is_newer_version(latest_str: &str, current_str: &str) -> bool {
    match (parse_version(latest_str), parse_version(current_str)) {
        (Some(latest), Some(current)) => latest > current,
        _ => false,
    }
}

fn fetch_latest_release() -> Result<ReleaseInfo> {
    if let Ok(mock_dir) = env::var("RTB_MOCK_RELEASE_DIR") {
        let release_json_path = PathBuf::from(mock_dir).join("release.json");
        let content = fs::read_to_string(release_json_path)?;
        let release: ReleaseInfo = serde_json::from_str(&content)?;
        return Ok(release);
    }

    let url = "https://api.github.com/repos/3mr-5aled/rtb/releases/latest";
    let user_agent = format!("rtb-cli/{}", env!("CARGO_PKG_VERSION"));

    let response = ureq::get(url)
        .set("User-Agent", &user_agent)
        .call()?;

    let release: ReleaseInfo = response.into_json()?;
    Ok(release)
}

fn download_bytes(asset_name: &str, download_url: &str) -> Result<Vec<u8>> {
    if let Ok(mock_dir) = env::var("RTB_MOCK_RELEASE_DIR") {
        let asset_path = PathBuf::from(mock_dir).join(asset_name);
        let bytes = fs::read(asset_path)?;
        return Ok(bytes);
    }

    let user_agent = format!("rtb-cli/{}", env!("CARGO_PKG_VERSION"));
    let response = ureq::get(download_url)
        .set("User-Agent", &user_agent)
        .call()?;

    let mut bytes = Vec::new();
    use std::io::Read;
    response.into_reader().read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn download_text(asset_name: &str, download_url: &str) -> Result<String> {
    if let Ok(mock_dir) = env::var("RTB_MOCK_RELEASE_DIR") {
        let asset_path = PathBuf::from(mock_dir).join(asset_name);
        let text = fs::read_to_string(asset_path)?;
        return Ok(text);
    }

    let user_agent = format!("rtb-cli/{}", env!("CARGO_PKG_VERSION"));
    let response = ureq::get(download_url)
        .set("User-Agent", &user_agent)
        .call()?;

    let text = response.into_string()?;
    Ok(text)
}

pub fn execute_upgrade(check: bool) -> Result<i32> {
    let manual_url = "https://github.com/3mr-5aled/rtb/releases/latest";

    let (bin_asset_name, sidecar_asset_name) = match target_asset_names() {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Manual download URL: {}", manual_url);
            return Ok(1);
        }
    };

    let release_info = match fetch_latest_release() {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Error: Failed to fetch latest release: {}", e);
            eprintln!("Manual download URL: {}", manual_url);
            return Ok(1);
        }
    };

    let current_ver = env!("CARGO_PKG_VERSION");
    let latest_ver_clean = release_info.tag_name.trim().trim_start_matches(['v', 'V']);

    if !is_newer_version(latest_ver_clean, current_ver) {
        println!("rtb is up-to-date (v{})", current_ver);
        return Ok(0);
    }

    if check {
        println!("Upgrade available: v{} -> v{}", current_ver, latest_ver_clean);
        return Ok(0);
    }

    println!("Downloading rtb v{} for platform...", latest_ver_clean);

    let bin_asset = release_info.assets.iter().find(|a| a.name == bin_asset_name);
    let sidecar_asset = release_info.assets.iter().find(|a| a.name == sidecar_asset_name);

    let (bin_url, sidecar_url) = match (bin_asset, sidecar_asset) {
        (Some(b), Some(s)) => (&b.browser_download_url, &s.browser_download_url),
        _ => {
            eprintln!(
                "Error: Release v{} missing assets for this platform ({}, {})",
                latest_ver_clean, bin_asset_name, sidecar_asset_name
            );
            eprintln!("Manual download URL: {}", manual_url);
            return Ok(1);
        }
    };

    let bin_bytes = match download_bytes(bin_asset_name, bin_url) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: Failed to download binary: {}", e);
            eprintln!("Manual download URL: {}", manual_url);
            return Ok(1);
        }
    };

    let sidecar_text = match download_text(sidecar_asset_name, sidecar_url) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error: Failed to download SHA256 sidecar: {}", e);
            eprintln!("Manual download URL: {}", manual_url);
            return Ok(1);
        }
    };

    println!("Verifying SHA256 checksum...");
    let expected_hash = sidecar_text
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_lowercase();

    let mut hasher = Sha256::new();
    hasher.update(&bin_bytes);
    let actual_hash = format!("{:x}", hasher.finalize());

    if actual_hash != expected_hash {
        eprintln!(
            "Error: SHA256 checksum mismatch (expected {}, got {})",
            expected_hash, actual_hash
        );
        eprintln!("Manual download URL: {}", manual_url);
        return Ok(1);
    }

    if env::var("RTB_MOCK_DO_NOT_REPLACE").is_err() {
        println!("Replacing binary...");
        let temp_dir = match tempfile::tempdir() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error: Failed to create temporary directory: {}", e);
                eprintln!("Manual download URL: {}", manual_url);
                return Ok(1);
            }
        };
        let temp_bin_path = temp_dir.path().join(bin_asset_name);
        if let Err(e) = fs::write(&temp_bin_path, &bin_bytes) {
            eprintln!("Error: Failed to write temporary binary: {}", e);
            eprintln!("Manual download URL: {}", manual_url);
            return Ok(1);
        }
        if let Err(e) = self_replace::self_replace(&temp_bin_path) {
            eprintln!("Error: Failed to replace binary: {}", e);
            eprintln!("Manual download URL: {}", manual_url);
            return Ok(1);
        }
    }

    println!("Successfully upgraded rtb to v{}!", latest_ver_clean);
    Ok(0)
}
