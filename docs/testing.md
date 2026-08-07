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
| `test_track_package_logic` | `src/state.rs` | Verifies package state registration, pre-existence flagging (`was_preexisting = true`), protection against accidental removal, and clean state removal. |
| `test_expand_home_utility` | `src/installer.rs` | Verifies path expansion from tilde paths (`~/.local/bin/nvim`) to absolute user paths (`/home/user/.local/bin/nvim`). |

---

## Exact Terminal Output (`cargo test -- --nocapture`)

```text
running 3 tests

🔍 [TEST] Embedded Default TOML Configuration Parsing
   Explanation: Verifies that the category catalog parses successfully and contains 'lazyvim-minimal'.

🔍 [TEST] System Path Expansion (~)
   Explanation: Verifies that paths such as '~/.local/bin/nvim' are converted to full absolute paths.
   Original path: ~/.local/bin/nvim
   Expanded path: /home/user/.local/bin/nvim
   ✓ Path expansion verified successfully.

🔍 [TEST] Safety Engine & Package State Tracking
   Explanation: Verifies that pre-existing system packages are flagged as protected to prevent accidental removal.
   ✓ Package 'git' tracked successfully.
   ✓ State 'was_preexisting': true (Protected against uninstallation)
   ✓ Package removed from state tracking registry cleanly.

test installer::tests::test_expand_home_utility ... ok
test state::tests::test_track_package_logic ... ok
   ✓ Valid TOML structure. Total categories loaded: 1
   ✓ Category 'lazyvim-minimal' verified in catalog.

test config::tests::test_embedded_config_parsing ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## TDD Red-Green Methodology Applied

1. **RED Stage**: Tests were initially authored with intentional failing assertions (e.g. asserting empty category lists or mismatched category strings) to confirm test failure detection.
2. **GREEN Stage**: Implementation logic and assertions were aligned to pass cleanly.
