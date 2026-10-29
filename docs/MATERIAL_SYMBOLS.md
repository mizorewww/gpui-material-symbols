# Material Symbols browser

A native GPUI icon browser for this crate's complete catalog, built with
components from [`gpui-kit-material`](https://github.com/mizorewww/gpui-kit-material)
on the same `gpui-pre` backend as Longbridge GPUI Kit. No WebView is used.

```sh
cargo run --no-default-features --features material-browser --example material_symbols
```

![Icon browser in the light theme](screenshots/material-symbols-light.png)

[Dark theme](screenshots/material-symbols-dark.png) · [Details](screenshots/material-symbols-detail.png) · [Search](screenshots/material-symbols-search.png)

## Features

- All 4,359 names in a virtualized grid grouped by category, including names
  that Google Fonts' public metadata does not list (under Other).
- Search by name or tag, sorted by popularity or name.
- Outlined, Rounded and Sharp, with fill, weight, grade and optical size.
- A details panel with name, code point, category and tags; SVG and PNG export
  at a chosen size and color; and copyable Rust that reproduces the selection.
- Light and dark themes, starting from the system appearance.
- The app bar is the window's title bar, drawn in the app's own colors: macOS
  keeps its traffic lights inside it, Windows and Linux get the kit's caption
  buttons, and dragging or double-clicking it moves or zooms the window.

SVG-only names show an unavailable state for axis combinations Google does not
publish; the browser never substitutes a different variant.

## Components

Every control comes from `gpui-kit-material`: the search field, connected
button groups (style, sort), switch, sliders, buttons, side sheet, caption
buttons and title-bar controls. Behavior and accessibility come from GPUI Base.

## Verification and captures

```sh
cargo clippy --no-default-features --features material-browser --all-targets -- -D warnings
cargo test --no-default-features --features material-browser --example material_symbols
cargo run --no-default-features --features material-browser --example material_symbols -- --capture target/captures
```

The capture command renders the light, dark, details and search states through
GPUI's native headless renderer; on macOS this needs a Metal device. Headless
captures show the window content only, without the system's traffic lights.
Interaction tests dispatch real clicks and input to open and close details,
change the sort order, search with no results and clear the search, and check
that resizing keeps the scroll position. Export tests decode the PNG and check
its dimensions and pixel colors.

## Data

- Symbol shapes: this crate's pinned Google Material Symbols snapshot
  (Apache-2.0).
- Categories, tags and popularity: Google Fonts
  `https://fonts.google.com/metadata/icons?key=material_symbols&incomplete=true`,
  fetched 2026-10-29. Refresh with `python3 scripts/update-symbol-catalog.py`.
