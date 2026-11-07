# gpui-material-symbols-sharp

The Sharp Material Symbols variable font, as a byte slice for
[`gpui-material-symbols`](https://crates.io/crates/gpui-material-symbols).
You normally do not depend on this crate directly: enable the `sharp` feature
(or the default `embed-assets`) of `gpui-material-symbols` instead.

```rust
assert_eq!(gpui_material_symbols_sharp::FILE_NAME, "MaterialSymbolsSharp.ttf");
assert_eq!(gpui_material_symbols_sharp::BYTES.len(), 8918100);
```

It is split from the main crate because crates.io limits each package to
10 MB compressed, and the three fonts and the SVG pack together exceed it.

## Provenance

`MaterialSymbolsSharp.ttf` is Google's unmodified `variablefont/MaterialSymbolsSharp[FILL,GRAD,opsz,wght].ttf` from
[google/material-design-icons](https://github.com/google/material-design-icons)
at commit `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`.

- Size: 8918100 bytes
- SHA-256: `1fd72f99ec1f44ec461c74f99341c1dcfa8c140d7087a2902e747d4ee01e379b`

The same digests are recorded in the main crate's `assets/manifest.json` and
checked by `scripts/sync_assets.py --verify` in the
[repository](https://github.com/mizorewww/gpui-material-symbols).

## License

Material Symbols are Google's work, licensed under Apache-2.0; `LICENSE` is
Google's license file from the same commit. This crate is an independent
packaging, not affiliated with or endorsed by Google.
