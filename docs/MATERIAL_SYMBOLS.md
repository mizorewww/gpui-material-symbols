# Material Symbols browser

A native GPUI reference of [Google Fonts Icons](https://fonts.google.com/icons),
owned by `gpui-material-symbols`, using its complete catalog and reusable
components from the sibling `gpui-kit-material` repository.
The reference desktop viewport is 1440 × 1000 logical pixels. No WebView is used.

Cargo fetches `gpui-kit-material` from Git (its `main` branch, pinned by
`Cargo.lock`), so no sibling checkout is required:

```sh
cargo run --no-default-features --features material-browser --example material_symbols
```

The example uses this crate without its optional `gpui-kit` adapter;
its `AssetSource` renders symbols directly into the Material kit's GPUI version. The
symbol crate's own optional adapter follows `longbridge/gpui-kit` main.

![Native GPUI light theme](screenshots/material-symbols-light.png)

[Dark theme](screenshots/material-symbols-dark.png) · [Details sheet](screenshots/material-symbols-detail.png) · [Search](screenshots/material-symbols-search.png)

## Interactions

- All 4,359 catalog names, virtualized and grouped by category, including names
  missing from Google Fonts' public search metadata (under Other).
- Name/tag search, popularity/name sorting, category selection and reset.
- Outlined, Rounded and Sharp; fill, weight, grade and optical-size controls.
- Light/dark themes, collapsible filters, animated non-modal details sheet.
- Preview size/color, SVG and PNG file export with those dimensions/colors.
- Copyable Rust/GPUI API reflecting the selected style and all four axes.

Web, Android and Apple installation tabs are replaced with GPUI instructions.
The legacy Material Icons family is outside the Material Symbols catalog.
SVG-only names show an explicit unavailable state for unsupported axis
combinations; the app does not silently substitute a different icon variant.
The metadata is a snapshot, so live popularity/new releases can change upstream.

## Components

The switch, slider, search field, buttons and side sheet come from
`gpui-kit-material` (`crates/material/src/components/controls.rs`) and appear in
that kit's `examples/gallery.rs`. The navigation rail, sort menu, filter chips,
cards and snackbar are still drawn inline in this example and should move to
the kit. Geometry was checked against [md3.io](https://md3.io/components):
52 × 32 switch with 16/24 thumb, 56-high search field, pill buttons, and a standard
side sheet. `MaterialSlider` offers the classic Google Fonts appearance and
an expressive 16-high track with a 4 × 44 handle. Behavior and accessibility
come from GPUI Base. Colors use semantic roles, geometry uses rems, switch/sheet
motion uses the kit's retargetable springs and respects reduced motion.

The expressive slider is the single-value variant; range sliders, tick marks and
value bubbles are not provided by this component. The browser uses Classic.

## Verification and captures

```sh
cargo fmt --all --check
cargo clippy --no-default-features --features material-browser --all-targets -- -D warnings
cargo test --no-default-features --features material-browser --example material_symbols
cargo run --no-default-features --features material-browser --example material_symbols -- --capture /tmp/material-symbols
```

The capture command renders light, dark, details and search states through the
native GPUI headless renderer. On macOS this requires a working Metal device.
Interaction tests exercise actual input/click dispatch: sort selection,
opening/closing details, empty results, clearing search and keeping the scroll
position when the grid reflows. Export tests decode the PNG and check dimensions and pixel colors.

## Assets and provenance

- Symbol shapes: this crate's pinned Google Material Symbols snapshot;
  Apache 2.0, with its license retained in that repository.
- Categories, tags and popularity: Google Fonts
  `https://fonts.google.com/metadata/icons?key=material_symbols&incomplete=true`,
  fetched 2026-10-29. Refresh using `python3 scripts/update-symbol-catalog.py`.
  Whitespace is normalized so tags cannot corrupt TSV columns.
- Google Sans Flex: [google/fonts](https://github.com/google/fonts/tree/main/ofl/googlesansflex),
  distributed with `assets/OFL.txt` under SIL Open Font License 1.1.
- Google Fonts logo: SVG from the reference site's header, retained solely to
  reproduce the requested reference example. Google branding is not this
  project's identity and is not covered by its Apache-2.0 license. This
  example is not affiliated with or endorsed by Google.

This repository remains Apache 2.0; the reusable Material kit remains MIT.
Third-party assets retain their own terms.
