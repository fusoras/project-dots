# CLI Command Reference & Manual — project-dots

`project-dots` provides a simple, safety-first command-line interface to manage system packages and dotfile dependencies across **Debian** (`apt`) and **Termux** (`pkg`).

## Overview of Commands

| Command | Purpose | Options |
| ------- | ------- | ------- |
| `project-dots list` | Displays available categories and contained packages in a concise single-line format | `-sh` / `--show-hidden` to display unsupported categories |
| `project-dots search <query>` | Searches categories by name, alias, or contained packages and prints matches in the same format as `list` | Query is matched case-insensitively against category names, aliases, and the current platform's packages |
| `project-dots show <category>` | Displays full detailed description, package tracking status, config files, and post-install commands | Accepts category name or alias (e.g. `shell-tokyonight`, `shell-tn`) |
| `project-dots add [category]` | Adds packages and configurations for a specific category or all categories | `--dry-run` / `-n` to preview actions without making changes |
| `project-dots remove <category>` | Safely uninstalls packages for a specific category or all categories | `--all` / `-a` to remove all, `--dry-run` / `-n` to preview |
| `project-dots self-update` | Checks GitHub Releases and updates the application binary in-place | `--dry-run` / `-n` to preview version update without downloading |
| `project-dots self-uninstall` | Safely removes project-dots binary executable and state/config directories | `--yes` / `-y` to confirm deletion, `--no` / `-n` to keep state/config, `--dry-run` / `-d` to preview |
| `project-dots --version` | Displays the current application version | `-v` |
| `project-dots --help` | Displays the command-line help summary | `-h` |

---

## Command Specifications & Exact Terminal Outputs

### 1. List Categories (Simplified Single-Line View)
**Command**:
```bash
project-dots list
```
**Exact Output**:
```text
lazyvim-minimal (lzv-min) / git, curl, clang, fd-find, lazygit, neovim (custom binary) [apply]
shell-tokyonight (shell-tn) / zsh, git, eza, zoxide, bat, starship, carapace [apply]
```

---

### 2. Search Categories (`search`)
**Command**:
```bash
project-dots search lazygit
project-dots search shell-tn
```
**Exact Output** (same format as `list`; matched categories unsupported on the platform keep the `[unsupported]` suffix):
```text
lazyvim-minimal (lzv-min) / git, curl, clang, fd-find, lazygit, neovim (custom binary) [apply]
shell-tokyonight (shell-tn) / zsh, git, eza, zoxide, bat, starship, atuin [apply]
```
The query matches case-insensitively against:
- the **category name** (e.g. `search lazy` → `lazyvim-minimal`)
- the **aliases** (e.g. `search lzv` → `lazyvim-minimal`)
- the **package list** for the current platform, including Debian custom binaries (e.g. `search lazygit` or `search neovim` → `lazyvim-minimal`, `search atuin` → `shell-tokyonight`)

---

### 3. Show Category Details (`show`)
**Command**:
```bash
project-dots show shell-tokyonight
# Or using an explicit alias:
project-dots show shell-tn
```
**Exact Output**:
```text
Category: shell-tokyonight
Description: Zsh terminal setup with TokyoNight Starship prompt, JetBrains Mono font, eza, zoxide, bat, git, and carapace
Aliases: shell-tn

Packages (Debian):
  - zsh [Pre-existing (system)]
  - git [Pre-existing (system)]
  - eza [Pre-existing (system)]
  - zoxide [Pre-existing (system)]
  - bat [Pre-existing (system)]
  - starship [Pre-existing (system)]
  - carapace-bin [Pre-existing (system)]

Config File Actions:
  - config/shell-tokyonight/starship.toml -> ~/.config/zsh/starship.toml
  - config/shell-tokyonight/font.ttf -> ~/.termux/font.ttf (Platform: termux) [skip if exists]

Post-Install Commands:
  - chsh -s $(which zsh) (Platform: debian)
  - chsh -s zsh (Platform: termux)
  - grep -qF 'eval "$(starship init zsh)"' "$HOME/.zshrc" 2>/dev/null || echo 'eval "$(starship init zsh)"' >> "$HOME/.zshrc"

Final Message: Close and reopen your terminal so Zsh and the Starship prompt take effect.
```

---

### 4. Remove Packages (`remove`)
**Command**:
```bash
project-dots remove lazyvim-minimal --dry-run
# Or remove all installed categories:
project-dots remove --all --dry-run
```

**Exact Terminal Output (Dry-Run Preview)**:
```text
=== DRY-RUN MODE ACTIVE: No system changes will be made ===
[Category: lazyvim-minimal]
  [Dry-Run] Would uninstall package: lazygit via apt
  [Dry-Run] Would remove custom binary symlink: /home/user/.local/bin/nvim

Removal processing completed successfully.
```

---

### 5. Self-Update Engine
**Command**:
```bash
project-dots self-update --dry-run
```
**Purpose**:
Queries the GitHub Releases API for `project-dots`, compares the current version against the latest release tag, and previews or performs binary replacement.

**Exact Terminal Output (Up-to-Date)**:
```text
Checking GitHub Releases for updates...
Current version: <current-version>
Latest release tag: v<current-version>

[Up-to-Date] project-dots is already running the latest version.
```

**Exact Terminal Output (Update Available - Dry Run)**:
```text
Checking GitHub Releases for updates...
Current version: <current-version>
Latest release tag: v<newer-version>

=== DRY-RUN MODE ACTIVE: No binary changes will be made ===
[Dry-Run] Would download pre-compiled release binary asset: project-dots-x86_64-unknown-linux-gnu.tar.gz
[Dry-Run] Would extract and replace executable at: /home/user/.local/bin/project-dots
```

---

### 6. Self-Uninstall Engine
**Command**:
```bash
project-dots self-uninstall --dry-run
# Or non-interactive confirmation:
project-dots self-uninstall --yes
# Or skip removing config/state directories:
project-dots self-uninstall --no
```

**Exact Terminal Output (Dry-Run Preview)**:
```text
=== DRY-RUN MODE ACTIVE: No files will be deleted ===
=== project-dots Self-Uninstall Engine ===
Target Binary Path: /home/user/.local/bin/project-dots
Target State Directory: /home/user/.local/state/project-dots
Target Config Directory: /home/user/.config/project-dots

=== DRY-RUN MODE ACTIVE: No files will be deleted ===
[Dry-Run] Would remove executable: /home/user/.local/bin/project-dots
[Dry-Run] Would remove state directory: /home/user/.local/state/project-dots
```

---

### 7. System Bootstrap Installation Script
**Command**:
```bash
curl -sSL https://raw.githubusercontent.com/fusoras/project-dots/develop/install.sh | sh
```
**Purpose**:
Installs `project-dots` on a fresh machine (Debian or Termux) in one command without requiring Rust or Cargo.

---

## Environment Variables & Token Security

`project-dots` respects environment variables such as `PROJECT_DOTS_REPO` and `GITHUB_TOKEN`. For full specification and token security guidelines, see [`docs/environment.md`](file:///home/uruka1/1v/Work/dots-install/docs/environment.md).
