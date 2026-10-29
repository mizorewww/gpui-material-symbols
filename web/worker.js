// Serves the site that `npm run deploy` builds (see ../docs/web.md).
//
// Files under /m/ have content-hashed names, so they are cached as immutable.
// The module and the icon fonts are stored Brotli-compressed as `NAME.br`
// (the module alone is larger than Cloudflare's per-file asset limit) and are
// served with `Content-Encoding: br` for the browser to decompress.
const TYPES = {
  wasm: "application/wasm",
  ttf: "font/ttf",
  bin: "application/octet-stream",
  js: "text/javascript; charset=utf-8",
};
const IMMUTABLE = "public, max-age=31536000, immutable";

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (!url.pathname.startsWith("/m/")) {
      return env.ASSETS.fetch(request);
    }
    const compressed = await env.ASSETS.fetch(new Request(new URL(`${url.pathname}.br`, url), request));
    if (compressed.status === 404) {
      const plain = await env.ASSETS.fetch(request);
      if (!plain.ok) {
        return plain;
      }
      const headers = new Headers(plain.headers);
      headers.set("Cache-Control", IMMUTABLE);
      return new Response(plain.body, { status: plain.status, headers });
    }
    if (compressed.status === 304 || !compressed.ok) {
      return compressed;
    }
    const headers = new Headers(compressed.headers);
    const extension = url.pathname.split(".").pop();
    headers.set("Content-Type", TYPES[extension] ?? "application/octet-stream");
    headers.set("Content-Encoding", "br");
    headers.set("Vary", "Accept-Encoding");
    headers.set("Cache-Control", IMMUTABLE);
    return new Response(compressed.body, {
      status: compressed.status,
      headers,
      encodeBody: "manual",
    });
  },
};
