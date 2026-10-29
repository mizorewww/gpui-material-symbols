#!/usr/bin/env bash
# Builds the Material Symbols browser for the web into web/dist.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
if [[ "$(wasm-bindgen --version 2>/dev/null)" != "wasm-bindgen 0.2.129" ]]; then
  echo "Install the matching CLI: cargo install wasm-bindgen-cli --version 0.2.129 --locked" >&2
  exit 1
fi
command -v brotli >/dev/null || { echo "Install brotli" >&2; exit 1; }

cargo build --locked --release -p gpui-material-symbols-web --target wasm32-unknown-unknown
dist=web/dist
rm -rf "$dist"
mkdir -p "$dist"
cp web/www/* "$dist/"
wasm-bindgen target/wasm32-unknown-unknown/release/gpui_material_symbols_web.wasm \
  --target web --no-typescript --out-dir "$dist/wasm"
# The module exceeds Cloudflare's 25 MiB asset limit; the Worker serves this
# Brotli file with `Content-Encoding: br`.
brotli --rm -q 11 "$dist/wasm/gpui_material_symbols_web_bg.wasm"
ls -l "$dist/wasm"
