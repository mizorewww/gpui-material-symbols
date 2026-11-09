# gpui-material-symbols

**English** | [简体中文](README.zh-CN.md)

Every Google Material Symbol, ready to use in
[Longbridge GPUI Kit](https://github.com/longbridge/gpui-kit) applications:
4,359 icon names in the Outlined, Rounded and Sharp styles, with continuously
adjustable fill, weight, grade and optical size. `Symbol` implements Kit's
`IconNamed`, so icons drop straight into buttons, menus and every other
standard icon slot.

**Try the icon browser in your browser: <https://m3symbols.mizore.blog>**
(WebAssembly, about 8 MB on the first visit).

## Run the examples

```sh
git clone https://github.com/mizorewww/gpui-material-symbols
cd gpui-material-symbols

# Icon browser for the whole catalog (built with gpui-kit-material)
cargo run -p gpui-material-symbols-browser --bin material_symbols

# Material Symbols in Longbridge GPUI Kit components
cargo run --features showcase --example showcase

# Export one SVG from the command line
cargo run --no-default-features --features embed-assets --example export -- home rounded > home.svg
```

The library itself builds against `gpui-kit` 0.9 from crates.io with no
patches. The browser and web consumers take `gpui-kit-material` 0.1 and
`gpui-kit` 0.9.1 from crates.io. The workspace patches Symbols to this checkout,
and `gpui-kit` to gpui-kit-material's vendored copy of the same 0.9.1 source at
a pinned revision, whose manifest keeps WebAssembly single-threaded: published
Kit builds its web platform multithreaded, which needs a nightly toolchain.

The browser also builds for the web; see the [web preview guide](docs/web.md).

## Screenshots

The native browser is the independent `gpui-material-symbols-browser` workspace
package. Its default `material-browser` feature embeds the assets. The former
root command `--features material-browser --example material_symbols` is replaced
by `-p gpui-material-symbols-browser --bin material_symbols`; the root library no
longer depends on Material, which can now consume Symbols without a reverse edge.
The `showcase` and `export` examples keep their existing entry points.

### Icon browser (`material_symbols`)

Search all 4,359 icons, switch style, adjust fill, weight, grade and optical
size, export SVG or PNG, and copy the matching Rust code. The theme follows the
system's light or dark appearance until you pick one.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screenshots/material-symbols-dark.png">
  <img src="docs/screenshots/material-symbols-light.png" alt="Icon browser: app bar with search and sort, a toolbar for style and axes, and a grid of UI action icons">
</picture>

The details panel, with export and the generated Rust code:

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screenshots/material-symbols-detail-dark.png">
  <img src="docs/screenshots/material-symbols-detail.png" alt="Details panel for the Search icon with export buttons and generated Rust code">
</picture>

Searching for "home":

![Search results for "home"](docs/screenshots/material-symbols-search.png)

### Kit integration (`showcase`)

All three styles in Kit `Button`s, the continuous fill axis, and an embedded
SVG icon.

![Showcase example: Kit buttons in three styles, a row of hearts with increasing fill, and a button with an embedded SVG icon](docs/screenshots/showcase.png)

## Features

- **Complete:** the full union of Google's variable-font names and SVG-only
  names, pinned to an exact upstream commit with recorded checksums.
- **Offline:** with the default `embed-assets` feature, the official fonts and
  SVGs are embedded; apps never download anything or register system fonts.
- **Validated:** out-of-range axes and unsupported variants are errors, never
  silently substituted icons.

| Axis | Range |
| --- | --- |
| FILL | `0..=1` |
| wght (weight) | `100..=700` |
| GRAD (grade) | `-50..=200` |
| opsz (optical size) | `20..=48` |

## Installation

```toml
[dependencies]
gpui-material-symbols = "0.1"
```

The crate supports [`gpui-kit`](https://crates.io/crates/gpui-kit) 0.9 from
crates.io and re-exports that exact Kit as `gpui_material_symbols::kit`. Using
the re-export keeps the types consistent; you do not need to declare GPUI
yourself. If your application patches `gpui-kit` (for example to Kit's Git
`main`), patch the crates.io source too so that both resolve to one Kit.

### Features

| Feature | Default | What it does |
| --- | --- | --- |
| `gpui-kit` | yes | `MaterialSymbols` asset source, `IconNamed`, `Icon::component` / `embedded` / `svg` |
| `embed-assets` | yes | Enables `outlined`, `rounded`, `sharp` and `supplemental` |
| `outlined` | via `embed-assets` | Embeds the Outlined font (10.7 MB) |
| `rounded` | via `embed-assets` | Embeds the Rounded font (15.2 MB) |
| `sharp` | via `embed-assets` | Embeds the Sharp font (8.9 MB) |
| `supplemental` | via `embed-assets` | Embeds the SVGs of the 55 SVG-only symbols (16.1 MB) |
| `gpui-fast` | no | Selects Kit's `gpui-fast` engine |

The fonts and the SVG pack are too large for one crates.io package, so each
lives in its own dependency-free data crate:
[`gpui-material-symbols-outlined`](https://crates.io/crates/gpui-material-symbols-outlined),
[`-rounded`](https://crates.io/crates/gpui-material-symbols-rounded),
[`-sharp`](https://crates.io/crates/gpui-material-symbols-sharp) and
[`-supplemental`](https://crates.io/crates/gpui-material-symbols-supplemental).
The features above pull them in; you never depend on them directly. Every
style and every symbol is always in the catalog: an asset whose feature is off
returns `Error::NotLoaded` until you supply it at runtime (see below).

For example, embed only the Outlined style:

```toml
[dependencies]
gpui-material-symbols = { version = "0.1", default-features = false, features = ["gpui-kit", "outlined"] }
```

## Usage

### Register the assets

Combine Material Symbols with Kit's own assets at startup:

```rust,no_run
use gpui_material_symbols::{kit, MaterialSymbols};

kit::application()
    .with_assets(MaterialSymbols.with_fallback(kit::assets::Assets))
    .run(|cx| {
        kit::init(cx);
        // kit::open_window(...)
    });
```

### Use icons in Kit components

A `Symbol` or a configured `Icon` works anywhere Kit accepts an icon:

```rust
use gpui_material_symbols::{kit, Style, Symbol};
use kit::component::{button::Button, menu::PopupMenuItem};

let home = Button::new("home").icon(Symbol::HOME);
let favorite = Button::new("favorite")
    .icon(Symbol::FAVORITE.icon().style(Style::Rounded).filled(true));
let settings = PopupMenuItem::new("Settings").icon(Symbol::SETTINGS);
```

### Styles and variable axes

```rust
use gpui_material_symbols::{kit, Axes, Style, Symbol};

let icon = Symbol::HOME
    .icon()
    .style(Style::Sharp)
    .axes(Axes::new(0.5, 550.0, 25.0, 32.0).unwrap()); // fill, weight, grade, optical size

let themed: kit::component::Icon = icon.component();          // loaded through the registered assets
let embedded: kit::component::Icon = icon.embedded().unwrap(); // SVG inlined; no asset registration
```

`Axes::new` rejects out-of-range and non-finite values. Optical size only
changes the glyph's design; the rendered size is still set with Kit's
`.size()` and related methods.

### Look up symbols by name

```rust
use gpui_material_symbols::Symbol;

let home: Symbol = "home".parse().unwrap();
assert_eq!(home, Symbol::HOME);
assert!(Symbol::named("does_not_exist").is_none());

let arrows = Symbol::all().filter(|s| s.name().contains("arrow")).count();
assert!(arrows > 0);
```

Constants are the upper-case official names. Names that start with a digit get
a leading underscore, for example `Symbol::_10K`.

### SVG-only symbols

> [!NOTE]
> 55 symbols, such as `Symbol::APP_SPARK`, have official SVGs but no
> variable-font glyph. They support only Google's discrete values: fill 0 or 1,
> weight 100–700 in steps of 100, grade -25, 0 or 200, and optical size 20, 24,
> 40 or 48.

Unsupported combinations are never replaced with a nearby variant:

```rust
use gpui_material_symbols::{Axes, Error, Symbol};

let spark = Symbol::APP_SPARK;
assert!(!spark.is_variable());
assert!(spark.available_variants().all(|icon| icon.is_supported()));

let odd = spark.icon().axes(Axes::new(0.0, 450.0, 0.0, 24.0).unwrap());
assert!(!odd.is_supported());
assert_eq!(odd.to_svg(), Err(Error::UnsupportedVariant));
```

`.component()` loads icons by path, so such a combination renders empty. Use
`.embedded()` to get the error when the icon is built.

### SVG export without Kit

```toml
[dependencies]
gpui-material-symbols = { version = "0.1", default-features = false, features = ["embed-assets"] }
```

```rust
use gpui_material_symbols::{Style, Symbol};

let svg = Symbol::HOME.icon().style(Style::Rounded).to_svg().unwrap();
assert!(svg.starts_with("<svg"));
```

### Load the fonts at runtime

The default `embed-assets` feature builds about 51 MB of fonts and SVGs into
the binary. Leave it out, or enable only some of the asset features, for
example on the web, and supply each missing `Asset` when you have its file;
until then `to_svg` returns `Error::NotLoaded`:

```rust,ignore
use gpui_material_symbols::{Asset, Style};

let font = Asset::Font(Style::Rounded);
assert!(!font.is_embedded()); // the `rounded` feature is off
font.provide(downloaded_bytes)?; // exactly MaterialSymbolsRounded.ttf from Google
assert!(font.is_available());
```

`Icon::required_asset` names the file an icon needs, `Asset::file_name` and
`Asset::size` identify it, and `Asset::feature` names the feature that would
embed it. The [web preview](docs/web.md) downloads only the styles it shows
this way.

## Help and contributing

Report bugs and ask questions in
[GitHub Issues](https://github.com/mizorewww/gpui-material-symbols/issues).
The project is maintained by [@mizorewww](https://github.com/mizorewww).
Before sending changes, read the [maintenance guide](docs/maintenance.md),
which covers the validation commands, updating Kit and the Google assets, and
regenerating the screenshots.

More background: [validation record](docs/validation.md) ·
[research notes](docs/research.md) · [icon browser](docs/MATERIAL_SYMBOLS.md) ·
[web preview](docs/web.md).
The icon data is pinned to `google/material-design-icons` commit `10ecc14`;
sources, sizes and checksums are in [`assets/manifest.json`](assets/manifest.json).
Release notes are in the [changelog](CHANGELOG.md).

## License

Apache-2.0, see [LICENSE](LICENSE). Google's icon assets are also Apache-2.0
([`assets/LICENSE`](assets/LICENSE)); third-party assets used by the examples
are listed in [NOTICE](NOTICE). This project is not affiliated with or
endorsed by Google.
