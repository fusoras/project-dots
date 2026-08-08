use crate::config::{Config, CustomInstaller};
use crate::platform::{check_apt_lock, command_exists, Platform};
use crate::state::State;
use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;

const RESET: &str = "\x1b[0m";
const DIM_GRAY: &str = "\x1b[90m";

/// Lists all categories and packages.
/// Default mode: Clean user view with comma-separated packages in dim gray.
/// Debug mode (`debug == true`): Detailed status for each package and system info.
pub fn list_categories(config: &Config, state: &State, platform: &Platform, debug: bool) {
    if config.categories.is_empty() {
        println!("No categories found in configuration.");
        return;
    }

    if debug {
        println!("\n=== project-dots: Categories & Package Status (DEBUG MODE) ===");
        println!("Platform Detected: {:?}\n", platform);

        for (cat_name, cat) in &config.categories {
            let alias_str = cat
                .aliases
                .as_ref()
                .map(|a| format!(" (Aliases: {})", a.join(", ")))
                .unwrap_or_default();

            println!("Category: {}{}", cat_name, alias_str);
            println!("  Description: {}", cat.description);

            let pkgs = match platform {
                Platform::Debian => cat.debian_packages.as_deref().unwrap_or(&[]),
                Platform::Termux => cat.termux_packages.as_deref().unwrap_or(&[]),
                Platform::Unsupported(_) => &[],
            };

            if !pkgs.is_empty() {
                println!("  Packages:");
                for pkg in pkgs {
                    let status_str = if let Some(tracked) = state.packages.get(pkg) {
                        if tracked.was_preexisting {
                            "Pre-existing (system)"
                        } else {
                            "Installed by project-dots"
                        }
                    } else if platform.is_package_installed(pkg) {
                        "Installed (untracked)"
                    } else {
                        "Not installed"
                    };
                    println!("    - {DIM_GRAY}{}{RESET} [{}]", pkg, status_str);
                }
            }

            if matches!(platform, Platform::Debian)
                && let Some(custom) = cat.custom.as_ref().and_then(|m| m.get("debian"))
            {
                let bin_path = expand_home(&custom.bin_symlink);
                let custom_status = if Path::new(&bin_path).exists() {
                    "Installed (Custom binary)"
                } else {
                    "Not installed"
                };
                println!("  Custom Installer (Debian):");
                println!("    - {DIM_GRAY}{}{RESET} ({}) [{}]", custom.name, custom.url, custom_status);
            }
            println!();
        }
    } else {
        println!("\n=== Available Categories ===");

        for (cat_name, cat) in &config.categories {
            let alias_str = cat
                .aliases
                .as_ref()
                .map(|a| format!(" (Aliases: {})", a.join(", ")))
                .unwrap_or_default();

            println!("\nCategory: {}{}", cat_name, alias_str);
            println!("  Description: {}", cat.description);

            let mut all_pkgs: Vec<String> = match platform {
                Platform::Debian => cat.debian_packages.clone().unwrap_or_default(),
                Platform::Termux => cat.termux_packages.clone().unwrap_or_default(),
                Platform::Unsupported(_) => Vec::new(),
            };

            if matches!(platform, Platform::Debian)
                && let Some(custom) = cat.custom.as_ref().and_then(|m| m.get("debian"))
            {
                all_pkgs.push(format!("{} (custom binary)", custom.name));
            }

            if !all_pkgs.is_empty() {
                println!("  Packages: {DIM_GRAY}{}{RESET}", all_pkgs.join(", "));
            }
        }
        println!();
    }
}

/// Installs all packages or a specific category safely.
pub fn install_category(
    category_filter: Option<&str>,
    config: &Config,
    state: &mut State,
    platform: &Platform,
    dry_run: bool,
) -> Result<(), String> {
    if let Platform::Unsupported(reason) = platform {
        return Err(format!("Unsupported platform: {}", reason));
    }

    if let Platform::Debian = platform {
        check_apt_lock()?;
    }

    let target_categories: Vec<(&String, &crate::config::Category)> = match category_filter {
        Some("all") | None => config.categories.iter().collect(),
        Some(cat) => {
            if let Some(canonical_key) = config.resolve_category_key(cat) {
                let category_def = config.categories.get(canonical_key).unwrap();
                vec![(canonical_key, category_def)]
            } else {
                return Err(format!("Category or alias '{}' not found in configuration.", cat));
            }
        }
    };

    for (cat_name, cat) in target_categories {
        println!("\n--> Processing Category: {}", cat_name);

        let pkgs = match platform {
            Platform::Debian => cat.debian_packages.as_deref().unwrap_or(&[]),
            Platform::Termux => cat.termux_packages.as_deref().unwrap_or(&[]),
            Platform::Unsupported(_) => &[],
        };

        for pkg in pkgs {
            if platform.is_package_installed(pkg) {
                println!("  [SKIP] Package '{}' is already installed on OS.", pkg);
                state.track_package(pkg, cat_name, true);
                state.save_atomic()?;
                continue;
            }

            match platform {
                Platform::Debian => {
                    let cmd_str = format!("sudo apt install -y {}", pkg);
                    if dry_run {
                        println!("  [Dry-Run] Would execute: {}", cmd_str);
                    } else {
                        println!("  [Installing] Executing: {}", cmd_str);
                        let status = Command::new("sudo")
                            .arg("apt")
                            .arg("install")
                            .arg("-y")
                            .arg(pkg)
                            .status()
                            .map_err(|e| format!("Failed to run apt install: {}", e))?;

                        if !status.success() {
                            return Err(format!("apt install failed for package '{}'", pkg));
                        }
                        state.track_package(pkg, cat_name, false);
                        state.save_atomic()?;
                    }
                }
                Platform::Termux => {
                    let cmd_str = format!("pkg install -y {}", pkg);
                    if dry_run {
                        println!("  [Dry-Run] Would execute: {}", cmd_str);
                    } else {
                        println!("  [Installing] Executing: {}", cmd_str);
                        let status = Command::new("pkg")
                            .arg("install")
                            .arg("-y")
                            .arg(pkg)
                            .status()
                            .map_err(|e| format!("Failed to run pkg install: {}", e))?;

                        if !status.success() {
                            return Err(format!("pkg install failed for package '{}'", pkg));
                        }
                        state.track_package(pkg, cat_name, false);
                        state.save_atomic()?;
                    }
                }
                Platform::Unsupported(_) => {}
            }
        }

        if matches!(platform, Platform::Debian)
            && let Some(custom) = cat.custom.as_ref().and_then(|m| m.get("debian"))
        {
            install_custom_debian(custom, cat_name, state, dry_run)?;
        }
    }

    Ok(())
}

/// Handles special binary installations (e.g. Neovim on Debian via tar.gz release).
fn install_custom_debian(
    custom: &CustomInstaller,
    cat_name: &str,
    state: &mut State,
    dry_run: bool,
) -> Result<(), String> {
    println!("  --> Custom Binary Installer: {}", custom.name);

    if !command_exists("curl") || !command_exists("tar") {
        return Err("Prerequisite binaries 'curl' and 'tar' are required for custom downloads. Please install them first.".to_string());
    }

    let bin_path_buf = Path::new(&expand_home(&custom.bin_symlink)).to_path_buf();
    let extract_dir_buf = Path::new(&custom.extract_dir).to_path_buf();

    if bin_path_buf.exists() && extract_dir_buf.exists() {
        println!("  [SKIP] Custom binary '{}' is already installed at {}", custom.name, extract_dir_buf.display());
        state.track_package(&custom.name, cat_name, true);
        state.save_atomic()?;
        return Ok(());
    }

    if dry_run {
        println!("  [Dry-Run] Would download release tarball from {}", custom.url);
        println!("  [Dry-Run] Would extract to {}", custom.extract_dir);
        println!("  [Dry-Run] Would create symlink: {} -> {}/bin/{}", bin_path_buf.display(), custom.extract_dir, custom.name);
        return Ok(());
    }

    let tmp_tarball = format!("/tmp/{}.tar.gz", custom.name);
    println!("  [Downloading] Fetching latest tarball from {}...", custom.url);
    let curl_status = Command::new("curl")
        .arg("-sSL")
        .arg("-o")
        .arg(&tmp_tarball)
        .arg(&custom.url)
        .status()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !curl_status.success() {
        return Err(format!("Failed to download tarball from {}", custom.url));
    }

    println!("  [Extracting] Moving to {}...", custom.extract_dir);
    let mkdir_status = Command::new("sudo")
        .arg("mkdir")
        .arg("-p")
        .arg(&custom.extract_dir)
        .status()
        .map_err(|e| format!("Failed to create extract directory: {}", e))?;

    if !mkdir_status.success() {
        return Err(format!("Failed to create directory {}", custom.extract_dir));
    }

    let tar_status = Command::new("sudo")
        .arg("tar")
        .arg("-xzf")
        .arg(&tmp_tarball)
        .arg("-C")
        .arg(&custom.extract_dir)
        .arg("--strip-components=1")
        .status()
        .map_err(|e| format!("Failed to extract tarball: {}", e))?;

    let _ = fs::remove_file(&tmp_tarball);

    if !tar_status.success() {
        return Err(format!("Failed to extract tarball to {}", custom.extract_dir));
    }

    let symlink_dir = bin_path_buf.parent().unwrap();
    fs::create_dir_all(symlink_dir)
        .map_err(|e| format!("Failed to create symlink parent directory {}: {}", symlink_dir.display(), e))?;

    if bin_path_buf.exists() || bin_path_buf.is_symlink() {
        let _ = fs::remove_file(&bin_path_buf);
    }

    let target_bin = format!("{}/bin/{}", custom.extract_dir, custom.name);
    symlink(&target_bin, &bin_path_buf)
        .map_err(|e| format!("Failed to create symlink at {}: {}", bin_path_buf.display(), e))?;

    println!("  [Success] Installed {} to {} with symlink at {}", custom.name, custom.extract_dir, bin_path_buf.display());
    state.track_package(&custom.name, cat_name, false);
    state.save_atomic()?;

    check_path_and_recommend(symlink_dir);

    Ok(())
}

/// Safely removes packages installed by project-dots.
pub fn remove_category(
    category_filter: Option<&str>,
    config: &Config,
    state: &mut State,
    platform: &Platform,
    dry_run: bool,
) -> Result<(), String> {
    if let Platform::Unsupported(reason) = platform {
        return Err(format!("Unsupported platform: {}", reason));
    }

    if let Platform::Debian = platform {
        check_apt_lock()?;
    }

    let target_categories: Vec<(&String, &crate::config::Category)> = match category_filter {
        Some("all") | None => config.categories.iter().collect(),
        Some(cat) => {
            if let Some(canonical_key) = config.resolve_category_key(cat) {
                let category_def = config.categories.get(canonical_key).unwrap();
                vec![(canonical_key, category_def)]
            } else {
                return Err(format!("Category or alias '{}' not found in configuration.", cat));
            }
        }
    };

    for (cat_name, cat) in target_categories {
        println!("\n--> Processing Removal for Category: {}", cat_name);

        let pkgs = match platform {
            Platform::Debian => cat.debian_packages.as_deref().unwrap_or(&[]),
            Platform::Termux => cat.termux_packages.as_deref().unwrap_or(&[]),
            Platform::Unsupported(_) => &[],
        };

        for pkg in pkgs {
            if let Some(tracked) = state.packages.get(pkg) {
                if tracked.was_preexisting {
                    println!("  [SKIP] Package '{}' was pre-existing on system before project-dots. Skipping removal.", pkg);
                    continue;
                }
            } else {
                println!("  [SKIP] Package '{}' was not installed by project-dots. Skipping removal.", pkg);
                continue;
            }

            match platform {
                Platform::Debian => {
                    let cmd_str = format!("sudo apt remove -y {}", pkg);
                    if dry_run {
                        println!("  [Dry-Run] Would execute: {}", cmd_str);
                    } else {
                        println!("  [Removing] Executing: {}", cmd_str);
                        let status = Command::new("sudo")
                            .arg("apt")
                            .arg("remove")
                            .arg("-y")
                            .arg(pkg)
                            .status()
                            .map_err(|e| format!("Failed to run apt remove: {}", e))?;

                        if !status.success() {
                            return Err(format!("apt remove failed for package '{}'", pkg));
                        }
                        state.remove_package(pkg);
                        state.save_atomic()?;
                    }
                }
                Platform::Termux => {
                    let cmd_str = format!("pkg remove -y {}", pkg);
                    if dry_run {
                        println!("  [Dry-Run] Would execute: {}", cmd_str);
                    } else {
                        println!("  [Removing] Executing: {}", cmd_str);
                        let status = Command::new("pkg")
                            .arg("remove")
                            .arg("-y")
                            .arg(pkg)
                            .status()
                            .map_err(|e| format!("Failed to run pkg remove: {}", e))?;

                        if !status.success() {
                            return Err(format!("pkg remove failed for package '{}'", pkg));
                        }
                        state.remove_package(pkg);
                        state.save_atomic()?;
                    }
                }
                Platform::Unsupported(_) => {}
            }
        }

        if matches!(platform, Platform::Debian)
            && let Some(custom) = cat.custom.as_ref().and_then(|m| m.get("debian"))
            && let Some(tracked) = state.packages.get(&custom.name)
            && !tracked.was_preexisting
        {
            let bin_path = expand_home(&custom.bin_symlink);
            if dry_run {
                println!("  [Dry-Run] Would remove symlink: {}", bin_path);
                println!("  [Dry-Run] Would execute: sudo rm -rf {}", custom.extract_dir);
            } else {
                println!("  [Removing] Removing custom binary '{}'...", custom.name);
                let _ = fs::remove_file(&bin_path);
                let _ = Command::new("sudo")
                    .arg("rm")
                    .arg("-rf")
                    .arg(&custom.extract_dir)
                    .status();
                state.remove_package(&custom.name);
                state.save_atomic()?;
            }
        }
    }

    Ok(())
}

/// Helper function to expand ~ to user's HOME directory in paths.
pub fn expand_home(path: &str) -> String {
    if let (Some(stripped), Ok(home)) = (path.strip_prefix("~/"), env::var("HOME")) {
        format!("{}/{}", home, stripped)
    } else {
        path.to_string()
    }
}

/// Helper to check if a directory is in $PATH and print setup recommendations if missing.
fn check_path_and_recommend(dir: &Path) {
    if let Ok(path_var) = env::var("PATH") {
        let dir_str = dir.to_string_lossy();
        if !path_var.split(':').any(|p| p == dir_str) {
            println!("\n  [TIP] '{}' is NOT in your current $PATH environment variable!", dir_str);
            println!("  To access binaries directly from anywhere, add it to your shell configuration:");
            println!("    - bash: echo 'export PATH=\"{}:$PATH\"' >> ~/.bashrc", dir_str);
            println!("    - zsh:  echo 'export PATH=\"{}:$PATH\"' >> ~/.zshrc", dir_str);
            println!("    - fish: fish_add_path {}\n", dir_str);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_home_utility() {
        let path_with_tilde = "~/.local/bin/nvim";
        let expanded = expand_home(path_with_tilde);
        assert!(!expanded.starts_with("~/"), "Tilde should be expanded to full path");
        assert!(expanded.ends_with(".local/bin/nvim"));

        let absolute_path = "/opt/nvim/bin/nvim";
        assert_eq!(expand_home(absolute_path), absolute_path);
    }
}
