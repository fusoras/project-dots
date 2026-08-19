mod colors;
mod config;
mod installer;
mod platform;
mod prompt;
mod state;
mod update;

use crate::colors::*;
use clap::{CommandFactory, Parser, Subcommand};
use config::Config;
use installer::{install_category, list_categories, remove_category, run_interactive_setup, show_category};
use platform::Platform;
use state::State;
use update::check_and_perform_update;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const DIM_GRAY: &str = "\x1b[90m";

#[derive(Parser)]
#[command(
    name = "dotss",
    author = "user",
    about = "Companion tool for system provisioning and dotfiles package management by categories on Debian and Termux",
    disable_version_flag = true
)]
struct Cli {
    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive setup wizard to select and install configuration presets or custom categories
    Setup {
        /// Preview actions without running system package commands
        #[arg(short = 'n', long = "dry-run")]
        dry_run: bool,
    },

    /// Lists categories and contained packages for current platform
    #[command(disable_help_flag = true)]
    List {
        /// Show categories not supported by current platform (e.g. i3wm on Termux)
        #[arg(short = 's', visible_short_alias = 'h', long = "show-hidden", action = clap::ArgAction::SetTrue, overrides_with = "show_hidden")]
        show_hidden: bool,

        /// Group categories by functional domain (e.g. AI & Agents, Shell & Terminal)
        #[arg(long = "categories", visible_alias = "ca")]
        group_by_category: bool,

        /// Print help information
        #[arg(long = "help")]
        help: bool,
    },

    /// Shows detailed description, packages, config files, and installation status for a specific category
    Show {
        /// Category name or alias to inspect (e.g. 'zsh-tokyonight', 'zsh-tn', 'lazyvim-minimal')
        category: String,
    },

    /// Searches categories by name, alias, or contained packages for the current platform
    Search {
        /// Query to match against category names, aliases, or package names (e.g. 'nvim', 'zsh-tn')
        query: String,
    },

    /// Adds packages and configurations for a specific category or all categories
    Add {
        /// Category name to add (e.g. 'zsh-tokyonight', 'lazyvim-minimal'), or 'all'
        #[arg(default_value = "all")]
        category: String,

        /// Additional categories specified after the primary category argument
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        trailing_categories: Vec<String>,

        /// Preview actions without running system package commands
        #[arg(short = 'n', long = "dry-run")]
        dry_run: bool,
    },

    /// Deprecated subcommand error hint
    #[command(hide = true)]
    Install {
        /// Catch trailing arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        _args: Vec<String>,
    },

    /// Safely removes packages installed by project-dots
    Remove {
        /// Category name to remove (e.g. 'shell', 'editors', 'cli-tools'), or 'all'
        category: Option<String>,

        /// Remove all installed categories
        #[arg(short = 'a', long = "all")]
        all: bool,

        /// Preview removal actions without running system package commands
        #[arg(short = 'n', long = "dry-run")]
        dry_run: bool,
    },

    /// Checks GitHub Releases and updates project-dots binary in-place
    SelfUpdate {
        /// Preview update check without replacing binary
        #[arg(short = 'n', long = "dry-run")]
        dry_run: bool,
    },

    /// Uninstalls project-dots executable and state/config directories from system
    SelfUninstall {
        /// Automatically confirm deletion of configuration files and package state registry
        #[arg(short = 'y', long)]
        yes: bool,

        /// Explicitly reject/skip deletion of configuration files and package state registry
        #[arg(short = 'n', long = "no")]
        no: bool,

        /// Preview uninstallation actions without deleting files
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,
    },
}

fn main() {
    let mut raw_args: Vec<String> = std::env::args().collect();
    if raw_args.iter().any(|arg| arg == "-s" || arg == "--sh") {
        eprintln!(
            "{BOLD_RED}error:{RESET} unexpected argument found for 'list'. Use '-sh' or '--show-hidden'."
        );
        std::process::exit(1);
    }

    for arg in &mut raw_args {
        if arg == "-ca" {
            *arg = "--categories".to_string();
        }
    }

    let cli = Cli::parse_from(raw_args);

    if cli.version {
        let current_version_tag = if VERSION.starts_with('v') {
            VERSION.to_string()
        } else {
            format!("v{VERSION}")
        };

        if let Some(latest) = crate::update::check_version_update(VERSION) {
            let latest_tag = if latest.starts_with('v') {
                latest
            } else {
                format!("v{latest}")
            };
            println!("{current_version_tag} -> {BOLD_YELLOW}Update: {latest_tag}{RESET}");
            println!("    Run 'dotss self-update' to update.");
        } else {
            println!("{current_version_tag}");
        }

        // Spawn non-blocking detached background check so state.toml is refreshed asynchronously
        crate::update::spawn_background_version_check(VERSION);
        return;
    }

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            let _ = Cli::command().print_help();
            return;
        }
    };

    let (config, config_source) = match Config::load() {
        Ok((cfg, src)) => (cfg, src),
        Err(e) => {
            eprintln!("{BOLD_RED}Error loading configuration:{RESET} {}", e);
            std::process::exit(1);
        }
    };

    if config_source != "Embedded default configuration" {
        println!(
            "{BOLD_YELLOW}[SECURITY WARNING] Loaded external configuration file: {config_source}{RESET}"
        );
        println!(
            "{BOLD_YELLOW}[SECURITY WARNING] Verify contents before running custom installers or post-install commands.{RESET}\n"
        );
    }

    let mut state = State::load();
    let platform = Platform::detect();

    match command {
        Commands::Setup { dry_run } => {
            if let Err(e) = run_interactive_setup(&config, &mut state, &platform, dry_run) {
                eprintln!("\n{BOLD_RED}Setup error:{RESET} {:?}", e);
                std::process::exit(1);
            }
        }
        Commands::List {
            show_hidden,
            group_by_category,
            help,
        } => {
            if help {
                use clap::CommandFactory;
                let mut cmd = Cli::command();
                if let Some(sub) = cmd.find_subcommand_mut("list") {
                    sub.print_help().ok();
                    println!();
                }
                return;
            }
            list_categories(
                &config,
                &state,
                &platform,
                show_hidden,
                None,
                group_by_category,
            );
        }
        Commands::Search { query } => {
            list_categories(&config, &state, &platform, true, Some(&query), false);
        }
        Commands::Show { category } => {
            if let Err(e) = show_category(&category, &config, &state, &platform) {
                eprintln!("{BOLD_RED}Error:{RESET} {:?}", e);
                std::process::exit(1);
            }
        }
        Commands::Add {
            category,
            trailing_categories,
            dry_run,
        } => {
            if !trailing_categories.is_empty() {
                eprintln!(
                    "{BOLD_RED}Error:{RESET} Cannot add multiple categories at the same time ('{}' and '{}'). Please run 'dotss add <category>' for one category at a time, or use 'dotss add all'.",
                    category,
                    trailing_categories.join(" ")
                );
                std::process::exit(1);
            }
            if dry_run {
                println!(
                    "{BOLD_YELLOW}=== DRY-RUN MODE ACTIVE: No system changes will be made ==={RESET}"
                );
            }
            let cat_arg = if category == "all" {
                None
            } else {
                Some(category.as_str())
            };
            if let Err(e) = install_category(cat_arg, &config, &mut state, &platform, dry_run) {
                eprintln!("\n{BOLD_RED}Addition error:{RESET} {:?}", e);
                std::process::exit(1);
            }
            println!("\n{BOLD_GREEN}Addition processing completed successfully.{RESET}");
        }
        Commands::Install { .. } => {
            eprintln!("{BOLD_RED}error:{RESET} unrecognized subcommand 'install'\n");
            eprintln!("  {BOLD_GREEN}tip:{RESET} a similar subcommand exists: 'add'");
            eprintln!("  {DIM_GRAY}tip: a similar subcommand exists: 'list'{RESET}\n");
            std::process::exit(1);
        }
        Commands::Remove {
            category,
            all,
            dry_run,
        } => {
            let cat_target = if all || category.as_deref() == Some("all") {
                println!(
                    "{BOLD_YELLOW}[WARNING] Removing ALL installed categories and packages managed by dotss!{RESET}"
                );
                None
            } else if let Some(ref cat) = category {
                Some(cat.as_str())
            } else {
                eprintln!(
                    "{BOLD_RED}Error:{RESET} Please specify a category to remove (e.g. 'dotss remove <category>') or use '--all' / 'all' to remove all categories."
                );
                std::process::exit(1);
            };

            if dry_run {
                println!(
                    "{BOLD_YELLOW}=== DRY-RUN MODE ACTIVE: No system changes will be made ==={RESET}"
                );
            }
            if let Err(e) = remove_category(cat_target, &config, &mut state, &platform, dry_run) {
                eprintln!("\n{BOLD_RED}Removal error:{RESET} {:?}", e);
                std::process::exit(1);
            }
            println!("\n{BOLD_GREEN}Removal processing completed successfully.{RESET}");
        }
        Commands::SelfUpdate { dry_run } => {
            if dry_run {
                println!(
                    "{BOLD_YELLOW}=== DRY-RUN MODE ACTIVE: No binary changes will be made ==={RESET}"
                );
            }
            if let Err(e) = check_and_perform_update(VERSION, &platform, dry_run) {
                eprintln!("\n{BOLD_RED}Self-update error:{RESET} {:?}", e);
                std::process::exit(1);
            }
            println!("\n{BOLD_GREEN}Self-update processing completed successfully.{RESET}");
        }
        Commands::SelfUninstall { yes, no, dry_run } => {
            if dry_run {
                println!(
                    "{BOLD_YELLOW}=== DRY-RUN MODE ACTIVE: No files will be deleted ==={RESET}"
                );
            }
            if let Err(e) =
                update::perform_self_uninstall(&config, &mut state, &platform, dry_run, yes, no)
            {
                eprintln!("\n{BOLD_RED}Self-uninstall error:{RESET} {:?}", e);
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_subcommand_should_parse_show_hidden_flag() {
        let cli = Cli::try_parse_from(["dotss", "list", "--show-hidden"])
            .expect("failed to parse list --show-hidden");
        if let Some(Commands::List { show_hidden, .. }) = cli.command {
            assert!(show_hidden);
        } else {
            panic!("Expected Commands::List");
        }

        let cli_short_alias =
            Cli::try_parse_from(["dotss", "list", "-sh"]).expect("failed to parse list -sh");
        if let Some(Commands::List { show_hidden, .. }) = cli_short_alias.command {
            assert!(show_hidden);
        } else {
            panic!("Expected Commands::List");
        }

        // --sh should fail parsing (not a valid long flag alias)
        assert!(Cli::try_parse_from(["dotss", "list", "--sh"]).is_err());
    }

    #[test]
    fn search_subcommand_should_parse_query() {
        let cli =
            Cli::try_parse_from(["dotss", "search", "nvim"]).expect("failed to parse search nvim");
        if let Some(Commands::Search { query }) = cli.command {
            assert_eq!(query, "nvim");
        } else {
            panic!("Expected Commands::Search");
        }
    }

    #[test]
    fn add_subcommand_should_parse_as_add_command() {
        let cli =
            Cli::try_parse_from(["dotss", "add", "zsh-tokyonight"]).expect("failed to parse add");
        assert!(matches!(cli.command, Some(Commands::Add { .. })));
    }

    #[test]
    fn add_subcommand_with_multiple_categories_should_capture_trailing() {
        let cli = Cli::try_parse_from(["dotss", "add", "zsh-tokyonight", "lazyvim-minimal"])
            .expect("failed to parse multiple add");
        if let Some(Commands::Add {
            category,
            trailing_categories,
            ..
        }) = cli.command
        {
            assert_eq!(category, "zsh-tokyonight");
            assert_eq!(trailing_categories, vec!["lazyvim-minimal".to_string()]);
        } else {
            panic!("Expected Commands::Add");
        }
    }

    #[test]
    fn install_subcommand_should_parse_as_deprecated_hint() {
        let cli = Cli::try_parse_from(["dotss", "install", "zsh-tokyonight"])
            .expect("failed to parse install");
        assert!(matches!(cli.command, Some(Commands::Install { .. })));
    }

    #[test]
    fn self_uninstall_subcommand_flags_parsing() {
        let cli = Cli::try_parse_from(["dotss", "self-uninstall", "-n", "-d"])
            .expect("failed to parse self-uninstall");
        if let Some(Commands::SelfUninstall { no, dry_run, yes }) = cli.command {
            assert!(no);
            assert!(dry_run);
            assert!(!yes);
        } else {
            panic!("Expected Commands::SelfUninstall");
        }
    }

    #[test]
    fn setup_subcommand_flags_parsing() {
        let cli = Cli::try_parse_from(["dotss", "setup"]).expect("failed to parse setup");
        if let Some(Commands::Setup { dry_run }) = cli.command {
            assert!(!dry_run);
        } else {
            panic!("Expected Commands::Setup");
        }

        let cli_dry = Cli::try_parse_from(["dotss", "setup", "-n"]).expect("failed to parse setup -n");
        if let Some(Commands::Setup { dry_run }) = cli_dry.command {
            assert!(dry_run);
        } else {
            panic!("Expected Commands::Setup");
        }
    }
}
