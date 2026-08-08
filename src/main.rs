mod config;
mod installer;
mod platform;
mod state;
mod update;

use clap::{CommandFactory, Parser, Subcommand};
use config::Config;
use installer::{install_category, list_categories, remove_category};
use platform::Platform;
use state::State;
use update::check_and_perform_update;

const VERSION: &str = "0.1.0-beta.3";

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
    /// Lists all categories and packages (use --debug for detailed package status)
    List {
        /// Show detailed package status and debug information
        #[arg(short = 'd', long = "debug")]
        debug: bool,
    },

    /// Installs packages for a specific category or all categories
    #[command(visible_alias = "i", short_flag = 'i', alias = "-i")]
    Install {
        /// Category name to install (e.g. 'shell', 'editors', 'cli-tools'), or 'all'
        #[arg(default_value = "all")]
        category: String,

        /// Preview actions without running system package commands
        #[arg(short = 'n', long = "dry-run")]
        dry_run: bool,
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

const RESET: &str = "\x1b[0m";
const BOLD_GREEN: &str = "\x1b[1;32m";
const BOLD_RED: &str = "\x1b[1;31m";
const BOLD_YELLOW: &str = "\x1b[1;33m";

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

    let mut state = State::load();
    let platform = Platform::detect();

    match command {
        Commands::List { debug } => {
            println!("Loaded configuration from: {}", config_source);
            list_categories(&config, &state, &platform, debug);
        }
        Commands::Install { category, dry_run } => {
            if dry_run {
                println!("{BOLD_YELLOW}=== DRY-RUN MODE ACTIVE: No system changes will be made ==={RESET}");
            }
            let cat_arg = if category == "all" { None } else { Some(category.as_str()) };
            if let Err(e) = install_category(cat_arg, &config, &mut state, &platform, dry_run) {
                eprintln!("\n{BOLD_RED}Installation error:{RESET} {}", e);
                std::process::exit(1);
            }
            println!("\n{BOLD_GREEN}Installation processing completed successfully.{RESET}");
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
                eprintln!("\n{BOLD_RED}Removal error:{RESET} {}", e);
                std::process::exit(1);
            }
            println!("\n{BOLD_GREEN}Removal processing completed successfully.{RESET}");
        }
        Commands::SelfUpdate { dry_run } => {
            if dry_run {
                println!("{BOLD_YELLOW}=== DRY-RUN MODE ACTIVE: No binary changes will be made ==={RESET}");
            }
            if let Err(e) = check_and_perform_update(VERSION, &platform, dry_run) {
                eprintln!("\n{BOLD_RED}Self-update error:{RESET} {}", e);
                std::process::exit(1);
            }
            println!("\n{BOLD_GREEN}Self-update processing completed successfully.{RESET}");
        }
        Commands::SelfUninstall { yes, dry_run } => {
            if dry_run {
                println!("{BOLD_YELLOW}=== DRY-RUN MODE ACTIVE: No files will be deleted ==={RESET}");
            }
            if let Err(e) = update::perform_self_uninstall(dry_run, yes) {
                eprintln!("\n{BOLD_RED}Self-uninstall error:{RESET} {}", e);
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_subcommand_alias() {
        let cli_dash = Cli::try_parse_from(["project-dots", "-i"]).expect("failed to parse -i");
        assert!(matches!(cli_dash.command, Some(Commands::Install { .. })));

        let cli_alias = Cli::try_parse_from(["project-dots", "i"]).expect("failed to parse i");
        assert!(matches!(cli_alias.command, Some(Commands::Install { .. })));
    }
}

