//! The Rounded glyphs Material's own controls draw in the browser: the close
//! buttons of the details sheet and the snackbar (`IconName::Close`), the only
//! Rounded path a capture of every browser state loads (the selected card's
//! check is the Outlined `check_circle`, whose font the web build has first). The web
//! build downloads the 5.6 MB Rounded font only when Rounded is chosen, so it
//! serves these glyphs from here until then, byte for byte what the font
//! renders (the test below checks them against the embedded native font).
//! `cargo run --no-default-features --features embed-assets --example export
//! -- close rounded` regenerates the file, without its final newline.

/// Asset path and SVG of each glyph, at Material's canonical Rounded axes.
#[cfg(any(target_family = "wasm", test))]
const GLYPHS: &[(&str, &str)] = &[(
    "material-symbols/rounded/close/0/400/0/24.svg",
    include_str!("assets/chrome/close-rounded.svg"),
)];

/// The embedded SVG for `path`, on the web.
#[cfg(target_family = "wasm")]
pub fn glyph(path: &str) -> Option<&'static str> {
    GLYPHS
        .iter()
        .find(|(glyph, _)| *glyph == path)
        .map(|(_, svg)| *svg)
}

/// The native build embeds every font, and so every glyph.
#[cfg(not(target_family = "wasm"))]
pub fn glyph(_: &str) -> Option<&'static str> {
    None
}

#[cfg(test)]
mod tests {
    use super::GLYPHS;
    use gpui_kit_material::icons::IconName;
    use gpui_material_symbols::Icon;

    #[test]
    fn embedded_glyphs_are_what_the_font_renders_at_materials_paths() {
        assert_eq!(GLYPHS[0].0, IconName::Close.path());
        for (path, svg) in GLYPHS {
            let rendered = Icon::from_asset_path(path).unwrap().to_svg().unwrap();
            assert_eq!(rendered, *svg, "{path}");
        }
    }
}
