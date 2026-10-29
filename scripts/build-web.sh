#!/usr/bin/env bash
# Builds the Material Symbols browser for the web into web/dist.
#
# The module, its JavaScript and the four assets the web build downloads at
# runtime go to dist/m/ under content-hashed names, served as immutable.
# Large files are stored only as Brotli (`NAME.br`); web/worker.js serves them
# with `Content-Encoding: br`. index.html and main.js are templates whose
# `%NAME%` placeholders become those hashed paths.
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
bindgen="$(mktemp -d)"
trap 'rm -rf "$bindgen"' EXIT
wasm-bindgen target/wasm32-unknown-unknown/release/gpui_material_symbols_web.wasm \
  --target web --no-typescript --out-dir "$bindgen"
rm -rf "$dist"
mkdir -p "$dist/m"
cp web/www/favicon.svg "$dist/"

# Copies $1 to dist/m/<stem>.<hash>.<ext>, Brotli-compressed when $2 is "br",
# and prints the served path.
publish() {
  local file="$1" name stem ext hash
  name="$(basename "$file")"
  stem="${name%.*}"
  ext="${name##*.}"
  hash="$(shasum -a 256 "$file" | cut -c1-12)"
  if [[ "$2" == br ]]; then
    brotli -q 11 -c "$file" > "$dist/m/$stem.$hash.$ext.br"
  else
    cp "$file" "$dist/m/$stem.$hash.$ext"
  fi
  echo "/m/$stem.$hash.$ext"
}

wasm="$(publish "$bindgen/gpui_material_symbols_web_bg.wasm" br)"
js="$(publish "$bindgen/gpui_material_symbols_web.js" plain)"
outlined="$(publish assets/MaterialSymbolsOutlined.ttf br)"
rounded="$(publish assets/MaterialSymbolsRounded.ttf br)"
sharp="$(publish assets/MaterialSymbolsSharp.ttf br)"
supplemental="$(publish assets/supplemental.bin br)"
assets="[\"MaterialSymbolsOutlined.ttf\", \"$outlined\", \"MaterialSymbolsRounded.ttf\", \"$rounded\", \"MaterialSymbolsSharp.ttf\", \"$sharp\", \"supplemental.bin\", \"$supplemental\"]"

for page in index.html main.js; do
  sed -e "s#%WASM%#$wasm#g" -e "s#%JS%#$js#g" -e "s#%OUTLINED%#$outlined#g" \
    -e "s#%ASSETS%#$assets#g" "web/www/$page" > "$dist/$page"
done
ls -l "$dist/m"
