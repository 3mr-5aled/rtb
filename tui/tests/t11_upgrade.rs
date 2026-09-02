use rtb::engine::RtbEngine;
use sha2::{Digest, Sha256};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_t11_upgrade_suite() {
    std::env::set_var("RTB_NON_INTERACTIVE", "1");

    // Case 1: Up to date
    {
        let temp = tempdir().expect("tempdir");
        let mock_dir = temp.path().join("mock_release");
        fs::create_dir_all(&mock_dir).expect("create mock_dir");

        let release_json = serde_json::json!({
            "tag_name": "v1.0.0",
            "html_url": "https://github.com/3mr-5aled/rtb/releases/tag/v1.0.0",
            "assets": []
        });
        fs::write(mock_dir.join("release.json"), serde_json::to_string(&release_json).unwrap()).unwrap();

        std::env::set_var("RTB_MOCK_RELEASE_DIR", mock_dir.to_str().unwrap());

        let exit_code = RtbEngine::dispatch_args(vec!["rtb", "upgrade"])
            .expect("dispatch upgrade");
        assert_eq!(exit_code, 0, "rtb upgrade should exit 0 when up-to-date");

        let exit_code = RtbEngine::dispatch_args(vec!["rtb", "upgrade", "--check"])
            .expect("dispatch upgrade --check");
        assert_eq!(exit_code, 0, "rtb upgrade --check should exit 0 when up-to-date");

        std::env::remove_var("RTB_MOCK_RELEASE_DIR");
    }

    // Case 2: Check newer available
    {
        let temp = tempdir().expect("tempdir");
        let mock_dir = temp.path().join("mock_release");
        fs::create_dir_all(&mock_dir).expect("create mock_dir");

        let release_json = serde_json::json!({
            "tag_name": "v1.1.0",
            "html_url": "https://github.com/3mr-5aled/rtb/releases/tag/v1.1.0",
            "assets": []
        });
        fs::write(mock_dir.join("release.json"), serde_json::to_string(&release_json).unwrap()).unwrap();

        std::env::set_var("RTB_MOCK_RELEASE_DIR", mock_dir.to_str().unwrap());

        let exit_code = RtbEngine::dispatch_args(vec!["rtb", "upgrade", "--check"])
            .expect("dispatch upgrade --check");
        assert_eq!(exit_code, 0, "rtb upgrade --check with newer release should exit 0");

        std::env::remove_var("RTB_MOCK_RELEASE_DIR");
    }

    // Case 3: Checksum mismatch
    {
        let temp = tempdir().expect("tempdir");
        let mock_dir = temp.path().join("mock_release");
        fs::create_dir_all(&mock_dir).expect("create mock_dir");

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        let (bin_asset, sidecar_asset) = ("rtb-windows-amd64.exe", "rtb-windows-amd64.exe.sha256");
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        let (bin_asset, sidecar_asset) = ("rtb-linux-amd64", "rtb-linux-amd64.sha256");
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        let (bin_asset, sidecar_asset) = ("rtb-macos-amd64", "rtb-macos-amd64.sha256");
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        let (bin_asset, sidecar_asset) = ("rtb-macos-arm64", "rtb-macos-arm64.sha256");

        let dummy_bin = b"dummy binary contents v1.1.0";
        fs::write(mock_dir.join(bin_asset), dummy_bin).unwrap();
        fs::write(mock_dir.join(sidecar_asset), "0000000000000000000000000000000000000000000000000000000000000000  ".to_string() + bin_asset).unwrap();

        let release_json = serde_json::json!({
            "tag_name": "v1.1.0",
            "html_url": "https://github.com/3mr-5aled/rtb/releases/tag/v1.1.0",
            "assets": [
                { "name": bin_asset, "browser_download_url": format!("http://mock/{}", bin_asset) },
                { "name": sidecar_asset, "browser_download_url": format!("http://mock/{}", sidecar_asset) }
            ]
        });
        fs::write(mock_dir.join("release.json"), serde_json::to_string(&release_json).unwrap()).unwrap();

        std::env::set_var("RTB_MOCK_RELEASE_DIR", mock_dir.to_str().unwrap());

        let exit_code = RtbEngine::dispatch_args(vec!["rtb", "upgrade"])
            .expect("dispatch upgrade");
        assert_eq!(exit_code, 1, "rtb upgrade should exit 1 on checksum mismatch");

        std::env::remove_var("RTB_MOCK_RELEASE_DIR");
    }

    // Case 4: Success
    {
        let temp = tempdir().expect("tempdir");
        let mock_dir = temp.path().join("mock_release");
        fs::create_dir_all(&mock_dir).expect("create mock_dir");

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        let (bin_asset, sidecar_asset) = ("rtb-windows-amd64.exe", "rtb-windows-amd64.exe.sha256");
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        let (bin_asset, sidecar_asset) = ("rtb-linux-amd64", "rtb-linux-amd64.sha256");
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        let (bin_asset, sidecar_asset) = ("rtb-macos-amd64", "rtb-macos-amd64.sha256");
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        let (bin_asset, sidecar_asset) = ("rtb-macos-arm64", "rtb-macos-arm64.sha256");

        let dummy_bin = b"dummy binary contents v1.1.0";
        fs::write(mock_dir.join(bin_asset), dummy_bin).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(dummy_bin);
        let correct_hash = format!("{:x}", hasher.finalize());

        fs::write(mock_dir.join(sidecar_asset), format!("{}  {}\n", correct_hash, bin_asset)).unwrap();

        let release_json = serde_json::json!({
            "tag_name": "v1.1.0",
            "html_url": "https://github.com/3mr-5aled/rtb/releases/tag/v1.1.0",
            "assets": [
                { "name": bin_asset, "browser_download_url": format!("http://mock/{}", bin_asset) },
                { "name": sidecar_asset, "browser_download_url": format!("http://mock/{}", sidecar_asset) }
            ]
        });
        fs::write(mock_dir.join("release.json"), serde_json::to_string(&release_json).unwrap()).unwrap();

        std::env::set_var("RTB_MOCK_RELEASE_DIR", mock_dir.to_str().unwrap());
        std::env::set_var("RTB_MOCK_DO_NOT_REPLACE", "1");

        let exit_code = RtbEngine::dispatch_args(vec!["rtb", "upgrade"])
            .expect("dispatch upgrade");
        assert_eq!(exit_code, 0, "rtb upgrade should exit 0 on successful upgrade");

        std::env::remove_var("RTB_MOCK_RELEASE_DIR");
        std::env::remove_var("RTB_MOCK_DO_NOT_REPLACE");
    }
}
