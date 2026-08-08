# AGENTS.md — project-dots

Companion tool for system provisioning and dotfiles package management by categories on Debian and Termux (temporary name: `project-dots`, version: `0.1.0-beta.2`).

## Project Facts
- Binary crate `project-dots` v0.1.0-beta.2, edition 2024 (`Cargo.toml`). Written in Rust with no heavy external dependencies.
- Target platforms: **Debian** (via `apt`) and **Termux** (via `pkg`).
- Entrypoint: `src/main.rs`.
- `.gitignore` ignores `/target` and `.codegraph`.

## Commands
- Build / Run (Dev): `cargo build` / `cargo run`
- Release Build: `cargo build --release` (binary at `./target/release/project-dots`)
- Local Installation: `cargo install --path .` (installs `project-dots` executable to `~/.cargo/bin` or `$PATH`)
- Code linting: `cargo clippy` (warnings)
- Type checking: `cargo check`
- Unit Testing: `cargo test -- --nocapture`

## CodeGraph (Fast file & symbol lookup)
CodeGraph v1.5.0 is installed globally (`~/.local/bin/codegraph`) with an active MCP server.
- Find files / symbols: `codegraph files`, `codegraph query <symbol>`
- Explore area: `codegraph explore "<topic>"`
- Sync index: `codegraph sync`

## Additional Documentation
For detailed architecture, roadmap, CLI reference, and unit testing specifications, consult:
- Planning and Roadmap: @docs/planning.md
- Categories and Packages: @docs/categories.md
- Debian & Termux Support: @docs/platforms.md
- CLI Reference & Commands: @docs/commands.md
- Unit Testing Guide: @docs/testing.md

## Rules and Conventions
- Keep the codebase lightweight and modular in Rust.
- Use concise bullet points for agent rules and documentation.
- Present user-facing commands using the compiled binary (`project-dots <command>`) rather than `cargo run --`.
- Always validate package names specific to Debian vs Termux.
- Always respond in Spanish to the user in chat.
- Confirm intended behavior with the user before adding major features (e.g., symlinks, backups).
- **Git Strategy & Modern Commands**:
  - Never use `git checkout`. Use modern git commands (`git switch`, `git restore`).
  - User Git Aliases: `git s` -> `git switch`, `git b` -> `git branch`.
  - All development edits must be conducted on a development branch (`develop` or feature branches). Only merge into `main` once code is fully tested and verified to work cleanly.
  - **Explicit Merge Restriction**: Never execute a branch merge (`git merge`) unless the user explicitly instructs to merge in their message.
  - **Explicit Push Restriction**: Never execute a remote push command (`git push`) unless the user explicitly instructs to push in their message. Before pushing, clearly explain what commits, branches, or tags will be pushed and request confirmation.
