use crate::colors::*;
use crate::platform::{Platform, command_exists};
use anyhow::Context;
use std::env;
use std::fs;
use std::process::Command;

/// Resolves the release asset name for a given platform.
pub fn resolve_asset_name(platform: &Platform) -> anyhow::Result<&'static str> {
    match platform {
        Platform::Debian => Ok("dotss-x86_64-unknown-linux-gnu.tar.gz"),
        Platform::Termux => Ok("dotss-aarch64-unknown-linux-musl.tar.gz"),
        Platform::Unsupported(reason) => {
            anyhow::bail!("Unsupported platform for self-update: {reason}")
        }
    }
}

/// Checks for updates via GitHub Releases API and performs self-update or dry-run simulation.
pub fn check_and_perform_update(
    current_version: &str,
    platform: &Platform,
    dry_run: bool,
) -> anyhow::Result<()> {
    println!("Checking GitHub Releases for updates...");
    println!("Current version: v{current_version}");

    if !command_exists("curl") {
        anyhow::bail!("Prerequisite binary 'curl' is required for self-update checks.");
    }

    // Default GitHub repository path (overridable via DOTSS_REPO / PROJECT_DOTS_REPO env var for testing)
    let repo = env::var("DOTSS_REPO")
        .or_else(|_| env::var("PROJECT_DOTS_REPO"))
        .unwrap_or_else(|_| "fusoras/project-dots".to_string());
    let api_url = format!("https://api.github.com/repos/{repo}/releases/latest");

    // Fetch latest release payload or construct tag query
    let latest_tag =
        fetch_latest_release_tag(&api_url, 10).unwrap_or_else(|_| format!("v{current_version}"));
    println!("Latest release tag: {latest_tag}");

    if !is_newer_version(&latest_tag, current_version) {
        println!(
            "\n[Up-to-Date] dotss is already running the latest version (v{current_version})."
        );
        Ok(())
    } else {
        let asset_name = resolve_asset_name(platform)?;
        let download_url =
            format!("https://github.com/{repo}/releases/download/{latest_tag}/{asset_name}");

        let current_exe = env::current_exe().context("Failed to locate current executable path")?;

        if dry_run {
            println!("\n[Dry-Run] Would download pre-compiled release binary asset: {asset_name}");
            println!("  URL: {download_url}");
            println!(
                "  [Dry-Run] Would extract and replace executable at: {}",
                current_exe.display()
            );
            Ok(())
        } else {
            println!("\n[Downloading] Fetching release binary from {download_url}...");
            let tmp_dir = env::temp_dir().join("dotss_update");
            fs::create_dir_all(&tmp_dir).context("Failed to create temp directory")?;
            let tmp_tarball = tmp_dir.join("update.tar.gz");

            let curl_status = Command::new("curl")
                .arg("-fsSL")
                .arg("-o")
                .arg(&tmp_tarball)
                .arg(&download_url)
                .status()
                .context("Failed to execute curl")?;

            if !curl_status.success() {
                anyhow::bail!("Failed to download update binary from {download_url}");
            }

            let extract_status = Command::new("tar")
                .arg("-xzf")
                .arg(&tmp_tarball)
                .arg("-C")
                .arg(&tmp_dir)
                .status()
                .context("Failed to extract update tarball")?;

            if !extract_status.success() {
                anyhow::bail!("Failed to extract update tarball payload.");
            }

            let new_binary = tmp_dir.join("dotss");
            if !new_binary.exists() {
                anyhow::bail!("Extracted tarball did not contain expected 'dotss' binary.");
            }

            // Atomic binary replacement
            let backup_exe = current_exe.with_extension("old");
            if let Err(e) = fs::rename(&current_exe, &backup_exe) {
                eprintln!("[WARN] Failed to backup current binary: {e}");
            }

            fs::copy(&new_binary, &current_exe).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to replace executable at {}: {e}",
                    current_exe.display()
                )
            })?;

            if let Err(e) = fs::remove_file(&backup_exe) {
                eprintln!("[WARN] Failed to remove backup file: {e}");
            }
            if let Err(e) = fs::remove_dir_all(&tmp_dir) {
                eprintln!("[WARN] Failed to clean up temp dir: {e}");
            }

            println!("\n{BOLD_GREEN}Self-update completed successfully!{RESET}");
            println!("Updated binary placed at: {}", current_exe.display());

            Ok(())
        }
    }
}

/// Safely removes dotss binary executable and state/config directories.
pub fn perform_self_uninstall(
    config: &crate::config::Config,
    state: &mut crate::state::State,
    platform: &crate::platform::Platform,
    dry_run: bool,
    auto_confirm: bool,
    auto_reject: bool,
) -> anyhow::Result<()> {
    let current_exe = env::current_exe().context("Failed to resolve current binary path")?;

    let state_path = crate::state::State::get_state_path();
    let state_dir = state_path.parent().unwrap_or(&state_path);
    let config_dir = crate::config::Config::get_user_config_dir();

    println!("=== dotss Self-Uninstall Engine ===");
    println!("Target Binary Path: {}", current_exe.display());
    println!("Target State Directory: {}", state_dir.display());
    if let Some(ref cfg_dir) = config_dir {
        println!("Target Config Directory: {}", cfg_dir.display());
    }

    let should_remove_packages_and_config = if auto_reject {
        false
    } else if auto_confirm {
        true
    } else {
        use std::io::{self, Write};
        print!(
            "\nDo you want to uninstall packages installed by dotss and remove configuration/state directories? [y/N]: "
        );
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let trimmed = input.trim().to_lowercase();
            trimmed == "y" || trimmed == "yes"
        } else {
            false
        }
    };

    if dry_run {
        println!("\n=== DRY-RUN MODE ACTIVE: No files will be deleted ===");
        if should_remove_packages_and_config {
            println!("[Dry-Run] Would uninstall all packages managed by dotss.");
            if state_dir.exists() {
                println!(
                    "[Dry-Run] Would remove state directory: {}",
                    state_dir.display()
                );
            }
            if config_dir.as_ref().is_some_and(|d| d.exists()) {
                println!(
                    "[Dry-Run] Would remove config directory: {}",
                    config_dir.as_ref().unwrap().display()
                );
            }
        } else {
            println!(
                "[Dry-Run] Managed packages, state directory, and config directory will be kept intact (--no / -n / declined)."
            );
        }
        println!(
            "[Dry-Run] Would remove executable: {}",
            current_exe.display()
        );
        return Ok(());
    }

    // 1. Uninstall packages managed by dotss if confirmed
    if should_remove_packages_and_config {
        println!("\n--> Uninstalling all packages managed by dotss...");
        if let Err(e) =
            crate::installer::remove_category(Some("all"), config, state, platform, dry_run)
        {
            eprintln!("[WARNING] Error while removing packages: {e}");
        }
    } else {
        println!("\n[Preserved] Managed packages kept intact.");
    }

    // 2. Remove binary executable
    if current_exe.exists() {
        fs::remove_file(&current_exe).map_err(|e| {
            anyhow::anyhow!("Failed to remove binary at {}: {e}", current_exe.display())
        })?;
        println!("✓ Executable removed: {}", current_exe.display());
    }

    // 3. Remove configuration and state directories if confirmed
    if should_remove_packages_and_config {
        if state_dir.exists() {
            fs::remove_dir_all(state_dir).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to remove state directory at {}: {e}",
                    state_dir.display()
                )
            })?;
            println!("✓ State directory removed: {}", state_dir.display());
        }
        if config_dir.as_ref().is_some_and(|d| d.exists()) {
            let cfg_dir = config_dir.as_ref().unwrap();
            fs::remove_dir_all(cfg_dir).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to remove config directory at {}: {e}",
                    cfg_dir.display()
                )
            })?;
            println!("✓ Config directory removed: {}", cfg_dir.display());
        }
    } else {
        println!("[Preserved] Configuration and state directories kept intact.");
    }

    println!("\n{BOLD_GREEN}dotss uninstalled successfully!{RESET}");
    println!(
        "Tip: Remember to remove PATH entries from ~/.zshrc or ~/.bashrc if no longer needed."
    );

    Ok(())
}

/// Returns cached latest version tag if a newer version is saved in state.toml. (0ms latency, zero IO delay).
pub fn check_version_update(current_version: &str) -> Option<String> {
    let state = crate::state::State::load();
    if let Some(cached) = state.cached_latest_version
        && is_newer_version(&cached, current_version)
    {
        return Some(cached);
    }
    None
}

/// Spawns a detached background thread to check GitHub Releases API and update state.toml asynchronously.
pub fn spawn_background_version_check(current_version: &str) {
    let current_ver = current_version.to_string();
    std::thread::spawn(move || {
        if !command_exists("curl") {
            return;
        }

        let repo = env::var("DOTSS_REPO")
            .or_else(|_| env::var("PROJECT_DOTS_REPO"))
            .unwrap_or_else(|_| "fusoras/project-dots".to_string());
        let api_url = format!("https://api.github.com/repos/{repo}/releases/latest");

        if let Ok(latest_tag) = fetch_latest_release_tag(&api_url, 3) {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let mut state = crate::state::State::load();
            if is_newer_version(&latest_tag, &current_ver) {
                state.cached_latest_version = Some(latest_tag);
            } else {
                state.cached_latest_version = None;
            }
            state.last_update_check_epoch = Some(now_secs);
            let _ = state.save_atomic();
        }
    });
}

/// Fetches the tag_name from GitHub Releases API response using curl.
fn fetch_latest_release_tag(url: &str, max_time_secs: u32) -> anyhow::Result<String> {
    let output = Command::new("curl")
        .arg("-fsSL")
        .arg("--max-time")
        .arg(max_time_secs.to_string())
        .arg("-H")
        .arg("User-Agent: dotss-cli")
        .arg(url)
        .output()
        .context("curl failed")?;

    if !output.status.success() {
        anyhow::bail!("curl failed to fetch releases");
    }

    let body = String::from_utf8_lossy(&output.stdout);
    if let Some(pos) = body.find("\"tag_name\":") {
        let remainder = &body[pos + 11..];
        let start = remainder.find('"').unwrap_or(0) + 1;
        let end = remainder[start..].find('"').unwrap_or(0) + start;
        return Ok(remainder[start..end].to_string());
    }

    anyhow::bail!("tag_name not found in release response")
}

/// Helper function to compare release tags against the current semver version string.
pub fn is_newer_version(latest_tag: &str, current_version: &str) -> bool {
    let tag = latest_tag.trim_start_matches('v');
    tag != current_version && semver_greater(tag, current_version)
}

/// Basic semver string comparison logic.
fn semver_greater(v1: &str, v2: &str) -> bool {
    let parse_parts = |v: &str| {
        let main_part = v.split('-').next().unwrap_or(v);
        let nums: Vec<u32> = main_part
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let build = if v.contains("-beta.") {
            v.split("-beta.")
                .nth(1)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0)
        } else {
            999
        };
        (nums, build)
    };

    let (p1, b1) = parse_parts(v1);
    let (p2, b2) = parse_parts(v2);

    if p1 != p2 { p1 > p2 } else { b1 > b2 }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod version_comparison {
        use super::*;

        #[test]
        fn newer_version_should_return_true_for_higher_beta() {
            assert!(is_newer_version("v0.1.0-beta.2", "0.1.0-beta.1"));
        }

        #[test]
        fn newer_version_should_return_false_for_same_version() {
            assert!(!is_newer_version("v0.1.0-beta.1", "0.1.0-beta.1"));
        }

        #[test]
        fn newer_version_should_return_false_for_older_version() {
            assert!(!is_newer_version("v0.0.9", "0.1.0-beta.1"));
        }

        #[test]
        fn check_version_update_should_return_none_for_matching_or_future_version() {
            assert_eq!(check_version_update("99.0.0"), None);
        }
    }

    mod asset_resolution {
        use super::*;

        #[test]
        fn asset_name_should_match_debian_x86_64_triple() {
            let asset = resolve_asset_name(&Platform::Debian).unwrap();
            assert_eq!(asset, "dotss-x86_64-unknown-linux-gnu.tar.gz");
        }

        #[test]
        fn asset_name_should_match_termux_aarch64_triple() {
            let asset = resolve_asset_name(&Platform::Termux).unwrap();
            assert_eq!(asset, "dotss-aarch64-unknown-linux-musl.tar.gz");
        }
    }

    mod self_uninstall {
        use super::*;
        use crate::config::Config;
        use crate::state::State;

        #[test]
        fn self_uninstall_should_preview_without_deleting_in_dry_run() {
            let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
            let mut state = State::load();
            let platform = Platform::Debian;
            let result = perform_self_uninstall(&config, &mut state, &platform, true, true, false);
            assert!(
                result.is_ok(),
                "Self-uninstall dry-run should complete cleanly"
            );
        }

        #[test]
        fn self_uninstall_should_respect_auto_reject_flag() {
            let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
            let mut state = State::load();
            let platform = Platform::Debian;
            let result = perform_self_uninstall(&config, &mut state, &platform, true, false, true);
            assert!(
                result.is_ok(),
                "Self-uninstall with auto_reject should complete cleanly"
            );
        }
    }
}
