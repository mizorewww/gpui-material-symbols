//! Browser entry point: the `material_symbols` consumer, compiled for the web.
#[cfg(target_family = "wasm")]
#[path = "../../browser/src/main.rs"]
mod browser;

/// Starts the browser. The page supplies the icon fonts and SVG pack as the
/// browser first needs them.
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run() {
    gpui_platform::web_init();
    browser::run_web(gpui_platform::single_threaded_web());
}
