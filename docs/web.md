# Web preview

The `material_symbols` icon browser also runs in the browser, compiled to
WebAssembly with GPUI's web platform. It is live at
<https://m3symbols.mizore.blog>.

The `web` crate includes `examples/material_symbols/main.rs` unchanged apart
from three web paths: the app bar links the GitHub repository, exports
download through the page instead of a save dialog, and the app registers IBM
Plex Sans (SIL OFL, `web/fonts/`), GPUI Web's default
UI font. It renders through WebGPU; GPUI Web can fall back to WebGL 2, which
this site has not been tested with.

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
through the Worker (`npm run dev`): `dist` holds only the Brotli module, so a
plain static server cannot load it.

## Deploy

The site is a Cloudflare Worker with static assets, configured in
`web/cloudflare.config.ts` and deployed with the
[Cloudflare CLI](https://developers.cloudflare.com/cf/) (`cf`, beta):

```sh
cd web
npx cf auth login
npm run deploy   # scripts/build-web.sh, then cf deploy
```

The module embeds the complete icon catalog and is about 71 MB, above
Cloudflare's 25 MiB per-file asset limit. The build compresses it with Brotli
(about 16.5 MB) and `web/worker.js` serves that file with
`Content-Encoding: br` and `Content-Type: application/wasm`.
