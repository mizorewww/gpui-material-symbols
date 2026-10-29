# Material Symbols native browser

This workspace consumer owns the `material_symbols` executable. The symbols
library has no dependency on Material or this package, allowing Material to
consume Symbols without a reverse dependency edge.

```sh
cargo run -p gpui-material-symbols-browser --bin material_symbols
cargo test -p gpui-material-symbols-browser --bin material_symbols
cargo run -p gpui-material-symbols-browser --bin material_symbols -- --capture target/captures
```

The default `material-browser` feature embeds Symbols assets for offline native
use. The former root-library `--features material-browser --example
material_symbols` invocation is replaced by the package/bin command above.
The native consumer and `../web` share `src/main.rs`; Web selects its own runtime
asset-loading dependencies and does not enable this package's embedding feature.

See [the browser guide](../docs/MATERIAL_SYMBOLS.md) and
[maintenance](../docs/maintenance.md) for the shared catalog and validation entry
points. The workspace root owns dependency patches; keep its Material, Symbols,
Kit and GPUI sources aligned rather than adding patches here.
