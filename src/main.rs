mod config;
mod installer;
mod platform;
mod state;

use clap::{CommandFactory, Parser, Subcommand};
use config::Config;
use installer::{install_category, list_categories, remove_category};
use platform::Platform;
use state::State;

const VERSION: &str = "0.1.0-beta.1";

#[derive(Parser)]
#[command(
    name = "project-dots",
    author = "user",
    about = "Companion tool for system provisioning and dotfiles package management by categories on Debian and Termux",
    disable_version_flag = true
)]
struct Cli {
    /// Print version
    #[arg(short = 'V', long = "version", action = clap::ArgAction::SetTrue)]
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
        #[arg(default_value = "all")]
        category: String,

        /// Preview removal actions without running system package commands
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
        Commands::Remove { category, dry_run } => {
            if dry_run {
                println!("{BOLD_YELLOW}=== DRY-RUN MODE ACTIVE: No system changes will be made ==={RESET}");
            }
            let cat_arg = if category == "all" { None } else { Some(category.as_str()) };
            if let Err(e) = remove_category(cat_arg, &config, &mut state, &platform, dry_run) {
                eprintln!("\n{BOLD_RED}Removal error:{RESET} {}", e);
                std::process::exit(1);
            }
            println!("\n{BOLD_GREEN}Removal processing completed successfully.{RESET}");
        }
    }
}
