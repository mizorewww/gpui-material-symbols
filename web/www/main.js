// Every downloadable file, split into 2 MiB chunks with content-hashed URLs
// (written by scripts/build-web.sh).
const MANIFEST = %MANIFEST%;

// Called by the Rust browser to save an exported icon.
globalThis.materialSymbolsDownload = (name, mime, bytes) => {
  const url = URL.createObjectURL(new Blob([bytes], { type: mime }));
  const link = Object.assign(document.createElement("a"), { href: url, download: name });
  document.body.append(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
};

// Chunks have content-hashed URLs and are served as immutable, so the browser
// caches each finished chunk: a retry or a reload downloads only the missing
// ones. Chunk downloads in flight are shared between callers.
const inflight = new Map();
const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const online = () =>
  navigator.onLine ? Promise.resolve() : new Promise((resolve) => addEventListener("online", resolve, { once: true }));

// The server no longer has the file: the site was redeployed since this page
// loaded. Retrying cannot help; the page must be reloaded.
class StaleError extends Error {}

// Downloads one chunk, reporting each decoded byte to `progress`.
async function fetchChunk(url, progress) {
  const response = await fetch(url);
  if (response.status >= 400 && response.status < 500) {
    throw new StaleError(`reload: ${url} returned HTTP ${response.status}`);
  }
  if (!response.ok) throw new Error(`${url}: HTTP ${response.status}`);
  const reader = response.body.getReader();
  const parts = [];
  let length = 0;
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      parts.push(value);
      length += value.length;
      progress(value.length);
    }
  } catch (error) {
    progress(-length);
    throw error;
  }
  const bytes = new Uint8Array(length);
  let offset = 0;
  for (const part of parts) {
    bytes.set(part, offset);
    offset += part.length;
  }
  return bytes;
}

function chunk(url, progress) {
  const shared = inflight.get(url);
  if (shared) return shared.then((bytes) => (progress(bytes.length), bytes));
  const promise = fetchChunk(url, progress).finally(() => inflight.delete(url));
  inflight.set(url, promise);
  return promise;
}

// Downloads every chunk of `entry`, retrying a failed chunk up to `tries`
// times with a growing pause, and joins them. A stale chunk fails at once.
async function download(entry, { progress = () => {}, retrying = () => {}, tries = 3 } = {}) {
  const parts = await Promise.all(
    entry.chunks.map(async (url) => {
      for (let attempt = 1; ; attempt++) {
        try {
          return await chunk(url, progress);
        } catch (error) {
          if (error instanceof StaleError || attempt >= tries) throw error;
          retrying(error);
          await online();
          await wait(Math.min(1000 * attempt, 10000));
        }
      }
    }),
  );
  const length = parts.reduce((sum, part) => sum + part.length, 0);
  if (length !== entry.size) {
    // Same-named chunks with other contents: only a redeploy does that.
    throw new StaleError(`reload: expected ${entry.size} bytes, received ${length}`);
  }
  const bytes = new Uint8Array(entry.size);
  let offset = 0;
  for (const part of parts) {
    bytes.set(part, offset);
    offset += part.length;
  }
  return bytes;
}

// Called by the Rust browser for an icon font or the SVG pack. Concurrent
// requests share one download, which is dropped here once delivered. It
// rejects with a string; the browser retries unless it starts with "reload:".
const pending = new Map();
globalThis.materialSymbolsFetchAsset = (name) => {
  const entry = MANIFEST.assets[name];
  if (!entry) return Promise.reject(`reload: unknown asset ${name}`);
  if (!pending.has(name)) {
    pending.set(name, download(entry));
  }
  const promise = pending.get(name);
  promise.then(
    () => pending.delete(name),
    () => pending.delete(name),
  );
  return promise.catch((error) => Promise.reject(String(error?.message ?? error)));
};

const loading = document.getElementById("loading");
const text = document.getElementById("loading-text");
const bar = loading?.querySelector('[role="progressbar"]');
let received = 0;
const total = MANIFEST.module.size + MANIFEST.assets["MaterialSymbolsOutlined.ttf"].size;
const progress = (bytes) => {
  received += bytes;
  const percent = Math.min(100, Math.round((received / total) * 100));
  if (bar) {
    bar.setAttribute("aria-valuenow", percent);
    bar.firstElementChild.style.width = `${percent}%`;
  }
  if (text) text.textContent = `Loading Material Symbols… ${percent}%`;
};

let started = false;
try {
  const retrying = () => text && (text.textContent = "Connection problem, resuming…");
  // The default style's font downloads alongside the module.
  const outlinedName = "MaterialSymbolsOutlined.ttf";
  // Kept until the browser asks for it, so it is not downloaded twice.
  const outlined = download(MANIFEST.assets[outlinedName], { progress, retrying, tries: Infinity });
  outlined.catch(() => pending.delete(outlinedName));
  pending.set(outlinedName, outlined);
  const [{ default: init, run }, module] = await Promise.all([
    import("%JS%"),
    download(MANIFEST.module, { progress, retrying, tries: Infinity }),
  ]);
  await init({ module_or_path: module });
  run();
  loading?.remove();
  started = true;
} catch (error) {
  console.error("Material Symbols failed to start", error);
  if (!loading) throw error;
  loading.classList.add("error");
  loading.replaceChildren();
  const message = document.createElement("p");
  message.textContent =
    error instanceof StaleError
      ? "The site was just updated. Reload the page to get the new version."
      : `Unable to start the icon browser: ${error}`;
  const source = Object.assign(document.createElement("a"), {
    href: error instanceof StaleError ? location.href : "https://github.com/mizorewww/gpui-material-symbols",
    textContent: error instanceof StaleError ? "Reload" : "Run it natively instead",
  });
  loading.append(message, source);
}
if (started) askForStar();
// Asks once per visit for a GitHub star, until the visitor stars or opts out.
function askForStar() {
  const key = "m3symbols.star-prompt";
  const read = () => { try { return localStorage.getItem(key); } catch { return null; } };
  const write = (value) => { try { localStorage.setItem(key, value); } catch {} };
  const dialog = document.getElementById("star");
  if (!dialog?.showModal || read() === "never") return;
  const never = document.getElementById("star-never");
  dialog.addEventListener("close", () => { if (never.checked) write("never"); });
  document.getElementById("star-later").addEventListener("click", () => dialog.close());
  document.getElementById("star-go").addEventListener("click", () => {
    never.checked = true;
    dialog.close();
  });
  dialog.showModal();
}
