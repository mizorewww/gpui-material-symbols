use gpui_material_symbols::{Style, Symbol};
use resvg::{tiny_skia, usvg};

fn raster(svg: &str) -> Vec<u8> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).unwrap();
    let mut pixmap = tiny_skia::Pixmap::new(96, 96).unwrap();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(96. / tree.size().width(), 96. / tree.size().height()),
        &mut pixmap.as_mut(),
    );
    pixmap.take()
}

#[test]
fn generated_geometry_matches_official_svg_pixels() {
    for (symbol, style, official) in [
        (
            Symbol::HOME,
            Style::Outlined,
            include_str!("fixtures/home-outlined.svg"),
        ),
        (
            Symbol::HOME,
            Style::Rounded,
            include_str!("fixtures/home-rounded.svg"),
        ),
        (
            Symbol::HOME,
            Style::Sharp,
            include_str!("fixtures/home-sharp.svg"),
        ),
        (
            Symbol::FAVORITE,
            Style::Outlined,
            include_str!("fixtures/favorite-outlined.svg"),
        ),
        (
            Symbol::FAVORITE,
            Style::Rounded,
            include_str!("fixtures/favorite-rounded.svg"),
        ),
        (
            Symbol::FAVORITE,
            Style::Sharp,
            include_str!("fixtures/favorite-sharp.svg"),
        ),
    ] {
        let actual = raster(&symbol.icon().style(style).to_svg().unwrap());
        let expected = raster(official);
        let error: u64 = actual
            .iter()
            .zip(&expected)
            .map(|(a, b)| a.abs_diff(*b) as u64)
            .sum();
        // Different path command encodings can yield tiny antialiasing differences.
        let mean = error as f64 / actual.len() as f64;
        assert!(mean < 0.5, "{symbol} {style:?}: mean pixel error {mean}");
        assert!(actual.as_chunks::<4>().0.iter().any(|p| p[3] > 0));
    }
}
