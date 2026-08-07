# AGENTS.md

Greenfield Rust project. Intended to become a dotfiles installer, but currently a "Hello, world!" stub — do not assume any conventions or tooling exist yet.

## Facts
- Binary crate `dots-install`, edition 2024 (`Cargo.toml`). No dependencies.
- No commits yet, no CI, no tests, no lint/format config. Only entrypoint is `src/main.rs`.
- `.gitignore` ignores `/target`.

## Commands
- Build/run: `cargo build` / `cargo run`
- Check: `cargo clippy` (warning-only, not enforced)

## Gotchas
- Before adding features (symlinks, shell profile edits, backup logic), confirm intended behavior with the user — the installer's contract is undefined.
- If you add dependencies, keep them minimal; the crate is deliberately dependency-free so far.
