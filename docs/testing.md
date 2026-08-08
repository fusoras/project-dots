# Unit Testing Guide & Specifications — project-dots

`project-dots` uses Rust's built-in unit testing framework (`#[cfg(test)]`) coupled with **Test-Driven Development (TDD Red-Green)** to ensure reliability across critical system components.

## Running Unit Tests

To run the unit test suite with human-readable explanatory descriptions printed directly in the console:

```bash
cargo test -- --nocapture
```

---

## Unit Test Inventory & Scope

| Test Name | Module | Primary Purpose & Verification |
| --------- | ------ | ----------------------------- |
| `test_embedded_config_parsing` | `src/config.rs` | Verifies TOML configuration deserialization, embedded fallback loader (`include_str!`), and presence of default categories like `lazyvim-minimal`. |
| `test_category_alias_resolution` | `src/config.rs` | Verifies resolution of category aliases (e.g., `lzv-min` -> `lazyvim-minimal`). |
| `test_track_package_logic` | `src/state.rs` | Verifies package state registration, pre-existence flagging (`was_preexisting = true`), protection against accidental removal, and clean state removal. |
| `test_expand_home_utility` | `src/installer.rs` | Verifies path expansion from tilde paths (`~/.local/bin/nvim`) to absolute user paths (`/home/user/.local/bin/nvim`). |
| `test_command_exists_utility` | `src/platform.rs` | Verifies PATH directory inspection for command presence without relying on external `which`. |
| `test_is_newer_version_logic` | `src/update.rs` | Verifies SemVer version comparison logic for `self-update`. |
| `test_platform_asset_resolution` | `src/update.rs` | Verifies target release asset resolution for Debian (`x86_64`) vs Termux (`aarch64-musl`). |
| `test_install_subcommand_alias` | `src/main.rs` | Verifies parsing of short flag `-i` and alias `i` for the `install` subcommand. |

---

## Exact Terminal Output (`cargo test -- --nocapture`)

```text
running 7 tests

🔍 [TEST] Embedded Default TOML Configuration Parsing
   Explanation: Verifies that the category catalog parses successfully and contains 'lazyvim-minimal'.

🔍 [TEST] System Command Presence Check (PATH Inspection)
   Explanation: Verifies that command_exists checks PATH directories directly without relying on external 'which'.

🔍 [TEST] Safety Engine & Package State Tracking
   Explanation: Verifies that pre-existing system packages are flagged as protected to prevent accidental removal.
   ✓ Package 'git' tracked successfully.
   ✓ Command 'cargo' found in system PATH.
   ✓ State 'was_preexisting': true (Protected against uninstallation)
   ✓ Package removed from state tracking registry cleanly.

🔍 [TEST] Platform Release Asset Resolution
   Explanation: Verifies that Debian resolves to the x86_64 tarball asset and Termux to aarch64.
   ✓ Debian target asset resolved correctly: project-dots-x86_64-unknown-linux-gnu.tar.gz
   ✓ Termux target asset resolved correctly: project-dots-aarch64-unknown-linux-musl.tar.gz

🔍 [TEST] SemVer Version Comparison for Self-Update
   Explanation: Verifies that release tag versions (e.g. v0.1.0-beta.2) are correctly identified as newer than v0.1.0-beta.1.
   ✓ v0.1.0-beta.2 recognized as newer than 0.1.0-beta.1
   ✓ Same version v0.1.0-beta.1 recognized as equal (not newer)
   ✓ Older version v0.0.9 recognized as not newer.

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
