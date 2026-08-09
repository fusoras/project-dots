> Companion tool for system provisioning and dotfiles package management by categories on **Debian** and **Termux**.

`project-dots` is a small CLI tool written in Rust that automates package installation, dotfile deployment, and post-install shell configuration across Debian Linux and Android Termux. It was built mostly with vibe coding.

---

# Install

> [!WARNING] Development version
> This installs the **development build** from the `develop` branch (pre-release, not a stable release).

To install the latest development build directly on your machine (Debian or Termux) without requiring Rust or Cargo:

```bash
curl -sSL https://raw.githubusercontent.com/fusoras/project-dots/develop/install.sh | sh
```

> [!NOTE]
> This command downloads the compiled pre-release binary from the active **`develop`** branch build assets and installs it to `~/.local/bin/dotss` (or `$PREFIX/bin` on Termux).

---

### Quick CLI Overview

| Command | Description | Notes |
|---|---|---|
| `dotss list` | Displays available categories and contained packages on a single line | Shows `[apply]` tag on applied categories |
| `dotss show <category>` | Displays full description, package tracking status, config file actions, and post-install commands | Accepts category name or alias (e.g. `shell-tokyonight`, `shell-tn`) |
| `dotss add [category]` | Adds packages and configurations for a specific category (or `all`) | `--dry-run` / `-n` to preview actions without system changes |
| `dotss remove <category>` | Safely uninstalls packages managed by `dotss` | `--all` / `-a` to remove all, `--dry-run` / `-n` to preview |
| `dotss self-update` | Checks GitHub Releases and updates `dotss` binary in-place | `--dry-run` / `-n` to preview update check |
| `dotss self-uninstall` | Safely removes `dotss` binary executable, state, and config directories | `--yes` / `-y` to confirm deletion, `--no` / `-n` to keep state/config |
