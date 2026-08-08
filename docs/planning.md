# Project Planning & Roadmap — project-dots

`project-dots` is a modular, category-based CLI installer designed to automate package provisioning and complementary tools for dotfiles on **Debian** and **Termux** systems.

## Core Features & Commands
- **`project-dots list`**: Displays available categories and contained packages on a single line per category.
- **`project-dots show <category>`**: Displays detailed description, package tracking status, config files, and post-install commands.
- **`project-dots install [category] [--dry-run / -n]` / `-i`**: Installs packages for a specific category (or all categories) with dry-run preview.
- **`project-dots remove <category> [--all / -a] [--dry-run / -n]`**: Safely removes packages installed by `project-dots`.
- **`project-dots self-update [--dry-run / -n]`**: Checks GitHub Releases for new versions and updates the binary in-place.
- **`project-dots self-uninstall [--yes / -y] [--dry-run / -n]`**: Safely removes binary executable and state/config directories.

## System Installation & Self-Update Architecture
1. **One-Line Bootstrap Script (`install.sh`)**:
   - POSIX shell script: `curl -sSL https://raw.githubusercontent.com/fusoras/project-dots/develop/install.sh | sh`
   - Detects architecture (Debian x86_64 vs Termux aarch64), fetches pre-compiled GitHub Release binary asset, places it in `~/.local/bin` (or `$PREFIX/bin` on Termux), and displays shell `$PATH` tips.
2. **Self-Update & Self-Uninstall Engine (`src/update.rs`)**:
   - Queries GitHub Releases API for `latest`.
   - Compares release tag with current binary crate version.
   - Replaces current executable atomically (`std::env::current_exe()`).
   - Self-uninstalls binary and prompts interactively to purge `~/.config/project-dots` and `~/.local/state/project-dots`.

## Completed & Roadmap Tasks

### Completed Core & Self-Update Tasks ✅
- [x] **Task 1: Project Setup, SemVer Packaging, Documentation & Git Strategy**
- [x] **Task 2: Dependencies & Default Configuration (`lazyvim-minimal`)**
- [x] **Task 3: Platform Detection & Pre-flight Checks (`src/platform.rs`)**
- [x] **Task 4: Configuration & Atomic State Engine (`src/config.rs`, `src/state.rs`)**
- [x] **Task 5: Core Installer Engine with `--dry-run` (`src/installer.rs`)**
- [x] **Task 6: Custom Binary Handler & `$PATH` Helper**
- [x] **Task 7: CLI Interface & Verification (`src/main.rs`)**
- [x] **Task 8: Unit Test Suite & TDD Verification (`cargo test -- --nocapture`)**
- [x] **Task 9: Self-Update Engine (`src/update.rs`)**
- [x] **Task 10: CLI `self-update` Subcommand Integration (`src/main.rs`)**
- [x] **Task 11: Bootstrap Installation Script (`install.sh`)**
- [x] **Task 12: GitHub Actions Release CI Pipeline (`.github/workflows/release.yml`)**
- [x] **Task 13: Self-Uninstall Engine & `--yes` Interactive Confirmation (`src/update.rs`)**
- [x] **Task 14: Category Config Copy (`copy_files`) & Post-Install Commands Engine (`src/installer.rs`)**
- [x] **Task 15: `shell-tokyonight` Category & Embedded Assets (`starship.toml`, `font.ttf`)**
