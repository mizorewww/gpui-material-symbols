# Web preview

The `material_symbols` icon browser also runs in the browser, compiled to
WebAssembly with GPUI's web platform. It is live at
<https://m3symbols.mizore.blog>.

The `web` crate includes `browser/src/main.rs` unchanged apart
from four web paths: the icon fonts download at runtime (below), the app bar
links the GitHub repository, exports download through the page instead of a
save dialog, and the app registers IBM Plex Sans (SIL OFL, `web/fonts/`), GPUI
Web's default UI font. It renders through WebGPU; GPUI Web can fall back to
WebGL 2, which this site has not been tested with.

The Web package disables default features on both Symbols and Material. This
keeps Material's default `embedded-symbols` feature from embedding the complete
font/SVG pack and bypassing the browser's on-demand asset loader. Build the Web
package explicitly with `-p gpui-material-symbols-web` (as `scripts/build-web.sh`
does). Building every workspace package for WASM can unify the native browser's
asset-embedding feature into the Web graph; it is not the lean Web build.

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

| File | Chunks | Downloaded |
| --- | --- | --- |
| Module (19.9 MB uncompressed) | 10 | 3.7 MB, at start |
| Outlined font | 6 | 4.1 MB, at start, in parallel with the module |
| Rounded font | 8 | 5.6 MB, when Rounded is first chosen |
| Sharp font | 5 | 3.6 MB, when Sharp is first chosen |
| SVG-only symbols | 8 | 2.4 MB, when one first scrolls into view |

`scripts/build-web.sh` splits each file into 2 MiB chunks, compresses each
with Brotli, and lists them in a manifest written into `main.js`. The page
downloads the module and the Outlined font together, with a progress bar.
The Rust browser asks the page for any other file when an icon first needs
it (`Icon::required_asset`), hands the bytes to `Asset::provide`, which
checks them, and redraws.

### Interrupted downloads

Chunks have content-hashed URLs under `/m/`, and `web/worker.js` serves them
as immutable with `Content-Encoding: br`. So:

- When the connection drops, the page retries only the failed chunks, with
  a pause that grows to 10 seconds; finished ones come from the browser
  cache. The loading screen says "Connection problem, resuming…" and waits
  for the browser to come back online.
- After a reload, finished chunks come from the browser cache; only missing
  ones are downloaded.
- A font that still fails after three tries shows "Could not download icons;
  retrying" in the browser, which asks again five seconds later.
- A chunk that returns a 4xx status, such as a tab left open across a
  redeploy, is not retried: the page asks the user to reload.

Splitting costs about 6% of compression against whole-file Brotli.

## Phones

Below 720px the browser switches to a compact layout: the app bar keeps the
search field and icon buttons, the style, sort and axis controls fold behind
the Tune button, and the details sheet covers the full width.

GPUI Web sizes its canvas from `devicePixelContentBoxSize` when the browser
has it. Chrome's DevTools device emulation reports CSS pixels there at a
device pixel ratio of 3, so the app (and gpui-kit.com's gallery) rendered
three times too large under emulation; with a real device scale factor
Chrome reports device pixels and renders correctly. `main.js` hides the
property, so GPUI takes its Safari path, CSS size times `devicePixelRatio`,
which gave the same canvas size as the device-pixel box at ratios from 1.1
to 3. The layout was checked only in Chrome's emulation, not on a real
phone.
