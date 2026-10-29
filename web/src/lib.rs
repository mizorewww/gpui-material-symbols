//! Browser entry point: the `material_symbols` example, compiled for the web.
#[cfg(target_family = "wasm")]
#[path = "../../examples/material_symbols/main.rs"]
mod browser;

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run() {
    gpui_platform::web_init();
    browser::run(gpui_platform::single_threaded_web());
}
