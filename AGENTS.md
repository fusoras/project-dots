# AGENTS.md — project-dots

Companion tool for system provisioning and dotfiles package management by categories on Debian and Termux (temporary name: `project-dots`, version: `0.1.0-beta.36`).

## Project Facts
- Binary crate `dotss` v0.1.0-beta.36, edition 2024 (`Cargo.toml`). Written in Rust with no heavy external dependencies.
- **Command Renaming Note**: Executable binary command was officially renamed from `project-dots` to `dotss` in `v0.1.0-beta.20` for CLI user convenience.
- Target platforms: **Debian** (via `apt`) and **Termux** (via `pkg`).
- Entrypoint: `src/main.rs`.
- `.gitignore` ignores `/target` and `.codegraph`.

## Commands
- Build / Run (Dev): `cargo build` / `cargo run`
- Release Build: `cargo build --release` (binary at `./target/release/dotss`)
- Local Installation: `cargo install --path .` (installs `dotss` executable to `~/.cargo/bin` or `$PATH`)
- Code linting: `cargo clippy` (warnings)
- Type checking: `cargo check`
- Unit Testing: `cargo test -- --nocapture`

## CodeGraph (Fast file & symbol lookup)
CodeGraph v1.5.0 is installed globally (`~/.local/bin/codegraph`) with an active MCP server.
- Find files / symbols: `codegraph files`, `codegraph query <symbol>`
- Explore area: `codegraph explore "<topic>"`
- Sync index: `codegraph sync`

## Additional Documentation
For detailed architecture, roadmap, CLI reference, unit testing, and release specifications, consult:
- Planning and Roadmap: @docs/planning.md
- Categories and Packages: @docs/categories.md
- Debian & Termux Support: @docs/platforms.md
- CLI Reference & Commands: @docs/commands.md
- Unit Testing Guide: @docs/testing.md
- Versioning & Release Guide: @docs/versioning.md
- Environment Variables & Token Security: @docs/environment.md

## Rules and Conventions
- Keep the codebase lightweight and modular in Rust.
- Use concise bullet points for agent rules and documentation.
- Present user-facing commands using the compiled binary (`dotss <command>`) rather than `cargo run --`.
- **Security Governance & Secret Leak Prevention**:
  - NEVER commit API keys, private keys (`id_*`), certificates (`*.key`, `*.pem`), or `.env` files within `config/<category>/` or project directories.
  - Template files in `config/` are embedded directly into the compiled executable release binary (`include_str!`/`include_bytes!`). Any committed secret will be permanently exposed in public release binaries.
  - Never log runtime tokens (e.g. `GITHUB_TOKEN`) in stdout, stderr, or `state.toml`.
  - Print a clear `[WARNING]` alert when loading external `./categories.toml` configurations before running post-install commands or custom installers.
- **Category & Configuration Naming Convention**:
  - Category names and configuration subdirectories under `config/` must NEVER be generic (e.g., avoid `shell`, `editor`, `config`).
  - Names must be simple but distinctive, combining the component/tool type with its specific variant, theme, or style (e.g., `shell-tokyonight`, `lazyvim-minimal`).
  - This prevents naming collisions when multiple distinct configurations exist for the same tool or component.
- **Explicit Alias Governance**:
  - Aliases for categories or commands must ONLY be created when explicitly defined by the user.
  - Never generate, infer, or automatically append unrequested aliases. Always consult or ask the user before defining aliases.
- **Git Strategy & Modern Commands**:
  - Never use `git checkout`. Use modern git commands (`git switch`, `git restore`).
  - User Git Aliases: `git s` -> `git switch`, `git b` -> `git branch`.
  - All development edits must be conducted on a development branch (`develop` or feature branches). Only merge into `main` once code is fully tested and verified to work cleanly.
  - **Explicit Merge Restriction**: Never execute a branch merge (`git merge`) unless the user explicitly instructs to merge in their message.
  - **Explicit Push Restriction**: Never execute a remote push command (`git push`) unless the user explicitly instructs to push in their message. Before pushing, clearly explain what commits, branches, or tags will be pushed and request confirmation.
- **Version Bumping Policy**:
  - Any code addition (`feat`), feature enhancement, or bug fix (`fix`) MUST increment/bump the application version number.
  - Documentation updates, additions, or Markdown edits (`docs`, `style`) NEVER trigger or alter the program version number.
  - **Bumps happen only in `develop`**: Version increments are executed exclusively on the `develop` branch when preparing a release. Feature branches and git worktrees MUST NOT modify the version number (`Cargo.toml`, `Cargo.lock`, `install.sh`, `AGENTS.md`). The bump is applied once on `develop` after merging feature work.
