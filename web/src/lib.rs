//! Browser entry point: the `material_symbols` example, compiled for the web.
#[cfg(target_family = "wasm")]
#[path = "../../examples/material_symbols/main.rs"]
mod browser;

/// Starts the browser. `assets` lists each asset's file name and then its URL;
/// the fonts and SVG pack download as the browser first needs them.
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run(assets: Vec<String>) {
    gpui_platform::web_init();
    let urls = assets
        .as_chunks::<2>()
        .0
        .iter()
        .map(|[name, url]| (name.clone(), url.clone()))
        .collect();
    browser::run_with_assets(gpui_platform::single_threaded_web(), urls);
}
