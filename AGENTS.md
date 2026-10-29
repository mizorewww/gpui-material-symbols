# Project identity and compatibility

This repository implements Google Material Symbols for **Longbridge GPUI Kit**:
https://github.com/longbridge/gpui-kit, tracking its `main` branch.

- Use the `gpui-kit` Git dependency with `branch = "main"`; commit Cargo.lock.
- Import the public `gpui_kit` facade and its `component` / `assets` modules.
- Let Kit choose its matching GPUI backend; do not add a competing GPUI package.
- Implement `IconNamed` for symbols and configured icons so standard Kit icon slots work.
- The `showcase` example uses Kit's `application`, `init`, `open_window`, and theme.
- The `material_symbols` consumer belongs to the independent `browser` workspace
  package in this repository and uses the sibling `gpui-kit-material` components
  and its matching GPUI backend. Reusable Material controls belong in that
  sibling, never the browser itself. Keep consumer dependencies in `browser`
  or `web`, separate from the root library and its default Kit adapter.
- Dependency direction is consumer → Material → symbols. The root symbols
  library must not depend on Material or its consumer packages. The
  `material-browser` feature belongs only to `gpui-material-symbols-browser`;
  run it with `cargo run -p gpui-material-symbols-browser --bin material_symbols`.
- On a Kit update, inspect current main APIs, run integration tests and rebuild the showcase.
- Do not substitute a similarly named repository or reuse its compatibility results.

Preserve the complete Google snapshot union: variable-font names plus SVG-only
names, all three styles and every official supplemental variant. Record source
commits/checksums, explicit variation limits, and actual validation results.
Keep changes in focused atomic commits.
