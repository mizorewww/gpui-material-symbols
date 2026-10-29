# Maintenance

## Validation

```sh
python3 scripts/sync_assets.py --verify     # local checksums and coverage; no network
cargo fmt --all -- --check
cargo test --no-default-features            # catalog, every variant, raster parity
cargo test --test integration               # Longbridge GPUI Kit slots and assets
cargo check --features showcase --example showcase
cargo test --no-default-features --features material-browser --example material_symbols
cargo clippy --all-targets --all-features -- -D warnings
```

The catalog tests check the complete name union, all three fonts' axes, four
axis sets for every variable symbol (51,648 SVG generations), all 27,720
supplemental SVG variants, real geometry changes, coordinates, invalid input
and asset-source composition. Record actual results in
[validation.md](validation.md).

## Updating Longbridge GPUI Kit

`Cargo.lock` pins the Kit `main` commit that was last validated.

```sh
cargo update -p gpui-kit
```

Inspect current Kit main APIs (`IconNamed`, `component::Icon`, `assets`,
`application`, `init`, `open_window`), then run the validation commands and
rebuild the showcase. The `material-browser` and `showcase-capture` features
pin `gpui-pre`/`gpui-pre-platform` exactly; keep that pin equal to the version
Kit and `gpui-kit-material` use, or those features stop resolving or pull in a
second backend.

## Updating Google assets

Pin a full commit SHA. Sync the fonts first, then extract the SVG-only
variants from the same commit:

```sh
SHA=<full-40-character-commit-sha>
python3 scripts/sync_assets.py --revision "$SHA"

git init .research/material-design-icons
git -C .research/material-design-icons fetch --depth 1 --filter=blob:none \
  https://github.com/google/material-design-icons "$SHA"
python3 scripts/sync_supplemental.py .research/material-design-icons

python3 scripts/sync_assets.py --verify
cargo test --no-default-features
```

The extractor reads Git objects directly, so the millions of SVG files are
never checked out. With the blob-less fetch above, `git cat-file` downloads
the roughly 18,000 needed SVG blobs on demand, one request at a time; expect
this step to take a while. `--verify` fails until both scripts have run for the same
revision. Review `assets/manifest.json`, the coverage counts and the snapshot
counts in `tests/catalog.rs` before committing. Cargo builds never run these
scripts or touch the network for assets.

## Browser catalog metadata

`examples/material_symbols/assets/catalog.tsv` holds categories, tags and
popularity from Google Fonts' live metadata endpoint, which is not pinned:

```sh
python3 scripts/update-symbol-catalog.py
```

## Screenshots

```sh
cargo run --no-default-features --features material-browser \
  --example material_symbols -- --capture target/captures
cargo run --features showcase-capture --example showcase -- \
  --capture docs/screenshots/showcase.png
```

Copy the browser captures to `docs/screenshots/material-symbols-*.png`. Both
use GPUI's native headless renderer; on macOS this needs a Metal device.

## CI

[ci.yml](ci.yml) is a ready GitHub Actions workflow. It needs a
`DEPENDENCY_READ_TOKEN` secret that can read the private `gpui-kit-material`
repository. Move it to `.github/workflows/ci.yml` once a token with the
`workflow` scope is available.
