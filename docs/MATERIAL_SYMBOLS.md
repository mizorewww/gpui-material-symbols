# Material Symbols browser

A native GPUI icon browser for this crate's complete catalog, built with
components from [`gpui-kit-material`](https://github.com/mizorewww/gpui-kit-material)
on the same `gpui-pre` backend as Longbridge GPUI Kit. No WebView is used.

```sh
cargo run --no-default-features --features material-browser --example material_symbols
```

![Icon browser in the light theme](screenshots/material-symbols-light.png)

[Dark theme](screenshots/material-symbols-dark.png) · [Details](screenshots/material-symbols-detail.png) · [Details, dark](screenshots/material-symbols-detail-dark.png) · [Search](screenshots/material-symbols-search.png)

## Features

- All 4,359 names in a virtualized grid grouped by category, including names
  that Google Fonts' public metadata does not list (under Other).
- Search by name or tag, sorted by popularity or name.
- Outlined, Rounded and Sharp, with fill, weight, grade and optical size.
- A details panel beside the grid with name, code point, category and tags;
  SVG and PNG export at a chosen size and color, previewed on a white canvas;
  and copyable Rust that reproduces the selection.
- Light and dark themes that follow the system appearance until you choose
  one with the theme button.
- The app bar is the window's title bar, drawn in the app's own colors: macOS
  keeps its traffic lights inside it, Windows and Linux get the kit's caption
  buttons, and dragging or double-clicking it moves or zooms the window.

SVG-only names show an unavailable state for axis combinations Google does not
publish; the browser never substitutes a different variant.

## Components

The search field, connected button groups (style, sort), switch, sliders,
buttons, side sheet, caption buttons and `title_bar_control` come from
`gpui-kit-material`; behavior and accessibility come from GPUI Base. Two
pieces are still written in this example and should move to the kit: the
status snackbar, and the app bar's window chrome (drag-to-move, double-click
zoom and the traffic-light inset).

## Verification and captures

```sh
cargo clippy --no-default-features --features material-browser --all-targets -- -D warnings
cargo test --no-default-features --features material-browser --example material_symbols
cargo run --no-default-features --features material-browser --example material_symbols -- --capture target/captures
```

The capture command renders the light, dark, light and dark details, and
search states through GPUI's native headless renderer; on macOS this needs a
Metal device. Headless captures show the window content only, without the
system's traffic lights. Interaction tests dispatch real clicks and input:
opening and closing details, choosing the A–Z sort, switching style and fill,
toggling the theme, searching with no results and clearing the search. They
also check that only empty app-bar space starts a window drag, that the open
details sheet takes columns from the grid, and that resizing keeps the scroll
position. Export tests decode the PNG and check its dimensions and pixel
colors.

## Data

- Symbol shapes: this crate's pinned Google Material Symbols snapshot
  (Apache-2.0).
- Categories, tags and popularity: Google Fonts
  `https://fonts.google.com/metadata/icons?key=material_symbols&incomplete=true`,
  fetched 2026-10-29. Refresh with `python3 scripts/update-symbol-catalog.py`.
