# Project Planning — project-dots

`project-dots` (temporary name, v0.1.0-beta.1) is a modular, category-based CLI installer designed to automate package provisioning and complementary tools for dotfiles on **Debian** and **Termux** systems.

## User-Facing CLI Commands
End users interact directly with the compiled `project-dots` binary:

- **`project-dots list`**: Displays available categories, descriptions, package mappings, and installation status.
- **`project-dots install [category] [--dry-run]`**: Installs packages for a specific category (or all categories) with optional dry-run preview.
- **`project-dots remove [category] [--dry-run]`**: Safely removes packages installed by `project-dots`.

## Binary Building & Installation

- **Build Release Binary**: `cargo build --release` (generates `./target/release/project-dots`)
- **Install to System PATH**: `cargo install --path .` (installs `project-dots` executable)

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

## Active Production Categories
- **`lazyvim-minimal`**: Minimal LazyVim setup (`neovim`, `git`, `curl`, `clang`, `fd`, `lazygit`).
