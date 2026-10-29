# Validation record — Longbridge GPUI Kit main

Validated locally on macOS on 2026-10-29 using rustc/Cargo 1.99.1.
This replaces the earlier validation against an unrelated similarly named toolkit.

Target: https://github.com/longbridge/gpui-kit, branch `main`, commit
`d983af26269fca2200600aced3e3f03849e60749` (also confirmed against remote main).
Kit's own backend is `gpui-pre 0.3.10`; no competing GPUI distribution is present.

| Executed check | Result |
| --- | --- |
| `python3 scripts/sync_assets.py --verify` | All resource SHA-256 values match; Google master still matches the pinned snapshot |
| `cargo fmt --all -- --check` and `git diff --check` | Passed |
| `cargo check --locked --no-default-features --all-targets` | Passed; standalone SVG functionality remains independent of Kit |
| `cargo test --locked --all-features --tests` | 10 tests passed: 7 catalog, 2 Longbridge integration, 1 raster comparison |
| `cargo check --locked --features showcase --example showcase` | Passed against Longbridge main |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | Passed |
| `cargo build --locked --features showcase --example showcase` | Passed, including native linking |
| Native showcase smoke check | Launched, remained alive for five seconds, no stderr, then terminated by the check |
| Dependency graph audit | `gpui-kit` and its component/assets crates come from the same main commit, with `gpui-pre 0.3.10` |

Coverage was rerun in this corrected build: all 4,304 variable names × three
styles × four representative axis sets (51,648 SVG generations), all 27,720
supplemental SVG variants, and the union of 4,359 names. Six official raster
fixtures verify shape and placement.

The integration tests exercise Kit's `IconNamed`, conversion into component
`Icon`, `Button::icon`, `PopupMenuItem::icon`, explicit `.component()` and
`.embedded()`, and composition with real `gpui_kit::assets::Assets`.
The native smoke check verifies startup, not exhaustive manual interaction or
accessibility review.

GitHub Actions has not run. The authenticated token still lacks `workflow`
scope; the usable workflow remains in `docs/ci.yml` until authorization completes.
All results reported here are local results.
