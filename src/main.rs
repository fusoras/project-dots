mod config;
mod installer;
mod platform;
mod state;

use clap::{Parser, Subcommand};
use config::Config;
use installer::{install_category, list_categories, remove_category};
use platform::Platform;
use state::State;

#[derive(Parser)]
#[command(
    name = "project-dots",
    author = "user",
    version = "0.1.0-beta.1",
    about = "Companion tool for system provisioning and dotfiles package management by categories on Debian and Termux"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lists all categories, descriptions, packages, and their current installation status
    List,

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

fn main() {
    let cli = Cli::parse();

    let (config, config_source) = match Config::load() {
        Ok((cfg, src)) => (cfg, src),
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            std::process::exit(1);
        }
    };

    let mut state = State::load();
    let platform = Platform::detect();

    match cli.command {
        Commands::List => {
            println!("Loaded configuration from: {}", config_source);
            list_categories(&config, &state, &platform);
        }
        Commands::Install { category, dry_run } => {
            if dry_run {
                println!("=== DRY-RUN MODE ACTIVE: No system changes will be made ===");
            }
            let cat_arg = if category == "all" { None } else { Some(category.as_str()) };
            if let Err(e) = install_category(cat_arg, &config, &mut state, &platform, dry_run) {
                eprintln!("\nInstallation error: {}", e);
                std::process::exit(1);
            }
            println!("\nInstallation processing completed successfully.");
        }
        Commands::Remove { category, dry_run } => {
            if dry_run {
                println!("=== DRY-RUN MODE ACTIVE: No system changes will be made ===");
            }
            let cat_arg = if category == "all" { None } else { Some(category.as_str()) };
            if let Err(e) = remove_category(cat_arg, &config, &mut state, &platform, dry_run) {
                eprintln!("\nRemoval error: {}", e);
                std::process::exit(1);
            }
            println!("\nRemoval processing completed successfully.");
        }
    }
}
