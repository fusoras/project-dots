# AGENTS.md — project-dots

Companion tool for system provisioning and dotfiles package management by categories on Debian and Termux (temporary name: `project-dots`, version: `0.1.0-beta.1`).

## Project Facts
- Binary crate `project-dots` v0.1.0-beta.1, edition 2024 (`Cargo.toml`). Written in Rust with no heavy external dependencies.
- Target platforms: **Debian** (via `apt`) and **Termux** (via `pkg`).
- Entrypoint: `src/main.rs`.
- `.gitignore` ignores `/target` and `.codegraph`.

## Commands
- Build / Run: `cargo build` / `cargo run`
- Code linting: `cargo clippy` (warnings)
- Type checking: `cargo check`

## CodeGraph (Fast file & symbol lookup)
CodeGraph v1.5.0 is installed globally (`~/.local/bin/codegraph`) with an active MCP server.
- Find files / symbols: `codegraph files`, `codegraph query <symbol>`
- Explore area: `codegraph explore "<topic>"`
- Sync index: `codegraph sync`

## Additional Documentation
For detailed architecture, roadmap, and platform specifications, consult:
- Planning and Roadmap: @docs/planning.md
- Categories and Packages: @docs/categories.md
- Debian & Termux Support: @docs/platforms.md

## Rules and Conventions
- Keep the codebase lightweight and modular in Rust.
- Use concise bullet points for agent rules and documentation.
- Always validate package names specific to Debian vs Termux.
- Always respond in Spanish to the user in chat.
- Confirm intended behavior with the user before adding major features (e.g., symlinks, backups).
- **Git**: Load the `git-workflow` skill for git commands, aliases, and branch strategy.
