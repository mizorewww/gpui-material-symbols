# Maintenance

## Validation

```sh
python3 scripts/sync_assets.py --verify     # local checksums and coverage; no network
cargo fmt --all -- --check
cargo test --no-default-features --features embed-assets  # catalog, every variant, raster parity
cargo test --no-default-features            # assets supplied at runtime
cargo test --no-default-features --features outlined --lib  # one style embedded, others supplied
cargo test --test integration               # Longbridge GPUI Kit slots and assets
cargo check --features showcase --example showcase
cargo test -p gpui-material-symbols-browser --bin material_symbols
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The workspace patches `gpui-kit` to the sibling's vendored copy for the
browser and web consumers. The published library must also build without
them; [ci.yml](ci.yml)'s `unpatched` job checks that from a consumer crate
outside the workspace, as in [Publishing](#publishing).

The catalog tests check the complete name union, all three fonts' axes, four
axis sets for every variable symbol (51,648 SVG generations), all 27,720
supplemental SVG variants, real geometry changes, coordinates, invalid input
and asset-source composition. Record actual results in
[validation.md](validation.md).

## Updating Longbridge GPUI Kit

The library depends on `gpui-kit` 0.9 from crates.io. In this workspace both
that and the consumers' Git `main` dependency are patched to
`../gpui-kit-material/vendor/gpui-kit`, so they resolve to one Kit; the
vendored copy must keep a 0.9 version for the crates.io patch to apply. A new
Kit minor release (0.10) is a breaking dependency change and needs a new
minor release of this crate.

Inspect the Kit APIs this crate uses (`IconNamed`, `component::Icon`, `assets`,
`application`, `init`, `open_window`), then run the validation commands and
rebuild the showcase. The independent `browser` package and the root
`showcase-capture` feature pin `gpui-pre`/`gpui-pre-platform` exactly; keep those
pins equal to the version Kit and `gpui-kit-material` use, or the consumers stop
resolving or pull in a second backend. The root symbols library must not depend
on `gpui-kit-material`: Material itself consumes this library.

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
cargo test --no-default-features --features embed-assets
```

The extractor reads Git objects directly, so the millions of SVG files are
never checked out. With the blob-less fetch above, `git cat-file` downloads
the roughly 18,000 needed SVG blobs on demand, one request at a time; expect
this step to take a while. `--verify` fails until both scripts have run for the same
revision. The scripts write the fonts and the SVG pack into their data crates
under `crates/`, Google's `LICENSE` next to them, and the other files into
`assets/`; `assets/manifest.json` records every file's SHA-256 and the data
files' sizes, which `build.rs` compiles in for `Asset::provide`. Review the
manifest, the coverage counts and the snapshot counts in `tests/catalog.rs`
before committing. New asset bytes need new versions of the data crates and
of this crate, whose dependencies on them are exact (`=x.y.z`). Cargo builds never run these
scripts or touch the network for assets.

## Browser catalog metadata

`browser/src/assets/catalog.tsv` holds categories, tags and
popularity from Google Fonts' live metadata endpoint, which is not pinned:

```sh
python3 scripts/update-symbol-catalog.py
```

## Screenshots

```sh
cargo run -p gpui-material-symbols-browser \
  --bin material_symbols -- --capture target/captures
cargo run --features showcase-capture --example showcase -- \
  --capture docs/screenshots/showcase.png
```

Copy the browser captures to `docs/screenshots/material-symbols-*.png`. Both
use GPUI's native headless renderer; on macOS this needs a Metal device.

## Publishing

Five crates are published: the four dependency-free data crates in `crates/`
(`gpui-material-symbols-outlined`, `-rounded`, `-sharp`, `-supplemental`,
each below crates.io's 10 MB compressed limit) and the root
`gpui-material-symbols`, which depends on them optionally at exact versions.
`browser` and `web` are `publish = false`.

```sh
for c in outlined rounded sharp supplemental; do
  cargo publish --dry-run -p gpui-material-symbols-$c
done
cargo package -p gpui-material-symbols --no-verify --list  # no docs/ or screenshots
```

Publish the data crates first, wait until each appears in the index, then
`cargo publish -p gpui-material-symbols` (its verification build resolves
the data crates from crates.io) and tag `vX.Y.Z`. Afterwards, build a scratch
consumer outside this workspace that depends on the published version with no
patches.

## CI

[ci.yml](ci.yml) is a ready GitHub Actions workflow. Move it to
`.github/workflows/ci.yml` once a token with the `workflow` scope is available.
