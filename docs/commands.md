# CLI Command Reference & Manual — project-dots

`project-dots` (v0.1.0-beta.1) provides a simple, safety-first command-line interface to manage system packages and dotfile dependencies across **Debian** (`apt`) and **Termux** (`pkg`).

## Overview of Commands

| Command | Purpose | Options |
| ------- | ------- | ------- |
| `project-dots list` | Displays available categories and package lists in a clean format | `--debug` / `-d` for detailed system package status |
| `project-dots install [category]` | Installs packages for a specific category or all categories | `--dry-run` / `-n` to preview actions without making changes |
| `project-dots remove [category]` | Safely uninstalls packages installed by `project-dots` | `--dry-run` / `-n` to preview removal without making changes |
| `project-dots --version` | Displays the current application version (`0.1.0-beta.1`) | `-V` |
| `project-dots --help` | Displays the command-line help summary | `-h` |

---

## Command Specifications & Exact Terminal Outputs

### 1. List Categories (Default User View)
**Command**:
```bash
cargo run -- list
# or using compiled binary:
project-dots list
```
**Purpose**:
Provides a clean, human-readable summary of available categories and their associated packages formatted on a single line in dim gray.

**Exact Terminal Output**:
```text
Loaded configuration from: ./categories.toml

=== Available Categories ===

Category: lazyvim-minimal
  Description: Minimal LazyVim dependencies and editor setup
  Packages: git, curl, clang, fd-find, lazygit, neovim (custom binary)
```

---

### 2. List Categories (Detailed Debug Mode)
**Command**:
```bash
cargo run -- list --debug
# or:
project-dots list -d
```
**Purpose**:
Provides developers and advanced users with exact system package tracking information (`Pre-existing (system)`, `Installed by project-dots`, `Not installed`) and custom installer metadata.

**Exact Terminal Output**:
```text
Loaded configuration from: ./categories.toml

=== project-dots: Categories & Package Status (DEBUG MODE) ===
Platform Detected: Debian

Category: lazyvim-minimal
  Description: Minimal LazyVim dependencies and editor setup
  Packages:
    - git [Pre-existing (system)]
    - curl [Pre-existing (system)]
    - clang [Not installed]
    - fd-find [Pre-existing (system)]
    - lazygit [Installed (untracked)]
  Custom Installer (Debian):
    - neovim (https://github.com/neovim/neovim/releases/latest/download/nvim-linux-x86_64.tar.gz) [Not installed]
```

---

### 3. Install Category (Dry-Run Simulation)
**Command**:
```bash
cargo run -- install lazyvim-minimal --dry-run
# or:
project-dots install lazyvim-minimal -n
```
**Purpose**:
Previews system package manager commands (`apt install -y` / `pkg install -y`), tarball extractions, and symlink creation without modifying the operating system.

**Exact Terminal Output**:
```text
=== DRY-RUN MODE ACTIVE: No system changes will be made ===

--> Processing Category: lazyvim-minimal
  [SKIP] Package 'git' is already installed on OS.
  [SKIP] Package 'curl' is already installed on OS.
  [Dry-Run] Would execute: sudo apt install -y clang
  [SKIP] Package 'fd-find' is already installed on OS.
  [SKIP] Package 'lazygit' is already installed on OS.
  --> Custom Binary Installer: neovim
  [Dry-Run] Would download release tarball from https://github.com/neovim/neovim/releases/latest/download/nvim-linux-x86_64.tar.gz
  [Dry-Run] Would extract to /opt/nvim
  [Dry-Run] Would create symlink: /home/user/.local/bin/nvim -> /opt/nvim/bin/neovim

Installation processing completed successfully.
```

---

### 4. Install Category (Live Execution)
**Command**:
```bash
cargo run -- install lazyvim-minimal
# or:
project-dots install all
```
**Purpose**:
Executes real package manager installation commands, downloads custom binary tarballs to `/opt/nvim`, creates symlinks to `~/.local/bin/nvim`, updates state tracking (`~/.local/state/project-dots/state.toml`), and displays `$PATH` helper recommendations if `~/.local/bin` is not in `$PATH`.

---

### 5. Remove Category (Safe Removal with Dry-Run)
**Command**:
```bash
cargo run -- remove lazyvim-minimal --dry-run
```
**Purpose**:
Simulates package removal while enforcing safety rules: packages that existed on the system prior to `project-dots` (`was_preexisting = true`) are automatically protected and skipped.

**Exact Terminal Output**:
```text
=== DRY-RUN MODE ACTIVE: No system changes will be made ===

--> Processing Removal for Category: lazyvim-minimal
  [SKIP] Package 'git' was pre-existing on system before project-dots. Skipping removal.
  [SKIP] Package 'curl' was pre-existing on system before project-dots. Skipping removal.
  [SKIP] Package 'clang' was not installed by project-dots. Skipping removal.
  [SKIP] Package 'fd-find' was pre-existing on system before project-dots. Skipping removal.
  [SKIP] Package 'lazygit' was pre-existing on system before project-dots. Skipping removal.

Removal processing completed successfully.
```

---

### 6. Display Version & Help
**Command**:
```bash
project-dots --version
project-dots --help
```
**Output**:
```text
0.1.0-beta.1
```
