use crate::config::{Config, CustomInstaller};
use crate::platform::{check_apt_lock, command_exists, Platform};
use crate::state::State;
use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;

const BOLD_GREEN: &str = "\x1b[1;32m";
const BOLD_BLUE: &str = "\x1b[1;34m";
const WHITE: &str = "\x1b[37m";
const RESET: &str = "\x1b[0m";
const DIM_GRAY: &str = "\x1b[90m";

/// Simplified list of all categories and contained packages on a single line per category.
/// Format: <bold-green-category-name> (aliases) / pkg1, pkg2, pkg3 [apply]
pub fn list_categories(config: &Config, state: &State, platform: &Platform) {
    if config.categories.is_empty() {
        println!("No categories found in configuration.");
        return;
    }

    for (cat_name, cat) in &config.categories {
        let alias_part = if let Some(aliases) = &cat.aliases {
            if !aliases.is_empty() {
                format!(" ({})", aliases.join(", "))
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let mut pkgs: Vec<String> = match platform {
            Platform::Debian => cat.debian_packages.clone().unwrap_or_default(),
            Platform::Termux => cat.termux_packages.clone().unwrap_or_default(),
            Platform::Unsupported(_) => Vec::new(),
        };

        if matches!(platform, Platform::Debian)
            && let Some(custom) = cat.custom.as_ref().and_then(|m| m.get("debian"))
        {
            pkgs.push(format!("{} (custom binary)", custom.name));
        }

        let pkgs_str = if pkgs.is_empty() {
            "none".to_string()
        } else {
            pkgs.join(", ")
        };

        let apply_suffix = if is_category_applied(cat_name, cat, state, platform) {
            format!(" {WHITE}[apply]{RESET}")
        } else {
            String::new()
        };

        println!("{BOLD_GREEN}{}{RESET}{} / {DIM_GRAY}{}{RESET}{}", cat_name, alias_part, pkgs_str, apply_suffix);
    }
}

fn is_category_applied(cat_name: &str, cat: &crate::config::Category, state: &State, platform: &Platform) -> bool {
    if state.packages.values().any(|p| p.category == cat_name) {
        return true;
    }

    let pkgs = match platform {
        Platform::Debian => cat.debian_packages.as_deref().unwrap_or(&[]),
        Platform::Termux => cat.termux_packages.as_deref().unwrap_or(&[]),
        Platform::Unsupported(_) => &[],
    };

    if !pkgs.is_empty() && pkgs.iter().all(|pkg| platform.is_package_installed(pkg)) {
        return true;
    }

    if matches!(platform, Platform::Debian)
        && let Some(custom) = cat.custom.as_ref().and_then(|m| m.get("debian"))
    {
        let bin_path = expand_home(&custom.bin_symlink);
        if Path::new(&bin_path).exists() {
            return true;
        }
    }

    false
}

/// Shows complete detailed information, description, packages, dotfiles, and installation status for a specific category.
pub fn show_category(
    category_query: &str,
    config: &Config,
    state: &State,
    platform: &Platform,
) -> Result<(), String> {
    let canonical_key = config
        .resolve_category_key(category_query)
        .ok_or_else(|| format!("Category or alias '{}' not found in configuration.", category_query))?;

    let cat = config.categories.get(canonical_key).unwrap();

    let aliases_str = cat
        .aliases
        .as_ref()
        .map(|a| a.join(", "))
        .unwrap_or_else(|| "none".to_string());

    println!("\n{BOLD_GREEN}Category:{RESET} {}", canonical_key);
    println!("{BOLD_BLUE}Description:{RESET} {}", cat.description);
    println!("{BOLD_BLUE}Aliases:{RESET} {}", aliases_str);

    let pkgs = match platform {
        Platform::Debian => cat.debian_packages.as_deref().unwrap_or(&[]),
        Platform::Termux => cat.termux_packages.as_deref().unwrap_or(&[]),
        Platform::Unsupported(_) => &[],
    };

    println!("\n{BOLD_BLUE}Packages ({:?}):{RESET}", platform);
    if pkgs.is_empty() {
        println!("  (No OS packages declared for this platform)");
    } else {
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
            println!("  - {DIM_GRAY}{}{RESET} [{}]", pkg, status_str);
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
        println!("\n{BOLD_BLUE}Custom Binary Installer (Debian):{RESET}");
        println!("  - {DIM_GRAY}{}{RESET} ({}) [{}]", custom.name, custom.url, custom_status);
        println!("    Symlink: {} -> {}/bin/{}", bin_path, custom.extract_dir, custom.name);
    }

    if let Some(copy_files) = &cat.copy_files {
        println!("\n{BOLD_BLUE}Config File Actions:{RESET}");
        for action in copy_files {
            let platform_info = action
                .platform
                .as_ref()
                .map(|p| format!(" (Platform: {})", p))
                .unwrap_or_default();
            let only_if_info = if action.only_if_not_exists.unwrap_or(false) {
                " [skip if exists]"
            } else {
                ""
            };
            println!("  - {} -> {}{}{}", action.src, action.dest, platform_info, only_if_info);
        }
    }

    if let Some(post_cmds) = &cat.post_install_commands {
        println!("\n{BOLD_BLUE}Post-Install Commands:{RESET}");
        for cmd in post_cmds {
            let platform_info = cmd
                .platform
                .as_ref()
                .map(|p| format!(" (Platform: {})", p))
                .unwrap_or_default();
            println!("  - {}{}", cmd.command, platform_info);
        }
    }

    if let Some(msg) = &cat.final_message {
        println!("\n{BOLD_BLUE}Final Message:{RESET} {}", msg);
    }

    println!();
    Ok(())
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

        if let Some(copy_files) = &cat.copy_files {
            process_copy_files(copy_files, platform, dry_run)?;
        }

        if let Some(post_cmds) = &cat.post_install_commands {
            process_post_install_commands(post_cmds, platform, dry_run)?;
        }

        if let Some(msg) = &cat.final_message {
            if dry_run {
                println!("  [Dry-Run] Would show final message: {}", msg);
            } else {
                println!("\n  {BOLD_BLUE}>>> {}{RESET}", msg);
            }
        }
    }

    Ok(())
}

fn process_copy_files(
    copy_files: &[crate::config::CopyFileAction],
    platform: &Platform,
    dry_run: bool,
) -> Result<(), String> {
    for action in copy_files {
        if let Some(target_platform) = &action.platform {
            let matches_platform = matches!((target_platform.to_lowercase().as_str(), platform), ("debian", Platform::Debian) | ("termux", Platform::Termux));
            if !matches_platform {
                continue;
            }
        }

        let dest_path_str = expand_home(&action.dest);
        let dest_path = Path::new(&dest_path_str);

        if action.only_if_not_exists.unwrap_or(false) && dest_path.exists() {
            println!("  [SKIP] Destination file '{}' already exists.", dest_path_str);
            continue;
        }

        if dry_run {
            println!("  [Dry-Run] Would copy configuration file '{}' -> '{}'", action.src, dest_path_str);
            continue;
        }

        println!("  [Copying] Configuration file '{}' -> '{}'", action.src, dest_path_str);

        let content_bytes: Vec<u8> = if Path::new(&action.src).exists() {
            fs::read(&action.src)
                .map_err(|e| format!("Failed to read source file {}: {}", action.src, e))?
        } else if action.src == "config/shell-tokyonight/starship.toml" {
            crate::config::EMBEDDED_STARSHIP.as_bytes().to_vec()
        } else if action.src == "config/shell-tokyonight/font.ttf" {
            crate::config::EMBEDDED_FONT.to_vec()
        } else {
            return Err(format!("Source file '{}' not found.", action.src));
        };

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
        }

        fs::write(dest_path, content_bytes)
            .map_err(|e| format!("Failed to write configuration to {}: {}", dest_path_str, e))?;

        println!("  [Success] Configuration file placed cleanly at '{}'", dest_path_str);
    }
    Ok(())
}

fn process_post_install_commands(
    commands: &[crate::config::PostInstallCommand],
    platform: &Platform,
    dry_run: bool,
) -> Result<(), String> {
    for cmd in commands {
        if let Some(target_platform) = &cmd.platform {
            let matches_platform = matches!((target_platform.to_lowercase().as_str(), platform), ("debian", Platform::Debian) | ("termux", Platform::Termux));
            if !matches_platform {
                continue;
            }
        }

        if cmd.command.contains("chsh") {
            let current_shell = env::var("SHELL").unwrap_or_default();
            if current_shell.ends_with("/zsh") {
                println!("  [SKIP] Default shell is already Zsh ('{}').", current_shell);
                continue;
            }
        }

        if dry_run {
            println!("  [Dry-Run] Would execute post-install command: {}", cmd.command);
            continue;
        }

        println!("  [Executing] Post-install command: {}", cmd.command);
        let status = Command::new("sh")
            .arg("-c")
            .arg(&cmd.command)
            .status()
            .map_err(|e| format!("Failed to execute post-install command '{}': {}", cmd.command, e))?;

        if !status.success() {
            println!("  [WARNING] Post-install command '{}' exited with status {}", cmd.command, status);
        } else {
            println!("  [Success] Post-install command completed: {}", cmd.command);
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

    #[test]
    fn test_process_copy_files_dry_run() {
        let copy_actions = vec![crate::config::CopyFileAction {
            src: "config/shell-tokyonight/starship.toml".to_string(),
            dest: "~/.config/starship.toml".to_string(),
            platform: None,
            only_if_not_exists: None,
        }];
        let res = process_copy_files(&copy_actions, &Platform::Debian, true);
        assert!(res.is_ok(), "Copy files dry-run should succeed");
    }

    #[test]
    fn test_process_copy_files_termux_font_dry_run() {
        let copy_actions = vec![crate::config::CopyFileAction {
            src: "config/shell-tokyonight/font.ttf".to_string(),
            dest: "~/.termux/font.ttf".to_string(),
            platform: Some("termux".to_string()),
            only_if_not_exists: Some(true),
        }];
        let res_debian = process_copy_files(&copy_actions, &Platform::Debian, true);
        assert!(res_debian.is_ok());

        let res_termux = process_copy_files(&copy_actions, &Platform::Termux, true);
        assert!(res_termux.is_ok());
    }

    #[test]
    fn test_process_post_install_commands_dry_run() {
        let commands = vec![crate::config::PostInstallCommand {
            command: "chsh -s $(which zsh)".to_string(),
            platform: Some("debian".to_string()),
        }];
        let res = process_post_install_commands(&commands, &Platform::Debian, true);
        assert!(res.is_ok(), "Post install commands dry-run should succeed");
    }

    #[test]
    fn test_process_post_install_commands_platform_filter() {
        let commands = vec![
            crate::config::PostInstallCommand {
                command: "chsh -s $(which zsh)".to_string(),
                platform: Some("debian".to_string()),
            },
            crate::config::PostInstallCommand {
                command: "chsh -s zsh".to_string(),
                platform: Some("termux".to_string()),
            },
            crate::config::PostInstallCommand {
                command: "echo common".to_string(),
                platform: None,
            },
        ];
        let res_debian = process_post_install_commands(&commands, &Platform::Debian, true);
        assert!(res_debian.is_ok());
        let res_termux = process_post_install_commands(&commands, &Platform::Termux, true);
        assert!(res_termux.is_ok());
    }

    #[test]
    fn test_list_categories_output() {
        let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
        let state = State::load();
        list_categories(&config, &state, &Platform::Debian);
    }

    #[test]
    fn test_show_category_resolution() {
        let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
        let state = State::load();
        let platform = Platform::Debian;

        assert!(show_category("shell-tokyonight", &config, &state, &platform).is_ok());
        assert!(show_category("shell-tn", &config, &state, &platform).is_ok());
        assert!(show_category("nonexistent", &config, &state, &platform).is_err());
    }
}
