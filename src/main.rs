mod colors;
mod config;
mod installer;
mod platform;
mod state;
mod update;

use crate::colors::*;
use clap::{CommandFactory, Parser, Subcommand};
use config::Config;
use installer::{install_category, list_categories, remove_category, show_category};
use platform::Platform;
use state::State;
use update::check_and_perform_update;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const DIM_GRAY: &str = "\x1b[90m";

#[derive(Parser)]
#[command(
    name = "project-dots",
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
    /// Lists all categories and contained packages on a single line
    List,

    /// Shows detailed description, packages, config files, and installation status for a specific category
    Show {
        /// Category name or alias to inspect (e.g. 'shell-tokyonight', 'shell-tn', 'lazyvim-minimal')
        category: String,
    },

    /// Adds packages and configurations for a specific category or all categories
    Add {
        /// Category name to add (e.g. 'shell-tokyonight', 'lazyvim-minimal'), or 'all'
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

        /// Preview uninstallation actions without deleting files
        #[arg(short = 'n', long = "dry-run")]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("{}", VERSION);
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
        println!("{BOLD_YELLOW}[SECURITY WARNING] Loaded external configuration file: {config_source}{RESET}");
        println!("{BOLD_YELLOW}[SECURITY WARNING] Verify contents before running custom installers or post-install commands.{RESET}\n");
    }

    let mut state = State::load();
    let platform = Platform::detect();

    match command {
        Commands::List => {
            list_categories(&config, &state, &platform);
        }
        Commands::Show { category } => {
            if let Err(e) = show_category(&category, &config, &state, &platform) {
                eprintln!("{BOLD_RED}Error:{RESET} {:?}", e);
                std::process::exit(1);
            }
        }
        Commands::Add { category, trailing_categories, dry_run } => {
            if !trailing_categories.is_empty() {
                eprintln!(
                    "{BOLD_RED}Error:{RESET} Cannot add multiple categories at the same time ('{}' and '{}'). Please run 'project-dots add <category>' for one category at a time, or use 'project-dots add all'.",
                    category,
                    trailing_categories.join(" ")
                );
                std::process::exit(1);
            }
            if dry_run {
                println!("{BOLD_YELLOW}=== DRY-RUN MODE ACTIVE: No system changes will be made ==={RESET}");
            }
            let cat_arg = if category == "all" { None } else { Some(category.as_str()) };
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
        Commands::Remove { category, all, dry_run } => {
            let cat_target = if all || category.as_deref() == Some("all") {
                println!("{BOLD_YELLOW}[WARNING] Removing ALL installed categories and packages managed by project-dots!{RESET}");
                None
            } else if let Some(ref cat) = category {
                Some(cat.as_str())
            } else {
                eprintln!(
                    "{BOLD_RED}Error:{RESET} Please specify a category to remove (e.g. 'project-dots remove <category>') or use '--all' / 'all' to remove all categories."
                );
                std::process::exit(1);
            };

            if dry_run {
                println!("{BOLD_YELLOW}=== DRY-RUN MODE ACTIVE: No system changes will be made ==={RESET}");
            }
            if let Err(e) = remove_category(cat_target, &config, &mut state, &platform, dry_run) {
                eprintln!("\n{BOLD_RED}Removal error:{RESET} {:?}", e);
                std::process::exit(1);
            }
            println!("\n{BOLD_GREEN}Removal processing completed successfully.{RESET}");
        }
        Commands::SelfUpdate { dry_run } => {
            if dry_run {
                println!("{BOLD_YELLOW}=== DRY-RUN MODE ACTIVE: No binary changes will be made ==={RESET}");
            }
            if let Err(e) = check_and_perform_update(VERSION, &platform, dry_run) {
                eprintln!("\n{BOLD_RED}Self-update error:{RESET} {:?}", e);
                std::process::exit(1);
            }
            println!("\n{BOLD_GREEN}Self-update processing completed successfully.{RESET}");
        }
        Commands::SelfUninstall { yes, dry_run } => {
            if dry_run {
                println!("{BOLD_YELLOW}=== DRY-RUN MODE ACTIVE: No files will be deleted ==={RESET}");
            }
            if let Err(e) = update::perform_self_uninstall(dry_run, yes) {
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
    fn add_subcommand_should_parse_as_add_command() {
        let cli = Cli::try_parse_from(["project-dots", "add", "shell-tokyonight"]).expect("failed to parse add");
        assert!(matches!(cli.command, Some(Commands::Add { .. })));
    }

    #[test]
    fn add_subcommand_with_multiple_categories_should_capture_trailing() {
        let cli = Cli::try_parse_from(["project-dots", "add", "shell-tokyonight", "lazyvim-minimal"]).expect("failed to parse multiple add");
        if let Some(Commands::Add { category, trailing_categories, .. }) = cli.command {
            assert_eq!(category, "shell-tokyonight");
            assert_eq!(trailing_categories, vec!["lazyvim-minimal".to_string()]);
        } else {
            panic!("Expected Commands::Add");
        }
    }

    #[test]
    fn install_subcommand_should_parse_as_deprecated_hint() {
        let cli = Cli::try_parse_from(["project-dots", "install", "shell-tokyonight"]).expect("failed to parse install");
        assert!(matches!(cli.command, Some(Commands::Install { .. })));
    }
}


