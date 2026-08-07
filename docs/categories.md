# Category Structure & Configuration

`categories.toml` is the central **declarative catalog** of `project-dots`. It defines the categories of applications to install, their human-readable descriptions, platform-specific package names (Debian vs Termux), and custom installation handlers.

## Purpose & Core Benefits of `categories.toml`

1. **Category Organization**: Groups system tools by logical purpose (e.g. `shell`, `cli-tools`, `editors`, `dev-tools`) instead of installing unorganized loose packages.
2. **Platform Translation**: Bridges naming differences between OS package managers (e.g. `fd-find` on Debian vs `fd` on Termux).
3. **Custom Installation Definitions**: Defines non-standard installations (e.g. Neovim binary tarballs from GitHub for Debian) alongside standard package manager entries.
4. **Developer Default + User Customization**:
   - **For Developers**: Embedded directly into the Rust binary via `include_str!()` so the CLI works out-of-the-box with zero configuration required.
   - **For Users / Dotfile Repositories**: Users can place a custom `categories.toml` at `~/.config/project-dots/categories.toml` or in the local working directory to add or override categories without modifying Rust code.

## Configuration File Resolution Order

1. `./categories.toml` (Current working directory)
2. `~/.config/project-dots/categories.toml` (XDG User Config)
3. Embedded default in binary (`include_str!("../categories.toml")`)

## State Management (`~/.local/state/project-dots/state.toml`)

```toml
[packages.git]
category = "shell"
was_preexisting = true
installed_by_dots = false
installed_at = "2026-08-07T17:48:00Z"

[packages.ripgrep]
category = "cli-tools"
was_preexisting = false
installed_by_dots = true
installed_at = "2026-08-07T17:48:05Z"
```

## Category Schema (`categories.toml`)

```toml
[categories.editors]
description = "Text editors and terminal multiplexers"
debian_packages = ["tmux"]
termux_packages = ["neovim", "tmux"]

[categories.editors.custom.debian]
name = "neovim"
type = "github_release_tarball"
url = "https://github.com/neovim/neovim/releases/latest/download/nvim-linux-x86_64.tar.gz"
extract_dir = "/opt/nvim"
bin_symlink = "~/.local/bin/nvim"
```

## Planned Default Categories

### 1. `shell`
- **Description**: Primary shell tools and prompt setup.
- **Debian**: `zsh`, `curl`, `git`
- **Termux**: `zsh`, `curl`, `git`

### 2. `cli-tools`
- **Description**: Modern command-line utilities.
- **Debian**: `ripgrep`, `fd-find`, `fzf`, `bat`
- **Termux**: `ripgrep`, `fd`, `fzf`, `bat`

### 3. `editors`
- **Description**: Neovim and Tmux.
- **Debian**: Neovim custom binary tarball (`/opt/nvim` + `~/.local/bin/nvim`), `tmux` via `apt`.
- **Termux**: `neovim`, `tmux` via `pkg`.

### 4. `dev-tools`
- **Description**: Compilations and basic development runtimes.
- **Debian**: `build-essential`, `git`, `curl`
- **Termux**: `build-essential`, `git`, `curl`
