# Web preview

The `material_symbols` icon browser also runs in the browser, compiled to
WebAssembly with GPUI's web platform. It is live at
<https://m3symbols.mizore.blog>.

The `web` crate includes `examples/material_symbols/main.rs` unchanged apart
from four web paths: the icon fonts download at runtime (below), the app bar
links the GitHub repository, exports download through the page instead of a
save dialog, and the app registers IBM Plex Sans (SIL OFL, `web/fonts/`), GPUI
Web's default UI font. It renders through WebGPU; GPUI Web can fall back to
WebGL 2, which this site has not been tested with.

On load the page asks for a GitHub star in a dialog (`web/www/index.html`).
Choosing "Star on GitHub" or "Don't show this again" stops it for that browser
through `localStorage`; "Not now" asks again on the next visit. Where the
browser blocks storage, the choice cannot be kept and the dialog returns.

## Build and run locally

Requirements: the `wasm32-unknown-unknown` target, `wasm-bindgen-cli` 0.2.129
(matching `Cargo.lock`), `brotli`, and Node.js 22.18 or later.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
cd web
npm install
npm run dev      # builds web/dist, then serves it with the Worker locally
```

`scripts/build-web.sh` alone builds `web/dist/` without Node.js. Serve it
through the Worker (`npm run dev`): `dist` holds the large files only as
Brotli, so a plain static server cannot load them.

## Deploy

The site is a Cloudflare Worker with static assets, configured in
`web/cloudflare.config.ts` and deployed with the
[Cloudflare CLI](https://developers.cloudflare.com/cf/) (`cf`, beta):

```sh
cd web
npx cf auth login
npm run deploy   # scripts/build-web.sh, then cf deploy
```

## Loading

The web crate builds `gpui-material-symbols` without `embed-assets`, so the
module carries the code and catalog but no icon fonts. Sizes as Brotli:

| File | Downloaded |
| --- | --- |
| Module (18 MB uncompressed) | 3.5 MB, at start |
| Outlined font | 4.1 MB, at start, in parallel with the module |
| Rounded font | 5.5 MB, when Rounded is first chosen |
| Sharp font | 3.6 MB, when Sharp is first chosen |
| SVG-only symbols | 2.2 MB, when one first scrolls into view |

`index.html` preloads the module and the Outlined font so they download
together. The browser fetches any other file when an icon first needs it
(`Icon::required_asset`), hands the bytes to `Asset::provide`, which checks
them, and redraws.

`scripts/build-web.sh` names these files by content hash under `/m/`, and
`web/worker.js` serves them as immutable, so repeat visits come from the
browser cache. They are stored only Brotli-compressed, because the module
built with every font was larger than Cloudflare's 25 MiB per-file asset
limit, and served with `Content-Encoding: br`.
