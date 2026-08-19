use crate::colors::*;
use crate::config::Config;
use crate::platform::Platform;
use crate::state::State;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write};

pub const FULL_CONFIG_CATEGORIES: &[&str] = &[
    "nvim-onedarkpro",
    "zsh-tokyonight",
    "term-flow",
    "nodejs-pnpm",
    "agents-flow",
];

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, cursor::Hide)?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresetOption {
    FullConfig,
    Custom,
}

struct PresetItem {
    name: &'static str,
    description: &'static str,
    option: PresetOption,
}

/// Prompt the user with a single-selection preset menu
pub fn select_preset() -> anyhow::Result<Option<PresetOption>> {
    let _guard = RawModeGuard::new()?;
    let mut stdout = io::stdout();

    let items = [
        PresetItem {
            name: "Full Config",
            description: "Curated complementary suite (nvim-onedarkpro, zsh-tokyonight, term-flow, nodejs-pnpm, agents-flow)",
            option: PresetOption::FullConfig,
        },
        PresetItem {
            name: "Custom",
            description: "Step-by-step custom configuration",
            option: PresetOption::Custom,
        },
    ];

    let mut selected_index = 0;
    let mut rendered_lines = 0;

    loop {
        // Clear previously rendered lines
        if rendered_lines > 0 {
            execute!(stdout, cursor::MoveToPreviousLine(rendered_lines as u16))?;
            for _ in 0..rendered_lines {
                execute!(stdout, Clear(ClearType::CurrentLine), cursor::MoveDown(1))?;
            }
            execute!(stdout, cursor::MoveToPreviousLine(rendered_lines as u16))?;
        }

        rendered_lines = 0;

        let header = format!(
            "{BOLD_CYAN}? What type of configuration do you want to install?{RESET}\r\n\r\n"
        );
        execute!(stdout, Print(&header))?;
        rendered_lines += 2;

        for (idx, item) in items.iter().enumerate() {
            let is_focused = idx == selected_index;
            let line = if is_focused {
                format!(
                    "  {BOLD_GREEN}❯ {BOLD}{}{RESET} {DIM_GRAY}- {}{RESET}\r\n",
                    item.name, item.description
                )
            } else {
                format!(
                    "    {WHITE}{}{RESET} {DIM_GRAY}- {}{RESET}\r\n",
                    item.name, item.description
                )
            };
            execute!(stdout, Print(&line))?;
            rendered_lines += 1;
        }

        let footer = format!(
            "\r\n{DIM_GRAY}  Use [↑/↓] or [j/k] to navigate, [Enter] to select, [q/Esc] to cancel{RESET}\r\n"
        );
        execute!(stdout, Print(&footer))?;
        rendered_lines += 2;
        stdout.flush()?;

        if let Event::Key(key_event) = event::read()? {
            if key_event.kind != KeyEventKind::Press {
                continue;
            }
            match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected_index == 0 {
                        selected_index = items.len() - 1;
                    } else {
                        selected_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected_index + 1 >= items.len() {
                        selected_index = 0;
                    } else {
                        selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    return Ok(Some(items[selected_index].option.clone()));
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    return Ok(None);
                }
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(None);
                }
                _ => {}
            }
        }
    }
}

pub struct CategorySelectOption {
    pub key: String,
    pub display_name: String,
    pub description: String,
    pub packages_label: String,
    pub is_installed: bool,
    pub checked: bool,
}

/// Prompt the user with a step-by-step section wizard for custom category selection
pub fn select_custom_categories(
    config: &Config,
    platform: &Platform,
    state: &State,
) -> anyhow::Result<Option<Vec<String>>> {
    let preferred_group_order = [
        "Shell & Terminal",
        "Editors & IDEs",
        "AI & Agents",
        "Runtimes & Languages",
        "Window Managers & Desktop",
        "Packs & Bundles",
    ];

    let mut grouped_categories: std::collections::BTreeMap<String, Vec<CategorySelectOption>> =
        std::collections::BTreeMap::new();

    for (cat_name, cat) in &config.categories {
        if !crate::installer::is_category_supported(cat, platform, config) {
            continue;
        }

        let grp = cat
            .group
            .clone()
            .unwrap_or_else(|| "Other Categories".to_string());

        let is_pack = cat.includes.is_some();
        let display_name = if is_pack {
            format!("{cat_name} (Pack)")
        } else {
            cat_name.clone()
        };

        let is_installed = crate::installer::is_category_applied(cat_name, cat, state, platform, config);

        let packages_label = if let Some(disp) = &cat.display_packages {
            if disp.is_empty() {
                String::new()
            } else {
                format!(" ({})", crate::installer::format_items_limited(disp, 6))
            }
        } else if let Some(inc_list) = &cat.includes {
            format!(" ({})", crate::installer::format_items_limited(inc_list, 6))
        } else {
            let pkgs = match platform {
                Platform::Debian => cat.debian_packages.as_deref().unwrap_or(&[]),
                Platform::Termux => cat.termux_packages.as_deref().unwrap_or(&[]),
                Platform::Unsupported(_) => &[],
            };
            if pkgs.is_empty() {
                String::new()
            } else {
                let formatted = crate::installer::format_items_limited(pkgs, 6);
                if formatted.eq_ignore_ascii_case(cat_name) {
                    String::new()
                } else {
                    format!(" ({formatted})")
                }
            }
        };

        grouped_categories.entry(grp).or_default().push(CategorySelectOption {
            key: cat_name.clone(),
            display_name,
            description: cat.description.clone(),
            packages_label,
            is_installed,
            checked: false,
        });
    }

    let mut ordered_groups: Vec<(String, Vec<CategorySelectOption>)> = Vec::new();
    for grp_name in preferred_group_order {
        if let Some(opts) = grouped_categories.remove(grp_name)
            && !opts.is_empty()
        {
            ordered_groups.push((grp_name.to_string(), opts));
        }
    }
    for (grp_name, opts) in grouped_categories {
        if !opts.is_empty() {
            ordered_groups.push((grp_name, opts));
        }
    }

    if ordered_groups.is_empty() {
        return Ok(Some(Vec::new()));
    }

    let mut all_selected = Vec::new();
    let total_steps = ordered_groups.len();

    let _guard = RawModeGuard::new()?;

    for (step_idx, (group_name, options)) in ordered_groups.into_iter().enumerate() {
        let step_num = step_idx + 1;
        let selected_in_group = match prompt_group_selection(&group_name, options, step_num, total_steps)? {
            Some(sel) => sel,
            None => return Ok(None),
        };
        all_selected.extend(selected_in_group);
    }

    Ok(Some(all_selected))
}

fn prompt_group_selection(
    group_name: &str,
    mut options: Vec<CategorySelectOption>,
    step_num: usize,
    total_steps: usize,
) -> anyhow::Result<Option<Vec<String>>> {
    let mut stdout = io::stdout();
    let mut cursor_index = 0;
    let mut rendered_lines = 0;

    loop {
        // Clear previously rendered lines
        if rendered_lines > 0 {
            execute!(stdout, cursor::MoveToPreviousLine(rendered_lines as u16))?;
            for _ in 0..rendered_lines {
                execute!(stdout, Clear(ClearType::CurrentLine), cursor::MoveDown(1))?;
            }
            execute!(stdout, cursor::MoveToPreviousLine(rendered_lines as u16))?;
        }

        rendered_lines = 0;

        let header = format!(
            "{BOLD_CYAN}? [{step_num}/{total_steps}] Select configuration for: {BOLD}{group_name}{RESET}\r\n\r\n"
        );
        execute!(stdout, Print(&header))?;
        rendered_lines += 2;

        let (_, term_height) = terminal::size().unwrap_or((80, 24));
        let max_visible = (term_height.saturating_sub(8) as usize).max(5).min(options.len());

        // Calculate scrolling window
        let start_index = if cursor_index >= max_visible {
            cursor_index + 1 - max_visible
        } else {
            0
        };
        let end_index = (start_index + max_visible).min(options.len());

        for (idx, opt) in options[start_index..end_index].iter().enumerate() {
            let actual_idx = start_index + idx;
            let is_focused = actual_idx == cursor_index;

            let check_mark = if opt.checked {
                format!("{BOLD_GREEN}[x]{RESET}")
            } else {
                format!("{DIM_GRAY}[ ]{RESET}")
            };

            let installed_badge = if opt.is_installed {
                format!(" {WHITE}[installed]{RESET}")
            } else {
                String::new()
            };

            let line = if is_focused {
                format!(
                    "  {BOLD_GREEN}❯{RESET} {check_mark} {BOLD}{}{RESET}{installed_badge} {DIM_GRAY}- {}{}{RESET}\r\n",
                    opt.display_name, opt.description, opt.packages_label
                )
            } else {
                format!(
                    "    {check_mark} {}{installed_badge} {DIM_GRAY}- {}{}{RESET}\r\n",
                    opt.display_name, opt.description, opt.packages_label
                )
            };
            execute!(stdout, Print(&line))?;
            rendered_lines += 1;
        }

        if options.len() > max_visible {
            let scroll_info = format!(
                "  {DIM_GRAY}(Showing {}-{} of {}){RESET}\r\n",
                start_index + 1,
                end_index,
                options.len()
            );
            execute!(stdout, Print(&scroll_info))?;
            rendered_lines += 1;
        }

        let footer = format!(
            "\r\n{DIM_GRAY}  [↑/↓] Navigate  [Space] Toggle  [a] Toggle All  [Enter] Next Step  [q/Esc] Cancel{RESET}\r\n"
        );
        execute!(stdout, Print(&footer))?;
        rendered_lines += 2;

        stdout.flush()?;

        if let Event::Key(key_event) = event::read()? {
            if key_event.kind != KeyEventKind::Press {
                continue;
            }
            match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if cursor_index == 0 {
                        cursor_index = options.len() - 1;
                    } else {
                        cursor_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if cursor_index + 1 >= options.len() {
                        cursor_index = 0;
                    } else {
                        cursor_index += 1;
                    }
                }
                KeyCode::Char(' ') => {
                    options[cursor_index].checked = !options[cursor_index].checked;
                }
                KeyCode::Char('a') => {
                    let any_unchecked = options.iter().any(|o| !o.checked);
                    for o in &mut options {
                        o.checked = any_unchecked;
                    }
                }
                KeyCode::Enter => {
                    // Clear the section from the screen before proceeding to the next step
                    if rendered_lines > 0 {
                        execute!(stdout, cursor::MoveToPreviousLine(rendered_lines as u16))?;
                        for _ in 0..rendered_lines {
                            execute!(stdout, Clear(ClearType::CurrentLine), cursor::MoveDown(1))?;
                        }
                        execute!(stdout, cursor::MoveToPreviousLine(rendered_lines as u16))?;
                    }

                    let selected: Vec<String> = options
                        .into_iter()
                        .filter(|o| o.checked)
                        .map(|o| o.key)
                        .collect();
                    return Ok(Some(selected));
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    return Ok(None);
                }
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(None);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_config_preset_categories_should_contain_exact_curated_set() {
        assert_eq!(
            FULL_CONFIG_CATEGORIES,
            &[
                "nvim-onedarkpro",
                "zsh-tokyonight",
                "term-flow",
                "nodejs-pnpm",
                "agents-flow"
            ]
        );
    }
}
