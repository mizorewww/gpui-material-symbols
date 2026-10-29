use gpui_material_symbols::{Axes, Icon, Style, Symbol};
use std::collections::BTreeSet;

#[derive(Clone)]
pub struct Entry {
    pub symbol: Symbol,
    pub category: &'static str,
    pub popularity: u32,
    pub tags: &'static str,
}
impl Entry {
    pub fn label(&self) -> String {
        self.symbol
            .name()
            .split('_')
            .map(|part| {
                let mut c = part.chars();
                c.next()
                    .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}
pub fn entries() -> Vec<Entry> {
    include_str!("assets/catalog.tsv")
        .lines()
        .map(|line| {
            let f: Vec<_> = line.split('\t').collect();
            Entry {
                symbol: Symbol::named(f[0]).unwrap(),
                category: f[1].split(',').next().unwrap(),
                popularity: f[2].parse().unwrap(),
                tags: f[3],
            }
        })
        .collect()
}
pub const CATEGORIES: &[&str] = &[
    "UI actions",
    "Social",
    "Actions",
    "Communicate",
    "Images",
    "Business",
    "Maps",
    "Text",
    "Audio & Video",
    "Transit",
    "Activities",
    "Android",
    "Hardware",
    "Privacy",
    "Travel",
    "Household",
    "Home",
    "Other",
];
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    #[default]
    Popular,
    Name,
}
pub fn filtered(
    entries: &[Entry],
    query: &str,
    categories: &BTreeSet<String>,
    sort: Sort,
) -> Vec<usize> {
    let q = query.to_lowercase().replace('_', " ");
    let mut result: Vec<_> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            (categories.is_empty() || categories.contains(e.category))
                && q.split_whitespace().all(|word| {
                    e.symbol.name().replace('_', " ").contains(word) || e.tags.contains(word)
                })
        })
        .map(|(i, _)| i)
        .collect();
    result.sort_by(|&a, &b| match sort {
        Sort::Name => entries[a].symbol.name().cmp(entries[b].symbol.name()),
        Sort::Popular => entries[b]
            .popularity
            .cmp(&entries[a].popularity)
            .then_with(|| entries[a].symbol.name().cmp(entries[b].symbol.name())),
    });
    result
}
pub fn configured(
    symbol: Symbol,
    style: Style,
    fill: bool,
    weight: f32,
    grade: f32,
    optical: f32,
) -> Icon {
    symbol
        .icon()
        .style(style)
        .axes(Axes::new(if fill { 1. } else { 0. }, weight, grade, optical).unwrap())
}
pub fn api(icon: Icon) -> String {
    let a = icon.variations();
    format!(
        "use gpui_material_symbols::{{Axes, Style, Symbol}};\n\nlet icon = Symbol::{}\n    .icon()\n    .style(Style::{:?})\n    .axes(Axes::new({:.1}, {:.1}, {:.1}, {:.1})?);\n\n// GPUI: serve icon.asset_path() from your AssetSource\nsvg().path(icon.asset_path());\n\n// Or use raw SVG bytes with your renderer\nlet svg = icon.to_svg()?;",
        icon.symbol()
            .name()
            .to_uppercase()
            .chars()
            .enumerate()
            .flat_map(|(i, c)| if i == 0 && c.is_ascii_digit() {
                vec!['_', c]
            } else {
                vec![c]
            })
            .collect::<String>(),
        icon.visual_style(),
        a.fill(),
        a.weight(),
        a.grade(),
        a.optical_size()
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_is_complete_and_searches_tags() {
        let e = entries();
        assert_eq!(e.len(), Symbol::all().len());
        let r = filtered(&e, "magnify", &BTreeSet::new(), Sort::Popular);
        assert!(r.iter().any(|i| e[*i].symbol == Symbol::SEARCH));
    }
    #[test]
    fn categories_and_sort_are_real() {
        let e = entries();
        let c = BTreeSet::from(["UI actions".into()]);
        let r = filtered(&e, "", &c, Sort::Name);
        assert!(!r.is_empty());
        assert!(r.iter().all(|i| e[*i].category == "UI actions"));
        assert!(
            r.windows(2)
                .all(|w| e[w[0]].symbol.name() <= e[w[1]].symbol.name())
        );
    }
    #[test]
    fn api_tracks_all_four_axes() {
        let i = configured(Symbol::HOME, Style::Rounded, true, 700., 200., 48.);
        let s = api(i);
        assert!(s.contains("Style::Rounded"));
        assert!(s.contains("1.0, 700.0, 200.0, 48.0"));
    }
}
