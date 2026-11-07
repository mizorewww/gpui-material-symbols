# gpui-material-symbols-supplemental

Google's official SVGs for the Material Symbols that have no variable-font glyph, as a byte slice for
[`gpui-material-symbols`](https://crates.io/crates/gpui-material-symbols).
You normally do not depend on this crate directly: enable the `supplemental` feature
(or the default `embed-assets`) of `gpui-material-symbols` instead.

```rust
assert_eq!(gpui_material_symbols_supplemental::FILE_NAME, "supplemental.bin");
assert_eq!(gpui_material_symbols_supplemental::BYTES.len(), 16062753);
```

It is split from the main crate because crates.io limits each package to
10 MB compressed, and the three fonts and the SVG pack together exceed it.

## Provenance

`supplemental.bin` concatenates, unmodified and deduplicated, every SVG under `symbols/web/`
in [google/material-design-icons](https://github.com/google/material-design-icons)
at commit `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7` for the 55 names without a variable-font glyph
(27720 official variants). The index that addresses each SVG,
`supplemental.tsv`, ships in `gpui-material-symbols`.

- Size: 16062753 bytes
- SHA-256: `7c398e5852960744417c59e37ff77991b6056bf85fb47e2a9e447f2f86931cb3`

The same digests are recorded in the main crate's `assets/manifest.json` and
checked by `scripts/sync_assets.py --verify` in the
[repository](https://github.com/mizorewww/gpui-material-symbols).

## License

Material Symbols are Google's work, licensed under Apache-2.0; `LICENSE` is
Google's license file from the same commit. This crate is an independent
packaging, not affiliated with or endorsed by Google.
