# Validation record — Longbridge GPUI Kit main

Validated locally on macOS on 2026-10-29 with rustc/Cargo 1.99.1, at the
commit that adds this record. It replaces the earlier record, which predated
the `material-browser` example and its dependencies.

Pinned by `Cargo.lock`:

- Longbridge GPUI Kit `main` at `d983af26269fca2200600aced3e3f03849e60749`,
  with its own backend `gpui-pre 0.3.10` (the only GPUI package in the graph).
- `gpui-kit-material` `main` at `0dd2d9f22baec8fa21f7d534c55da791cc2c5846`,
  fetched from Git; no sibling checkout is required. It supplies the
  browser's title bar and snackbar.
- Google `material-design-icons` at `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`
  for fonts and supplemental SVGs (`assets/manifest.json`).

| Executed check | Result |
| --- | --- |
| `python3 scripts/sync_assets.py --verify` (also with `python3 -O`) | All ten SHA-256 values, matching font/SVG revisions, sorted in-bounds SVG index and coverage counts verified. Local only; no comparison against Google master |
| `cargo fmt --all -- --check`, `git diff --check` | Passed |
| `cargo test --locked --no-default-features` | 8 passed: 7 catalog, 1 raster comparison |
| `cargo test --locked --test integration` | 2 passed |
| `cargo test --locked --doc` | 12 README examples passed (6 in `README.md`, 6 in `README.zh-CN.md`) |
| `cargo check --locked --features showcase --example showcase` | Passed |
| `cargo test --locked --no-default-features --features material-browser --example material_symbols` | 9 passed: 3 catalog, 2 export, 4 interaction |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | Passed |
| `cargo build --locked --features showcase --example showcase` | Passed, including native linking |
| `material_symbols --capture` | Rendered through the Metal headless renderer; the five browser captures in `docs/screenshots/` were regenerated from this output. `showcase.png` was not recaptured for this record |
| `cargo metadata --locked` in a fresh clone without `../gpui-kit-material` | Resolved |

Coverage: the union of 4,359 names; all 4,304 variable names × three styles ×
four representative axis sets (51,648 SVG generations); all 27,720
supplemental SVG variants. Six official raster fixtures verify shape and
placement.

The integration tests exercise Kit's `IconNamed`, conversion into component
`Icon`, `Button::icon`, `PopupMenuItem::icon`, `.component()`, `.embedded()`,
and composition with real `gpui_kit::assets::Assets`, including asset listing
for unrelated prefixes. Browser interaction tests dispatch real clicks and
input for details, sort, style, fill, theme and search, and check the details
sheet's effect on the grid and scroll preservation on resize. The kit's own
gallery, which draws the same title bar, was launched natively and stayed alive
for six seconds; the browser window itself was not relaunched. Neither covers
exhaustive manual interaction or accessibility review.

Not exercised: dragging or double-clicking the browser's custom title bar,
and the caption buttons on Windows and Linux, which need a real desktop
session. Not run: GitHub Actions (the workflow in `ci.yml` is not enabled), Linux or
Windows builds, and `scripts/sync_supplemental.py` against the upstream
repository (only its missing-revision error path was exercised).
