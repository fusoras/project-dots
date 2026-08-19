# CLI Command Reference & Manual — project-dots

`project-dots` provides a simple, safety-first command-line interface to manage system packages and dotfile dependencies across **Debian** (`apt`) and **Termux** (`pkg`).

## Overview of Commands

| Command | Purpose | Options |
| ------- | ------- | ------- |
| `dotss setup` | Interactive setup wizard with keyboard navigation (`↑`/`↓`, `Space`, `Enter`) to select presets or custom categories | `--dry-run` / `-n` to preview actions without making changes |
| `dotss list` | Displays available categories and contained packages in a concise single-line format (routed through system `$PAGER` / `less` when on TTY) | `-sh` / `--show-hidden` to display unsupported categories, `-ca` / `--categories` to group by functional domain |
| `dotss search <query>` | Searches categories by name, alias, or contained packages and prints matches in the same format as `list` | Query is matched case-insensitively against category names, aliases, and the current platform's packages |
| `dotss show <category>` | Displays full detailed description, package tracking status, config files, and post-install commands | Accepts category name or alias (e.g. `zsh-tokyonight`, `zsh-tn`) |
| `dotss add [category]` | Adds packages and configurations for a specific category or all categories | `--dry-run` / `-n` to preview actions without making changes |
| `dotss remove <category>` | Safely uninstalls packages for a specific category or all categories | `--all` / `-a` to remove all, `--dry-run` / `-n` to preview |
| `dotss self-update` | Checks GitHub Releases and updates the application binary in-place | `--dry-run` / `-n` to preview version update without downloading |
| `dotss self-uninstall` | Safely removes dotss binary executable and state/config directories | `--yes` / `-y` to confirm deletion, `--no` / `-n` to keep state/config, `--dry-run` / `-d` to preview |
| `dotss --version` | Displays the current application version | `-v` |
| `dotss --help` | Displays the command-line help summary | `-h` |

---

## Command Specifications & Exact Terminal Outputs

### 0. Interactive Setup Wizard (`dotss setup`)
**Command**:
```bash
dotss setup
# Preview actions without executing system package commands:
dotss setup --dry-run
```

**Features & Keyboard Controls**:
1. **Preset Selection Screen**:
   - `Full Config`: Curated complementary suite (`nvim-onedarkpro`, `zsh-tokyonight`, `term-flow`, `nodejs-pnpm`, `agents-flow`).
   - `Custom`: Step-by-step custom configuration.
   - Controls: `[↑/↓]` or `[j/k]` to navigate, `[Enter]` to confirm, `[q/Esc]` to cancel.

2. **Custom Step-by-Step Domain Wizard**:
   - Prompts category by domain section (e.g. `[1/5] Shell & Terminal`, `[2/5] Editors & IDEs`, `[3/5] AI & Agents`, `[4/5] Runtimes & Languages`, `[5/5] Window Managers & Desktop`).
   - `[ ]` / `[x]` Checkboxes with category names, `(Pack)` indicator for pack bundles, and group descriptions.
   - Controls: `[↑/↓]` navigate, `[Space]` toggle item, `[a]` toggle select/deselect all, `[Enter]` next step / confirm, `[q/Esc]` cancel.

---

### 1. List Categories (Simplified Single-Line View & Grouped View)
**Command**:
```bash
dotss list
# Grouped by functional categories:
dotss list --categories
```
*Note: When executed in an interactive terminal (TTY), `dotss list` routes its output through the system `$PAGER` (default `less -FRX`) to allow smooth navigation (`Enter`, `Space`, `q`) without screen clutter.*

**Exact Output (`dotss list -ca`)**:
```text
╭─────────────╮
│ AI & Agents │
╰─────────────╯
  - agents-flow / pack: opencode, engram, herdr, codegraph [apply]
  - agents-tools (ai-tools) / engram, herdr, codegraph [apply]
  - antigravity-cli
  - opencode [apply]

╭──────────────────╮
│ Shell & Terminal │
╰──────────────────╯
  - term-flow / zellij, zoxide, atuin
  - zsh-minimal (zsh-min) / zsh, zoxide, bat [apply]
  - zsh-tokyonight (zsh-tn) / zsh, git, eza, zoxide, bat, starship, atuin, fzf [apply]

╭─────────────────╮
│ Packs & Bundles │
╰─────────────────╯
  - mydots-termux (dots-termux) / pack: zsh-tokyonight, lazyvim-minimal, term-flow
```

---

### 2. Search Categories (`search`)
**Command**:
```bash
dotss search lazygit
dotss search zsh-tn
```
**Exact Output** (same format as `list`; matched categories unsupported on the platform keep the `[unsupported]` suffix):
```text
lazyvim-minimal, lzv-min / git, curl, clang, fd-find, lazygit, neovim (custom binary) [apply]
zsh-tokyonight, zsh-tn / zsh, git, eza, zoxide, bat, starship, atuin [apply]
```
The query matches case-insensitively against:
- the **category name** (e.g. `search lazy` → `lazyvim-minimal`)
- the **aliases** (e.g. `search lzv` → `lazyvim-minimal`)
- the **package list** for the current platform, including Debian custom binaries (e.g. `search lazygit` or `search neovim` → `lazyvim-minimal`, `search atuin` → `zsh-tokyonight`)

---

### 3. Show Category Details (`show`)
**Command**:
```bash
dotss show zsh-tokyonight
# Or using an explicit alias:
dotss show zsh-tn
```
**Exact Output**:
```text
Category: zsh-tokyonight
Description: Zsh terminal setup with TokyoNight Starship prompt, JetBrains Mono font, eza, zoxide, bat, git, and atuin
Aliases: zsh-tn

Packages (Debian):
  - zsh [Pre-existing (system)]
  - git [Pre-existing (system)]
  - eza [Pre-existing (system)]
  - zoxide [Pre-existing (system)]
  - bat [Pre-existing (system)]
  - starship [Pre-existing (system)]
  - atuin [Pre-existing (system)]

Config File Actions:
  - Add ~/.config/zsh/starship.toml
  - Add ~/.termux/font.ttf (Platform: termux) [skip if exists]

Post-Install Commands:
  - chsh -s $(which zsh) (Platform: debian)
  - chsh -s zsh (Platform: termux)

Section Injections:
  - Configure Zsh persistent history options at top of ~/.config/zsh/.zshrc under section '# History configuration' -> ~/.config/zsh/.zshrc under section '# History configuration'
  - Initialize Starship prompt in ~/.config/zsh/.zshrc under section '# Fast init tools' -> ~/.config/zsh/.zshrc under section '# Fast init tools'
  - Initialize zoxide in ~/.config/zsh/.zshrc under section '# Fast init tools' -> ~/.config/zsh/.zshrc under section '# Fast init tools'
  - Enable zsh-autosuggestions plugin in ~/.config/zsh/.zshrc under section '# Zsh plugins' -> ~/.config/zsh/.zshrc under section '# Zsh plugins'
  - Enable fast-syntax-highlighting plugin in ~/.config/zsh/.zshrc under section '# Zsh plugins' -> ~/.config/zsh/.zshrc under section '# Zsh plugins'
  - Initialize Atuin shell history in ~/.config/zsh/.zshrc under section '# Fast init tools' -> ~/.config/zsh/.zshrc under section '# Fast init tools'

Final Message: Restart your terminal or run: exec zsh
```

---

### 4. Remove Packages (`remove`)
**Command**:
```bash
dotss remove lazyvim-minimal --dry-run
# Or remove all installed categories:
dotss remove --all --dry-run
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
dotss self-update --dry-run
```
**Purpose**:
Queries the GitHub Releases API for `dotss`, compares the current version against the latest release tag, and previews or performs binary replacement.

**Exact Terminal Output (Up-to-Date)**:
```text
Checking GitHub Releases for updates...
Current version: <current-version>
Latest release tag: v<current-version>

[Up-to-Date] dotss is already running the latest version.
```

**Exact Terminal Output (Update Available - Dry Run)**:
```text
Checking GitHub Releases for updates...
Current version: <current-version>
Latest release tag: v<newer-version>

=== DRY-RUN MODE ACTIVE: No binary changes will be made ===
[Dry-Run] Would download pre-compiled release binary asset: dotss-x86_64-unknown-linux-gnu.tar.gz
[Dry-Run] Would extract and replace executable at: /home/user/.local/bin/dotss
```

---

### 6. Self-Uninstall Engine
**Command**:
```bash
dotss self-uninstall --dry-run
# Or non-interactive confirmation:
dotss self-uninstall --yes
# Or skip removing config/state directories:
dotss self-uninstall --no
```

**Exact Terminal Output (Dry-Run Preview)**:
```text
=== DRY-RUN MODE ACTIVE: No files will be deleted ===
=== dotss Self-Uninstall Engine ===
Target Binary Path: /home/user/.local/bin/dotss
Target State Directory: /home/user/.local/state/dotss
Target Config Directory: /home/user/.config/dotss

=== DRY-RUN MODE ACTIVE: No files will be deleted ===
[Dry-Run] Would remove executable: /home/user/.local/bin/dotss
[Dry-Run] Would remove state directory: /home/user/.local/state/dotss
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

`project-dots` respects environment variables such as `PROJECT_DOTS_REPO` and `GITHUB_TOKEN`. For full specification and token security guidelines, see [`docs/environment.md`](environment.md).
