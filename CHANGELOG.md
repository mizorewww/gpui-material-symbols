# Changelog

All notable changes to `gpui-material-symbols` and its data crates are
recorded here. The project follows [Semantic Versioning](https://semver.org/).

## 0.1.0 — 2026-11-07

First release on crates.io.

- The complete Google Material Symbols snapshot at
  `google/material-design-icons` commit `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`:
  4,359 names (4,304 variable-font symbols and 55 SVG-only symbols with
  27,720 official variants) in the Outlined, Rounded and Sharp styles.
- `Symbol` constants and name lookup; `Icon` with style and the continuous
  FILL, wght, GRAD and opsz axes; SVG output with no system font registration.
- Longbridge GPUI Kit integration (`gpui-kit` feature, default) for
  `gpui-kit` 0.9 from crates.io: the `MaterialSymbols` asset source, `IconNamed`
  for `Symbol` and `Icon`, and `Icon::component`, `Icon::embedded` and
  `Icon::svg`.
- The fonts and the SVG pack are split into four dependency-free data crates,
  `gpui-material-symbols-outlined`, `-rounded`, `-sharp` and `-supplemental`,
  to fit crates.io's package size limit. The `outlined`, `rounded`, `sharp`
  and `supplemental` features embed them one by one; the default
  `embed-assets` feature enables all four. Any asset that is not embedded can
  be supplied at runtime with `Asset::provide`.
- `Asset::feature`, `Asset::crate_name` and `Asset::is_embedded`, and an
  `Error::NotLoaded` message that names the feature to enable.
