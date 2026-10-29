// Serves the site that `npm run deploy` builds (see ../docs/web.md).
//
// Files under /m/ have content-hashed names, so they are cached as immutable.
// The chunks of the module and the icon assets are stored Brotli-compressed as
// `NAME.br`, served with `Content-Encoding: br` for the browser to decompress.

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
      if (plain.status !== 304 && !plain.ok) {
        return plain;
      }
      const headers = new Headers(plain.headers);
      headers.set("Cache-Control", IMMUTABLE);
      return new Response(plain.body, { status: plain.status, headers });
    }
    if (compressed.status === 304) {
      // Keep the stored response immutable after a revalidation.
      const headers = new Headers(compressed.headers);
      headers.set("Cache-Control", IMMUTABLE);
      return new Response(null, { status: 304, headers });
    }
    if (!compressed.ok) {
      return compressed;
    }
    const headers = new Headers(compressed.headers);
    headers.set("Content-Type", "application/octet-stream");
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
