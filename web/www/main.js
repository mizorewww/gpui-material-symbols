// Called by the Rust browser to save an exported icon.
globalThis.materialSymbolsDownload = (name, mime, bytes) => {
  const url = URL.createObjectURL(new Blob([bytes], { type: mime }));
  const link = Object.assign(document.createElement("a"), { href: url, download: name });
  document.body.append(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
};

const loading = document.getElementById("loading");
let started = false;
try {
  const { default: init, run } = await import("./wasm/gpui_material_symbols_web.js");
  await init();
  run();
  loading?.remove();
  started = true;
} catch (error) {
  console.error("Material Symbols failed to start", error);
  if (!loading) throw error;
  loading.classList.add("error");
  loading.replaceChildren();
  const message = document.createElement("p");
  message.textContent = `Unable to start the icon browser: ${error}`;
  const source = Object.assign(document.createElement("a"), {
    href: "https://github.com/mizorewww/gpui-material-symbols",
    textContent: "Run it natively instead",
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
