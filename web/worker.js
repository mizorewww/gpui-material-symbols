// Serves the site that `npm run deploy` builds (see ../docs/web.md). The
// WebAssembly module is larger than Cloudflare's per-file asset limit, so it
// is uploaded Brotli-compressed and served with `Content-Encoding: br` for
// the browser to decompress.
const WASM = /\/wasm\/[^/]+\.wasm$/;

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (!WASM.test(url.pathname)) {
      return env.ASSETS.fetch(request);
    }
    const compressed = new URL(`${url.pathname}.br`, url);
    const asset = await env.ASSETS.fetch(new Request(compressed, request));
    if (asset.status === 304) {
      return asset;
    }
    if (!asset.ok) {
      return asset;
    }
    const headers = new Headers(asset.headers);
    headers.set("Content-Type", "application/wasm");
    headers.set("Content-Encoding", "br");
    headers.set("Vary", "Accept-Encoding");
    return new Response(asset.body, { status: asset.status, headers, encodeBody: "manual" });
  },
};
