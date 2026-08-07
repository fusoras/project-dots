# Project Planning — project-dots

`project-dots` (temporary name, v0.1.0-beta.1) is a modular, category-based CLI installer designed to automate package provisioning and complementary tools for dotfiles on **Debian** and **Termux** systems.

## Core Features & Commands
- **`project-dots list`**: Displays available categories, descriptions, package mappings, and installation status.
- **`project-dots install [category] [--dry-run]`**: Installs packages for a specific category (or all categories) with optional dry-run preview.
- **`project-dots remove [category] [--dry-run]`**: Safely removes packages installed by `project-dots`.

## Preventive & Safety-First Architecture
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

## Package Installation Modes
1. **Standard Package Manager**:
   - **Debian**: `sudo apt install -y <packages>` / `sudo apt remove -y <packages>`
   - **Termux**: `pkg install -y <packages>` / `pkg remove -y <packages>`
2. **Custom Binary Downloads (Special Cases)**:
   - **Neovim on Debian**: Downloads latest release tarball from GitHub (`https://github.com/neovim/neovim/releases/latest/download/nvim-linux-x86_64.tar.gz`), extracts to `/opt/nvim`, and symlinks to `~/.local/bin/nvim`.
   - **Neovim on Termux**: Installs directly via `pkg install neovim`.

## Development Roadmap (TODO Checklist)

### Task 1: Project Setup, SemVer, Docs & Git Branch (`develop`)
- [x] Configured `project-dots` v0.1.0-beta.1, created English `docs/`, registered Git rules in `AGENTS.md` and Engram, switched to `develop`.

### Task 2: Dependencies & Default Configuration
- [ ] Add `clap`, `serde`, and `toml` to `Cargo.toml`.
- [ ] Create default `categories.toml`.

### Task 3: Platform Detection & Pre-flight Checks
- [ ] Implement `src/platform.rs` (`command_exists`, `is_package_installed`, `check_apt_lock`).

### Task 4: Configuration & Atomic State Engine
- [ ] Implement `src/config.rs` (User XDG config -> Embedded fallback).
- [ ] Implement `src/state.rs` (`state.toml.tmp` -> `state.toml`).

### Task 5: Core Installer Engine with `--dry-run`
- [ ] Implement `src/installer.rs` with `apt`/`pkg` abstractions, `list`, safe `install`, and safe `remove`.

### Task 6: Custom Binary Handler & `$PATH` Helper
- [ ] Implement Neovim tarball downloader/extractor to `/opt/nvim` and `$PATH` helper.

### Task 7: CLI Interface & Verification
- [ ] Wire up `clap` CLI in `src/main.rs`.
- [ ] Run full automated and manual testing.
