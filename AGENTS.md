# AGENTS.md

Companion tool to the user's dotfiles: its purpose is to **install the packages/programs that complement the dotfiles** (bootstrap a fresh machine). Implementation is early-stage (WIP) — do not assume any conventions or tooling exist yet.

## Facts
- Binary crate `dots-install`, edition 2024 (`Cargo.toml`). No dependencies.
- No CI, no tests, no lint/format config. Only entrypoint is `src/main.rs` (currently a WIP shell-command runner).
- `.gitignore` ignores `/target` and `.codegraph`.

## Commands
- Build/run: `cargo build` / `cargo run`
- Check: `cargo clippy` (warning-only, not enforced)

## CodeGraph (fast file/symbol lookup)
CodeGraph v1.5.0 is installed globally (`~/.local/bin/codegraph`) and its MCP server is registered in opencode, so `codegraph_*` MCP tools are available. Index lives in `.codegraph/` (gitignored).
- Find files / symbols: `codegraph files`, `codegraph query <symbol>`
- Explore an area (source + call paths): `codegraph explore "<topic>"`
- Single symbol trail: `codegraph node <symbol>`
- Freshness: `codegraph status`; run `codegraph sync` after edits (the MCP watcher also keeps the index fresh while opencode is running).

## Gotchas
- Before adding features (symlinks, shell profile edits, backup logic), confirm intended behavior with the user — the installer's contract is undefined.
- If you add dependencies, keep them minimal; the crate is deliberately dependency-free so far.
