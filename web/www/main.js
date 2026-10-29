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
try {
  const { default: init, run } = await import("./wasm/gpui_material_symbols_web.js");
  await init();
  run();
  loading?.remove();
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
