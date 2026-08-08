# Cybersecurity & Secret Governance — project-dots

Rules and constraints to prevent credential leaks, arbitrary code execution (RCE) via untrusted configuration files, and unverified binary downloads.

- **No Secrets or Credentials in `config/`**:
  - Never commit API keys, tokens, private SSH keys (`id_*`), certificates (`*.pem`, `*.key`), `.env` files, or history files inside `config/<category>/` subdirectories.
  - Category template files embedded via `include_str!` or `include_bytes!` are compiled directly into the release executable. Any committed secret is permanently baked into public binary releases.
- **Environment Token Protection**:
  - Never log, display, or persist runtime tokens (e.g. `GITHUB_TOKEN`, API keys) in stdout, stderr, debug logs, or state files (`state.toml`).
- **Untrusted Configuration Warning**:
  - When loading `categories.toml` from local path (`./categories.toml` or `~/.config/project-dots/categories.toml`) instead of the embedded default configuration, print a clear `[WARNING]` alert before processing custom installers or executing `post_install_commands`.
- **Integrity & Checksum Governance**:
  - Custom binary tarball releases or self-update binaries must be fetched strictly over HTTPS from verified GitHub Release assets or pinned URLs.
