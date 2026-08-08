# Project Planning & Roadmap — project-dots

`project-dots` (v0.1.0-beta.1) is a modular, category-based CLI installer designed to automate package provisioning and complementary tools for dotfiles on **Debian** and **Termux** systems.

## Core Features & Commands
- **`project-dots list [--debug / -d]`**: Displays available categories, descriptions, and package listings.
- **`project-dots install [category] [--dry-run / -n]`**: Installs packages for a specific category (or all categories) with dry-run preview.
- **`project-dots remove [category] [--dry-run / -n]`**: Safely removes packages installed by `project-dots`.
- **`project-dots self-update [--dry-run / -n]`**: Checks GitHub Releases for new versions and updates the binary in-place.

## System Installation & Self-Update Architecture
1. **One-Line Bootstrap Script (`install.sh`)**:
   - POSIX shell script: `curl -sSL https://raw.githubusercontent.com/<user>/project-dots/main/install.sh | sh`
   - Detects architecture (Debian x86_64 vs Termux aarch64), fetches pre-compiled GitHub Release binary asset, places it in `~/.local/bin/project-dots`, and checks `$PATH`.
2. **Self-Update Engine (`src/update.rs`)**:
   - Queries GitHub Releases API for `latest`.
   - Compares release tag with current crate version (`0.1.0-beta.1`).
   - Replaces current executable atomically (`std::env::current_exe()`).

## Completed & Upcoming Tasks Roadmap

### Completed Tasks ✅
- [x] **Task 1: Project Setup, SemVer (`v0.1.0-beta.1`), Documentation & Git Strategy**
- [x] **Task 2: Dependencies & Default Configuration (`lazyvim-minimal`)**
- [x] **Task 3: Platform Detection & Pre-flight Checks (`src/platform.rs`)**
- [x] **Task 4: Configuration & Atomic State Engine (`src/config.rs`, `src/state.rs`)**
- [x] **Task 5: Core Installer Engine with `--dry-run` (`src/installer.rs`)**
- [x] **Task 6: Custom Binary Handler & `$PATH` Helper**
- [x] **Task 7: CLI Interface & Verification (`src/main.rs`)**
- [x] **Task 8: Unit Test Suite & TDD Verification (`cargo test -- --nocapture`)**

### Upcoming Self-Update Tasks 🚀
- [ ] **Task 9: Self-Update Engine (`src/update.rs`)**
- [ ] **Task 10: CLI `self-update` Subcommand Integration (`src/main.rs`)**
- [ ] **Task 11: Bootstrap Installation Script (`install.sh`)**
- [ ] **Task 12: GitHub Actions Release CI Pipeline (`.github/workflows/release.yml`)**
