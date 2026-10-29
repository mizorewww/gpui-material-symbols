use gpui_material_symbols::{Axes, Icon, Style, Symbol};

#[derive(Clone)]
pub struct Entry {
    pub symbol: Symbol,
    pub category: &'static str,
    pub popularity: u32,
    pub tags: &'static str,
    /// Lowercase name words and tags; Google's tags include "Wi-Fi", "Android", ….
    search_text: String,
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
                search_text: format!("{} {}", f[0].replace('_', " "), f[3]).to_lowercase(),
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
pub fn filtered(entries: &[Entry], query: &str, sort: Sort) -> Vec<usize> {
    let q = query.to_lowercase().replace('_', " ");
    let mut result: Vec<_> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            q.split_whitespace()
                .all(|word| e.search_text.contains(word))
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
/// Rust for the selected icon, in the README's style.
pub fn api(icon: Icon) -> String {
    let a = icon.variations();
    let name = icon.symbol().name().to_uppercase();
    let constant = if name.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{name}")
    } else {
        name
    };
    format!(
        "use gpui_material_symbols::{{Axes, Style, Symbol}};\n\n\
         let icon = Symbol::{constant}\n    .icon()\n    .style(Style::{:?})\n    \
         .axes(Axes::new({:?}, {:?}, {:?}, {:?})?);\n\n\
         // Longbridge GPUI Kit: any icon slot, e.g. Button::new(\"id\").icon(icon)\n\
         // Standalone: let svg = icon.to_svg()?;",
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
        let r = filtered(&e, "magnify", Sort::Popular);
        assert!(r.iter().any(|i| e[*i].symbol == Symbol::SEARCH));
        let tagged = e.iter().position(|e| e.tags.contains("Android")).unwrap();
        let r = filtered(&e, "ANDROID", Sort::Name);
        assert!(r.contains(&tagged));
    }
    #[test]
    fn sort_orders_by_name_or_popularity() {
        let e = entries();
        let r = filtered(&e, "", Sort::Name);
        assert_eq!(r.len(), e.len());
        assert!(
            r.windows(2)
                .all(|w| e[w[0]].symbol.name() <= e[w[1]].symbol.name())
        );
        let r = filtered(&e, "", Sort::Popular);
        assert!(
            r.windows(2)
                .all(|w| e[w[0]].popularity >= e[w[1]].popularity)
        );
    }
    #[test]
    fn api_tracks_all_four_axes() {
        let i = configured(Symbol::HOME, Style::Rounded, true, 700., 200., 48.);
        let s = api(i);
        assert!(s.contains("Symbol::HOME"));
        assert!(s.contains("Style::Rounded"));
        assert!(s.contains("1.0, 700.0, 200.0, 48.0"));
        assert!(api(Symbol::_10K.icon()).contains("Symbol::_10K"));
    }
}
