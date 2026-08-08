# Platform Support — Debian vs Termux

`project-dots` adapts package operations dynamically based on the detected operating system.

## Pre-flight Check Strategy

Before initiating installation steps, `project-dots` runs pre-flight checks:
- **Prerequisite Binaries**: Verifies presence of `curl` and `tar` by inspecting `$PATH` directories directly via `std::env::split_paths`.
- **Package Presence Check**:
  - **Debian**: `dpkg-query -W -f='${db:Status-Status}' <package>`
  - **Termux**: `dpkg-query -W -f='${db:Status-Status}' <package>`
- **Package Manager Locks**: Checks `/var/lib/dpkg/lock-frontend` on Debian to avoid conflicts with background updates.

## Platform Command Matrix

| Feature | Debian | Termux |
| ------- | ------ | ------ |
| **Detection Method** | `/etc/debian_version` or `/etc/os-release` | `$TERMUX_VERSION` env var or `/data/data/com.termux/` |
| **Package Manager** | `apt` / `apt-get` | `pkg` |
| **Package Check** | `dpkg-query -W` | `dpkg-query -W` |
| **Install Command** | `sudo apt install -y <packages>` | `pkg install -y <packages>` |
| **Safe Remove Command**| `sudo apt remove -y <only-installed-by-dots>` | `pkg remove -y <only-installed-by-dots>` |
| **Neovim Strategy** | GitHub release tarball -> `/opt/nvim` -> `~/.local/bin/nvim` | Official `pkg install neovim` |
| **Binary Symlinks** | `~/.local/bin/` | `$PREFIX/bin/` (`/data/data/com.termux/files/usr/bin`) |
| **Platform Config Copies** | Config files (`~/.config/starship.toml`) | Config files + Termux font (`~/.termux/font.ttf`) |
