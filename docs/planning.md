# Project Planning & Status — project-dots

`project-dots` (v0.1.0-beta.1) is a modular, category-based CLI installer designed to automate package provisioning and complementary tools for dotfiles on **Debian** and **Termux** systems.

## Project Status: All Tasks Completed ✅

### Core Features & Commands
- **`project-dots list [--debug / -d]`**: Displays available categories, descriptions, and package listings (clean comma-separated view for users, `--debug` for detailed system package status).
- **`project-dots install [category] [--dry-run / -n]`**: Installs packages for a specific category (or all categories) with `--dry-run` simulation support.
- **`project-dots remove [category] [--dry-run / -n]`**: Safely removes packages installed by `project-dots`.

### Preventive & Safety Architecture
1. **Dual Configuration Resolution**:
   - Primary: User config at `~/.config/project-dots/categories.toml` or `./categories.toml`.
   - Fallback: Embedded default `categories.toml` built directly into the Rust binary (`include_str!`).
2. **Pre-flight System Verification**:
   - Verifies system binaries (`curl`, `tar`) before executing custom binary handlers.
   - Detects `apt` lock files on Debian to prevent execution during background updates.
3. **Atomic Safe State Tracking**:
   - `~/.local/state/project-dots/state.toml` records package pre-existence.
   - Atomically written (`.tmp` file rename) to prevent state file corruption.
4. **Smart `$PATH` Helper**:
   - Checks if `~/.local/bin` is in `$PATH` and provides shell configuration export recommendations (`~/.bashrc`, `~/.zshrc`).
5. **English TDD Unit Test Suite**:
   - Run via `cargo test -- --nocapture` to view human-readable test explanations.

## Completed Tasks Checklist

- [x] **Task 1: Project Setup, SemVer (`v0.1.0-beta.1`), Documentation & Git Strategy**
- [x] **Task 2: Dependencies & Default Configuration (`lazyvim-minimal`)**
- [x] **Task 3: Platform Detection & Pre-flight Checks (`src/platform.rs`)**
- [x] **Task 4: Configuration & Atomic State Engine (`src/config.rs`, `src/state.rs`)**
- [x] **Task 5: Core Installer Engine with `--dry-run` (`src/installer.rs`)**
- [x] **Task 6: Custom Binary Handler & `$PATH` Helper**
- [x] **Task 7: CLI Interface & Verification (`src/main.rs`)**
- [x] **Task 8: Unit Test Suite & TDD Verification (`cargo test -- --nocapture`)**
