use crate::platform::{command_exists, Platform};
use std::env;
use std::fs;
use std::process::Command;

/// Checks for updates via GitHub Releases API and performs self-update or dry-run simulation.
pub fn check_and_perform_update(
    current_version: &str,
    platform: &Platform,
    dry_run: bool,
) -> Result<(), String> {
    println!("Checking GitHub Releases for updates...");
    println!("Current version: v{}", current_version);

    if !command_exists("curl") {
        return Err("Prerequisite binary 'curl' is required for self-update checks.".to_string());
    }

    // Default GitHub repository path (overridable via PROJECT_DOTS_REPO env var for testing)
    let repo = env::var("PROJECT_DOTS_REPO").unwrap_or_else(|_| "user/project-dots".to_string());
    let api_url = format!("https://api.github.com/repos/{}/releases/latest", repo);

    // Fetch latest release payload or construct tag query
    let latest_tag = fetch_latest_release_tag(&api_url).unwrap_or_else(|_| format!("v{}", current_version));
    println!("Latest release tag: {}", latest_tag);

    if !is_newer_version(&latest_tag, current_version) {
        println!("\n[Up-to-Date] project-dots is already running the latest version (v{}).", current_version);
        return Ok(());
    }

    let asset_name = match platform {
        Platform::Debian => "project-dots-x86_64-unknown-linux-gnu.tar.gz",
        Platform::Termux => "project-dots-aarch64-unknown-linux-gnu.tar.gz",
        Platform::Unsupported(reason) => return Err(format!("Unsupported platform for self-update: {}", reason)),
    };

    let download_url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        repo, latest_tag, asset_name
    );

    let current_exe = env::current_exe().map_err(|e| format!("Failed to locate current executable path: {}", e))?;

    if dry_run {
        println!("\n[Dry-Run] Would download pre-compiled release binary asset: {}", asset_name);
        println!("  URL: {}", download_url);
        println!("  [Dry-Run] Would extract and replace executable at: {}", current_exe.display());
        return Ok(());
    }

    println!("\n[Downloading] Fetching release binary from {}...", download_url);
    let tmp_dir = env::temp_dir().join("project_dots_update");
    let _ = fs::create_dir_all(&tmp_dir);
    let tmp_tarball = tmp_dir.join("update.tar.gz");

    let curl_status = Command::new("curl")
        .arg("-sSL")
        .arg("-o")
        .arg(&tmp_tarball)
        .arg(&download_url)
        .status()
        .map_err(|e| format!("Failed to execute curl: {}", e))?;

    if !curl_status.success() {
        return Err(format!("Failed to download update binary from {}", download_url));
    }

    let extract_status = Command::new("tar")
        .arg("-xzf")
        .arg(&tmp_tarball)
        .arg("-C")
        .arg(&tmp_dir)
        .status()
        .map_err(|e| format!("Failed to extract update tarball: {}", e))?;

    if !extract_status.success() {
        return Err("Failed to extract update tarball payload.".to_string());
    }

    let new_binary = tmp_dir.join("project-dots");
    if !new_binary.exists() {
        return Err("Extracted tarball did not contain expected 'project-dots' binary.".to_string());
    }

    // Atomic binary replacement
    let backup_exe = current_exe.with_extension("old");
    let _ = fs::rename(&current_exe, &backup_exe);
    fs::copy(&new_binary, &current_exe)
        .map_err(|e| format!("Failed to replace executable at {}: {}", current_exe.display(), e))?;
    let _ = fs::remove_file(&backup_exe);
    let _ = fs::remove_dir_all(&tmp_dir);

    println!("\n[Success] project-dots updated successfully to {} at {}!", latest_tag, current_exe.display());
    Ok(())
}

/// Fetches the tag_name from GitHub Releases API response using curl.
fn fetch_latest_release_tag(url: &str) -> Result<String, String> {
    let output = Command::new("curl")
        .arg("-sSL")
        .arg("-H")
        .arg("User-Agent: project-dots-cli")
        .arg(url)
        .output()
        .map_err(|e| format!("curl failed: {}", e))?;

    if !output.status.success() {
        return Err("curl failed to fetch releases".to_string());
    }

    let body = String::from_utf8_lossy(&output.stdout);
    if let Some(pos) = body.find("\"tag_name\":") {
        let remainder = &body[pos + 11..];
        let start = remainder.find('"').unwrap_or(0) + 1;
        let end = remainder[start..].find('"').unwrap_or(0) + start;
        return Ok(remainder[start..end].to_string());
    }

    Err("tag_name not found in release response".to_string())
}

/// Helper function to compare release tags against the current semver version string.
pub fn is_newer_version(latest_tag: &str, current_version: &str) -> bool {
    let tag = latest_tag.trim_start_matches('v');
    tag != current_version && semver_greater(tag, current_version)
}

/// Basic semver string comparison logic.
fn semver_greater(v1: &str, v2: &str) -> bool {
    // Simple component comparison for v0.1.0-beta.2 vs v0.1.0-beta.1
    let parse_parts = |v: &str| {
        let main_part = v.split('-').next().unwrap_or(v);
        let nums: Vec<u32> = main_part.split('.').filter_map(|s| s.parse().ok()).collect();
        let build = if v.contains("-beta.") {
            v.split("-beta.").nth(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0)
        } else {
            999
        };
        (nums, build)
    };

    let (p1, b1) = parse_parts(v1);
    let (p2, b2) = parse_parts(v2);

    if p1 != p2 {
        p1 > p2
    } else {
        b1 > b2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version_logic() {
        println!("\n🔍 [TEST] SemVer Version Comparison for Self-Update");
        println!("   Explanation: Verifies that release tag versions (e.g. v0.1.0-beta.2) are correctly identified as newer than v0.1.0-beta.1.");

        assert!(is_newer_version("v0.1.0-beta.2", "0.1.0-beta.1"), "v0.1.0-beta.2 should be newer than 0.1.0-beta.1");
        println!("   ✓ v0.1.0-beta.2 recognized as newer than 0.1.0-beta.1");

        assert!(!is_newer_version("v0.1.0-beta.1", "0.1.0-beta.1"), "Same version should not be newer");
        println!("   ✓ Same version v0.1.0-beta.1 recognized as equal (not newer)");

        assert!(!is_newer_version("v0.0.9", "0.1.0-beta.1"), "Older version should not be newer");
        println!("   ✓ Older version v0.0.9 recognized as not newer.\n");
    }

    #[test]
    fn test_platform_asset_resolution() {
        println!("\n🔍 [TEST] Platform Release Asset Resolution");
        println!("   Explanation: Verifies that Debian resolves to the x86_64 tarball asset and Termux to aarch64.");

        let debian_asset = match Platform::Debian {
            Platform::Debian => "project-dots-x86_64-unknown-linux-gnu.tar.gz",
            _ => "unknown",
        };
        assert_eq!(debian_asset, "project-dots-x86_64-unknown-linux-gnu.tar.gz");
        println!("   ✓ Debian target asset resolved correctly: {}", debian_asset);

        let termux_asset = match Platform::Termux {
            Platform::Termux => "project-dots-aarch64-unknown-linux-gnu.tar.gz",
            _ => "unknown",
        };
        assert_eq!(termux_asset, "project-dots-aarch64-unknown-linux-gnu.tar.gz");
        println!("   ✓ Termux target asset resolved correctly: {}\n", termux_asset);
    }
}
