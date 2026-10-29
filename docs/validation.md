# Validation record

Validated locally on macOS on 2026-10-29 with rustc 1.99.1 and Cargo 1.99.1.

| Check | Result |
| --- | --- |
| `python3 scripts/sync_assets.py --verify` | All pinned upstream asset SHA-256 hashes match |
| `cargo fmt --all -- --check` | Passed |
| `git diff --check` | Passed |
| `cargo test --no-default-features` (catalog suite) | 7 tests passed; 51,648 font SVG generations and all 27,720 supplemental variants |
| `cargo test --no-default-features --test rendering` | Passed; six raster comparisons against official SVG fixtures |
| `cargo test --test integration` | Passed with the pinned GPUI 1.18.0 dependency; fallback resources and path errors verified |
| `cargo check --features showcase --example showcase` | Passed with GPUiKit 0.9.0 and GPUI 1.18.0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passed |
| `cargo build --features showcase --example showcase` | Passed, including native linking |
| Native showcase smoke check | Executable launched, stayed alive for five seconds, emitted no stderr, then was terminated by the check |
| GitHub repository | `private: true`, default branch `main`, detected license `Apache-2.0` |

The native smoke check verifies startup, not an exhaustive manual interaction/accessibility review. Pixel comparisons independently check actual icon geometry and placement. The core and runtime integration tests do not require a display server.

GitHub Actions has not run: the authenticated OAuth token lacks `workflow` scope. The ready-to-use workflow is retained in `docs/ci.yml` rather than the restricted workflow directory. The reported results above are local results.
