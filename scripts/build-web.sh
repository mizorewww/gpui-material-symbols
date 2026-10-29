#!/usr/bin/env bash
# Builds the Material Symbols browser for the web into web/dist.
#
# The module and the four assets the web build downloads at runtime are split
# into 2 MiB chunks, each Brotli-compressed and stored as `NAME.br`; web/worker.js
# serves them with `Content-Encoding: br`. Chunks and the module's JavaScript
# go to dist/m/ under content-hashed names and are served as immutable, so a
# dropped connection or a reload only fetches the chunks still missing.
# index.html and main.js are templates: `%JS%` and `%MANIFEST%` become the
# JavaScript's path and the chunk manifest.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
if [[ "$(wasm-bindgen --version 2>/dev/null)" != "wasm-bindgen 0.2.129" ]]; then
  echo "Install the matching CLI: cargo install wasm-bindgen-cli --version 0.2.129 --locked" >&2
  exit 1
fi
command -v brotli >/dev/null || { echo "Install brotli" >&2; exit 1; }

cargo build --locked --release -p gpui-material-symbols-web --target wasm32-unknown-unknown
bindgen="$(mktemp -d)"
trap 'rm -rf "$bindgen"' EXIT
wasm-bindgen target/wasm32-unknown-unknown/release/gpui_material_symbols_web.wasm \
  --target web --no-typescript --out-dir "$bindgen"

python3 -I - "$bindgen" <<'PY'
import hashlib, json, pathlib, shutil, subprocess, sys

CHUNK = 2 * 1024 * 1024
bindgen = pathlib.Path(sys.argv[1])
dist = pathlib.Path("web/dist")
shutil.rmtree(dist, ignore_errors=True)
(dist / "m").mkdir(parents=True)
shutil.copy("web/www/favicon.svg", dist)

def chunked(path):
    data = path.read_bytes()
    digest = hashlib.sha256(data).hexdigest()[:12]
    chunks = []
    for index in range(0, len(data), CHUNK):
        name = f"{path.stem}.{digest}.{index // CHUNK}"
        compressed = subprocess.run(
            ["brotli", "-q", "11", "-c"], input=data[index:index + CHUNK],
            capture_output=True, check=True).stdout
        (dist / "m" / f"{name}.br").write_bytes(compressed)
        chunks.append(f"/m/{name}")
    return {"size": len(data), "chunks": chunks}

js = bindgen / "gpui_material_symbols_web.js"
js_name = f"{js.stem}.{hashlib.sha256(js.read_bytes()).hexdigest()[:12]}.js"
shutil.copy(js, dist / "m" / js_name)
manifest = {
    "module": chunked(bindgen / "gpui_material_symbols_web_bg.wasm"),
    "assets": {
        name: chunked(pathlib.Path("assets") / name)
        for name in ["MaterialSymbolsOutlined.ttf", "MaterialSymbolsRounded.ttf",
                     "MaterialSymbolsSharp.ttf", "supplemental.bin"]
    },
}
for page in ["index.html", "main.js"]:
    text = (pathlib.Path("web/www") / page).read_text()
    text = text.replace("%JS%", f"/m/{js_name}").replace("%MANIFEST%", json.dumps(manifest))
    (dist / page).write_text(text)
for name, entry in [("module", manifest["module"]), *manifest["assets"].items()]:
    compressed = sum((dist / f"{c[1:]}.br").stat().st_size for c in entry["chunks"])
    print(f"{name}: {entry['size']:,} bytes, {len(entry['chunks'])} chunks, {compressed:,} as Brotli")
PY
