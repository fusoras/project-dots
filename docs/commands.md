# CLI Command Reference & Manual — project-dots

`project-dots` provides a simple, safety-first command-line interface to manage system packages and dotfile dependencies across **Debian** (`apt`) and **Termux** (`pkg`).

## Overview of Commands

| Command | Purpose | Options |
| ------- | ------- | ------- |
| `project-dots list` | Displays available categories and package lists in a clean format | `--debug` / `-d` for detailed system package status |
| `project-dots install [category]` | Installs packages for a specific category or all categories | `--dry-run` / `-n` to preview actions without making changes |
| `project-dots remove [category]` | Safely uninstalls packages installed by `project-dots` | `--dry-run` / `-n` to preview removal without making changes |
| `project-dots self-update` | Checks GitHub Releases and updates the application binary in-place | `--dry-run` / `-n` to preview version update without downloading |
| `project-dots --version` | Displays the current application version | `-V` |
| `project-dots --help` | Displays the command-line help summary | `-h` |

---

## Command Specifications & Exact Terminal Outputs

### 1. List Categories (Default User View)
**Command**:
```bash
project-dots list
```

---

### 2. Install Category (Dry-Run & Live)
**Command**:
```bash
project-dots install lazyvim-minimal --dry-run
```

---

### 3. Self-Update Engine
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

### 4. System Bootstrap Installation Script
**Command**:
```bash
curl -sSL https://raw.githubusercontent.com/fusoras/project-dots/develop/install.sh | sh
```
**Purpose**:
Installs `project-dots` on a fresh machine (Debian or Termux) in one command without requiring Rust or Cargo.
