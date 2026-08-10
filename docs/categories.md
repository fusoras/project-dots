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
[categories.shell-tokyonight]
description = "Zsh terminal setup with TokyoNight Starship prompt and JetBrains Mono font"
aliases = ["shell", "zsh", "terminal", "tokyonight-shell"]
debian_packages = ["zsh", "git", "bat", "zoxide"]
termux_packages = ["zsh", "git", "bat", "zoxide", "starship", "eza"]
final_message = "Close and reopen your terminal so Zsh and the Starship prompt take effect."

[[categories.shell-tokyonight.copy_files]]
src = "config/shell-tokyonight/starship.toml"
dest = "~/.config/zsh/starship.toml"

[[categories.shell-tokyonight.copy_files]]
src = "config/shell-tokyonight/font.ttf"
dest = "~/.termux/font.ttf"
platform = "termux"
only_if_not_exists = true

[[categories.shell-tokyonight.post_install_commands]]
command = "chsh -s $(which zsh)"
platform = "debian"

[[categories.shell-tokyonight.post_install_commands]]
command = "chsh -s zsh"
platform = "termux"

[[categories.shell-tokyonight.section_injections]]
file = "~/.zshrc"
section = "# Fast init tools"
line = 'eval "$(starship init zsh)"'
description = "Initialize Starship prompt in ~/.zshrc under section '# Fast init tools'"

[[categories.shell-tokyonight.section_injections]]
file = "~/.zshrc"
section = "# Fast init tools"
line = 'eval "$(zoxide init zsh)"'
description = "Initialize zoxide in ~/.zshrc under section '# Fast init tools'"

[[categories.shell-tokyonight.section_injections]]
file = "~/.zshrc"
section = "# Zsh plugins"
line = 'source ~/.config/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh'
description = "Enable zsh-autosuggestions plugin in ~/.zshrc under section '# Zsh plugins'"

[[categories.shell-tokyonight.section_injections]]
file = "~/.zshrc"
section = "# Zsh plugins"
line = 'source ~/.config/zsh/plugins/fast-syntax-highlighting/fast-syntax-highlighting.plugin.zsh'
description = "Enable fast-syntax-highlighting plugin in ~/.zshrc under section '# Zsh plugins'"

[[categories.shell-tokyonight.section_injections]]
file = "~/.zshrc"
section = "# Fast init tools"
line = 'eval "$(atuin init zsh)"'
description = "Initialize Atuin shell history in ~/.zshrc under section '# Fast init tools'"

[[categories.shell-tokyonight.post_install_commands]]
command = "git clone --depth=1 https://github.com/zsh-users/zsh-autosuggestions ~/.config/zsh/plugins/zsh-autosuggestions 2>/dev/null || true; rm -rf ~/.config/zsh/plugins/zsh-autosuggestions/.git"

[[categories.shell-tokyonight.post_install_commands]]
command = "grep -qF 'source ~/.config/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh' \"$HOME/.zshrc\" 2>/dev/null || echo 'source ~/.config/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh' >> \"$HOME/.zshrc\""

[[categories.shell-tokyonight.post_install_commands]]
command = "git clone --depth=1 https://github.com/zdharma-continuum/fast-syntax-highlighting ~/.config/zsh/plugins/fast-syntax-highlighting 2>/dev/null || true; rm -rf ~/.config/zsh/plugins/fast-syntax-highlighting/.git"

[[categories.shell-tokyonight.post_install_commands]]
command = "grep -qF 'source ~/.config/zsh/plugins/fast-syntax-highlighting/fast-syntax-highlighting.plugin.zsh' \"$HOME/.zshrc\" 2>/dev/null || echo 'source ~/.config/zsh/plugins/fast-syntax-highlighting/fast-syntax-highlighting.plugin.zsh' >> \"$HOME/.zshrc\""
```

## Category Aliases
Each category can declare optional short aliases via the `aliases` array property in `categories.toml`. Users can invoke `project-dots install <alias>` or `project-dots remove <alias>` interchangeably with the canonical category name.

## Active Production Categories

### 1. `lazyvim-minimal` (Minimal LazyVim Setup)
- **Description**: Minimal required dependencies to run LazyVim.
- **Termux Packages**: `neovim`, `git`, `curl`, `clang`, `fd`, `lazygit`
- **Debian Packages**: `git`, `curl`, `clang`, `fd-find`, `lazygit` + Neovim custom binary tarball (`/opt/nvim` + `~/.local/bin/nvim`).

### 2. `shell-tokyonight` (Zsh & TokyoNight Starship Setup)
- **Description**: Zsh terminal setup with TokyoNight Starship prompt, JetBrains Mono font, eza, zoxide, bat, and git.
- **Aliases**: `tokyonight-shell`, `zsh-tokyonight`, `shell-tokyo`, `terminal`, `shell`, `zsh`
- **Packages**: `zsh`, `git`, `eza`, `zoxide`, `bat`, `starship`
- **Config Copies**:
  - `config/shell-tokyonight/starship.toml` -> `~/.config/zsh/starship.toml`
  - `config/shell-tokyonight/font.ttf` -> `~/.termux/font.ttf` (Termux only, skipped if `font.ttf` already exists)
- **Post-Install**:
  - Debian: `chsh -s $(which zsh)` (sets Zsh as default shell)
  - Termux: `chsh -s zsh` (sets Zsh as default shell)
  - All: ensures `~/.zshrc` exists and adds `eval "$(starship init zsh)"` so Starship starts with Zsh
- **Final Message**: Close and reopen the terminal so Zsh and the Starship prompt take effect.

### 3. `nodejs-pnpm` (Node.js & pnpm Setup)
- **Description**: Node.js environment with NVM (Debian) or Node.js LTS (Termux) and pnpm enabled via Corepack.
- **Termux Packages**: `nodejs-lts`
- **Debian Packages**: `curl`
- **Post-Install**:
  - Debian: Installs NVM v0.40.6, installs Node.js v24, enables pnpm via Corepack, prepares `pnpm@latest`, and verifies version (`pnpm -v`).
  - Termux: Enables pnpm via Corepack, prepares `pnpm@latest`, and verifies version (`pnpm -v`).



