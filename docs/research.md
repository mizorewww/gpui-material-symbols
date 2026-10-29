# Upstream investigation — Longbridge GPUI Kit main

The integration target is exclusively https://github.com/longbridge/gpui-kit,
branch `main`. Earlier work targeted an unrelated similarly named project;
that integration and its compatibility conclusions have been replaced.

## Kit architecture and integration

Inspected commit `d983af26269fca2200600aced3e3f03849e60749` on 2026-10-29.
The application-facing package is `gpui-kit`, currently version 0.8.1 on main.
Use `gpui_kit::*`, `gpui_kit::component`, and `gpui_kit::assets`. The Kit's
workspace pins the matching `gpui-pre-*` snapshots to 0.3.10; this library does
not pin or import a separate GPUI distribution.

`component::IconNamed` supplies an asset path. Kit blanket conversions accept
any implementation as a themed `component::Icon`, a button icon or a menu icon.
Both `Symbol` and configured `Icon` implement this trait. `.component()` gives
an explicit component; `.embedded()` uses main's `Icon::data` API for callers
who do not want an application asset source. Path-based components participate
in Kit's icon resolver; embedded data intentionally bypasses it.

`MaterialSymbols.with_fallback(gpui_kit::assets::Assets)` composes the generated
SVG namespace with the default component assets. The native example uses
`gpui_kit::application`, `init` and `open_window` so Base Root and Kit theme
initialization are installed correctly.

The dependency declares `branch = "main"`; Cargo.lock records the tested Git
commit. Updating with `cargo update -p gpui-kit` advances the lock to current
main and must be followed by integration tests and a showcase build. A Git
branch dependency is not an automatic daily updater.

Sources: `crates/kit/src/lib.rs`, `crates/component/src/icon.rs`,
`crates/component/src/button/button_icon.rs`, `crates/assets/src/icon.rs`,
`examples/hello_world/src/main.rs`, and the official assets guide at
https://github.com/longbridge/gpui-kit/blob/main/website/docs/assets.md .
Context7 documentation was cross-checked against these current source files.

## Google resource coverage

Google source: https://github.com/google/material-design-icons at
`10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`. Downloaded Git objects, complete TTFs
and codepoint catalogs. Sparse checkout avoids expanding about 4.87 million
files. All resources are pinned and checksummed in `assets/manifest.json`.

All three font catalogs contain 4,304 names including aliases. The independent
`symbols/web` tree contains 4,157 names: 55 absent from fonts, while 202 font
names are absent from the SVG tree. The union is therefore 4,359 names.
The 55 supplemental names have 27,720 SVG files: 3 styles × 7 weights × 3 grades
× 2 fills × 4 optical sizes. Every variant is embedded byte-for-byte in a
deduplicated binary pack. `assets/svg-names.txt` records the upstream tree names.

The official guide https://developers.google.com/fonts/docs/material_symbols
mentions ROND in its online API, but the three pinned repository fonts expose
only FILL, GRAD, opsz and wght. Tests inspect actual fvar metadata. No ROND or
legacy Material Icons Two Tone support is claimed.

The variable renderer uses ttf-parser 0.25.1, verified against its official
source, to set continuous variation coordinates and extract glyph outlines.
Coordinates use the 960-unit design square, not the 1056-unit font ascender.
Six official SVG fixtures independently check rasterized geometry and placement.
Some glyphs intentionally share geometry across styles.

## Resource tradeoffs

Full offline coverage embeds approximately 51 MB of raw font and SVG data.
There is no per-icon subsetting. Font glyphs support continuous axes; SVG-only
names support only their exact official discrete variants and return explicit
errors for other combinations. No silent rounding or fabricated interpolation.
The renderer does not register system fonts or maintain an unbounded global
cache; Kit/GPUI owns rendering caches. Future upstream additions require a
resource refresh and a new coverage audit.
