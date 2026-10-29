use gpui_material_symbols::Icon;

/// Material Symbols are monochrome. Apply the preview color to every path,
/// including Google's SVG-only resources, which can carry their own fill.
pub fn encode(icon: Icon, size: u32, color: &str, png: bool) -> Result<Vec<u8>, String> {
    if !(16..=96).contains(&size) {
        return Err("Invalid size".into());
    }
    let hex = color.trim_start_matches('#');
    let color = if hex.len() == 6 && u32::from_str_radix(hex, 16).is_ok() {
        format!("#{hex}")
    } else {
        "#1F1F1F".into()
    };
    let source = icon.to_svg().map_err(|e| e.to_string())?;
    let end = source.find('>').ok_or("Missing SVG root")?;
    let svg = format!(
        "{}<style>path {{ fill: {color}; }}</style>{}",
        &source[..=end],
        &source[end + 1..]
    );
    let tree = resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default())
        .map_err(|e| e.to_string())?;
    // Serialize through usvg so root width/height and path colors are explicit.
    let mut normalized = tree.to_string(&resvg::usvg::WriteOptions::default());
    for (name, value) in [
        ("width", tree.size().width()),
        ("height", tree.size().height()),
    ] {
        normalized = normalized.replacen(
            &format!("{name}=\"{value}\""),
            &format!("{name}=\"{size}\""),
            1,
        );
    }
    if !png {
        return Ok(normalized.into_bytes());
    }
    let mut pixmap = tiny_skia::Pixmap::new(size, size).ok_or("Invalid dimensions")?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(
            size as f32 / tree.size().width(),
            size as f32 / tree.size().height(),
        ),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    use gpui_material_symbols::Symbol;
    #[test]
    fn exported_color_and_dimensions_follow_preview() {
        let svg = encode(Symbol::HOME.icon(), 32, "#ff0000", false).unwrap();
        let tree = resvg::usvg::Tree::from_data(&svg, &Default::default()).unwrap();
        assert_eq!(tree.size().width(), 32.);
        let png = encode(Symbol::HOME.icon(), 32, "#ff0000", true).unwrap();
        let pixels = tiny_skia::Pixmap::decode_png(&png).unwrap();
        assert_eq!(pixels.width(), 32);
        assert!(pixels.pixels().iter().any(|p| p.alpha() > 0));
        assert!(
            pixels
                .pixels()
                .iter()
                .filter(|p| p.alpha() > 0)
                .all(|p| p.red() > 0 && p.green() == 0 && p.blue() == 0)
        );
    }
}
