# Category Structure & Configuration

`categories.toml` is the central **declarative catalog** of `project-dots`. It defines the categories of applications to install, their human-readable descriptions, platform-specific package names (Debian vs Termux), and custom installation handlers.

## Configuration File Resolution Order

1. `./categories.toml` (Current working directory)
2. `~/.config/project-dots/categories.toml` (XDG User Config)
3. Embedded default in binary (`include_str!("../categories.toml")`)

## State Management (`~/.local/state/project-dots/state.toml`)

```toml
[packages.git]
category = "lazyvim-minimal"
was_preexisting = true
installed_by_dots = false
installed_at = "2026-08-07T17:48:00Z"

[packages.fd]
category = "lazyvim-minimal"
was_preexisting = false
installed_by_dots = true
installed_at = "2026-08-07T17:48:05Z"
```

## Category Schema (`categories.toml`)

```toml
[categories.lazyvim-minimal]
description = "Minimal LazyVim dependencies and editor setup"
aliases = ["lzv-min", "lazy-min"]
debian_packages = ["git", "curl", "clang", "fd-find", "lazygit"]
termux_packages = ["neovim", "git", "curl", "clang", "fd", "lazygit"]

[categories.lazyvim-minimal.custom.debian]
name = "neovim"
type = "github_release_tarball"
url = "https://github.com/neovim/neovim/releases/latest/download/nvim-linux-x86_64.tar.gz"
extract_dir = "/opt/nvim"
bin_symlink = "~/.local/bin/nvim"
```

## Category Aliases
Each category can declare optional short aliases via the `aliases` array property in `categories.toml`. Users can invoke `project-dots install <alias>` or `project-dots remove <alias>` interchangeably with the canonical category name.

## Active Production Categories

### 1. `lazyvim-minimal` (Minimal LazyVim Setup)
- **Description**: Minimal required dependencies to run LazyVim.
- **Termux Packages**: `neovim`, `git`, `curl`, `clang`, `fd`, `lazygit`
- **Debian Packages**: `git`, `curl`, `clang`, `fd-find`, `lazygit` + Neovim custom binary tarball (`/opt/nvim` + `~/.local/bin/nvim`).
