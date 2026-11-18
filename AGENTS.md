# Project identity and compatibility

This repository implements Google Material Symbols for **Longbridge GPUI Kit**:
https://github.com/longbridge/gpui-kit.

- The published root library depends on `gpui-kit` from crates.io (0.10) and
  must build without any `[patch]`; commit Cargo.lock. The `browser` and `web`
  consumers (`publish = false`) use crates.io `gpui-kit-material` 0.2 and
  `gpui-kit` 0.10.0. Each is a Cargo workspace of its own with its own
  Cargo.lock, so the root can move to a new Kit before Material does (one graph
  cannot hold two Kit releases). They patch only Symbols (to this checkout) and
  `gpui-kit` (to gpui-kit-material's vendored 0.10.0 at a pinned revision, which
  keeps WASM single-threaded so the web build stays on stable).
- The fonts and SVG pack live in the dependency-free data crates under
  `crates/` (crates.io limits each package to 10 MB compressed); keep each
  file in one place and its size and SHA-256 in `assets/manifest.json`.
- Import the public `gpui_kit` facade and its `component` / `assets` modules.
- Let Kit choose its matching GPUI backend; do not add a competing GPUI package.
- Implement `IconNamed` for symbols and configured icons so standard Kit icon slots work.
- The `showcase` example uses Kit's `application`, `init`, `open_window`, and theme.
- The `material_symbols` consumer belongs to the independent `browser` workspace
  in this repository and uses the `gpui-kit-material` components
  and its matching GPUI backend. Reusable Material controls belong in that
  sibling, never the browser itself. Keep consumer dependencies in `browser`
  or `web`, separate from the root library and its default Kit adapter.
- Dependency direction is consumer → Material → symbols. The root symbols
  library must not depend on Material or its consumer packages. The
  `material-browser` feature belongs only to `gpui-material-symbols-browser`;
  run it with `cargo run --manifest-path browser/Cargo.toml --bin material_symbols`.
- On a Kit update, inspect the APIs used, run integration tests and rebuild the showcase.
- Do not substitute a similarly named repository or reuse its compatibility results.

Preserve the complete Google snapshot union: variable-font names plus SVG-only
names, all three styles and every official supplemental variant. Record source
commits/checksums, explicit variation limits, and actual validation results.
Keep changes in focused atomic commits.
