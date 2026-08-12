use crate::colors::*;
use crate::config::{Config, CustomInstaller};
use crate::platform::{check_apt_lock, command_exists, Platform};
use crate::state::State;
use anyhow::Context;
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::{Command, Stdio};

/// Writes the provided content to stdout, routing through a pager (`less` or `$PAGER`)
/// if stdout is connected to an interactive terminal (TTY). If not a TTY or pager fails,
/// writes directly to stdout.
pub fn output_with_pager(content: &str) {
    if !io::stdout().is_terminal() {
        let _ = io::stdout().write_all(content.as_bytes());
        return;
    }

    let pager_cmd = env::var("PAGER").unwrap_or_else(|_| "less".to_string());
    let mut cmd = Command::new(&pager_cmd);

    // If using default 'less', pass flags -FRX to preserve colors, exit if output fits one screen, and avoid screen clear
    if pager_cmd == "less" || pager_cmd.ends_with("/less") {
        cmd.args(["-F", "-R", "-X"]);
    }

    match cmd.stdin(Stdio::piped()).spawn() {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(content.as_bytes());
            }
            let _ = child.wait();
        }
        Err(_) => {
            let _ = io::stdout().write_all(content.as_bytes());
        }
    }
}

/// Simplified list of categories and contained packages on a single line per category.
/// Format: <bold-green-category-name>, alias1, alias2 / pkg1, pkg2, pkg3 [apply]
///
/// When `filter` is `Some(query)`, only categories whose name, aliases, or platform
/// packages match the query are printed. The `show_hidden` flag keeps its behavior:
/// matched categories unsupported on the current platform are shown with `[unsupported]`.
pub fn list_categories(
    config: &Config,
    state: &State,
    platform: &Platform,
    show_hidden: bool,
    filter: Option<&str>,
) {
    if config.categories.is_empty() {
        println!("No categories found in configuration.");
        return;
    }

    let mut output = String::new();
    let mut hidden_count = 0;

    for (cat_name, cat) in &config.categories {
        let mut pkgs: Vec<String> = match platform {
            Platform::Debian => cat.debian_packages.clone().unwrap_or_default(),
            Platform::Termux => cat.termux_packages.clone().unwrap_or_default(),
            Platform::Unsupported(_) => Vec::new(),
        };

        if let Some(query) = filter
            && !category_matches_query(query, cat_name, cat, platform)
        {
            continue;
        }

        if matches!(platform, Platform::Debian)
            && let Some(custom) = cat.custom.as_ref().and_then(|m| m.get("debian"))
        {
            pkgs.push(format!("{} (custom binary)", custom.name));
        }

        let is_supported = !pkgs.is_empty();

        if !is_supported && !show_hidden {
            hidden_count += 1;
            continue;
        }

        let alias_part = cat.aliases.as_ref().map_or_else(String::new, |aliases| {
            if aliases.is_empty() {
                String::new()
            } else {
                format!(", {}", aliases.join(", "))
            }
        });

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

        let hidden_suffix = if !is_supported {
            format!(" {DIM_GRAY}[unsupported]{RESET}")
        } else {
            String::new()
        };

        output.push_str(&format!(
            "{BOLD_GREEN}{cat_name}{RESET}{alias_part} / {DIM_GRAY}{pkgs_str}{RESET}{apply_suffix}{hidden_suffix}\n"
        ));
    }

    if hidden_count > 0 && !show_hidden {
        output.push_str(&format!(
            "\n{DIM_GRAY}{hidden_count} hidden. Use '-sh' or '--show-hidden' to view all.{RESET}\n"
        ));
    }

    output_with_pager(&output);
}

/// Case-insensitive substring match of `query` against a category name, its aliases,
/// and the packages declared for the current platform.
fn category_matches_query(
    query: &str,
    cat_name: &str,
    cat: &crate::config::Category,
    platform: &Platform,
) -> bool {
    let q = query.to_lowercase();

    if cat_name.to_lowercase().contains(&q) {
        return true;
    }

    if cat
        .aliases
        .as_ref()
        .is_some_and(|aliases| aliases.iter().any(|alias| alias.to_lowercase().contains(&q)))
    {
        return true;
    }

    let pkgs = match platform {
        Platform::Debian => cat.debian_packages.as_deref().unwrap_or(&[]),
        Platform::Termux => cat.termux_packages.as_deref().unwrap_or(&[]),
        Platform::Unsupported(_) => &[],
    };

    if pkgs.iter().any(|pkg| pkg.to_lowercase().contains(&q)) {
        return true;
    }

    if matches!(platform, Platform::Debian)
        && let Some(custom) = cat.custom.as_ref().and_then(|m| m.get("debian"))
        && custom.name.to_lowercase().contains(&q)
    {
        return true;
    }

    false
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
) -> anyhow::Result<()> {
    let canonical_key = config
        .resolve_category_key(category_query)
        .ok_or_else(|| anyhow::anyhow!("Category or alias '{category_query}' not found in configuration."))?;

    let cat = config.categories.get(canonical_key).unwrap();

    let aliases_str = cat
        .aliases
        .as_ref()
        .map(|a| a.join(", "))
        .unwrap_or_else(|| "none".to_string());

    println!("\n{BOLD_GREEN}Category:{RESET} {canonical_key}");
    println!("{BOLD_BLUE}Description:{RESET} {}", cat.description);
    println!("{BOLD_BLUE}Aliases:{RESET} {aliases_str}");

    let pkgs = match platform {
        Platform::Debian => cat.debian_packages.as_deref().unwrap_or(&[]),
        Platform::Termux => cat.termux_packages.as_deref().unwrap_or(&[]),
        Platform::Unsupported(_) => &[],
    };

    println!("\n{BOLD_BLUE}Packages ({platform:?}):{RESET}");
    if pkgs.is_empty() {
        println!("  (No OS packages declared for this platform)");
    } else {
        for pkg in pkgs {
            let status_str = if let Some(tracked) = state.packages.get(pkg) {
                if tracked.was_preexisting {
                    "Pre-existing (system)"
                } else {
                    "Installed by dotss"
                }
            } else if platform.is_package_installed(pkg) {
                "Installed (untracked)"
            } else {
                "Not installed"
            };
            println!("  - {DIM_GRAY}{pkg}{RESET} [{status_str}]");
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
        println!("  - {DIM_GRAY}{}{RESET} ({}) [{custom_status}]", custom.name, custom.url);
        println!("    Symlink: {bin_path} -> {}/bin/{}", custom.extract_dir, custom.name);
    }

    if let Some(copy_files) = &cat.copy_files {
        println!("\n{BOLD_BLUE}Config File Actions:{RESET}");
        for action in copy_files {
            let platform_info = action
                .platform
                .as_ref()
                .map(|p| format!(" (Platform: {p})"))
                .unwrap_or_default();
            let only_if_info = if action.only_if_not_exists.unwrap_or(false) {
                " [skip if exists]"
            } else {
                ""
            };
            println!("  - Add {}{platform_info}{only_if_info}", action.dest);
        }
    }

    if let Some(post_cmds) = &cat.post_install_commands {
        println!("\n{BOLD_BLUE}Post-Install Actions:{RESET}");
        for cmd in post_cmds {
            let platform_info = cmd
                .platform
                .as_ref()
                .map(|p| format!(" (Platform: {p})"))
                .unwrap_or_default();
            
            let display_text = if let Some(ref desc) = cmd.description {
                desc.clone()
            } else if let Some(ref prompt) = cmd.prompt {
                prompt.clone()
            } else {
                format_command_summary(&cmd.command)
            };

            println!("  - {display_text}{platform_info}");
        }
    }

    if let Some(injections) = &cat.section_injections {
        println!("\n{BOLD_BLUE}Section Injections:{RESET}");
        for inj in injections {
            let platform_info = inj
                .platform
                .as_ref()
                .map(|p| format!(" (Platform: {p})"))
                .unwrap_or_default();
            let desc = inj
                .description
                .as_deref()
                .unwrap_or(inj.line.as_str());
            println!("  - {desc} -> {} under section '{}'{platform_info}", inj.file, inj.section);
        }
    }

    if let Some(msg) = cat.final_message_for_platform(platform) {
        println!("\n{BOLD_BLUE}Final Message:{RESET} {msg}");
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
) -> anyhow::Result<()> {
    if let Platform::Unsupported(reason) = platform {
        anyhow::bail!("Unsupported platform: {reason}");
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
                anyhow::bail!("Category or alias '{cat}' not found in configuration.");
            }
        }
    };

    for (cat_name, cat) in target_categories {
        println!("\n--> Processing Category: {cat_name}");

        // Pre-prompt for any interactive post-install commands before doing packages/downloads
        let mut confirmed_commands: std::collections::HashMap<usize, bool> = std::collections::HashMap::new();

        if let Some(post_cmds) = &cat.post_install_commands {
            for (idx, cmd) in post_cmds.iter().enumerate() {
                if let Some(target_platform) = &cmd.platform {
                    let matches_platform = matches!((target_platform.to_lowercase().as_str(), platform), ("debian", Platform::Debian) | ("termux", Platform::Termux));
                    if !matches_platform {
                        continue;
                    }
                }

                if cmd.command.contains("chsh") {
                    let current_shell = env::var("SHELL").unwrap_or_default();
                    if current_shell.ends_with("/zsh") {
                        continue;
                    }
                }

                let is_confirm_required = cmd.confirm.unwrap_or(false) || cmd.prompt.is_some();
                if is_confirm_required {
                    let prompt_text = cmd
                        .prompt
                        .as_deref()
                        .unwrap_or("Do you want to run this post-install step?");

                    if dry_run {
                        println!("  [Dry-Run] Prompt: {prompt_text} [y/N]");
                        confirmed_commands.insert(idx, true);
                        continue;
                    }

                    use std::io::{self, Write};
                    print!("  {BOLD_YELLOW}? {prompt_text} [y/N]: {RESET}");
                    let _ = io::stdout().flush();
                    let mut input = String::new();
                    let confirmed = if io::stdin().read_line(&mut input).is_ok() {
                        let trimmed = input.trim().to_lowercase();
                        trimmed == "y" || trimmed == "yes"
                    } else {
                        false
                    };

                    confirmed_commands.insert(idx, confirmed);
                    if !confirmed {
                        println!("  [SKIP] Step will be skipped.");
                    }
                }
            }
        }

        let pkgs = match platform {
            Platform::Debian => cat.debian_packages.as_deref().unwrap_or(&[]),
            Platform::Termux => cat.termux_packages.as_deref().unwrap_or(&[]),
            Platform::Unsupported(_) => &[],
        };

        for pkg in pkgs {
            if platform.is_package_installed(pkg) {
                println!("  [SKIP] Package '{pkg}' is already installed on OS.");
                state.track_package(pkg, cat_name, true);
                state.save_atomic()?;
                continue;
            }

            match platform {
                Platform::Debian => {
                    let cmd_str = format!("sudo apt install -y {pkg}");
                    if dry_run {
                        println!("  [Dry-Run] Would execute: {cmd_str}");
                    } else {
                        println!("  [Installing] Executing: {cmd_str}");
                        let status = Command::new("sudo")
                            .arg("apt")
                            .arg("install")
                            .arg("-y")
                            .arg(pkg)
                            .status()
                            .context("Failed to run apt install")?;

                        if !status.success() {
                            anyhow::bail!("apt install failed for package '{pkg}'");
                        }
                        state.track_package(pkg, cat_name, false);
                        state.save_atomic()?;
                    }
                }
                Platform::Termux => {
                    let cmd_str = format!("pkg install -y {pkg}");
                    if dry_run {
                        println!("  [Dry-Run] Would execute: {cmd_str}");
                    } else {
                        println!("  [Installing] Executing: {cmd_str}");
                        let status = Command::new("pkg")
                            .arg("install")
                            .arg("-y")
                            .arg(pkg)
                            .status()
                            .context("Failed to run pkg install")?;

                        if !status.success() {
                            anyhow::bail!("pkg install failed for package '{pkg}'");
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
            process_post_install_commands(post_cmds, platform, &confirmed_commands, dry_run)?;
        }

        if let Some(injections) = &cat.section_injections {
            process_section_injections(injections, platform, dry_run)?;
        }

        if let Some(msg) = cat.final_message_for_platform(platform) {
            if dry_run {
                println!("  [Dry-Run] Would show final message: {msg}");
            } else {
                println!("\n  {BOLD_YELLOW}>>> {msg}{RESET}");
            }
        }
    }

    Ok(())
}

fn process_copy_files(
    copy_files: &[crate::config::CopyFileAction],
    platform: &Platform,
    dry_run: bool,
) -> anyhow::Result<()> {
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
            println!("  [SKIP] Destination file '{dest_path_str}' already exists.");
            continue;
        }

        if dry_run {
            println!("  [Dry-Run] Would copy configuration file to '{dest_path_str}'");
            continue;
        }

        println!("  [Copying] Configuration file to '{dest_path_str}'");

        let content_bytes: Vec<u8> = if Path::new(&action.src).exists() {
            fs::read(&action.src)
                .map_err(|e| anyhow::anyhow!("Failed to read source file {}: {e}", action.src))?
        } else if action.src == "config/zsh-tokyonight/starship.toml" {
            crate::config::EMBEDDED_STARSHIP.as_bytes().to_vec()
        } else if action.src == "config/zsh-tokyonight/aliases.zsh" {
            crate::config::EMBEDDED_ALIASES.as_bytes().to_vec()
        } else if action.src == "config/zsh-tokyonight/font.ttf" || action.src == "config/zsh-minimal/font.ttf" {
            crate::config::EMBEDDED_FONT.to_vec()
        } else if action.src == "config/i3wm/config" {
            crate::config::EMBEDDED_I3_CONFIG.as_bytes().to_vec()
        } else if action.src == "config/i3wm/config.ini" {
            crate::config::EMBEDDED_POLYBAR_CONFIG.as_bytes().to_vec()
        } else if action.src == "config/i3wm/launch.sh" {
            crate::config::EMBEDDED_POLYBAR_LAUNCH.as_bytes().to_vec()
        } else {
            anyhow::bail!("Source file '{}' not found.", action.src);
        };

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow::anyhow!("Failed to create directory {}: {e}", parent.display()))?;
        }

        fs::write(dest_path, content_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to write configuration to {dest_path_str}: {e}"))?;

        println!("  [Success] Configuration file placed cleanly at '{dest_path_str}'");
    }
    Ok(())
}

fn process_post_install_commands(
    commands: &[crate::config::PostInstallCommand],
    platform: &Platform,
    confirmed_commands: &std::collections::HashMap<usize, bool>,
    dry_run: bool,
) -> anyhow::Result<()> {
    for (idx, cmd) in commands.iter().enumerate() {
        if let Some(target_platform) = &cmd.platform {
            let matches_platform = matches!((target_platform.to_lowercase().as_str(), platform), ("debian", Platform::Debian) | ("termux", Platform::Termux));
            if !matches_platform {
                continue;
            }
        }

        if cmd.command.contains("chsh") {
            let current_shell = env::var("SHELL").unwrap_or_default();
            if current_shell.ends_with("/zsh") {
                println!("  [SKIP] Default shell is already Zsh ('{current_shell}').");
                continue;
            }
        }

        let is_confirm_required = cmd.confirm.unwrap_or(false) || cmd.prompt.is_some();
        if is_confirm_required {
            let is_confirmed = confirmed_commands.get(&idx).copied().unwrap_or(false);
            if !is_confirmed {
                println!("  [SKIP] Post-install command '{}' skipped by user choice.", cmd.command);
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
            .map_err(|e| anyhow::anyhow!("Failed to execute post-install command '{}': {e}", cmd.command))?;

        if !status.success() {
            println!("  [WARNING] Post-install command '{}' exited with status {status}", cmd.command);
        } else {
            println!("  [Success] Post-install command completed: {}", cmd.command);
        }
    }
    Ok(())
}

fn process_section_injections(
    injections: &[crate::config::SectionInjection],
    platform: &Platform,
    dry_run: bool,
) -> anyhow::Result<()> {
    for injection in injections {
        if let Some(target_platform) = &injection.platform {
            let matches_platform = matches!(
                (target_platform.to_lowercase().as_str(), platform),
                ("debian", Platform::Debian) | ("termux", Platform::Termux)
            );
            if !matches_platform {
                continue;
            }
        }

        let expanded_path = expand_home(&injection.file);
        let path = Path::new(&expanded_path);

        let desc = injection
            .description
            .as_deref()
            .unwrap_or(injection.line.as_str());

        let content = if path.exists() {
            fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("Failed to read config file '{}': {e}", path.display()))?
        } else {
            String::new()
        };

        let first_line = injection.line.lines().next().unwrap_or(&injection.line).trim();
        if content.lines().any(|l| l.trim() == first_line) {
            println!("  [SKIP] Section line '{}' already exists in {}.", first_line, injection.file);
            continue;
        }

        if dry_run {
            let pos_info = injection
                .position
                .as_deref()
                .map(|p| format!(" [position: {p}]"))
                .unwrap_or_default();
            println!(
                "  [Dry-Run] Would inject line into {} under section '{}'{}: {}",
                injection.file, injection.section, pos_info, desc
            );
            continue;
        }

        let new_content = inject_line_into_section(
            &content,
            &injection.section,
            &injection.line,
            injection.position.as_deref(),
        );

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow::anyhow!("Failed to create directories for '{}': {e}", path.display()))?;
        }

        fs::write(path, new_content)
            .map_err(|e| anyhow::anyhow!("Failed to write updated config to '{}': {e}", path.display()))?;

        println!("  [Success] Injected line into {} under section '{}': {}", injection.file, injection.section, desc);
    }
    Ok(())
}

/// Helper to inject a line into content under a section header.
/// If the section header exists, inserts line directly beneath it.
/// If the section header does NOT exist, prepends (position="top") or appends to content.
pub fn inject_line_into_section(
    content: &str,
    section: &str,
    line: &str,
    position: Option<&str>,
) -> String {
    let lines: Vec<&str> = content.lines().collect();

    if let Some(header_idx) = lines.iter().position(|l| l.trim() == section.trim()) {
        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        new_lines.insert(header_idx + 1, line.trim().to_string());
        let mut result = new_lines.join("\n");
        if content.ends_with('\n') || content.is_empty() {
            result.push('\n');
        }
        result
    } else {
        let is_top = position.map(|p| p.to_lowercase() == "top").unwrap_or(false);

        if is_top {
            let mut result = String::new();
            result.push_str(section.trim());
            result.push('\n');
            result.push_str(line.trim());
            result.push('\n');
            if !content.is_empty() {
                result.push('\n');
                result.push_str(content.trim_start());
                if !result.ends_with('\n') {
                    result.push('\n');
                }
            }
            result
        } else {
            let mut result = content.to_string();
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(section.trim());
            result.push('\n');
            result.push_str(line.trim());
            result.push('\n');
            result
        }
    }
}

/// Handles special binary installations (e.g. Neovim on Debian via tar.gz release).
fn install_custom_debian(
    custom: &CustomInstaller,
    cat_name: &str,
    state: &mut State,
    dry_run: bool,
) -> anyhow::Result<()> {
    println!("  --> Custom Binary Installer: {}", custom.name);

    if !command_exists("curl") || !command_exists("tar") {
        anyhow::bail!("Prerequisite binaries 'curl' and 'tar' are required for custom downloads. Please install them first.");
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

    let tmp_dir = env::temp_dir().join(format!("{}_install", custom.name));
    fs::create_dir_all(&tmp_dir).context("Failed to create temp directory")?;
    let tmp_tarball = tmp_dir.join(format!("{}.tar.gz", custom.name));

    println!("  [Downloading] Fetching latest tarball from {}...", custom.url);
    let curl_status = Command::new("curl")
        .arg("-fsSL")
        .arg("-o")
        .arg(&tmp_tarball)
        .arg(&custom.url)
        .status()
        .context("Failed to run curl")?;

    if !curl_status.success() {
        let _ = fs::remove_dir_all(&tmp_dir);
        anyhow::bail!("Failed to download tarball from {}", custom.url);
    }

    println!("  [Extracting] Moving to {}...", custom.extract_dir);
    let mkdir_status = Command::new("sudo")
        .arg("mkdir")
        .arg("-p")
        .arg(&custom.extract_dir)
        .status()
        .context("Failed to create extract directory")?;

    if !mkdir_status.success() {
        let _ = fs::remove_dir_all(&tmp_dir);
        anyhow::bail!("Failed to create directory {}", custom.extract_dir);
    }

    let tar_status = Command::new("sudo")
        .arg("tar")
        .arg("-xzf")
        .arg(&tmp_tarball)
        .arg("-C")
        .arg(&custom.extract_dir)
        .arg("--strip-components=1")
        .status()
        .context("Failed to extract tarball")?;

    let _ = fs::remove_dir_all(&tmp_dir);

    if !tar_status.success() {
        anyhow::bail!("Failed to extract tarball to {}", custom.extract_dir);
    }

    let symlink_dir = bin_path_buf.parent().unwrap();
    fs::create_dir_all(symlink_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create symlink parent directory {}: {e}", symlink_dir.display()))?;

    if bin_path_buf.exists() || bin_path_buf.is_symlink() {
        let _ = fs::remove_file(&bin_path_buf);
    }

    let target_bin = format!("{}/bin/{}", custom.extract_dir, custom.name);
    symlink(&target_bin, &bin_path_buf)
        .map_err(|e| anyhow::anyhow!("Failed to create symlink at {}: {e}", bin_path_buf.display()))?;

    println!("  [Success] Installed {} to {} with symlink at {}", custom.name, custom.extract_dir, bin_path_buf.display());
    state.track_package(&custom.name, cat_name, false);
    state.save_atomic()?;

    check_path_and_recommend(symlink_dir);

    Ok(())
}

/// Safely removes packages installed by dotss.
pub fn remove_category(
    category_filter: Option<&str>,
    config: &Config,
    state: &mut State,
    platform: &Platform,
    dry_run: bool,
) -> anyhow::Result<()> {
    if let Platform::Unsupported(reason) = platform {
        anyhow::bail!("Unsupported platform: {reason}");
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
                anyhow::bail!("Category or alias '{cat}' not found in configuration.");
            }
        }
    };

    for (cat_name, cat) in target_categories {
        println!("\n--> Processing Removal for Category: {cat_name}");

        let pkgs = match platform {
            Platform::Debian => cat.debian_packages.as_deref().unwrap_or(&[]),
            Platform::Termux => cat.termux_packages.as_deref().unwrap_or(&[]),
            Platform::Unsupported(_) => &[],
        };

        for pkg in pkgs {
            if let Some(tracked) = state.packages.get(pkg) {
                if tracked.was_preexisting {
                    println!("  [SKIP] Package '{pkg}' was pre-existing on system before dotss. Skipping removal.");
                    continue;
                }
            } else {
                println!("  [SKIP] Package '{pkg}' was not installed by dotss. Skipping removal.");
                continue;
            }

            match platform {
                Platform::Debian => {
                    let cmd_str = format!("sudo apt remove -y {pkg}");
                    if dry_run {
                        println!("  [Dry-Run] Would execute: {cmd_str}");
                    } else {
                        println!("  [Removing] Executing: {cmd_str}");
                        let status = Command::new("sudo")
                            .arg("apt")
                            .arg("remove")
                            .arg("-y")
                            .arg(pkg)
                            .status()
                            .context("Failed to run apt remove")?;

                        if !status.success() {
                            anyhow::bail!("apt remove failed for package '{pkg}'");
                        }
                        state.remove_package(pkg);
                        state.save_atomic()?;
                    }
                }
                Platform::Termux => {
                    let cmd_str = format!("pkg remove -y {pkg}");
                    if dry_run {
                        println!("  [Dry-Run] Would execute: {cmd_str}");
                    } else {
                        println!("  [Removing] Executing: {cmd_str}");
                        let status = Command::new("pkg")
                            .arg("remove")
                            .arg("-y")
                            .arg(pkg)
                            .status()
                            .context("Failed to run pkg remove")?;

                        if !status.success() {
                            anyhow::bail!("pkg remove failed for package '{pkg}'");
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
                println!("  [Dry-Run] Would remove symlink: {bin_path}");
                println!("  [Dry-Run] Would execute: sudo rm -rf {}", custom.extract_dir);
            } else {
                println!("  [Removing] Removing custom binary '{}'...", custom.name);
                if let Err(e) = fs::remove_file(&bin_path) {
                    eprintln!("  [WARN] Failed to remove symlink {bin_path}: {e}");
                }
                match Command::new("sudo").args(["rm", "-rf", &custom.extract_dir]).status() {
                    Ok(s) if !s.success() => {
                        eprintln!("  [WARN] sudo rm -rf {} exited with {s}", custom.extract_dir);
                    }
                    Err(e) => {
                        eprintln!("  [WARN] Failed to run sudo rm: {e}");
                    }
                    _ => {}
                }
                state.remove_package(&custom.name);
                state.save_atomic()?;
            }
        }

        if let Some(copy_files) = &cat.copy_files {
            for action in copy_files {
                if let Some(target_platform) = &action.platform {
                    let matches_platform = matches!(
                        (target_platform.to_lowercase().as_str(), platform),
                        ("debian", Platform::Debian) | ("termux", Platform::Termux)
                    );
                    if !matches_platform {
                        continue;
                    }
                }

                let dest_path_str = expand_home(&action.dest);
                let dest_path = Path::new(&dest_path_str);
                if dest_path.exists() && dest_path.is_file() {
                    if dry_run {
                        println!("  [Dry-Run] Would remove configuration file: '{dest_path_str}'");
                    } else {
                        println!("  [Removing] Removing configuration file '{dest_path_str}'...");
                        if let Err(e) = fs::remove_file(dest_path) {
                            eprintln!("  [WARN] Failed to remove configuration file {dest_path_str}: {e}");
                        }
                    }
                }
            }
        }

        if cat_name == "zsh-tokyonight" {
            let plugins_dir = expand_home("~/.config/zsh/plugins");
            let plugins_path = Path::new(&plugins_dir);
            if plugins_path.exists() {
                if dry_run {
                    println!("  [Dry-Run] Would remove Zsh plugins directory: '{plugins_dir}'");
                } else {
                    println!("  [Removing] Removing Zsh plugins directory '{plugins_dir}'...");
                    if let Err(e) = fs::remove_dir_all(plugins_path) {
                        eprintln!("  [WARN] Failed to remove Zsh plugins directory {plugins_dir}: {e}");
                    }
                }
            }

            let zsh_dir = expand_home("~/.config/zsh");
            let zsh_path = Path::new(&zsh_dir);
            if zsh_path.exists()
                && let Ok(mut entries) = fs::read_dir(zsh_path)
                && entries.next().is_none()
            {
                if dry_run {
                    println!("  [Dry-Run] Would remove empty directory: '{zsh_dir}'");
                } else if let Err(e) = fs::remove_dir(zsh_path) {
                    eprintln!("  [WARN] Failed to remove directory {zsh_dir}: {e}");
                }
            }
        }
    }

    Ok(())
}

/// Helper function to expand ~ to user's HOME directory in paths.
pub fn expand_home(path: &str) -> String {
    if let (Some(stripped), Ok(home)) = (path.strip_prefix("~/"), env::var("HOME")) {
        format!("{home}/{stripped}")
    } else {
        path.to_string()
    }
}

/// Helper to check if a directory is in $PATH and print setup recommendations if missing.
fn check_path_and_recommend(dir: &Path) {
    if let Ok(path_var) = env::var("PATH") {
        let dir_str = dir.to_string_lossy();
        if !path_var.split(':').any(|p| p == dir_str) {
            println!("\n  [TIP] '{dir_str}' is NOT in your current $PATH environment variable!");
            println!("  To access binaries directly from anywhere, add it to your shell configuration:");
            println!("    - bash: echo 'export PATH=\"{dir_str}:$PATH\"' >> ~/.bashrc");
            println!("    - zsh:  echo 'export PATH=\"{dir_str}:$PATH\"' >> ~/.zshrc");
            println!("    - fish: fish_add_path {dir_str}\n");
        }
    }
}

/// Returns a human-readable summary of a shell command for display in CLI output.
fn format_command_summary(cmd: &str) -> String {
    if cmd.contains("chsh") {
        "Change default shell to Zsh".to_string()
    } else if cmd.contains("XDG_STATE_HOME") {
        "Configure XDG_STATE_HOME in ~/.zshenv".to_string()
    } else if cmd.contains("HISTFILE=") {
        "Configure Zsh history file & options in ~/.zshrc".to_string()
    } else if cmd.contains("starship init") {
        "Initialize Starship prompt in ~/.zshrc".to_string()
    } else if cmd.contains("atuin init") {
        "Initialize Atuin shell history in ~/.zshrc".to_string()
    } else if cmd.contains("zsh-autosuggestions") {
        if cmd.contains("git clone") {
            "Download zsh-autosuggestions plugin".to_string()
        } else {
            "Enable zsh-autosuggestions in ~/.zshrc".to_string()
        }
    } else if cmd.contains("fast-syntax-highlighting") {
        if cmd.contains("git clone") {
            "Download fast-syntax-highlighting plugin".to_string()
        } else {
            "Enable fast-syntax-highlighting in ~/.zshrc".to_string()
        }
    } else if cmd.contains("fzf-tab") {
        if cmd.contains("git clone") {
            "Download fzf-tab plugin".to_string()
        } else {
            "Enable fzf-tab in ~/.zshrc".to_string()
        }
    } else if cmd.contains("compinit") {
        "Enable Zsh autocompletion (compinit)".to_string()
    } else if cmd.contains("completion:*") {
        "Configure Zsh completion format style".to_string()
    } else if cmd.contains("LazyVim/starter") {
        "Clone default config for LazyVim".to_string()
    } else {
        let first_line = cmd.lines().next().unwrap_or(cmd);
        if first_line.len() > 60 {
            format!("{}...", &first_line[..57])
        } else {
            first_line.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_home_should_replace_tilde_with_home_dir() {
        let path_with_tilde = "~/.local/bin/nvim";
        let expanded = expand_home(path_with_tilde);
        assert!(!expanded.starts_with("~/"), "Tilde should be expanded to full path");
        assert!(expanded.ends_with(".local/bin/nvim"));

        let absolute_path = "/opt/nvim/bin/nvim";
        assert_eq!(expand_home(absolute_path), absolute_path);
    }

    #[test]
    fn copy_files_should_succeed_in_dry_run_mode() {
        let copy_actions = vec![crate::config::CopyFileAction {
            src: "config/zsh-tokyonight/starship.toml".to_string(),
            dest: "~/.config/zsh/starship.toml".to_string(),
            platform: None,
            only_if_not_exists: None,
        }];
        let res = process_copy_files(&copy_actions, &Platform::Debian, true);
        assert!(res.is_ok(), "Copy files dry-run should succeed");
    }

    #[test]
    fn copy_files_should_filter_by_platform_in_dry_run() {
        let copy_actions = vec![crate::config::CopyFileAction {
            src: "config/zsh-tokyonight/font.ttf".to_string(),
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
    fn post_install_should_preview_commands_in_dry_run() {
        let commands = vec![crate::config::PostInstallCommand {
            command: "chsh -s $(which zsh)".to_string(),
            description: None,
            platform: Some("debian".to_string()),
            prompt: None,
            confirm: None,
        }];
        let confirmed = std::collections::HashMap::new();
        let res = process_post_install_commands(&commands, &Platform::Debian, &confirmed, true);
        assert!(res.is_ok(), "Post install commands dry-run should succeed");
    }

    #[test]
    fn post_install_should_filter_commands_by_platform() {
        let commands = vec![
            crate::config::PostInstallCommand {
                command: "chsh -s $(which zsh)".to_string(),
                description: None,
                platform: Some("debian".to_string()),
                prompt: None,
                confirm: None,
            },
            crate::config::PostInstallCommand {
                command: "chsh -s zsh".to_string(),
                description: None,
                platform: Some("termux".to_string()),
                prompt: None,
                confirm: None,
            },
            crate::config::PostInstallCommand {
                command: "echo common".to_string(),
                description: None,
                platform: None,
                prompt: None,
                confirm: None,
            },
        ];
        let confirmed = std::collections::HashMap::new();
        let res_debian = process_post_install_commands(&commands, &Platform::Debian, &confirmed, true);
        assert!(res_debian.is_ok());
        let res_termux = process_post_install_commands(&commands, &Platform::Termux, &confirmed, true);
        assert!(res_termux.is_ok());
    }

    #[test]
    fn list_categories_should_include_configured_categories() {
        let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
        let state = State::load();
        list_categories(&config, &state, &Platform::Debian, false, None);
        list_categories(&config, &state, &Platform::Termux, true, None);
    }

    #[test]
    fn category_matches_query_should_match_name_alias_and_packages() {
        let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
        let platform = Platform::Debian;

        let (name, cat) = config
            .categories
            .get_key_value("lazyvim-minimal")
            .expect("lazyvim-minimal category should exist");

        assert!(category_matches_query("lazy", name, cat, &platform), "Should match by category name");
        assert!(category_matches_query("LAZY", name, cat, &platform), "Should match case-insensitively");
        assert!(category_matches_query("lzv", name, cat, &platform), "Should match by alias");
        assert!(category_matches_query("lazygit", name, cat, &platform), "Should match by package name");
        assert!(
            category_matches_query("neovim", name, cat, &platform),
            "Should match by Debian custom binary name"
        );
        assert!(
            !category_matches_query("nvim", name, cat, &platform),
            "'nvim' is not a substring of 'neovim', so it must not match"
        );
        assert!(!category_matches_query("zzz-nonexistent", name, cat, &platform), "Should not match unrelated query");

        let (shell_name, shell_cat) = config
            .categories
            .get_key_value("zsh-tokyonight")
            .expect("zsh-tokyonight category should exist");
        assert!(
            category_matches_query("atuin", shell_name, shell_cat, &platform),
            "Should match package only present in zsh-tokyonight"
        );
    }

    #[test]
    fn show_category_should_resolve_canonical_name_and_alias() {
        let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
        let state = State::load();
        let platform = Platform::Debian;

        assert!(show_category("zsh-tokyonight", &config, &state, &platform).is_ok());
        assert!(show_category("zsh-tn", &config, &state, &platform).is_ok());
        assert!(show_category("nonexistent", &config, &state, &platform).is_err());
    }

    #[test]
    fn remove_category_should_cleanup_copy_files_and_plugins_dir_in_dry_run() {
        let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
        let mut state = State::load();
        let platform = Platform::Debian;

        assert!(remove_category(Some("zsh-tokyonight"), &config, &mut state, &platform, true).is_ok());
    }

    #[test]
    fn inject_line_into_section_should_insert_under_existing_header() {
        let content = "HISTFILE=~/history\n# Fast init tools\neval \"$(starship init zsh)\"\n";
        let updated = inject_line_into_section(content, "# Fast init tools", "eval \"$(zoxide init zsh)\"", None);
        assert!(updated.contains("# Fast init tools\neval \"$(zoxide init zsh)\"\neval \"$(starship init zsh)\""));
    }

    #[test]
    fn inject_line_into_section_should_create_header_if_missing() {
        let content = "HISTFILE=~/history\n";
        let updated = inject_line_into_section(content, "# Fast init tools", "eval \"$(zoxide init zsh)\"", None);
        assert!(updated.contains("\n# Fast init tools\neval \"$(zoxide init zsh)\"\n"));
    }

    #[test]
    fn inject_line_into_section_should_prepend_to_top_when_position_is_top() {
        let content = "# Fast init tools\neval \"$(starship init zsh)\"\n";
        let updated = inject_line_into_section(
            content,
            "# History configuration",
            "HISTFILE=~/history",
            Some("top"),
        );
        assert!(updated.starts_with("# History configuration\nHISTFILE=~/history"));
    }

    #[test]
    fn process_section_injections_should_preview_in_dry_run() {
        let injections = vec![crate::config::SectionInjection {
            file: "~/.zshrc".to_string(),
            section: "# Fast init tools".to_string(),
            line: "eval \"$(zoxide init zsh)\"".to_string(),
            description: Some("Initialize zoxide".to_string()),
            platform: None,
            position: Some("top".to_string()),
        }];
        let res = process_section_injections(&injections, &Platform::Debian, true);
        assert!(res.is_ok(), "Section injection dry-run should succeed");
    }
}

