use crate::colors::*;
use crate::config::{Config, CustomInstaller};
use crate::platform::{Platform, check_apt_lock, command_exists};
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
    group_by_category: bool,
) {
    if config.categories.is_empty() {
        println!("No categories found in configuration.");
        return;
    }

    let mut output = String::new();
    let mut hidden_count = 0;

    if group_by_category {
        let mut grouped: std::collections::BTreeMap<
            String,
            Vec<(&String, &crate::config::Category)>,
        > = std::collections::BTreeMap::new();

        for (cat_name, cat) in &config.categories {
            if let Some(query) = filter
                && !category_matches_query(query, cat_name, cat, platform)
            {
                continue;
            }

            let grp_name = cat
                .group
                .clone()
                .unwrap_or_else(|| "Other Categories".to_string());
            grouped.entry(grp_name).or_default().push((cat_name, cat));
        }

        for (grp_name, cats) in grouped {
            let visible_cats: Vec<_> = cats
                .into_iter()
                .filter(|(_, cat)| {
                    let is_supported = is_category_supported(cat, platform, config);
                    if !is_supported && !show_hidden {
                        hidden_count += 1;
                        false
                    } else {
                        true
                    }
                })
                .collect();

            if visible_cats.is_empty() {
                continue;
            }

            let border = "─".repeat(grp_name.chars().count() + 2);
            output.push_str(&format!(
                "\n{BOLD_CYAN}╭{border}╮\n│ {grp_name} │\n╰{border}╯{RESET}\n"
            ));

            for (cat_name, cat) in visible_cats {
                let mut pkgs: Vec<String> = if let Some(disp) = &cat.display_packages {
                    disp.clone()
                } else {
                    match platform {
                        Platform::Debian => cat.debian_packages.clone().unwrap_or_default(),
                        Platform::Termux => cat.termux_packages.clone().unwrap_or_default(),
                        Platform::Unsupported(_) => Vec::new(),
                    }
                };

                if matches!(platform, Platform::Debian)
                    && let Some(custom) = cat.custom.as_ref().and_then(|m| m.get("debian"))
                    && cat.display_packages.is_none()
                {
                    pkgs.push(format!("{} (custom binary)", custom.name));
                }

                let is_supported = is_category_supported(cat, platform, config);

                let pack_label = if cat.includes.is_some() {
                    " (Pack)"
                } else {
                    ""
                };

                let alias_part = cat.aliases.as_ref().map_or_else(String::new, |aliases| {
                    if aliases.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", aliases.join(", "))
                    }
                });

                let (full_detail_str, count_str, total_count, is_redundant) =
                    if let Some(inc_list) = &cat.includes {
                        let mut expanded: Vec<String> = Vec::new();
                        if let Some(disp) = &cat.display_packages {
                            expanded = disp.clone();
                        } else {
                            for inc in inc_list {
                                if let Some(sub) = config.categories.get(inc) {
                                    if let Some(disp) = &sub.display_packages {
                                        expanded.extend(disp.clone());
                                    } else {
                                        expanded.push(inc.clone());
                                    }
                                } else {
                                    expanded.push(inc.clone());
                                }
                            }
                        }
                        let mut unique_items = Vec::new();
                        for item in expanded {
                            if !unique_items.contains(&item) {
                                unique_items.push(item);
                            }
                        }
                        let cnt = unique_items.len();
                        (
                            format!("pack: {}", format_items_limited(&unique_items, 6)),
                            format!("pack: {cnt} items"),
                            cnt,
                            false,
                        )
                    } else if pkgs.is_empty() {
                        ("none".to_string(), "0 pkgs".to_string(), 0, true)
                    } else {
                        let cnt = pkgs.len();
                        let formatted = format_items_limited(&pkgs, 6);
                        let redundant = formatted.eq_ignore_ascii_case(cat_name);
                        let count_label = if cnt == 1 {
                            "1 pkg".to_string()
                        } else {
                            format!("{cnt} pkgs")
                        };
                        (formatted, count_label, cnt, redundant)
                    };

                let apply_suffix = if is_category_applied(cat_name, cat, state, platform, config) {
                    format!(" {WHITE}[apply]{RESET}")
                } else {
                    String::new()
                };
                let apply_suffix_len =
                    if is_category_applied(cat_name, cat, state, platform, config) {
                        8
                    } else {
                        0
                    };

                let hidden_suffix = if !is_supported {
                    format!(" {DIM_GRAY}[unsupported]{RESET}")
                } else {
                    String::new()
                };
                let hidden_suffix_len = if !is_supported { 14 } else { 0 };

                let term_width = get_terminal_width();
                let base_header_len =
                    4 + cat_name.len() + pack_label.len() + alias_part.len() + 2 + cat.description.len();

                let pkg_part = if is_redundant || total_count == 0 {
                    String::new()
                } else {
                    let full_candidate_len = 3 + full_detail_str.len();
                    if base_header_len + full_candidate_len + apply_suffix_len + hidden_suffix_len
                        <= term_width
                    {
                        format!(" {DIM_GRAY}({full_detail_str}){RESET}")
                    } else {
                        format!(" {DIM_GRAY}({count_str}){RESET}")
                    }
                };

                output.push_str(&format!(
                    "  - {BOLD_GREEN}{cat_name}{RESET}{pack_label}{alias_part}: {}{pkg_part}{apply_suffix}{hidden_suffix}\n",
                    cat.description
                ));
            }
        }
    } else {
        for (cat_name, cat) in &config.categories {
            let mut pkgs: Vec<String> = if let Some(disp) = &cat.display_packages {
                disp.clone()
            } else {
                match platform {
                    Platform::Debian => cat.debian_packages.clone().unwrap_or_default(),
                    Platform::Termux => cat.termux_packages.clone().unwrap_or_default(),
                    Platform::Unsupported(_) => Vec::new(),
                }
            };

            if let Some(query) = filter
                && !category_matches_query(query, cat_name, cat, platform)
            {
                continue;
            }

            if matches!(platform, Platform::Debian)
                && let Some(custom) = cat.custom.as_ref().and_then(|m| m.get("debian"))
                && cat.display_packages.is_none()
            {
                pkgs.push(format!("{} (custom binary)", custom.name));
            }

            let is_supported = is_category_supported(cat, platform, config);

            if !is_supported && !show_hidden {
                hidden_count += 1;
                continue;
            }

            let pack_label = if cat.includes.is_some() {
                " (Pack)"
            } else {
                ""
            };

            let alias_part = cat.aliases.as_ref().map_or_else(String::new, |aliases| {
                if aliases.is_empty() {
                    String::new()
                } else {
                    format!(", {}", aliases.join(", "))
                }
            });

            let pkgs_str = if let Some(inc_list) = &cat.includes {
                if let Some(disp) = &cat.display_packages {
                    format!("pack: {}", format_items_limited(disp, 6))
                } else {
                    let mut expanded: Vec<String> = Vec::new();
                    for inc in inc_list {
                        if let Some(sub) = config.categories.get(inc) {
                            if let Some(disp) = &sub.display_packages {
                                expanded.extend(disp.clone());
                            } else {
                                expanded.push(inc.clone());
                            }
                        } else {
                            expanded.push(inc.clone());
                        }
                    }
                    let mut unique_items = Vec::new();
                    for item in expanded {
                        if !unique_items.contains(&item) {
                            unique_items.push(item);
                        }
                    }
                    format!("pack: {}", format_items_limited(&unique_items, 6))
                }
            } else if pkgs.is_empty() {
                "none".to_string()
            } else {
                format_items_limited(&pkgs, 6)
            };

            let is_redundant = pkgs_str.eq_ignore_ascii_case(cat_name);
            let detail_part = if is_redundant {
                String::new()
            } else {
                format!(" / {DIM_GRAY}{pkgs_str}{RESET}")
            };

            let apply_suffix = if is_category_applied(cat_name, cat, state, platform, config) {
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
                "{BOLD_GREEN}{cat_name}{RESET}{pack_label}{alias_part}{detail_part}{apply_suffix}{hidden_suffix}\n"
            ));
        }
    }

    if hidden_count > 0 && !show_hidden {
        output.push_str(&format!(
            "\n{DIM_GRAY}{hidden_count} hidden. Use '-sh' or '--show-hidden' to view all.{RESET}\n"
        ));
    }

    output_with_pager(&output);
}

/// Case-insensitive substring match of `query` against a category name, its aliases,
/// group, included categories, and the packages declared for the current platform.
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

    if cat.aliases.as_ref().is_some_and(|aliases| {
        aliases
            .iter()
            .any(|alias| alias.to_lowercase().contains(&q))
    }) {
        return true;
    }

    if let Some(group) = &cat.group
        && group.to_lowercase().contains(&q)
    {
        return true;
    }

    if let Some(inc_list) = &cat.includes
        && inc_list.iter().any(|inc| inc.to_lowercase().contains(&q))
    {
        return true;
    }

    if let Some(disp) = &cat.display_packages
        && disp.iter().any(|p| p.to_lowercase().contains(&q))
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

fn is_command_in_path(cmd: &str) -> bool {
    if let Ok(path_var) = env::var("PATH") {
        for dir in env::split_paths(&path_var) {
            let full_path = dir.join(cmd);
            if full_path.is_file() {
                return true;
            }
        }
    }
    false
}

pub fn is_category_supported(
    cat: &crate::config::Category,
    platform: &Platform,
    config: &Config,
) -> bool {
    if let Some(target_platform) = &cat.platform {
        let matches_platform = matches!(
            (target_platform.to_lowercase().as_str(), platform),
            ("debian", Platform::Debian) | ("termux", Platform::Termux)
        );
        if !matches_platform {
            return false;
        }
    }

    let has_os_packages = match platform {
        Platform::Debian => cat.debian_packages.as_ref().is_some_and(|p| !p.is_empty()),
        Platform::Termux => cat.termux_packages.as_ref().is_some_and(|p| !p.is_empty()),
        Platform::Unsupported(_) => false,
    };

    let has_custom_binary = matches!(platform, Platform::Debian)
        && cat
            .custom
            .as_ref()
            .is_some_and(|m| m.contains_key("debian"));

    if has_os_packages || has_custom_binary {
        return true;
    }

    if let Some(inc_list) = &cat.includes {
        return inc_list.iter().any(|inc_name| {
            if let Some(sub_cat) = config.categories.get(inc_name) {
                is_category_supported(sub_cat, platform, config)
            } else {
                false
            }
        });
    }

    false
}

pub fn format_items_limited(items: &[String], limit: usize) -> String {
    if items.is_empty() {
        return "none".to_string();
    }
    if items.len() <= limit {
        items.join(", ")
    } else {
        let shown = &items[..limit];
        let remaining = items.len() - limit;
        format!("{}, (+{} more)", shown.join(", "), remaining)
    }
}

pub fn get_terminal_width() -> usize {
    if let Ok(cols) = env::var("COLUMNS")
        && let Ok(w) = cols.parse::<usize>()
        && w > 20
    {
        return w;
    }

    if let Ok(output) = std::process::Command::new("tput").arg("cols").output()
        && output.status.success()
        && let Ok(s) = std::str::from_utf8(&output.stdout)
        && let Ok(w) = s.trim().parse::<usize>()
        && w > 20
    {
        return w;
    }

    80
}

pub fn is_category_applied(
    cat_name: &str,
    cat: &crate::config::Category,
    state: &State,
    platform: &Platform,
    config: &Config,
) -> bool {
    if let Some(inc_list) = &cat.includes {
        if inc_list.is_empty() {
            return false;
        }
        return inc_list.iter().all(|inc_name| {
            if let Some((resolved_key, sub_cat)) = config.categories.get_key_value(inc_name) {
                is_category_applied(resolved_key, sub_cat, state, platform, config)
            } else {
                false
            }
        });
    }

    let recorded_in_state = state.is_category_applied(cat_name)
        || state.packages.values().any(|p| p.category == cat_name);

    if !recorded_in_state {
        return false;
    }

    if let Platform::Debian = platform
        && let Some(custom) = cat.custom.as_ref().and_then(|m| m.get("debian"))
    {
        let bin_path = expand_home(&custom.bin_symlink);
        if !Path::new(&bin_path).exists() && !is_command_in_path(&custom.name) {
            return false;
        }
    } else if let Platform::Termux = platform
        && let Some(custom) = cat.custom.as_ref().and_then(|m| m.get("termux"))
    {
        let bin_path = expand_home(&custom.bin_symlink);
        if !Path::new(&bin_path).exists() && !is_command_in_path(&custom.name) {
            return false;
        }
    }

    true
}

/// Shows complete detailed information, description, packages, dotfiles, and installation status for a specific category.
pub fn show_category(
    category_query: &str,
    config: &Config,
    state: &State,
    platform: &Platform,
) -> anyhow::Result<()> {
    let canonical_key = config.resolve_category_key(category_query).ok_or_else(|| {
        anyhow::anyhow!("Category or alias '{category_query}' not found in configuration.")
    })?;

    let cat = config.categories.get(canonical_key).unwrap();

    let aliases_str = cat
        .aliases
        .as_ref()
        .map(|a| a.join(", "))
        .unwrap_or_else(|| "none".to_string());

    println!("\n{BOLD_GREEN}Category:{RESET} {canonical_key}");
    println!("{BOLD_BLUE}Description:{RESET} {}", cat.description);
    println!("{BOLD_BLUE}Aliases:{RESET} {aliases_str}");

    if let Some(inc_list) = &cat.includes {
        println!("\n{BOLD_BLUE}Included Categories (Pack):{RESET}");
        for inc_name in inc_list {
            let status_str = if let Some(sub_cat) = config.categories.get(inc_name) {
                if is_category_applied(inc_name, sub_cat, state, platform, config) {
                    "Applied"
                } else {
                    "Not applied"
                }
            } else {
                "Unknown"
            };
            println!("  - {DIM_GRAY}{inc_name}{RESET} [{status_str}]");
        }
    }

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
        println!(
            "  - {DIM_GRAY}{}{RESET} ({}) [{custom_status}]",
            custom.name, custom.url
        );
        println!(
            "    Symlink: {bin_path} -> {}/bin/{}",
            custom.extract_dir, custom.name
        );
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
            let desc = inj.description.as_deref().unwrap_or(inj.line.as_str());
            println!(
                "  - {desc} -> {} under section '{}'{platform_info}",
                inj.file, inj.section
            );
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
        if let Some(inc_list) = &cat.includes {
            println!(
                "\n{BOLD_BLUE}--> Installing Pack '{cat_name}' (Includes {} categories)...{RESET}",
                inc_list.len()
            );
            for inc_name in inc_list {
                install_category(Some(inc_name.as_str()), config, state, platform, dry_run)?;
            }
            continue;
        }

        println!("\n--> Processing Category: {cat_name}");

        // Pre-prompt for any interactive post-install commands before doing packages/downloads
        let mut confirmed_commands: std::collections::HashMap<usize, bool> =
            std::collections::HashMap::new();

        if let Some(post_cmds) = &cat.post_install_commands {
            for (idx, cmd) in post_cmds.iter().enumerate() {
                if let Some(target_platform) = &cmd.platform {
                    let matches_platform = matches!(
                        (target_platform.to_lowercase().as_str(), platform),
                        ("debian", Platform::Debian) | ("termux", Platform::Termux)
                    );
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
                if !dry_run {
                    state.track_package(pkg, cat_name, true);
                    state.save_atomic()?;
                }
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

        if !dry_run {
            state.mark_category_applied(cat_name);
            state.save_atomic()?;
        }
    }

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
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
        let src_path = Path::new(&action.src);

        if action.only_if_not_exists.unwrap_or(false) && dest_path.exists() {
            println!("  [SKIP] Destination '{dest_path_str}' already exists.");
            continue;
        }

        if action.backup.unwrap_or(false) && dest_path.exists() {
            let backup_path_str = if !Path::new(&format!("{dest_path_str}.bak")).exists() {
                format!("{dest_path_str}.bak")
            } else {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                format!("{dest_path_str}.bak.{timestamp}")
            };

            if dry_run {
                println!(
                    "  [Dry-Run] Would back up existing '{dest_path_str}' to '{backup_path_str}'"
                );
            } else {
                println!(
                    "  [Backup] Backing up existing '{dest_path_str}' -> '{backup_path_str}'"
                );
                fs::rename(dest_path, &backup_path_str).map_err(|e| {
                    anyhow::anyhow!("Failed to back up {dest_path_str} to {backup_path_str}: {e}")
                })?;
            }
        }

        if src_path.is_dir() {
            if dry_run {
                println!(
                    "  [Dry-Run] Would copy directory '{}' to '{dest_path_str}'",
                    action.src
                );
                continue;
            }

            println!(
                "  [Copying] Directory '{}' to '{dest_path_str}'",
                action.src
            );
            copy_dir_all(src_path, dest_path).map_err(|e| {
                anyhow::anyhow!("Failed to copy directory {} to {dest_path_str}: {e}", action.src)
            })?;
            println!("  [Success] Directory placed cleanly at '{dest_path_str}'");
            continue;
        }

        if dry_run {
            println!("  [Dry-Run] Would copy configuration file to '{dest_path_str}'");
            continue;
        }

        println!("  [Copying] Configuration file to '{dest_path_str}'");

        let content_bytes: Vec<u8> = if src_path.exists() {
            fs::read(&action.src)
                .map_err(|e| anyhow::anyhow!("Failed to read source file {}: {e}", action.src))?
        } else if action.src == "config/zsh-tokyonight/starship.toml" {
            crate::config::EMBEDDED_STARSHIP.as_bytes().to_vec()
        } else if action.src == "config/zsh-tokyonight/aliases.zsh" {
            crate::config::EMBEDDED_ALIASES.as_bytes().to_vec()
        } else if action.src == "config/zsh-tokyonight/font.ttf"
            || action.src == "config/zsh-minimal/font.ttf"
        {
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
            fs::create_dir_all(parent).map_err(|e| {
                anyhow::anyhow!("Failed to create directory {}: {e}", parent.display())
            })?;
        }

        fs::write(dest_path, content_bytes).map_err(|e| {
            anyhow::anyhow!("Failed to write configuration to {dest_path_str}: {e}")
        })?;

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
            let matches_platform = matches!(
                (target_platform.to_lowercase().as_str(), platform),
                ("debian", Platform::Debian) | ("termux", Platform::Termux)
            );
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
                println!(
                    "  [SKIP] Post-install command '{}' skipped by user choice.",
                    cmd.command
                );
                continue;
            }
        }

        if dry_run {
            println!(
                "  [Dry-Run] Would execute post-install command: {}",
                cmd.command
            );
            continue;
        }

        println!("  [Executing] Post-install command: {}", cmd.command);
        let status = Command::new("sh")
            .arg("-c")
            .arg(&cmd.command)
            .status()
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to execute post-install command '{}': {e}",
                    cmd.command
                )
            })?;

        if !status.success() {
            anyhow::bail!(
                "Post-install command '{}' failed with status {status}",
                cmd.command
            );
        }

        println!(
            "  [Success] Post-install command completed: {}",
            cmd.command
        );
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
            fs::read_to_string(path).map_err(|e| {
                anyhow::anyhow!("Failed to read config file '{}': {e}", path.display())
            })?
        } else {
            String::new()
        };

        let first_line = injection
            .line
            .lines()
            .next()
            .unwrap_or(&injection.line)
            .trim();
        if content.lines().any(|l| l.trim() == first_line) {
            println!(
                "  [SKIP] Section line '{}' already exists in {}.",
                first_line, injection.file
            );
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
            fs::create_dir_all(parent).map_err(|e| {
                anyhow::anyhow!("Failed to create directories for '{}': {e}", path.display())
            })?;
        }

        fs::write(path, new_content).map_err(|e| {
            anyhow::anyhow!(
                "Failed to write updated config to '{}': {e}",
                path.display()
            )
        })?;

        println!(
            "  [Success] Injected line into {} under section '{}': {}",
            injection.file, injection.section, desc
        );
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
        anyhow::bail!(
            "Prerequisite binaries 'curl' and 'tar' are required for custom downloads. Please install them first."
        );
    }

    let bin_path_buf = Path::new(&expand_home(&custom.bin_symlink)).to_path_buf();
    let extract_dir_buf = Path::new(&custom.extract_dir).to_path_buf();

    if bin_path_buf.exists() && extract_dir_buf.exists() {
        println!(
            "  [SKIP] Custom binary '{}' is already installed at {}",
            custom.name,
            extract_dir_buf.display()
        );
        if !dry_run {
            state.track_package(&custom.name, cat_name, true);
            state.save_atomic()?;
        }
        return Ok(());
    }

    if dry_run {
        println!(
            "  [Dry-Run] Would download release tarball from {}",
            custom.url
        );
        println!("  [Dry-Run] Would extract to {}", custom.extract_dir);
        println!(
            "  [Dry-Run] Would create symlink: {} -> {}/bin/{}",
            bin_path_buf.display(),
            custom.extract_dir,
            custom.name
        );
        return Ok(());
    }

    let tmp_dir = env::temp_dir().join(format!("{}_install", custom.name));
    fs::create_dir_all(&tmp_dir).context("Failed to create temp directory")?;
    let tmp_tarball = tmp_dir.join(format!("{}.tar.gz", custom.name));

    println!("  [Downloading] Downloading {}...", custom.name);
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

    // Extract tarball with --strip-components=1 first (for nested release archives like Neovim)
    let _ = Command::new("sudo")
        .arg("tar")
        .arg("-xzf")
        .arg(&tmp_tarball)
        .arg("-C")
        .arg(&custom.extract_dir)
        .arg("--strip-components=1")
        .status();

    let bin_in_subfolder = format!("{}/bin/{}", custom.extract_dir, custom.name);
    let bin_at_root = format!("{}/{}", custom.extract_dir, custom.name);

    // If neither path exists (e.g. flat tarball like zellij), extract directly without --strip-components=1
    if !Path::new(&bin_in_subfolder).exists() && !Path::new(&bin_at_root).exists() {
        let tar_status = Command::new("sudo")
            .arg("tar")
            .arg("-xzf")
            .arg(&tmp_tarball)
            .arg("-C")
            .arg(&custom.extract_dir)
            .status()
            .context("Failed to extract flat tarball")?;

        if !tar_status.success() {
            let _ = fs::remove_dir_all(&tmp_dir);
            anyhow::bail!("Failed to extract tarball to {}", custom.extract_dir);
        }
    }

    let _ = fs::remove_dir_all(&tmp_dir);

    // Resolve target binary path and verify it exists
    let target_bin = if Path::new(&bin_in_subfolder).exists() {
        bin_in_subfolder
    } else if Path::new(&bin_at_root).exists() {
        bin_at_root
    } else {
        anyhow::bail!(
            "Extracted binary '{}' was not found in {} or {}/bin",
            custom.name,
            custom.extract_dir,
            custom.extract_dir
        );
    };

    // Ensure the binary is executable
    let _ = Command::new("sudo")
        .arg("chmod")
        .arg("+x")
        .arg(&target_bin)
        .status();

    let symlink_dir = bin_path_buf.parent().unwrap();
    fs::create_dir_all(symlink_dir).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create symlink parent directory {}: {e}",
            symlink_dir.display()
        )
    })?;

    if bin_path_buf.exists() || bin_path_buf.is_symlink() {
        let _ = fs::remove_file(&bin_path_buf);
    }

    symlink(&target_bin, &bin_path_buf).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create symlink at {}: {e}",
            bin_path_buf.display()
        )
    })?;

    println!(
        "  [Success] Installed {} to {} with symlink at {}",
        custom.name,
        custom.extract_dir,
        bin_path_buf.display()
    );
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
        if let Some(inc_list) = &cat.includes {
            println!(
                "\n{BOLD_YELLOW}--> Removing Pack '{cat_name}' (Includes {} categories)...{RESET}",
                inc_list.len()
            );
            for inc_name in inc_list {
                remove_category(Some(inc_name.as_str()), config, state, platform, dry_run)?;
            }
            continue;
        }

        println!("\n--> Processing Removal for Category: {cat_name}");

        let pkgs = match platform {
            Platform::Debian => cat.debian_packages.as_deref().unwrap_or(&[]),
            Platform::Termux => cat.termux_packages.as_deref().unwrap_or(&[]),
            Platform::Unsupported(_) => &[],
        };

        for pkg in pkgs {
            if let Some(tracked) = state.packages.get(pkg) {
                if tracked.was_preexisting {
                    println!(
                        "  [SKIP] Package '{pkg}' was pre-existing on system before dotss. Skipping removal."
                    );
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
                println!(
                    "  [Dry-Run] Would execute: sudo rm -rf {}",
                    custom.extract_dir
                );
            } else {
                println!("  [Removing] Removing custom binary '{}'...", custom.name);
                if let Err(e) = fs::remove_file(&bin_path) {
                    eprintln!("  [WARN] Failed to remove symlink {bin_path}: {e}");
                }
                match Command::new("sudo")
                    .args(["rm", "-rf", &custom.extract_dir])
                    .status()
                {
                    Ok(s) if !s.success() => {
                        eprintln!(
                            "  [WARN] sudo rm -rf {} exited with {s}",
                            custom.extract_dir
                        );
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
                if dest_path.exists() {
                    if dest_path.is_dir() {
                        if dry_run {
                            println!("  [Dry-Run] Would remove configuration directory: '{dest_path_str}'");
                        } else {
                            println!("  [Removing] Removing configuration directory '{dest_path_str}'...");
                            if let Err(e) = fs::remove_dir_all(dest_path) {
                                eprintln!(
                                    "  [WARN] Failed to remove configuration directory {dest_path_str}: {e}"
                                );
                            }
                        }
                    } else if dest_path.is_file() {
                        if dry_run {
                            println!("  [Dry-Run] Would remove configuration file: '{dest_path_str}'");
                        } else {
                            println!("  [Removing] Removing configuration file '{dest_path_str}'...");
                            if let Err(e) = fs::remove_file(dest_path) {
                                eprintln!(
                                    "  [WARN] Failed to remove configuration file {dest_path_str}: {e}"
                                );
                            }
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
                        eprintln!(
                            "  [WARN] Failed to remove Zsh plugins directory {plugins_dir}: {e}"
                        );
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

        if !dry_run {
            state.unmark_category_applied(cat_name);
            state.save_atomic()?;
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
            println!(
                "  To access binaries directly from anywhere, add it to your shell configuration:"
            );
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
    } else if cmd.contains("opencode.ai/install") {
        "Install or update opencode".to_string()
    } else if cmd.contains("engram") {
        "Install or update engram".to_string()
    } else if cmd.contains("herdr.dev/install") {
        "Install or update herdr".to_string()
    } else if cmd.contains("codegraph") {
        "Install or update codegraph".to_string()
    } else {
        let first_line = cmd.lines().next().unwrap_or(cmd);
        if first_line.len() > 60 {
            format!("{}...", &first_line[..57])
        } else {
            first_line.to_string()
        }
    }
}

/// Runs the interactive setup wizard for selecting presets or custom categories.
pub fn run_interactive_setup(
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

    if !std::io::stdout().is_terminal() {
        anyhow::bail!("Interactive setup requires an interactive terminal (TTY).");
    }

    let preset_choice = crate::prompt::select_preset()?;
    let categories_to_install: Vec<String> = match preset_choice {
        Some(crate::prompt::PresetOption::FullConfig) => {
            crate::prompt::FULL_CONFIG_CATEGORIES
                .iter()
                .filter(|cat_name| {
                    if let Some(cat) = config.categories.get(**cat_name) {
                        is_category_supported(cat, platform, config)
                    } else {
                        false
                    }
                })
                .map(|s| s.to_string())
                .collect()
        }
        Some(crate::prompt::PresetOption::Custom) => {
            match crate::prompt::select_custom_categories(config, platform, state)? {
                Some(selected) => selected,
                None => {
                    println!("\n{DIM_GRAY}Setup cancelled.{RESET}");
                    return Ok(());
                }
            }
        }
        None => {
            println!("\n{DIM_GRAY}Setup cancelled.{RESET}");
            return Ok(());
        }
    };

    if categories_to_install.is_empty() {
        println!("\n{DIM_GRAY}No categories selected for installation.{RESET}");
        return Ok(());
    }

    println!("\n{BOLD_CYAN}Selected categories to install:{RESET}");
    for cat_name in &categories_to_install {
        let is_pack = config
            .categories
            .get(cat_name)
            .and_then(|c| c.includes.as_ref())
            .is_some();
        let pack_suffix = if is_pack { " (Pack)" } else { "" };
        println!("  {BOLD_GREEN}•{RESET} {cat_name}{pack_suffix}");
    }

    if dry_run {
        println!(
            "\n{BOLD_YELLOW}=== DRY-RUN MODE ACTIVE: Simulating installation ==={RESET}"
        );
    } else {
        use std::io::{self, Write};
        print!("\n{BOLD_YELLOW}? Proceed with installation? [Y/n]: {RESET}");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let trimmed = input.trim().to_lowercase();
            if !trimmed.is_empty() && trimmed != "y" && trimmed != "yes" {
                println!("\n{DIM_GRAY}Installation cancelled.{RESET}");
                return Ok(());
            }
        }
    }

    for cat_name in &categories_to_install {
        install_category(Some(cat_name), config, state, platform, dry_run)?;
    }

    println!("\n{BOLD_GREEN}Interactive setup completed successfully!{RESET}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_home_should_replace_tilde_with_home_dir() {
        let path_with_tilde = "~/.local/bin/nvim";
        let expanded = expand_home(path_with_tilde);
        assert!(
            !expanded.starts_with("~/"),
            "Tilde should be expanded to full path"
        );
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
            backup: None,
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
            backup: None,
        }];
        let res_debian = process_copy_files(&copy_actions, &Platform::Debian, true);
        assert!(res_debian.is_ok());

        let res_termux = process_copy_files(&copy_actions, &Platform::Termux, true);
        assert!(res_termux.is_ok());
    }

    #[test]
    fn copy_files_with_backup_and_directory_should_succeed_in_dry_run() {
        let copy_actions = vec![crate::config::CopyFileAction {
            src: "config/nvim-onedarkpro".to_string(),
            dest: "~/.config/nvim".to_string(),
            platform: None,
            only_if_not_exists: None,
            backup: Some(true),
        }];
        let res = process_copy_files(&copy_actions, &Platform::Debian, true);
        assert!(res.is_ok(), "Directory copy with backup dry-run should succeed");
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
        let res_debian =
            process_post_install_commands(&commands, &Platform::Debian, &confirmed, true);
        assert!(res_debian.is_ok());
        let res_termux =
            process_post_install_commands(&commands, &Platform::Termux, &confirmed, true);
        assert!(res_termux.is_ok());
    }

    #[test]
    fn list_categories_should_include_configured_categories() {
        let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
        let state = State::load();
        list_categories(&config, &state, &Platform::Debian, false, None, false);
        list_categories(&config, &state, &Platform::Termux, true, None, false);
    }

    #[test]
    fn category_matches_query_should_match_name_alias_and_packages() {
        let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
        let platform = Platform::Debian;

        let (name, cat) = config
            .categories
            .get_key_value("lazyvim-minimal")
            .expect("lazyvim-minimal category should exist");

        assert!(
            category_matches_query("lazy", name, cat, &platform),
            "Should match by category name"
        );
        assert!(
            category_matches_query("LAZY", name, cat, &platform),
            "Should match case-insensitively"
        );
        assert!(
            category_matches_query("lzv", name, cat, &platform),
            "Should match by alias"
        );
        assert!(
            category_matches_query("lazygit", name, cat, &platform),
            "Should match by package name"
        );
        assert!(
            category_matches_query("neovim", name, cat, &platform),
            "Should match by Debian custom binary name"
        );
        assert!(
            !category_matches_query("nvim", name, cat, &platform),
            "'nvim' is not a substring of 'neovim', so it must not match"
        );
        assert!(
            !category_matches_query("zzz-nonexistent", name, cat, &platform),
            "Should not match unrelated query"
        );

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

        assert!(
            remove_category(Some("zsh-tokyonight"), &config, &mut state, &platform, true).is_ok()
        );
    }

    #[test]
    fn inject_line_into_section_should_insert_under_existing_header() {
        let content = "HISTFILE=~/history\n# Fast init tools\neval \"$(starship init zsh)\"\n";
        let updated = inject_line_into_section(
            content,
            "# Fast init tools",
            "eval \"$(zoxide init zsh)\"",
            None,
        );
        assert!(updated.contains(
            "# Fast init tools\neval \"$(zoxide init zsh)\"\neval \"$(starship init zsh)\""
        ));
    }

    #[test]
    fn inject_line_into_section_should_create_header_if_missing() {
        let content = "HISTFILE=~/history\n";
        let updated = inject_line_into_section(
            content,
            "# Fast init tools",
            "eval \"$(zoxide init zsh)\"",
            None,
        );
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

    #[test]
    fn is_category_applied_should_not_false_positive_on_generic_packages() {
        let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
        let state = State::default();
        let platform = Platform::Debian;

        let (cat_name, cat) = config
            .categories
            .get_key_value("opencode")
            .expect("opencode category should exist");

        if !is_command_in_path("opencode") {
            assert!(
                !is_category_applied(cat_name, cat, &state, &platform, &config),
                "opencode should NOT be marked as applied when opencode binary is missing, even if curl/git/bash are installed"
            );
        }

        let mut applied_state = State::default();
        applied_state.mark_category_applied("opencode");
        assert!(
            is_category_applied(cat_name, cat, &applied_state, &platform, &config),
            "opencode should be marked as applied when recorded in state"
        );
    }

    #[test]
    fn dry_run_mode_must_not_modify_state_or_track_packages() {
        let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
        let mut state = State::default();
        let platform = Platform::Debian;

        let initial_packages_count = state.packages.len();
        let initial_applied_count = state.applied_categories.len();

        let res = install_category(Some("opencode"), &config, &mut state, &platform, true);
        assert!(res.is_ok(), "install_category in dry_run should succeed");

        assert_eq!(
            state.packages.len(),
            initial_packages_count,
            "dry_run MUST NOT track packages in state"
        );
        assert_eq!(
            state.applied_categories.len(),
            initial_applied_count,
            "dry_run MUST NOT mark categories as applied in state"
        );
    }

    #[test]
    fn format_items_limited_should_cap_overflowing_items() {
        let items = vec![
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string(),
            "item4".to_string(),
            "item5".to_string(),
            "item6".to_string(),
            "item7".to_string(),
            "item8".to_string(),
        ];
        assert_eq!(
            format_items_limited(&items, 6),
            "item1, item2, item3, item4, item5, item6, (+2 more)"
        );
        assert_eq!(format_items_limited(&items[..3], 6), "item1, item2, item3");
        assert_eq!(format_items_limited(&[], 6), "none");
    }

    #[test]
    fn pack_categories_should_be_identifiable_with_includes() {
        let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
        let agents_flow = config.categories.get("agents-flow").unwrap();
        assert!(agents_flow.includes.is_some(), "agents-flow must be a pack");

        let mydots_termux = config.categories.get("mydots-termux").unwrap();
        assert!(mydots_termux.includes.is_some(), "mydots-termux must be a pack");

        let zsh_tn = config.categories.get("zsh-tokyonight").unwrap();
        assert!(zsh_tn.includes.is_none(), "zsh-tokyonight is an individual category");
    }

    #[test]
    fn full_config_preset_categories_must_exist_in_config() {
        let config: Config = toml::from_str(crate::config::EMBEDDED_CONFIG).unwrap();
        for cat_name in crate::prompt::FULL_CONFIG_CATEGORIES {
            assert!(
                config.categories.contains_key(*cat_name),
                "Category '{cat_name}' in FULL_CONFIG_CATEGORIES must exist in categories.toml"
            );
        }
    }
}
