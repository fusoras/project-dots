---
name: project-versioning
description: Execute project version bumps and release tagging following SemVer, authority rules, and the 4-file checklist. Use when asked to "bump version", "release", "actualizar version", "subir version", "crear tag", or "hacer release".
---

# Project Versioning & Release Protocol

This skill governs how version increments, release tagging, and distribution pipelines are executed in `project-dots` (`dotss`).

## 1. User Authority & Governance Rules

- **Exclusive User Authority**: The user exclusively determines, authorizes, and defines version numbers (e.g. `v0.1.0-beta.40`) and release triggers.
- **No Autonomous Bumping**: Never increment or change version numbers autonomously for intermediate local commits or minor edits.
- **Strict Branch Restriction**: All release commits and tag creations must occur on the development branch (`develop`). **NEVER** switch to, merge into, or touch the `main` branch unless explicitly instructed.
- **Push Restriction**: Only push commits or tags to the remote repository when the user explicitly requests it.

## 2. Version Bump 4-File Checklist

When a version update is authorized by the user, update the version string across all authoritative files:

1. **`Cargo.toml`**:
   Update `version = "0.1.0-beta.X"`.
2. **`Cargo.lock`**:
   Automatically updated by running `cargo check` or `cargo build`.
3. **`install.sh`**:
   Update the header comment (`# project-dots (v0.1.0-beta.X) System Installer Script`).
4. **`AGENTS.md`**:
   Update header and Project Facts lines (`version: 0.1.0-beta.X`, `crate dotss v0.1.0-beta.X`).

## 3. Step-by-Step Release Procedure

```bash
# Step 1: Update version in Cargo.toml, install.sh, and AGENTS.md

# Step 2: Validate compilation, linting, and tests (also updates Cargo.lock)
cargo check && cargo clippy && cargo test -- --nocapture

# Step 3: Reinstall local binary to ~/.cargo/bin and ~/.local/bin
cargo install --path .
cp ~/.cargo/bin/dotss ~/.local/bin/dotss

# Step 4: Stage authoritative files and commit with conventional message
git add Cargo.toml Cargo.lock install.sh AGENTS.md .agents/skills/
git commit -m "update: bump version to v0.1.0-beta.X"

# Step 5: Push develop branch to remote (with user authorization)
git push origin develop

# Step 6: Create and push annotated release tag (triggers CI release workflow)
git tag -a v0.1.0-beta.X -m "Release v0.1.0-beta.X"
git push origin v0.1.0-beta.X
```

## 4. Anti-patterns

- Do NOT bump version without explicit version definition from the user.
- Do NOT commit directly to or switch to `main`.
- Do NOT push tags before testing and verifying the release locally.
- Do NOT create lightweight (unannotated) git tags — always use `git tag -a <tag> -m "<message>"`.
