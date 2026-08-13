# Category Structure & Configuration

`categories.toml` is the central **declarative catalog** of `project-dots`. It defines the categories of applications to install, their human-readable descriptions, platform-specific package names (Debian vs Termux), and custom installation handlers.

## Configuration File Resolution Order

1. **Base Catalog**:
   - `./categories.toml` (Current working directory)
   - `~/.config/dotss/categories.toml` (XDG User Config)
   - Embedded default in binary (`include_str!("../categories.toml")`)

2. **Modular Directories (`categories.d/`)**:
   - `./categories.d/*.toml` (Local modular configurations, loaded alphabetically)
   - `~/.config/dotss/categories.d/*.toml` (XDG User modular configurations)

Modular `.toml` files allow breaking down large configurations into clean, domain-specific files (e.g. `editors.toml`, `shell.toml`). Categories declared in `categories.d/*.toml` are merged into the main category catalog.

## State Management (`~/.local/state/dotss/state.toml`)

`dotss` tracks applied categories and package installation metadata deterministically in `~/.local/state/dotss/state.toml`:

```toml
applied_categories = ["lazyvim-minimal", "zsh-tokyonight"]

[packages.git]
category = "lazyvim-minimal"
was_preexisting = true
installed_by_dots = false
installed_at = "Timestamp(1770925680)"

[packages.fd]
category = "lazyvim-minimal"
was_preexisting = false
installed_by_dots = true
installed_at = "Timestamp(1770925685)"
```

Category applied status in `dotss list` is verified strictly against `applied_categories` in `state.toml`. This guarantees zero false-positive status indicators on clean systems that already contain pre-existing system packages (such as `curl` or `git`). Executing commands with `--dry-run` (`-n`) maintains 100% state immutability without modifying `state.toml`.

## Category Schema (`categories.toml`)

```toml
[categories.zsh-tokyonight]
description = "Zsh terminal setup with TokyoNight Starship prompt and JetBrains Mono font"
aliases = ["zsh-tn", "zsh", "terminal", "tokyonight-shell"]
debian_packages = ["zsh", "git", "bat", "zoxide"]
termux_packages = ["zsh", "git", "bat", "zoxide", "starship", "eza"]
final_message = "Restart your terminal or run: exec zsh"

[[categories.zsh-tokyonight.copy_files]]
src = "config/zsh-tokyonight/starship.toml"
dest = "~/.config/zsh/starship.toml"

[[categories.zsh-tokyonight.copy_files]]
src = "config/zsh-tokyonight/font.ttf"
dest = "~/.termux/font.ttf"
platform = "termux"
only_if_not_exists = true

[[categories.zsh-tokyonight.post_install_commands]]
command = "chsh -s $(which zsh)"
platform = "debian"

[[categories.zsh-tokyonight.post_install_commands]]
command = "chsh -s zsh"
platform = "termux"

[[categories.zsh-tokyonight.section_injections]]
file = "~/.config/zsh/.zshrc"
section = "# Fast init tools"
line = 'eval "$(starship init zsh)"'
description = "Initialize Starship prompt in ~/.config/zsh/.zshrc under section '# Fast init tools'"

[[categories.zsh-tokyonight.section_injections]]
file = "~/.config/zsh/.zshrc"
section = "# Fast init tools"
line = 'eval "$(zoxide init zsh)"'
description = "Initialize zoxide in ~/.config/zsh/.zshrc under section '# Fast init tools'"

[[categories.zsh-tokyonight.section_injections]]
file = "~/.config/zsh/.zshrc"
section = "# Zsh plugins"
line = 'source ~/.config/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh'
description = "Enable zsh-autosuggestions plugin in ~/.config/zsh/.zshrc under section '# Zsh plugins'"

[[categories.zsh-tokyonight.section_injections]]
file = "~/.config/zsh/.zshrc"
section = "# Zsh plugins"
line = 'source ~/.config/zsh/plugins/fast-syntax-highlighting/fast-syntax-highlighting.plugin.zsh'
description = "Enable fast-syntax-highlighting plugin in ~/.config/zsh/.zshrc under section '# Zsh plugins'"

[[categories.zsh-tokyonight.section_injections]]
file = "~/.config/zsh/.zshrc"
section = "# Fast init tools"
line = 'eval "$(atuin init zsh)"'
description = "Initialize Atuin shell history in ~/.config/zsh/.zshrc under section '# Fast init tools'"

[[categories.zsh-tokyonight.post_install_commands]]
command = "git clone --depth=1 https://github.com/zsh-users/zsh-autosuggestions ~/.config/zsh/plugins/zsh-autosuggestions 2>/dev/null || true; rm -rf ~/.config/zsh/plugins/zsh-autosuggestions/.git"

## Section Injections (`section_injections`)
Categories can declare structured line injections under specific comment headers (e.g. `# History configuration` or `# Fast init tools`).
- **File**: Path to target file (e.g. `~/.zshrc`).
- **Section**: Header comment string (e.g. `# History configuration`).
- **Line**: Content line(s) to insert directly beneath the header comment.
- **Position** (Optional): Optional placement preference when creating a new section comment (`"top"` or `"bottom"`, default `"bottom"`). Setting `position = "top"` ensures history or env configuration sections are prepended to the very beginning of `~/.zshrc`.
- **Auto Header Generation**: If the section comment header does not exist in the file, `dotss` automatically inserts the header comment before inserting the line beneath it (at the top if `position = "top"` or bottom otherwise).
- **Idempotency**: If the line already exists anywhere in the file, insertion is skipped (`[SKIP]`).
Each category can declare optional short aliases via the `aliases` array property in `categories.toml`. Users can invoke `project-dots install <alias>` or `project-dots remove <alias>` interchangeably with the canonical category name.

## Active Production Categories

### 1. `lazyvim-minimal` (Minimal LazyVim Setup)
- **Description**: Minimal required dependencies to run LazyVim.
- **Termux Packages**: `neovim`, `git`, `curl`, `clang`, `fd`, `lazygit`
- **Debian Packages**: `git`, `curl`, `clang`, `fd-find`, `lazygit` + Neovim custom binary tarball (`/opt/nvim` + `~/.local/bin/nvim`).
- **Post-Install**: Clones default LazyVim config to `~/.config/nvim` (automatically creates timestamped backups if existing Neovim config is found).

### 2. `zsh-tokyonight` (Zsh & TokyoNight Starship Setup)
- **Description**: Zsh terminal setup with TokyoNight Starship prompt, JetBrains Mono font, eza, zoxide, bat, git, atuin, and fzf.
- **Aliases**: `zsh-tn`
- **Packages**: `zsh`, `git`, `eza`, `zoxide`, `bat`, `starship`, `atuin`, `fzf`
- **Config Copies**:
  - `config/zsh-tokyonight/starship.toml` -> `~/.config/zsh/starship.toml`
  - `config/zsh-tokyonight/font.ttf` -> `~/.termux/font.ttf` (Termux only, skipped if `font.ttf` already exists)
- **Post-Install**:
  - Debian: `chsh -s $(which zsh)` (sets Zsh as default shell)
  - Termux: `chsh -s zsh` (sets Zsh as default shell)
  - All: ensures `~/.zshrc` exists and adds `eval "$(starship init zsh)"` so Starship starts with Zsh
- **Final Message**: Restart your terminal or run: exec zsh (or Restart Termux or run: exec zsh on Termux).

### 3. `nodejs-pnpm` (Node.js & pnpm Setup)
- **Description**: Node.js environment with NVM (Debian) or Node.js LTS (Termux) and pnpm enabled via Corepack.
- **Termux Packages**: `nodejs-lts`
- **Debian Packages**: `curl`
- **Post-Install**:
  - Debian: Installs NVM v0.40.6, installs Node.js v24, enables pnpm via Corepack, prepares `pnpm@latest`, and verifies version (`pnpm -v`).
  - Termux: Enables pnpm via Corepack, prepares `pnpm@latest`, and verifies version (`pnpm -v`).

### 4. `term-flow` (Terminal Navigation & Session Suite)
- **Description**: Agile terminal navigation and session management suite with zellij, zoxide, and atuin.
- **Aliases**: `terminal-flow`, `nav-flow`
- **Debian Packages**: `zoxide`, `atuin` + `zellij` custom binary installer (`https://github.com/zellij-org/zellij/releases/latest/download/zellij-x86_64-unknown-linux-musl.tar.gz` -> `/opt/zellij` -> `~/.local/bin/zellij`).
- **Termux Packages**: `zellij`, `zoxide`, `atuin`

### 5. `opencode` (OpenCode AI Coding Assistant)
- **Description**: OpenCode AI coding assistant for Debian.
- **Aliases**: `open-code`, `opencode-cli`
- **Debian Packages**: `curl`, `git`, `bash`
- **Post-Install**: `curl -fsSL https://opencode.ai/install | bash`

### 6. `agent-tools` (Token-Saving Agentic Helper Tools)
- **Description**: Token-saving agentic helper tools suite for Debian (engram, herdr, codegraph).
- **Aliases**: `ai-tools`
- **Debian Packages**: `curl`, `git`, `tar`, `grep`
- **Post-Install**: Executes non-interactively (no confirmation prompts) to install helper utilities:
  - `engram`: Queries latest release tag via GitHub API, downloads `engram_*_linux_amd64.tar.gz`, and places binary at `~/.local/bin/engram`
  - `herdr`: `curl -fsSL https://herdr.dev/install.sh | sh`
  - `codegraph`: `curl -fsSL https://raw.githubusercontent.com/colbymchenry/codegraph/main/install.sh | sh`

### 7. `agent-flow` (Complete Agentic AI Suite Pack)
- **Description**: Complete Agentic AI suite combining opencode and token-saving agent-tools (engram, herdr, codegraph).
- **Includes**: `opencode`, `agent-tools`



