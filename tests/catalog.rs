use gpui_material_symbols::{Axes, Error, Icon, Style, Symbol};
use ttf_parser::Face;

#[test]
fn catalog_exactly_matches_every_upstream_manifest() {
    for (style, data) in [
        (
            Style::Outlined,
            include_str!("../assets/MaterialSymbolsOutlined.codepoints"),
        ),
        (
            Style::Rounded,
            include_str!("../assets/MaterialSymbolsRounded.codepoints"),
        ),
        (
            Style::Sharp,
            include_str!("../assets/MaterialSymbolsSharp.codepoints"),
        ),
    ] {
        assert_eq!(
            Symbol::all().filter(|s| s.is_variable()).count(),
            data.lines().count()
        );
        for line in data.lines() {
            let mut parts = line.split_whitespace();
            let name = parts.next().unwrap();
            let cp = u32::from_str_radix(parts.next().unwrap(), 16).unwrap();
            let symbol = Symbol::named(name).unwrap();
            assert_eq!(symbol.name(), name);
            assert_eq!(symbol.codepoint(style).unwrap() as u32, cp);
        }
    }
    assert_eq!(Symbol::named("home"), Some(Symbol::HOME));
    assert_eq!(Symbol::named("not_a_real_symbol"), None);
}

#[test]
fn declared_axes_match_all_official_fonts() {
    for style in Style::ALL {
        let face = Face::parse(style.font_bytes(), 0).unwrap();
        let axes: Vec<_> = face
            .variation_axes()
            .into_iter()
            .map(|a| (a.tag.to_string(), a.min_value, a.max_value))
            .collect();
        assert_eq!(
            axes.len(),
            4,
            "new upstream axes need API support: {axes:?}"
        );
        for expected in [
            ("FILL", 0., 1.),
            ("wght", 100., 700.),
            ("GRAD", -50., 200.),
            ("opsz", 20., 48.),
        ] {
            assert!(
                axes.iter()
                    .any(|a| a.0 == expected.0 && a.1 == expected.1 && a.2 == expected.2),
                "{style:?}: {axes:?}"
            );
        }
    }
}

#[test]
fn every_symbol_renders_in_every_style_and_representative_variations() {
    let settings = [
        Axes::default(),
        Axes::new(1., 700., 200., 48.).unwrap(),
        Axes::new(0., 100., -50., 20.).unwrap(),
        Axes::new(0.5, 350.5, 25.5, 32.).unwrap(),
    ];
    for style in Style::ALL {
        for symbol in Symbol::all().filter(|s| s.is_variable()) {
            for axes in settings {
                let icon = symbol.icon().style(style).axes(axes);
                let svg = icon.to_svg().unwrap_or_else(|e| panic!("{icon:?}: {e}"));
                let doc = roxmltree::Document::parse(&svg).unwrap();
                let path = doc.descendants().find(|n| n.has_tag_name("path")).unwrap();
                assert!(!path.attribute("d").unwrap().is_empty(), "{icon:?}");
                assert!(!svg.contains("NaN") && !svg.contains("inf"));
                assert_eq!(Icon::from_asset_path(&icon.asset_path()).unwrap(), icon);
            }
        }
    }
}

#[test]
fn variations_change_real_geometry() {
    let base = Symbol::HOME.icon();
    let baseline = base.to_svg().unwrap();
    for axes in [
        Axes::new(1., 400., 0., 24.),
        Axes::new(0., 700., 0., 24.),
        Axes::new(0., 400., 200., 24.),
        Axes::new(0., 400., 0., 48.),
    ] {
        assert_ne!(baseline, base.axes(axes.unwrap()).to_svg().unwrap());
    }
    for style in [Style::Rounded, Style::Sharp] {
        assert!(
            Symbol::all().take(100).any(|symbol| {
                symbol.icon().to_svg().unwrap() != symbol.icon().style(style).to_svg().unwrap()
            }),
            "style {style:?} must have distinct geometry somewhere in the catalog"
        );
    }
}

#[test]
fn invalid_input_is_rejected() {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.1, 1.1] {
        assert!(Axes::new(value, 400., 0., 24.).is_err());
    }
    assert_eq!(Axes::new(0., 99., 0., 24.), Err(Error::InvalidAxis("wght")));
    assert!(Axes::new(0., 400., 201., 24.).is_err());
    assert!(Axes::new(0., 400., 0., 49.).is_err());
    for path in [
        "",
        "../home.svg",
        "material-symbols/outlined/home/NaN/400/0/24.svg",
        "material-symbols/outlined/home/0/400/0/24.svg/extra",
        "material-symbols/outlined/home/0/400/0/024.svg",
    ] {
        assert!(Icon::from_asset_path(path).is_err(), "{path}");
    }
}

#[test]
fn supplemental_catalog_covers_every_official_svg_name_and_variant() {
    for name in include_str!("../assets/svg-names.txt").lines() {
        assert!(Symbol::named(name).is_some(), "missing {name}");
    }
    let mut count = 0;
    for symbol in Symbol::all().filter(|s| !s.is_variable()) {
        for style in Style::ALL {
            assert!(
                symbol.icon().style(style).is_supported(),
                "{symbol} {style:?}"
            );
        }
        for icon in symbol.available_variants() {
            assert!(icon.is_supported(), "{icon:?}");
            let svg = icon.to_svg().unwrap();
            let document = roxmltree::Document::parse(&svg).unwrap();
            assert!(document.descendants().any(|n| n.has_tag_name("path")));
            assert_eq!(Icon::from_asset_path(&icon.asset_path()).unwrap(), icon);
            count += 1;
        }
    }
    assert_eq!(
        count,
        include_str!("../assets/supplemental.tsv").lines().count()
    );
    assert_eq!(Symbol::all().count(), 4359);
    let unsupported = Symbol::APP_SPARK
        .icon()
        .axes(Axes::new(0.5, 400., 0., 24.).unwrap());
    assert!(!unsupported.is_supported());
    assert_eq!(unsupported.to_svg(), Err(Error::UnsupportedVariant));
}

#[test]
fn icon_geometry_uses_design_square_not_typographic_ascender() {
    let svg = Symbol::HOME.icon().to_svg().unwrap();
    let doc = roxmltree::Document::parse(&svg).unwrap();
    assert_eq!(doc.root_element().attribute("viewBox"), Some("0 0 960 960"));
    let path = doc.descendants().find(|n| n.has_tag_name("path")).unwrap();
    assert_eq!(
        path.attribute("transform"),
        Some("translate(0 960) scale(1 -1)")
    );
}
