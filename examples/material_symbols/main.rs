//! Native reference implementation of fonts.google.com/icons, with GPUI API output.
// GPUI macros prefer an installed Kit facade even when its optional feature is
// disabled. Keep their generated path on the same backend for this example.
#[cfg(not(feature = "gpui-kit"))]
extern crate gpui as gpui_kit;
mod catalog;
mod export;
use catalog::{Entry, Sort};
use gpui::{prelude::*, *};
use gpui_base::{
    input::{InputEvent, InputState},
    slider::{SliderEvent, SliderState},
};
use gpui_kit_material::{icons::Assets, prelude::*, theme};
use gpui_material_symbols::{Icon as SymbolIcon, Style, Symbol};
use std::{borrow::Cow, collections::BTreeSet, path::Path};

fn brand_image(dark: bool) -> std::sync::Arc<RenderImage> {
    let source = include_str!("assets/google-fonts.svg")
        .replace("currentColor", if dark { "#e3e3e3" } else { "#444746" });
    let tree = resvg::usvg::Tree::from_str(&source, &Default::default()).unwrap();
    let mut pixmap = tiny_skia::Pixmap::new(350, 62).unwrap();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(2., 2.),
        &mut pixmap.as_mut(),
    );
    let mut bytes = pixmap.data().to_vec();
    for pixel in bytes.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
    }
    std::sync::Arc::new(RenderImage::new(vec![image::Frame::new(
        image::RgbaImage::from_raw(350, 62, bytes).unwrap(),
    )]))
}
struct SymbolsSource;
impl AssetSource for SymbolsSource {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if !path.starts_with("material-symbols/") {
            return Ok(None);
        }
        Ok(Some(Cow::Owned(
            SymbolIcon::from_asset_path(path)?.to_svg()?.into_bytes(),
        )))
    }
    fn list(&self, _: &str) -> Result<Vec<SharedString>> {
        Ok(vec![])
    }
}
fn glyph(symbol: Symbol, size: f32) -> Svg {
    svg()
        .path(symbol.icon().asset_path())
        .size(rem(size))
        .text_color(color(ColorRole::INK))
        .flex_shrink_0()
}
fn button(id: impl Into<ElementId>, symbol: Symbol, label: &str) -> gpui_base::Button {
    material_button(id, ButtonVariant::Text)
        .accessibility_label(label.to_owned())
        .w(rem(40.))
        .px_0()
        .child(glyph(symbol, 24.))
}
fn scheme(dark: bool) -> gpui_kit_material::color::Scheme {
    use gpui_kit_material::color::{Color, Scheme};
    let values = if dark {
        [
            0x121212, 0x1b1b1b, 0x232427, 0x1b1b1b, 0xc4c7c5, 0xc4c7c5, 0x444746, 0x8e918f,
            0xa8c7fa, 0x062e6f, 0x004a77, 0xc2e7ff,
        ]
    } else {
        [
            0xffffff, 0xf8fafd, 0xf3f6fc, 0xf8fafd, 0x444746, 0x444746, 0xc4c7c5, 0x747775,
            0x0b57d0, 0xffffff, 0xc2e7ff, 0x001d35,
        ]
    };
    let roles = [
        ColorRole::CANVAS,
        ColorRole::SURFACE,
        ColorRole::SURFACE_HIGH,
        ColorRole::INSET,
        ColorRole::INK,
        ColorRole::GRAY,
        ColorRole::HAIRLINE,
        ColorRole::CONTROL,
        ColorRole::ACCENT,
        ColorRole::ON_PRIMARY,
        ColorRole::SECONDARY_CONTAINER,
        ColorRole::ON_SECONDARY_CONTAINER,
    ];
    roles.into_iter().zip(values).fold(
        Scheme::from_palette(if dark {
            PaletteId::Ink
        } else {
            PaletteId::Paper
        }),
        |s, (r, c)| s.with_role(r, Color::hex(c)),
    )
}
#[derive(Clone)]
enum Row {
    Heading(&'static str),
    Icons(Vec<usize>),
    Empty,
}
struct Browser {
    logo: std::sync::Arc<RenderImage>,
    dark_logo: std::sync::Arc<RenderImage>,
    entries: Vec<Entry>,
    rows: Vec<Row>,
    list: ListState,
    columns: usize,
    search: Entity<InputState>,
    weight: Entity<SliderState>,
    grade: Entity<SliderState>,
    optical: Entity<SliderState>,
    color_input: Entity<InputState>,
    style: Style,
    filled: bool,
    dark: bool,
    filters: bool,
    style_menu: bool,
    sort_menu: bool,
    categories: BTreeSet<String>,
    sort: Sort,
    selected: Option<usize>,
    detail_open: bool,
    preview_size: f32,
    status: SharedString,
    _subscriptions: Vec<Subscription>,
}
impl Browser {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search icons"));
        let color_input = cx.new(|cx| {
            let mut s = InputState::new(window, cx).placeholder("#1F1F1F");
            s.set_value("#1F1F1F", window, cx);
            s
        });
        let weight = cx.new(|_| {
            SliderState::new()
                .min(100.)
                .max(700.)
                .step(100.)
                .default_value(400.)
        });
        let grade = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(2.)
                .step(1.)
                .default_value(1.)
        });
        let optical = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(3.)
                .step(1.)
                .default_value(1.)
        });
        let mut subscriptions = vec![
            cx.subscribe(&search, |this, _, event, cx| {
                if matches!(event, InputEvent::Change) {
                    this.rebuild(cx);
                    cx.notify();
                }
            }),
            cx.subscribe(&color_input, |_, _, _: &InputEvent, cx| cx.notify()),
        ];
        for state in [&weight, &grade, &optical] {
            subscriptions.push(cx.subscribe(state, |_, _, event, cx| {
                if matches!(event, SliderEvent::Change(_)) {
                    cx.notify();
                }
            }));
        }
        let mut this = Self {
            logo: brand_image(false),
            dark_logo: brand_image(true),
            entries: catalog::entries(),
            rows: vec![],
            list: ListState::new(0, ListAlignment::Top, px(500.)),
            columns: 7,
            search,
            weight,
            grade,
            optical,
            color_input,
            style: Style::Outlined,
            filled: false,
            dark: false,
            filters: true,
            style_menu: false,
            sort_menu: false,
            categories: BTreeSet::new(),
            sort: Sort::Popular,
            selected: None,
            detail_open: false,
            preview_size: 24.,
            status: "".into(),
            _subscriptions: subscriptions,
        };
        this.rebuild(cx);
        this
    }
    fn rebuild(&mut self, cx: &App) {
        let indices = catalog::filtered(
            &self.entries,
            &self.search.read(cx).value(),
            &self.categories,
            self.sort,
        );
        self.rows.clear();
        for category in catalog::CATEGORIES {
            let group: Vec<_> = indices
                .iter()
                .copied()
                .filter(|i| self.entries[*i].category == *category)
                .collect();
            if !group.is_empty() {
                self.rows.push(Row::Heading(category));
                for chunk in group.chunks(self.columns) {
                    self.rows.push(Row::Icons(chunk.to_vec()));
                }
            }
        }
        if self.rows.is_empty() {
            self.rows.push(Row::Empty)
        }
        self.list.reset(self.rows.len());
    }
    fn icon(&self, symbol: Symbol, cx: &App) -> SymbolIcon {
        catalog::configured(
            symbol,
            self.style,
            self.filled,
            self.weight.read(cx).value().end(),
            [-25., 0., 200.][self.grade.read(cx).value().end() as usize],
            [20., 24., 40., 48.][self.optical.read(cx).value().end() as usize],
        )
    }
    fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.style = Style::Outlined;
        self.filled = false;
        self.categories.clear();
        self.search.update(cx, |s, cx| s.set_value("", window, cx));
        for (s, v) in [(&self.weight, 400.), (&self.grade, 1.), (&self.optical, 1.)] {
            s.update(cx, |s, cx| s.set_value(v, window, cx));
        }
        self.rebuild(cx);
        cx.notify();
    }
    fn rail(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .w(rem(96.))
            .h_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .items_center()
            .bg(color(ColorRole::SURFACE_HIGH))
            .py(rem(20.))
            .gap(rem(12.))
            .children(
                [
                    (Symbol::FONT_DOWNLOAD, "Fonts", "https://fonts.google.com"),
                    (Symbol::LANGUAGE, "Noto", "https://fonts.google.com/noto"),
                    (Symbol::CATEGORY, "Icons", ""),
                    (
                        Symbol::SCHOOL,
                        "Knowledge",
                        "https://fonts.google.com/knowledge",
                    ),
                    (
                        Symbol::HELP,
                        "FAQ",
                        "https://fonts.google.com/knowledge/using_type",
                    ),
                ]
                .map(|(icon, label, url)| {
                    material_button(
                        SharedString::from(format!("nav-{label}")),
                        ButtonVariant::Text,
                    )
                    .flex_col()
                    .h(rem(56.))
                    .w(rem(80.))
                    .px_0()
                    .gap(rem(3.))
                    .accessibility_label(label)
                    .on_click(move |_, _, cx| {
                        if !url.is_empty() {
                            cx.open_url(url)
                        }
                    })
                    .child(
                        div()
                            .w(rem(56.))
                            .h(rem(32.))
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(label == "Icons", |e| {
                                e.bg(color(ColorRole::SECONDARY_CONTAINER))
                            })
                            .child(glyph(icon, 24.)),
                    )
                    .child(div().text_size(rem(12.)).child(label))
                }),
            )
            .child(div().flex_1())
            .child(
                button(
                    "theme",
                    if self.dark {
                        Symbol::LIGHT_MODE
                    } else {
                        Symbol::DARK_MODE
                    },
                    "Toggle light and dark theme",
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.dark = !this.dark;
                    cx.notify();
                })),
            )
            .into_any_element()
    }
    fn axis(
        &self,
        label: &'static str,
        info: &'static str,
        state: &Entity<SliderState>,
        low: &'static str,
        high: &'static str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap(rem(8.))
            .mb(rem(24.))
            .child(
                div().flex().items_center().gap(rem(8.)).child(label).child(
                    button(
                        SharedString::from(format!("axis-info-{label}")),
                        Symbol::INFO,
                        "Axis information",
                    )
                    .w(rem(24.))
                    .h(rem(24.))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.status = info.into();
                        cx.notify();
                    })),
                ),
            )
            .child(MaterialSlider::new(state, label).appearance(SliderAppearance::Classic))
            .child(div().flex().justify_between().child(low).child(high))
            .into_any_element()
    }
    fn filters(&self, cx: &mut Context<Self>) -> AnyElement {
        let current = match self.style {
            Style::Outlined => "Outlined",
            Style::Rounded => "Rounded",
            Style::Sharp => "Sharp",
        };
        div()
            .id("filters-panel")
            .w(rem(320.))
            .h_full()
            .flex_shrink_0()
            .overflow_y_scroll()
            .bg(color(ColorRole::SURFACE))
            .rounded_tr(rem(24.))
            .px(rem(24.))
            .text_size(rem(16.))
            .child(
                div()
                    .h(rem(80.))
                    .flex()
                    .items_center()
                    .justify_end()
                    .child(
                        material_button("reset", ButtonVariant::Text)
                            .child(glyph(Symbol::REFRESH, 20.))
                            .child("Reset all")
                            .on_click(cx.listener(Self::reset_click)),
                    )
                    .child(
                        button("close-filters", Symbol::CLOSE, "Close filters").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.filters = false;
                                cx.notify();
                            }),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap(rem(16.))
                    .mb(rem(40.))
                    .child(glyph(Symbol::TUNE, 24.))
                    .child("Customize"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb(rem(24.))
                    .child("Fill")
                    .child(
                        MaterialSwitch::new("fill", "Fill")
                            .checked(self.filled)
                            .on_change(cx.listener(|this, value: &bool, _, cx| {
                                this.filled = *value;
                                cx.notify();
                            })),
                    ),
            )
            .child(self.axis(
                "Weight",
                "Weight controls stroke thickness from 100 to 700.",
                &self.weight,
                "100",
                "700",
                cx,
            ))
            .child(self.axis(
                "Grade",
                "Grade adjusts emphasis without changing the icon's size.",
                &self.grade,
                "-25 (low)",
                "200 (high emphasis)",
                cx,
            ))
            .child(self.axis(
                "Optical Size",
                "Optical size adjusts details for the intended display size.",
                &self.optical,
                "20px",
                "48px",
                cx,
            ))
            .child(div().h(px(1.)).bg(color(ColorRole::HAIRLINE)).my(rem(32.)))
            .child(div().text_size(rem(14.)).mb(rem(40.)).child("Filter"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(rem(16.))
                    .mb(rem(24.))
                    .child(glyph(Symbol::CHECKROOM, 24.))
                    .child("Style"),
            )
            .child(
                div()
                    .h(rem(48.))
                    .px(rem(16.))
                    .flex()
                    .items_center()
                    .rounded(rem(8.))
                    .bg(color(ColorRole::SURFACE_HIGH))
                    .text_size(rem(14.))
                    .child("Material Symbols (new)"),
            )
            .child(
                material_button("style", ButtonVariant::Text)
                    .mt(rem(4.))
                    .w_full()
                    .h(rem(48.))
                    .rounded(rem(8.))
                    .bg(color(ColorRole::SURFACE_HIGH))
                    .justify_between()
                    .child(current)
                    .child(glyph(Symbol::ARROW_DROP_DOWN, 20.))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.style_menu = !this.style_menu;
                        cx.notify();
                    })),
            )
            .when(self.style_menu, |panel| {
                panel.children(Style::ALL.map(|style| {
                    material_button(
                        SharedString::from(format!("style-{}", style.name())),
                        ButtonVariant::Text,
                    )
                    .w_full()
                    .justify_start()
                    .child(style.name())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.style = style;
                        this.style_menu = false;
                        cx.notify();
                    }))
                }))
            })
            .child(
                div()
                    .flex()
                    .gap(rem(16.))
                    .mt(rem(48.))
                    .mb(rem(24.))
                    .child(glyph(Symbol::FOLDER, 24.))
                    .child("Category"),
            )
            .child(div().flex().flex_wrap().gap(rem(8.)).pb(rem(24.)).children(
                catalog::CATEGORIES.iter().map(|&category| {
                    let selected = self.categories.contains(category);
                    material_button(
                        SharedString::from(format!("category-{category}")),
                        if selected {
                            ButtonVariant::Tonal
                        } else {
                            ButtonVariant::Text
                        },
                    )
                    .h(rem(32.))
                    .rounded(rem(8.))
                    .bg(color(if selected {
                        ColorRole::SECONDARY_CONTAINER
                    } else {
                        ColorRole::SURFACE_HIGH
                    }))
                    .child(category)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if !this.categories.remove(category) {
                            this.categories.insert(category.into());
                        }
                        this.rebuild(cx);
                        cx.notify();
                    }))
                }),
            ))
            .into_any_element()
    }
    fn reset_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.reset(window, cx)
    }
    fn header(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .h(rem(152.))
            .px(rem(60.))
            .pt(rem(16.))
            .gap(rem(24.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(rem(24.))
                    .h(rem(56.))
                    .child(
                        img(if self.dark {
                            self.dark_logo.clone()
                        } else {
                            self.logo.clone()
                        })
                        .w(rem(175.))
                        .h(rem(31.))
                        .flex_shrink_0(),
                    )
                    .child(
                        div().flex_1().min_w_0().child(
                            SearchField::new(&self.search)
                                .leading(glyph(Symbol::SEARCH, 24.))
                                .trailing(
                                    button("clear-search", Symbol::CLOSE, "Clear search").on_click(
                                        cx.listener(|this, _, w, cx| {
                                            this.search.update(cx, |s, cx| s.set_value("", w, cx));
                                            this.rebuild(cx);
                                            cx.notify();
                                        }),
                                    ),
                                ),
                        ),
                    )
                    .child(
                        div()
                            .relative()
                            .child(
                                material_button("sort", ButtonVariant::Text)
                                    .h(rem(56.))
                                    .bg(color(ColorRole::SURFACE_HIGH))
                                    .child(if self.sort == Sort::Popular {
                                        "Most popular"
                                    } else {
                                        "Name"
                                    })
                                    .child(glyph(Symbol::ARROW_DROP_DOWN, 20.))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.sort_menu = !this.sort_menu;
                                        cx.notify();
                                    })),
                            )
                            .when(self.sort_menu, |e| {
                                e.child(
                                    deferred(
                                        div()
                                            .absolute()
                                            .right_0()
                                            .top(rem(60.))
                                            .w(rem(180.))
                                            .p(rem(8.))
                                            .rounded(rem(12.))
                                            .shadow_lg()
                                            .bg(color(ColorRole::SURFACE_HIGH))
                                            .flex()
                                            .flex_col()
                                            .children(
                                                [
                                                    (Sort::Popular, "Most popular"),
                                                    (Sort::Name, "Name"),
                                                ]
                                                .into_iter()
                                                .enumerate()
                                                .map(|(i, (sort, label))| {
                                                    material_button(
                                                        ("sort-option", i),
                                                        ButtonVariant::Text,
                                                    )
                                                    .justify_start()
                                                    .child(label)
                                                    .on_click(cx.listener(move |this, _, _, cx| {
                                                        this.sort = sort;
                                                        this.sort_menu = false;
                                                        this.rebuild(cx);
                                                        cx.notify();
                                                    }))
                                                }),
                                            ),
                                    )
                                    .with_priority(2),
                                )
                            }),
                    ),
            )
            .child(
                material_button(
                    "filters",
                    if self.filters {
                        ButtonVariant::Filled
                    } else {
                        ButtonVariant::Tonal
                    },
                )
                .h(rem(48.))
                .w(rem(106.))
                .child(
                    glyph(
                        if self.filters {
                            Symbol::CLOSE
                        } else {
                            Symbol::TUNE
                        },
                        18.,
                    )
                    .text_color(color(if self.filters {
                        ColorRole::ON_PRIMARY
                    } else {
                        ColorRole::ON_SECONDARY_CONTAINER
                    })),
                )
                .child("Filters")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.filters = !this.filters;
                    cx.notify();
                })),
            )
            .into_any_element()
    }
    fn resources(&self) -> AnyElement {
        div()
            .id("resources")
            .flex()
            .gap(rem(12.))
            .h(rem(144.))
            .px(rem(60.))
            .pt(rem(24.))
            .overflow_x_scroll()
            .children(
                [
                    (
                        "Material Design icon guidelines",
                        "Learn about the latest best practices for icons",
                        "https://m3.material.io/styles/icons/overview",
                        Symbol::DESIGN_SERVICES,
                    ),
                    (
                        "GPUI API",
                        "Use Material Symbols in your native applications",
                        "https://github.com/mizorewww/gpui-material-symbols",
                        Symbol::CODE,
                    ),
                    (
                        "GitHub repo",
                        "The complete variable fonts and SVG resources",
                        "https://github.com/google/material-design-icons",
                        Symbol::NORTH_EAST,
                    ),
                    (
                        "Apache license",
                        "Material Symbols are available under Apache 2.0",
                        "https://www.apache.org/licenses/LICENSE-2.0",
                        Symbol::DESCRIPTION,
                    ),
                ]
                .map(|(title, description, url, icon)| {
                    material_button(SharedString::from(title), ButtonVariant::Text)
                        .flex_shrink_0()
                        .w(rem(256.))
                        .h(rem(120.))
                        .rounded(rem(24.))
                        .bg(color(ColorRole::SURFACE))
                        .px(rem(24.))
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(rem(4.))
                                .flex_1()
                                .min_w_0()
                                .whitespace_normal()
                                .child(div().text_size(rem(16.)).whitespace_normal().child(title))
                                .child(
                                    div()
                                        .text_size(rem(12.))
                                        .whitespace_normal()
                                        .child(description),
                                ),
                        )
                        .child(glyph(icon, 32.))
                        .on_click(move |_, _, cx| cx.open_url(url))
                }),
            )
            .into_any_element()
    }
    fn row(&mut self, index: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match self.rows[index].clone() {
            Row::Heading(category) => div()
                .h(rem(80.))
                .pt(rem(24.))
                .text_size(rem(14.))
                .child(category)
                .into_any_element(),
            Row::Empty => div()
                .p(rem(40.))
                .child("No icons found. Try another search or reset your filters.")
                .into_any_element(),
            Row::Icons(indices) => div()
                .flex()
                .h(rem(152.))
                .gap(rem(20.))
                .children(indices.into_iter().map(|index| {
                    let e = &self.entries[index];
                    let icon = self.icon(e.symbol, cx);
                    let supported = icon.is_supported();
                    let name = e.label();
                    material_button(("icon", index), ButtonVariant::Text)
                        .w(rem(112.))
                        .h(rem(128.))
                        .flex_shrink_0()
                        .rounded(rem(8.))
                        .flex_col()
                        .gap(rem(24.))
                        .px(rem(8.))
                        .py(rem(16.))
                        .justify_start()
                        .accessibility_label(name.clone())
                        .border_color(if self.selected == Some(index) {
                            color(ColorRole::ACCENT)
                        } else {
                            gpui::rgba(0)
                        })
                        .when(self.selected == Some(index), |e| {
                            e.bg(color(ColorRole::SURFACE_HIGH))
                        })
                        .child(if supported {
                            svg()
                                .path(icon.asset_path())
                                .size(rem(48.))
                                .text_color(color(ColorRole::INK))
                                .into_any_element()
                        } else {
                            glyph(Symbol::BLOCK, 40.).into_any_element()
                        })
                        .child(
                            div()
                                .text_size(rem(14.))
                                .text_align(TextAlign::Center)
                                .whitespace_normal()
                                .child(name),
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.selected = Some(index);
                            this.detail_open = true;
                            this.preview_size = 24.;
                            cx.notify();
                        }))
                }))
                .into_any_element(),
        }
    }
    fn detail(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(index) = self.selected else {
            return div().into_any_element();
        };
        let entry = &self.entries[index];
        let icon = self.icon(entry.symbol, cx);
        let api = catalog::api(icon);
        let text = entry.label();
        let color_text = self.color_input.read(cx).value();
        let preview_color = rgb(export::parse_hex(&color_text).unwrap_or(0x1f1f1f));
        let supported = icon.is_supported();
        div().id("details").absolute().right_0().top_0().bottom_0().w(rem(320.)).bg(color(ColorRole::CANVAS)).border_l_1().border_color(color(ColorRole::HAIRLINE)).overflow_y_scroll()
   .child(div().h(rem(64.)).flex().justify_end().items_center().px(rem(8.)).border_b_1().border_color(color(ColorRole::HAIRLINE)).child(button("close-detail",Symbol::CLOSE,"Close icon details").on_click(cx.listener(|this,_,_,cx|{this.detail_open=false;cx.notify();}))))
   .child(div().p(rem(16.)).flex().flex_col().gap(rem(20.))
    .child(div().flex().justify_between().items_center().child(div().text_size(rem(22.)).child(text)).child(button("reset-preview",Symbol::REFRESH,"Reset color and size").on_click(cx.listener(|this,_,w,cx|{this.preview_size=24.;this.color_input.update(cx,|s,cx|s.set_value("#1F1F1F",w,cx));cx.notify();}))))
    .child(div().flex().gap(rem(16.)).child(div().w(rem(136.)).h(rem(156.)).rounded(rem(16.)).border_1().border_color(color(ColorRole::HAIRLINE)).flex().items_center().justify_center().child(if supported{svg().path(icon.asset_path()).size(px(self.preview_size)).text_color(preview_color).into_any_element()}else{glyph(Symbol::BLOCK,32.).into_any_element()}))
     .child(div().flex_1().flex().flex_col().gap(rem(12.)).child("Size").child(div().flex().items_center().child(button("size-down",Symbol::REMOVE,"Decrease icon size").on_click(cx.listener(|this,_,_,cx|{this.preview_size=(this.preview_size-4.).max(16.);cx.notify();}))).child(format!("{}",self.preview_size)).child(button("size-up",Symbol::ADD,"Increase icon size").on_click(cx.listener(|this,_,_,cx|{this.preview_size=(this.preview_size+4.).min(96.);cx.notify();})))).child("Color").child(SearchField::new(&self.color_input))))
    .child(div().text_size(rem(14.)).max_h(rem(40.)).overflow_hidden().child(entry.tags.to_owned()))
    .when(!supported,|e|e.child(div().text_color(color(ColorRole::DANGER)).child("This SVG-only symbol has no exact variant for these axes.")))
    .child(div().flex().gap(rem(16.)).child(material_button("save-svg",ButtonVariant::Filled).flex_1().h(rem(48.)).disabled(!supported).child(glyph(Symbol::DOWNLOAD,18.).text_color(color(ColorRole::ON_PRIMARY))).child("SVG").on_click(cx.listener(move|this,_,_,cx|this.export(icon,false,cx))))
     .child(material_button("save-png",ButtonVariant::Filled).flex_1().h(rem(48.)).disabled(!supported).child(glyph(Symbol::DOWNLOAD,18.).text_color(color(ColorRole::ON_PRIMARY))).child("PNG").on_click(cx.listener(move|this,_,_,cx|this.export(icon,true,cx)))))
   )
   .child(div().h(rem(64.)).flex().items_center().justify_center().border_b_2().border_color(color(ColorRole::ACCENT)).child("GPUI"))
   .child(div().p(rem(16.)).flex().flex_col().gap(rem(20.)).child(div().text_size(rem(18.)).child("Instructions")).child("Use the current symbol and variation settings with GPUI. Copy the Rust API below.")
    .child(material_button("copy-api",ButtonVariant::Tonal).child(glyph(Symbol::CONTENT_COPY,18.)).child("Copy GPUI API").on_click(cx.listener(move|this,_,_,cx|{cx.write_to_clipboard(ClipboardItem::new_string(api.clone()));this.status="GPUI API copied".into();cx.notify();})))
    .child(div().p(rem(16.)).rounded(rem(8.)).bg(color(ColorRole::SURFACE_HIGH)).font_family("Menlo").text_size(rem(12.)).whitespace_normal().child(catalog::api(icon)))
    .child("Icon name").child(entry.symbol.name())
    .child("Code point").child(entry.symbol.codepoint(self.style).map(|c|format!("{:x}",c as u32)).unwrap_or_else(||"SVG-only".into())))
   .into_any_element()
    }
    fn export(&mut self, icon: SymbolIcon, png: bool, cx: &mut Context<Self>) {
        let color = self.color_input.read(cx).value();
        let bytes = export::encode(icon, self.preview_size as u32, &color, png);
        let Ok(bytes) = bytes else {
            self.status = "Could not render icon".into();
            cx.notify();
            return;
        };
        let extension = if png { "png" } else { "svg" };
        let receiver = cx.prompt_for_new_path(
            Path::new("."),
            Some(&format!("{}.{extension}", icon.symbol().name())),
        );
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(path))) = receiver.await {
                let result = std::fs::write(path, bytes);
                let _ = this.update(cx, |this, cx| {
                    this.status = if result.is_ok() {
                        "Icon saved".into()
                    } else {
                        "Could not save icon".into()
                    };
                    cx.notify();
                });
            }
        })
        .detach();
    }
}
impl Render for Browser {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        theme::apply_scheme(scheme(self.dark), window, cx);
        theme::apply_scale(1., window);
        let width = f32::from(window.viewport_size().width);
        let sidebar = if self.filters { 320. } else { 0. };
        let columns = ((width - 96. - sidebar - 100.) / 132.).floor().max(1.) as usize;
        if columns != self.columns {
            // Reflowing rows must not throw the reader back to the top of the catalog.
            let top = self
                .rows
                .get(self.list.logical_scroll_top().item_ix)
                .cloned();
            self.columns = columns;
            self.rebuild(cx);
            let item_ix = match top {
                Some(Row::Heading(name)) => self
                    .rows
                    .iter()
                    .position(|row| matches!(row, Row::Heading(n) if *n == name)),
                Some(Row::Icons(old)) => self
                    .rows
                    .iter()
                    .position(|row| matches!(row, Row::Icons(r) if r.contains(&old[0]))),
                _ => None,
            };
            if let Some(item_ix) = item_ix {
                self.list.scroll_to(ListOffset {
                    item_ix,
                    offset_in_item: px(0.),
                });
            }
        }
        let weak = cx.entity().downgrade();
        let list_state = self.list.clone();
        let detail = self.detail(cx);
        let rail = self.rail(cx);
        let filters = if self.filters {
            Some(self.filters(cx))
        } else {
            None
        };
        let header = self.header(cx);
        let resources = self.resources();
        div()
            .id("icons-browser")
            .size_full()
            .relative()
            .overflow_hidden()
            .flex()
            .font_family("Google Sans Flex")
            .text_size(rem(14.))
            .bg(color(ColorRole::CANVAS))
            .text_color(color(ColorRole::INK))
            .child(rail)
            .children(filters)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .flex_col()
                    .child(header)
                    .child(resources)
                    .child(
                        div().flex_1().min_h_0().px(rem(60.)).child(
                            list(list_state, move |index, w, cx| {
                                weak.update(cx, |this, cx| this.row(index, w, cx))
                                    .unwrap_or_else(|_| div().into_any_element())
                            })
                            .size_full(),
                        ),
                    ),
            )
            .child(
                MaterialSideSheet::new("details-sheet")
                    .open(self.detail_open)
                    .child(detail),
            )
            .when(!self.status.is_empty(), |root| {
                root.child(
                    div()
                        .absolute()
                        .bottom(rem(24.))
                        .left(rem(440.))
                        .rounded(rem(8.))
                        .bg(color(ColorRole::INK))
                        .text_color(color(ColorRole::CANVAS))
                        .px(rem(20.))
                        .py(rem(12.))
                        .flex()
                        .items_center()
                        .gap(rem(12.))
                        .child(self.status.clone())
                        .child(
                            button("dismiss-status", Symbol::CLOSE, "Dismiss message").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.status = "".into();
                                    cx.notify();
                                }),
                            ),
                        ),
                )
            })
    }
}
fn main() {
    if let Some(path) = std::env::args()
        .nth(1)
        .filter(|s| s == "--capture")
        .and_then(|_| std::env::args().nth(2))
    {
        capture(&path);
        return;
    }
    gpui_platform::application()
        .with_assets(Assets::with(SymbolsSource))
        .run(|cx| {
            gpui_base::init(cx);
            gpui_kit_material::init(cx);
            cx.text_system()
                .add_fonts(vec![Cow::Borrowed(include_bytes!(
                    "assets/GoogleSansFlex.ttf"
                ))])
                .expect("load bundled Google Sans Flex");
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1440.), px(1000.)),
                        cx,
                    ))),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Material Symbols · GPUI".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| Browser::new(window, cx)),
            )
            .unwrap();
            cx.activate(true);
        });
}

fn capture(directory: &str) {
    use std::sync::Arc;
    std::fs::create_dir_all(directory).unwrap();
    let mut app = HeadlessAppContext::with_platform(
        gpui_platform::current_platform(true).text_system(),
        Arc::new(Assets::with(SymbolsSource)),
        gpui_platform::current_headless_renderer,
    );
    app.update(|cx| {
        gpui_base::init(cx);
        gpui_kit_material::init(cx);
        cx.set_reduce_motion(true);
        cx.text_system()
            .add_fonts(vec![Cow::Borrowed(include_bytes!(
                "assets/GoogleSansFlex.ttf"
            ))])
            .unwrap();
    });
    let window = app
        .open_window(size(px(1440.), px(1000.)), |w, cx| {
            cx.new(|cx| Browser::new(w, cx))
        })
        .unwrap();
    for state in ["light", "dark", "detail", "search"] {
        window
            .update(&mut app, |view, w, cx| {
                view.dark = state == "dark";
                if state == "detail" {
                    view.detail_open = true;
                    view.selected = view.entries.iter().position(|e| e.symbol == Symbol::SEARCH);
                }
                if state == "search" {
                    view.selected = None;
                    view.detail_open = false;
                    view.search.update(cx, |s, cx| s.set_value("home", w, cx));
                    view.rebuild(cx);
                }
                cx.notify();
            })
            .unwrap();
        app.update_window(window.into(), |_, w, cx| {
            w.draw(cx).clear(cx);
        })
        .unwrap();

        app.update_window(window.into(), |_, w, cx| {
            w.refresh();
            w.draw(cx).clear(cx);
        })
        .unwrap();
        app.capture_screenshot(window.into())
            .unwrap()
            .save(Path::new(directory).join(format!("{state}.png")))
            .unwrap();
    }
    println!("Captured four native GPUI states to {directory}");
}

#[cfg(test)]
mod interaction_tests {
    use super::*;
    use core::prelude::v1::test;
    fn draw(cx: &mut VisualTestContext) {
        cx.update(|w, cx| {
            w.refresh();
            w.draw(cx).clear(cx);
        });
    }
    fn click(cx: &mut VisualTestContext, id: ElementId) {
        let point = cx.update(|w, _| {
            gpui_base::test_support::find(w, &[], &id)
                .expect("visible control")
                .bounds()
                .center()
        });
        cx.simulate_click(point, Modifiers::default());
        draw(cx);
    }
    #[gpui::test]
    fn reflowing_columns_keeps_scroll_position(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            gpui_kit_material::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(Browser::new);
        draw(cx);
        let entry = view.update(cx, |v, _| {
            // An icon row halfway down the catalog, not its first heading.
            let (ix, entry) = v
                .rows
                .iter()
                .enumerate()
                .skip(v.rows.len() / 2)
                .find_map(|(ix, row)| match row {
                    Row::Icons(row) => Some((ix, row[0])),
                    _ => None,
                })
                .unwrap();
            v.list.scroll_to(ListOffset {
                item_ix: ix,
                offset_in_item: px(0.),
            });
            entry
        });
        draw(cx);
        let columns = view.read_with(cx, |v, _| v.columns);
        view.update(cx, |v, cx| {
            v.filters = !v.filters;
            cx.notify();
        });
        draw(cx);
        view.read_with(cx, |v, _| {
            assert_ne!(v.columns, columns, "toggling filters must reflow the grid");
            let top = v.list.logical_scroll_top().item_ix;
            assert!(matches!(&v.rows[top], Row::Icons(r) if r.contains(&entry)));
        });
    }
    #[gpui::test]
    fn selection_sheet_sort_and_search_are_connected(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            gpui_kit_material::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(Browser::new);
        draw(cx);
        click(cx, "sort".into());
        cx.update(|_, cx| assert!(view.read(cx).sort_menu));
        click(cx, ("sort-option", 1usize).into());
        cx.update(|_, cx| {
            assert!(view.read(cx).sort == Sort::Name);
            assert!(!view.read(cx).sort_menu);
        });
        view.update(cx, |v, cx| {
            v.sort = Sort::Popular;
            v.rebuild(cx);
            cx.notify();
        });
        draw(cx);
        let index = cx.update(|_, cx| {
            view.read(cx)
                .entries
                .iter()
                .position(|e| e.symbol == Symbol::SEARCH)
                .unwrap()
        });
        click(cx, ("icon", index).into());
        cx.update(|_, cx| assert!(view.read(cx).detail_open));
        click(cx, "close-detail".into());
        cx.update(|_, cx| assert!(!view.read(cx).detail_open));
        cx.update(|w, cx| {
            let input = view.read(cx).search.clone();
            input.update(cx, |s, cx| s.focus(w, cx));
        });
        cx.simulate_input("zzzz_no_icon");
        draw(cx);
        cx.update(|_, cx| assert!(matches!(view.read(cx).rows.as_slice(), [Row::Empty])));
        click(cx, "clear-search".into());
        cx.update(|_, cx| assert!(view.read(cx).rows.len() > 1));
    }
}
