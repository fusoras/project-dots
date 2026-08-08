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
> This command downloads the compiled pre-release binary from the active **`develop`** branch build assets and installs it to `~/.local/bin/project-dots` (or `$PREFIX/bin` on Termux).

---

## 🛠️ Quick CLI Reference

| Command | Purpose | Options |
| ------- | ------- | ------- |
| `project-dots list` | Displays available categories and contained packages on a single line | Shows `[apply]` tag on applied categories |
| `project-dots show <category>` | Displays full description, package tracking status, config file actions, and post-install commands | Accepts category name or alias (e.g. `shell-tokyonight`, `shell-tn`) |
| `project-dots add [category]` | Adds packages and configurations for a specific category (or `all`) | `--dry-run` / `-n` to preview actions without system changes |
| `project-dots remove <category>` | Safely uninstalls packages managed by `project-dots` | `--all` / `-a` to remove all, `--dry-run` / `-n` to preview |
| `project-dots self-update` | Checks GitHub Releases and updates `project-dots` binary in-place | `--dry-run` / `-n` to preview update check |
| `project-dots self-uninstall` | Safely removes `project-dots` binary executable, state, and config directories | `--yes` / `-y` for non-interactive confirmation |
