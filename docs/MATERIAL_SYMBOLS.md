# Material Symbols browser

A native GPUI icon browser for this crate's complete catalog, built with
components from [`gpui-kit-material`](https://github.com/mizorewww/gpui-kit-material)
on the same `gpui-pre` backend as Longbridge GPUI Kit. No WebView is used.

The native executable lives in the independent `gpui-material-symbols-browser`
workspace package (`browser/`). Its default `material-browser` feature embeds
assets; it is no longer a feature or example target of the root symbols library.
This keeps the consumer → Material → symbols dependency direction acyclic.

```sh
cargo run -p gpui-material-symbols-browser --bin material_symbols
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

Every control comes from `gpui-kit-material`: the title bar and its caption
buttons, search field, connected button groups (style, sort), switch, sliders,
buttons, side sheet and snackbar. Behavior and accessibility come from GPUI
Base.

## Verification and captures

```sh
cargo clippy -p gpui-material-symbols-browser --all-targets -- -D warnings
cargo test -p gpui-material-symbols-browser --bin material_symbols
cargo run -p gpui-material-symbols-browser --bin material_symbols -- --capture target/captures
```

The capture command renders the light, dark, light and dark details, and
search states through GPUI's native headless renderer; on macOS this needs a
Metal device. Headless captures show the window content only, without the
system's traffic lights. Interaction tests dispatch real clicks and input:
opening and closing details, choosing the A–Z sort, switching style and fill,
toggling the theme, searching with no results and clearing the search. They
also check that the open details sheet takes columns from the grid and that
resizing keeps the scroll position; the kit tests its title bar and snackbar.
Export tests decode the PNG and check its dimensions and pixel colors.

## Data

- Symbol shapes: this crate's pinned Google Material Symbols snapshot
  (Apache-2.0).
- Categories, tags and popularity: Google Fonts
  `https://fonts.google.com/metadata/icons?key=material_symbols&incomplete=true`,
  fetched 2026-10-29. Refresh with `python3 scripts/update-symbol-catalog.py`.
