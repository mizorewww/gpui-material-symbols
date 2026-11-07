# gpui-material-symbols-rounded

The Rounded Material Symbols variable font, as a byte slice for
[`gpui-material-symbols`](https://crates.io/crates/gpui-material-symbols).
You normally do not depend on this crate directly: enable the `rounded` feature
(or the default `embed-assets`) of `gpui-material-symbols` instead.

```rust
assert_eq!(gpui_material_symbols_rounded::FILE_NAME, "MaterialSymbolsRounded.ttf");
assert_eq!(gpui_material_symbols_rounded::BYTES.len(), 15235612);
```

It is split from the main crate because crates.io limits each package to
10 MB compressed, and the three fonts and the SVG pack together exceed it.

## Provenance

`MaterialSymbolsRounded.ttf` is Google's unmodified `variablefont/MaterialSymbolsRounded[FILL,GRAD,opsz,wght].ttf` from
[google/material-design-icons](https://github.com/google/material-design-icons)
at commit `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`.

- Size: 15235612 bytes
- SHA-256: `7e74e36533ccee12c2b81e75bfcf3ccf123d3bb7e7e46d65d94445f23db0c97c`

The same digests are recorded in the main crate's `assets/manifest.json` and
checked by `scripts/sync_assets.py --verify` in the
[repository](https://github.com/mizorewww/gpui-material-symbols).

## License

Material Symbols are Google's work, licensed under Apache-2.0; `LICENSE` is
Google's license file from the same commit. This crate is an independent
packaging, not affiliated with or endorsed by Google.
