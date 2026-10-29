//! A native Material Symbols browser built with gpui-kit-material components.
//!
//! `cargo run -p gpui-material-symbols-browser --bin material_symbols`
//!
//! The `web` crate builds the same browser for the web with `run_web`.
#[cfg(not(target_family = "wasm"))]
extern crate gpui_kit as gpui;

mod catalog;
mod export;
use catalog::{Entry, Sort};
use gpui::{prelude::*, *};
use gpui_base::{
    input::{InputEvent, InputState},
    slider::{SliderEvent, SliderState},
};
use gpui_kit_material::{color::Scheme, icons::Assets, prelude::*, theme};
use gpui_material_symbols::{Icon as SymbolIcon, Style, Symbol};
use std::borrow::Cow;

/// The app bar doubles as the window title bar.
const APP_BAR: Pixels = px(64.);
const TILE: f32 = 104.;
const TILE_GAP: f32 = 8.;
const GRID_INSET: f32 = 24.;
const GRADES: [f32; 3] = [-25., 0., 200.];
const OPTICAL_SIZES: [f32; 4] = [20., 24., 40., 48.];
const DEFAULT_COLOR: &str = "#1F1F1F";
const SHEET: f32 = 360.;
/// Below this window width the browser uses its compact phone layout: the
/// controls fold behind a button, and details cover the grid.
const COMPACT: f32 = 720.;
const COMPACT_INSET: f32 = 12.;
/// Linked from the web build's app bar.
const REPOSITORY: &str = "https://github.com/mizorewww/gpui-material-symbols";
/// The GitHub mark (Primer Octicons, MIT) for that link.
const GITHUB_MARK: &str = "github-mark.svg";

struct SymbolsSource;
impl AssetSource for SymbolsSource {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        #[cfg(target_family = "wasm")]
        if path == GITHUB_MARK {
            return Ok(Some(Cow::Borrowed(include_bytes!(
                "assets/github-mark.svg"
            ))));
        }
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

/// The icon's SVG, or an empty one of the same size until the web build has
/// downloaded the asset it needs.
fn symbol_svg(icon: SymbolIcon) -> Svg {
    let svg = svg();
    if loader::ready(icon.required_asset()) {
        svg.path(icon.asset_path())
    } else {
        svg
    }
}

fn glyph(symbol: Symbol, size: f32) -> Svg {
    symbol_svg(symbol.icon())
        .size(rem(size))
        .text_color(color(ColorRole::ON_SURFACE))
        .flex_shrink_0()
}

fn icon_button(id: impl Into<ElementId>, symbol: Symbol, label: &str) -> MaterialButton {
    material_button(id, ButtonVariant::Text)
        .accessibility_label(label.to_owned())
        .w(rem(40.))
        .px_0()
        .child(glyph(symbol, 24.))
}

/// A labelled control with its current value, for the toolbar.
fn labelled(label: &'static str, value: String, control: impl IntoElement) -> Div {
    div()
        .flex()
        .items_center()
        .gap(rem(12.))
        .child(
            div()
                .flex()
                .gap(rem(6.))
                .child(label)
                .when(!value.is_empty(), |e| {
                    e.child(
                        div()
                            .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                            .child(value),
                    )
                }),
        )
        .child(control)
}

#[derive(Clone)]
enum Row {
    Heading(&'static str),
    Icons(Vec<usize>),
    Empty,
}

struct Browser {
    entries: Vec<Entry>,
    rows: Vec<Row>,
    shown: usize,
    list: ListState,
    columns: usize,
    search: Entity<InputState>,
    weight: Entity<SliderState>,
    grade: Entity<SliderState>,
    optical: Entity<SliderState>,
    color_input: Entity<InputState>,
    style: Style,
    filled: bool,
    theme: Appearance,
    sort: Sort,
    selected: Option<usize>,
    detail_open: bool,
    /// The compact layout's style and axis controls are shown.
    controls_open: bool,
    preview_size: f32,
    status: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl Browser {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search icons"));
        let color_input = cx.new(|cx| {
            let mut s = InputState::new(window, cx).placeholder(DEFAULT_COLOR);
            s.set_value(DEFAULT_COLOR, window, cx);
            s
        });
        let slider = |max: f32, step: f32, value: f32| {
            SliderState::new()
                .min(0.)
                .max(max)
                .step(step)
                .default_value(value)
        };
        let weight = cx.new(|_| slider(700., 100., 400.).min(100.));
        let grade = cx.new(|_| slider(2., 1., 1.));
        let optical = cx.new(|_| slider(3., 1., 1.));
        let mut subscriptions = vec![
            cx.subscribe(&search, |this, _, event, cx| {
                if matches!(event, InputEvent::Change) {
                    this.rebuild(cx);
                    cx.notify();
                }
            }),
            cx.subscribe(&color_input, |_, _, _: &InputEvent, cx| cx.notify()),
            // Repaint when the system switches between light and dark.
            cx.observe_window_appearance(window, |_, _, cx| cx.notify()),
        ];
        for state in [&weight, &grade, &optical] {
            subscriptions.push(cx.subscribe(state, |_, _, event, cx| {
                if matches!(event, SliderEvent::Change(_)) {
                    cx.notify();
                }
            }));
        }
        let mut this = Self {
            entries: catalog::entries(),
            rows: vec![],
            shown: 0,
            list: ListState::new(0, ListAlignment::Top, px(500.)),
            columns: 8,
            search,
            weight,
            grade,
            optical,
            color_input,
            style: Style::Outlined,
            filled: false,
            theme: Appearance::System,
            sort: Sort::Popular,
            selected: None,
            detail_open: false,
            controls_open: false,
            preview_size: 48.,
            status: "".into(),
            _subscriptions: subscriptions,
        };
        this.rebuild(cx);
        // The text scale never changes, so set it once; this also paints the
        // first palette without a transition.
        theme::advance_component_theme(Scheme::from_palette(this.palette(cx)), 1., window, cx);
        this
    }

    /// Group the matching entries by category into rows of `columns` tiles.
    fn rebuild(&mut self, cx: &App) {
        let indices = catalog::filtered(&self.entries, &self.search.read(cx).value(), self.sort);
        self.shown = indices.len();
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

    /// Reflow the grid for a new column count without losing the reader's place.
    fn reflow(&mut self, columns: usize, cx: &App) {
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

    fn palette(&self, cx: &App) -> PaletteId {
        ThemePreferences {
            mode: self.theme,
            ..Default::default()
        }
        .resolve(theme::system_dark(cx))
    }

    fn icon(&self, symbol: Symbol, cx: &App) -> SymbolIcon {
        catalog::configured(
            symbol,
            self.style,
            self.filled,
            self.weight.read(cx).value().end(),
            GRADES[self.grade.read(cx).value().end() as usize],
            OPTICAL_SIZES[self.optical.read(cx).value().end() as usize],
        )
    }

    fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.style = Style::Outlined;
        self.filled = false;
        self.sort = Sort::Popular;
        self.preview_size = 48.;
        self.search.update(cx, |s, cx| s.set_value("", window, cx));
        self.color_input
            .update(cx, |s, cx| s.set_value(DEFAULT_COLOR, window, cx));
        for (s, v) in [(&self.weight, 400.), (&self.grade, 1.), (&self.optical, 1.)] {
            s.update(cx, |s, cx| s.set_value(v, window, cx));
        }
        self.rebuild(cx);
        cx.notify();
    }

    /// The app bar is the window's title bar, drawn by the kit's `TitleBar`
    /// on the same surface as the content.
    fn app_bar(&self, compact: bool, cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new(
            "app-bar",
            APP_BAR,
            CaptionLabels {
                minimize: "Minimize".into(),
                maximize: "Maximize".into(),
                restore: "Restore".into(),
                close: "Close".into(),
            },
        )
        .gap(rem(if compact { 4. } else { 16. }))
        .when(!compact, |bar| bar.child(glyph(Symbol::INTERESTS, 24.)))
        .when(!compact, |bar| {
            bar.child(
                div()
                    .flex()
                    .flex_col()
                    .flex_shrink_0()
                    .child(div().text_size(rem(16.)).child("Material Symbols"))
                    .child(
                        div()
                            .text_size(rem(12.))
                            .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                            .child(format!("{} of {} icons", self.shown, self.entries.len())),
                    ),
            )
        })
        .child(
            title_bar_control(
                div()
                    .id("search")
                    .flex_1()
                    .min_w_0()
                    .max_w(rem(640.))
                    .mx_auto(),
            )
            .child(
                SearchField::new(&self.search)
                    .leading(glyph(Symbol::SEARCH, 24.))
                    .trailing(
                        icon_button("clear-search", Symbol::CLOSE, "Clear search").on_click(
                            cx.listener(|this, _, w, cx| {
                                this.search.update(cx, |s, cx| s.set_value("", w, cx));
                                this.rebuild(cx);
                                cx.notify();
                            }),
                        ),
                    ),
            ),
        )
        .when(!compact, |bar| {
            bar.child(title_bar_control(div().child(self.sort_group(cx))))
        })
        .when(compact, |bar| {
            bar.child(title_bar_control(
                icon_button("controls", Symbol::TUNE, "Style and axes")
                    .when(self.controls_open, |b| {
                        b.bg(color(ColorRole::SECONDARY_CONTAINER))
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.controls_open = !this.controls_open;
                        cx.notify();
                    })),
            ))
        })
        .child(title_bar_control(
            icon_button(
                "theme",
                if self.palette(cx) == PaletteId::Ink {
                    Symbol::LIGHT_MODE
                } else {
                    Symbol::DARK_MODE
                },
                "Toggle light and dark theme",
            )
            .on_click(cx.listener(|this, _, _, cx| {
                // An explicit choice overrides the system appearance.
                this.theme = if this.palette(cx) == PaletteId::Ink {
                    Appearance::Light
                } else {
                    Appearance::Dark
                };
                cx.notify();
            })),
        ))
        .when(cfg!(target_family = "wasm"), |bar| {
            bar.child(title_bar_control(
                material_button("github", ButtonVariant::Tonal)
                    .accessibility_label(format!("Source code on GitHub: {REPOSITORY}"))
                    .child(
                        svg()
                            .path(GITHUB_MARK)
                            .size(rem(18.))
                            .text_color(color(ColorRole::ON_SECONDARY_CONTAINER))
                            .flex_shrink_0(),
                    )
                    .when(!compact, |b| b.child("GitHub"))
                    .when(compact, |b| b.w(rem(40.)).px_0())
                    .on_click(|_, _, cx| cx.open_url(REPOSITORY)),
            ))
        })
    }

    fn sort_group(&self, cx: &mut Context<Self>) -> ConnectedButtonGroup {
        ConnectedButtonGroup::new("sort", "Sort")
            .options([("popular", "Popular"), ("name", "A–Z")])
            .selected(match self.sort {
                Sort::Popular => "popular",
                Sort::Name => "name",
            })
            .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                this.sort = if value == "name" {
                    Sort::Name
                } else {
                    Sort::Popular
                };
                this.rebuild(cx);
                cx.notify();
            }))
    }

    /// Style, fill, the three numeric axes and reset; in the compact layout
    /// also the sort order.
    fn toolbar(&self, compact: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let axis = |state: &Entity<SliderState>, label: &'static str, width: f32| {
            div()
                .w(rem(width))
                .child(MaterialSlider::new(state, label).appearance(SliderAppearance::Classic))
        };
        let weight = self.weight.read(cx).value().end();
        let grade = GRADES[self.grade.read(cx).value().end() as usize];
        let optical = OPTICAL_SIZES[self.optical.read(cx).value().end() as usize];
        div()
            .flex()
            .flex_wrap()
            .flex_shrink_0()
            .items_center()
            .gap_x(rem(if compact { 16. } else { 24. }))
            .gap_y(rem(12.))
            .px(px(inset(compact)))
            .py(rem(12.))
            .border_b_1()
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .child(
                ConnectedButtonGroup::new("style", "Style")
                    .options(Style::ALL.map(|s| (s.name(), title_case(s.name()))))
                    .selected(self.style.name())
                    .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                        if let Some(style) =
                            Style::ALL.into_iter().find(|s| s.name() == value.as_ref())
                        {
                            this.style = style;
                            cx.notify();
                        }
                    })),
            )
            .when(compact, |bar| bar.child(self.sort_group(cx)))
            .child(labelled(
                "Fill",
                String::new(),
                MaterialSwitch::new("fill", "Fill")
                    .checked(self.filled)
                    .on_change(cx.listener(|this, value: &bool, _, cx| {
                        this.filled = *value;
                        cx.notify();
                    })),
            ))
            .child(labelled(
                "Weight",
                format!("{weight}"),
                axis(&self.weight, "Weight", 160.),
            ))
            .child(labelled(
                "Grade",
                format!("{grade}"),
                axis(&self.grade, "Grade", 96.),
            ))
            .child(labelled(
                "Optical size",
                format!("{optical}"),
                axis(&self.optical, "Optical size", 112.),
            ))
            .child(
                material_button("reset", ButtonVariant::Text)
                    .child(glyph(Symbol::REFRESH, 20.))
                    .child("Reset")
                    .on_click(cx.listener(|this, _, window, cx| this.reset(window, cx))),
            )
    }

    fn row(&mut self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        match self.rows[index].clone() {
            Row::Heading(category) => div()
                .pt(rem(20.))
                .pb(rem(8.))
                .text_size(rem(14.))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                .child(category)
                .into_any_element(),
            Row::Empty => div()
                .py(rem(48.))
                .flex()
                .justify_center()
                .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                .child("No icons match your search.")
                .into_any_element(),
            Row::Icons(indices) => div()
                .flex()
                .gap(px(TILE_GAP))
                .pb(px(TILE_GAP))
                .children(
                    indices
                        .into_iter()
                        .map(|index| self.tile(index, cx).into_any_element())
                        .collect::<Vec<_>>(),
                )
                .into_any_element(),
        }
    }

    fn tile(&self, index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let entry = &self.entries[index];
        let icon = self.icon(entry.symbol, cx);
        let selected = self.selected == Some(index) && self.detail_open;
        let name = entry.label();
        material_button(("icon", index), ButtonVariant::Text)
            .w(px(TILE))
            .h(px(TILE))
            .flex_shrink_0()
            .flex_col()
            .gap(rem(10.))
            .px(rem(6.))
            .rounded(rem(12.))
            .accessibility_label(name.clone())
            .when(selected, |e| e.bg(color(ColorRole::SECONDARY_CONTAINER)))
            .child(if icon.is_supported() {
                symbol_svg(icon)
                    .size(rem(36.))
                    .text_color(color(ColorRole::ON_SURFACE))
                    .into_any_element()
            } else {
                glyph(Symbol::BLOCK, 28.)
                    .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                    .into_any_element()
            })
            .child(
                div()
                    .h(rem(32.))
                    .text_size(rem(12.))
                    .line_height(rem(16.))
                    .text_align(TextAlign::Center)
                    .whitespace_normal()
                    .overflow_hidden()
                    .child(name),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.selected = Some(index);
                this.detail_open = true;
                cx.notify();
            }))
    }

    fn detail(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(index) = self.selected else {
            return div().into_any_element();
        };
        let entry = &self.entries[index];
        let icon = self.icon(entry.symbol, cx);
        let supported = icon.is_supported();
        let code = catalog::api(icon);
        let code_text = code.clone();
        let preview_color =
            rgb(export::parse_hex(&self.color_input.read(cx).value()).unwrap_or(0x1f1f1f));
        let codepoint = entry
            .symbol
            .codepoint(self.style)
            .map(|c| format!("U+{:04X}", c as u32))
            .unwrap_or_else(|| "SVG only".into());
        let fact = |label: &'static str, value: String| {
            div()
                .flex()
                .justify_between()
                .gap(rem(16.))
                .child(
                    div()
                        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                        .child(label),
                )
                .child(value)
        };
        let export_button = |id: &'static str, label: &'static str, png: bool| {
            material_button(id, ButtonVariant::Filled)
                .flex_1()
                .disabled(!supported)
                .child(glyph(Symbol::DOWNLOAD, 18.).text_color(color(ColorRole::ON_PRIMARY)))
                .child(label)
                .on_click(cx.listener(move |this, _, _, cx| this.export(icon, png, cx)))
        };
        let content = div()
            .flex()
            .flex_col()
            .gap(rem(20.))
            .p(rem(20.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().text_size(rem(20.)).child(entry.label()))
                    .child(
                        icon_button("close-detail", Symbol::CLOSE, "Close details").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.detail_open = false;
                                cx.notify();
                            }),
                        ),
                    ),
            )
            .child(
                div()
                    .h(rem(176.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(rem(16.))
                    // A fixed canvas, so the export color reads in either theme.
                    .bg(rgb(0xffffff))
                    .border_1()
                    .border_color(color(ColorRole::OUTLINE_VARIANT))
                    .child(if supported {
                        symbol_svg(icon)
                            .size(px(self.preview_size))
                            .text_color(preview_color)
                            .into_any_element()
                    } else {
                        div()
                            .px(rem(24.))
                            .text_align(TextAlign::Center)
                            .text_color(color(ColorRole::ERROR))
                            .child("This SVG-only symbol has no official variant for these axes.")
                            .into_any_element()
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(rem(8.))
                    .child(fact("Name", entry.symbol.name().to_owned()))
                    .child(fact("Code point", codepoint))
                    .child(fact("Category", entry.category.to_owned()))
                    .when(!entry.tags.is_empty(), |facts| {
                        facts.child(
                            div()
                                .text_size(rem(12.))
                                .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                                .line_clamp(3)
                                .child(entry.tags),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(rem(12.))
                    .child(
                        icon_button("size-down", Symbol::REMOVE, "Decrease export size").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.preview_size = (this.preview_size - 8.).max(16.);
                                cx.notify();
                            }),
                        ),
                    )
                    .child(format!("{} px", self.preview_size))
                    .child(
                        icon_button("size-up", Symbol::ADD, "Increase export size").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.preview_size = (this.preview_size + 8.).min(96.);
                                cx.notify();
                            }),
                        ),
                    )
                    .child(div().flex_1().child(SearchField::new(&self.color_input))),
            )
            .child(
                div()
                    .flex()
                    .gap(rem(12.))
                    .child(export_button("save-svg", "SVG", false))
                    .child(export_button("save-png", "PNG", true)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().text_size(rem(16.)).child("Rust"))
                    .child(
                        material_button("copy-code", ButtonVariant::Tonal)
                            .child(glyph(Symbol::CONTENT_COPY, 18.))
                            .child("Copy")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(code.clone()));
                                this.status = "Code copied".into();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .p(rem(16.))
                    .rounded(rem(12.))
                    .bg(color(ColorRole::SURFACE_CONTAINER_HIGHEST))
                    .font_family("Menlo")
                    .text_size(rem(12.))
                    .whitespace_normal()
                    .child(code_text),
            );
        div()
            .id("details")
            .size_full()
            .overflow_y_scroll()
            .bg(color(ColorRole::SURFACE_CONTAINER))
            .border_l_1()
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .child(content)
            .into_any_element()
    }

    fn export(&mut self, icon: SymbolIcon, png: bool, cx: &mut Context<Self>) {
        let color = self.color_input.read(cx).value();
        if export::parse_hex(&color).is_none() {
            self.status = "Enter the color as #RRGGBB".into();
            cx.notify();
            return;
        }
        let Ok(bytes) = export::encode(icon, self.preview_size as u32, &color, png) else {
            self.status = if loader::ready(icon.required_asset()) {
                "Could not render icon"
            } else {
                "The icon is still loading"
            }
            .into();
            cx.notify();
            return;
        };
        let extension = if png { "png" } else { "svg" };
        let name = format!("{}.{extension}", icon.symbol().name());
        #[cfg(target_family = "wasm")]
        {
            let mime = if png { "image/png" } else { "image/svg+xml" };
            web::download(&name, mime, &bytes);
            self.status = "Icon downloaded".into();
            cx.notify();
        }
        #[cfg(not(target_family = "wasm"))]
        self.save(&name, bytes, cx);
    }

    #[cfg(not(target_family = "wasm"))]
    fn save(&mut self, name: &str, bytes: Vec<u8>, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_new_path(std::path::Path::new("."), Some(name));
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

/// Space between the window edge and the grid or toolbar.
fn inset(compact: bool) -> f32 {
    if compact { COMPACT_INSET } else { GRID_INSET }
}

fn title_case(name: &str) -> String {
    let mut chars = name.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

impl Render for Browser {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(failure) = loader::take_failure() {
            self.status = failure;
        }
        let palette = self.palette(cx);
        theme::apply_scheme(Scheme::from_palette(palette), window, cx);
        let window_width = f32::from(window.viewport_size().width);
        let compact = window_width < COMPACT;
        let sheet_width = f32::from(rem(SHEET).to_pixels(window.rem_size()));
        // The details sheet sits beside the grid, so tiles never hide under
        // it; in the compact layout it covers the whole width instead.
        let sheet = if self.detail_open && !compact {
            sheet_width
        } else {
            0.
        };
        let width = window_width - sheet;
        let columns = ((width - 2. * inset(compact) + TILE_GAP) / (TILE + TILE_GAP))
            .floor()
            .max(1.) as usize;
        if columns != self.columns {
            self.reflow(columns, cx);
        }
        let weak = cx.entity().downgrade();
        let grid = list(self.list.clone(), move |index, _, cx| {
            weak.update(cx, |this, cx| this.row(index, cx))
                .unwrap_or_else(|_| div().into_any_element())
        })
        .size_full();
        div()
            .id("icons-browser")
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .text_size(rem(14.))
            .bg(color(ColorRole::SURFACE_CONTAINER))
            .text_color(color(ColorRole::ON_SURFACE))
            .child(self.app_bar(compact, cx))
            .when(!compact || self.controls_open, |root| {
                root.child(self.toolbar(compact, cx))
            })
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .child(
                        div()
                            .size_full()
                            .pl(px(inset(compact)))
                            .pr(px(inset(compact) + sheet))
                            .pb(rem(16.))
                            .child(grid),
                    )
                    .child(
                        MaterialSideSheet::new("details-sheet")
                            .open(self.detail_open)
                            // The kit's `rem` takes 14px-based design units.
                            .width(rem(if compact {
                                SHEET * window_width / sheet_width
                            } else {
                                SHEET
                            }))
                            .child(self.detail(cx)),
                    )
                    .when(!self.status.is_empty(), |body| {
                        body.child(
                            div()
                                .absolute()
                                .bottom(rem(24.))
                                .left(px(inset(compact)))
                                .right(px(inset(compact)))
                                .child(Snackbar::new("status", self.status.clone()).close(
                                    "Dismiss",
                                    cx.listener(|this, _, _, cx| {
                                        this.status = "".into();
                                        cx.notify();
                                    }),
                                )),
                        )
                    }),
            )
    }
}

fn window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        window_min_size: Some(size(px(720.), px(480.))),
        ..title_bar_window_options("Material Symbols", APP_BAR, size(px(1280.), px(860.)), cx)
    }
}

/// The native build embeds every asset.
#[cfg(not(target_family = "wasm"))]
mod loader {
    pub fn ready(asset: gpui_material_symbols::Asset) -> bool {
        asset.is_available()
    }

    pub fn take_failure() -> Option<gpui::SharedString> {
        None
    }
}

/// Asks the page for the assets the web build does not embed when an icon
/// first needs one, and redraws when it arrives. The page downloads them in
/// resumable chunks; a failure is reported and retried after a pause.
#[cfg(target_family = "wasm")]
pub mod loader {
    use gpui::{AsyncApp, SharedString};
    use gpui_material_symbols::Asset;
    use std::{cell::RefCell, time::Duration};

    const RETRY: Duration = Duration::from_secs(5);

    struct Loader {
        app: AsyncApp,
        requested: Vec<Asset>,
        failure: Option<SharedString>,
    }

    thread_local! {
        static LOADER: RefCell<Option<Loader>> = const { RefCell::new(None) };
    }

    pub fn init(cx: &gpui::App) {
        LOADER.set(Some(Loader {
            app: cx.to_async(),
            requested: Vec::new(),
            failure: None,
        }));
    }

    /// A download failure not yet shown to the user.
    pub fn take_failure() -> Option<SharedString> {
        LOADER.with_borrow_mut(|loader| loader.as_mut()?.failure.take())
    }

    /// Whether `asset` is ready, starting its download if it is not.
    pub fn ready(asset: Asset) -> bool {
        if asset.is_available() {
            return true;
        }
        LOADER.with_borrow_mut(|loader| {
            let Some(loader) = loader else { return };
            if loader.requested.contains(&asset) {
                return;
            }
            loader.requested.push(asset);
            let app = loader.app.clone();
            app.foreground_executor()
                .spawn({
                    let app = app.clone();
                    async move {
                        let result = match super::web::fetch_asset(asset.file_name()).await {
                            Ok(bytes) => asset.provide(bytes).map_err(|e| e.to_string()),
                            Err(error) => Err(error),
                        };
                        if let Err(error) = &result
                            && error.starts_with("reload:")
                        {
                            // The site was redeployed; retrying cannot help.
                            log::error!("{}: {error}", asset.file_name());
                            LOADER.with_borrow_mut(|loader| {
                                if let Some(loader) = loader {
                                    loader.failure = Some(
                                        "The site was updated. Reload the page to see these icons."
                                            .into(),
                                    );
                                }
                            });
                            app.update(|cx| cx.refresh_windows());
                            return;
                        }
                        if let Err(error) = result {
                            log::error!("{}: {error}", asset.file_name());
                            LOADER.with_borrow_mut(|loader| {
                                if let Some(loader) = loader {
                                    loader.failure =
                                        Some("Could not download icons; retrying".into());
                                }
                            });
                            app.update(|cx| cx.refresh_windows());
                            app.background_executor().timer(RETRY).await;
                            LOADER.with_borrow_mut(|loader| {
                                if let Some(loader) = loader {
                                    loader.requested.retain(|a| *a != asset);
                                }
                            });
                        }
                        app.update(|cx| cx.refresh_windows());
                    }
                })
                .detach();
        });
        false
    }
}

#[cfg(target_family = "wasm")]
mod web {
    use wasm_bindgen::prelude::*;

    /// The asset file `name`, downloaded by the page (`web/www/main.js`).
    pub async fn fetch_asset(name: &str) -> Result<Vec<u8>, String> {
        let bytes = wasm_bindgen_futures::JsFuture::from(fetch_asset_promise(name))
            .await
            .map_err(|error| error.as_string().unwrap_or_else(|| format!("{error:?}")))?;
        Ok(js_sys::Uint8Array::new(&bytes).to_vec())
    }

    #[wasm_bindgen]
    extern "C" {
        /// Saves `bytes` through the browser's download, defined by the page.
        #[wasm_bindgen(js_name = materialSymbolsDownload)]
        pub fn download(name: &str, mime: &str, bytes: &[u8]);
        /// Resolves to the asset's bytes; see `materialSymbolsFetchAsset`.
        #[wasm_bindgen(js_name = materialSymbolsFetchAsset)]
        fn fetch_asset_promise(name: &str) -> js_sys::Promise;
    }
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    if std::env::args().nth(1).as_deref() == Some("--capture") {
        let directory = std::env::args()
            .nth(2)
            .expect("usage: --capture <directory>");
        capture(&directory);
        return;
    }
    run(gpui_kit::platform::application());
}

/// Opens the browser window natively.
#[cfg(not(target_family = "wasm"))]
fn run(application: Application) {
    start(application, |_| {});
}

/// Opens the browser on the web, which asks the page for each asset when an
/// icon first needs it.
#[cfg(target_family = "wasm")]
pub fn run_web(application: Application) {
    start(application, |cx| loader::init(cx));
}

fn start(application: Application, setup: impl FnOnce(&mut App) + 'static) {
    application
        .with_assets(Assets::with(SymbolsSource))
        .run(|cx| {
            setup(cx);
            // The web has no system fonts; this is its default UI family.
            #[cfg(target_family = "wasm")]
            cx.text_system()
                .add_fonts(vec![Cow::Borrowed(include_bytes!(
                    "../../web/fonts/IBMPlexSans-Regular.ttf"
                ))])
                .expect("load the UI font");
            gpui_base::init(cx);
            gpui_kit_material::init(cx);
            cx.open_window(window_options(cx), |window, cx| {
                cx.new(|cx| Browser::new(window, cx))
            })
            .expect("open browser window");
            cx.activate(true);
        });
}

/// Render the README states through the native headless renderer.
#[cfg(not(target_family = "wasm"))]
fn capture(directory: &str) {
    use std::{path::Path, sync::Arc};
    std::fs::create_dir_all(directory).unwrap();
    let mut app = HeadlessAppContext::with_platform(
        gpui_kit::platform::current_platform(true).text_system(),
        Arc::new(Assets::with(SymbolsSource)),
        gpui_kit::platform::current_headless_renderer,
    );
    app.update(|cx| {
        gpui_base::init(cx);
        gpui_kit_material::init(cx);
        cx.set_reduce_motion(true);
    });
    let window = app
        .open_window(size(px(1280.), px(860.)), |w, cx| {
            cx.new(|cx| Browser::new(w, cx))
        })
        .unwrap();
    for state in ["light", "dark", "detail", "detail-dark", "search"] {
        window
            .update(&mut app, |view, w, cx| {
                view.theme = if state.ends_with("dark") {
                    Appearance::Dark
                } else {
                    Appearance::Light
                };
                view.detail_open = state.starts_with("detail");
                view.selected = view
                    .detail_open
                    .then(|| view.entries.iter().position(|e| e.symbol == Symbol::SEARCH))
                    .flatten();
                let query = if state == "search" { "home" } else { "" };
                view.search.update(cx, |s, cx| s.set_value(query, w, cx));
                view.rebuild(cx);
                cx.notify();
            })
            .unwrap();
        // The first frame requests SVG assets; the second paints them.
        for _ in 0..2 {
            app.update_window(window.into(), |_, w, cx| {
                w.refresh();
                w.draw(cx).clear(cx);
            })
            .unwrap();
        }
        app.capture_screenshot(window.into())
            .unwrap()
            .save(Path::new(directory).join(format!("{state}.png")))
            .unwrap();
    }
    println!("Captured five native GPUI states to {directory}");
}

#[cfg(test)]
mod interaction_tests {
    use super::*;
    use core::prelude::v1::test;
    fn setup(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            gpui_kit_material::init(cx);
            cx.set_reduce_motion(true);
        });
    }
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
        setup(cx);
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
        cx.simulate_resize(size(px(760.), px(700.)));
        draw(cx);
        view.read_with(cx, |v, _| {
            assert_ne!(v.columns, columns, "resizing must reflow the grid");
            let top = v.list.logical_scroll_top().item_ix;
            assert!(matches!(&v.rows[top], Row::Icons(r) if r.contains(&entry)));
        });
    }
    #[gpui::test]
    fn details_sort_and_search_are_connected(cx: &mut TestAppContext) {
        setup(cx);
        let (view, cx) = cx.add_window_view(Browser::new);
        draw(cx);
        let index = view.read_with(cx, |v, _| {
            v.entries
                .iter()
                .position(|e| e.symbol == Symbol::SEARCH)
                .unwrap()
        });
        click(cx, ("icon", index).into());
        view.read_with(cx, |v, _| {
            assert!(v.detail_open);
            assert_eq!(v.selected, Some(index));
        });
        click(cx, "close-detail".into());
        view.read_with(cx, |v, _| assert!(!v.detail_open));

        click(cx, "name".into());
        view.read_with(cx, |v, _| {
            assert!(v.sort == Sort::Name);
            let Some(Row::Icons(first)) = v.rows.get(1) else {
                panic!("expected an icon row after the first heading");
            };
            let names: Vec<_> = first.iter().map(|&i| v.entries[i].symbol.name()).collect();
            assert!(names.windows(2).all(|w| w[0] <= w[1]));
        });

        cx.update(|w, cx| {
            let input = view.read(cx).search.clone();
            input.update(cx, |s, cx| s.focus(w, cx));
        });
        cx.simulate_input("zzzz_no_icon");
        draw(cx);
        view.read_with(cx, |v, _| {
            assert!(matches!(v.rows.as_slice(), [Row::Empty]));
            assert_eq!(v.shown, 0);
        });
        click(cx, "clear-search".into());
        view.read_with(cx, |v, _| assert_eq!(v.shown, v.entries.len()));
    }
    #[gpui::test]
    fn details_sheet_takes_columns_from_the_grid(cx: &mut TestAppContext) {
        setup(cx);
        let (view, cx) = cx.add_window_view(Browser::new);
        draw(cx);
        let (columns, first) = view.read_with(cx, |v, _| {
            let Some(Row::Icons(row)) = v.rows.get(1) else {
                panic!("expected an icon row after the first heading");
            };
            (v.columns, row[0])
        });
        click(cx, ("icon", first).into());
        draw(cx);
        view.read_with(cx, |v, _| {
            assert!(v.detail_open);
            assert!(v.columns < columns, "{} should be < {columns}", v.columns);
        });
    }
    #[gpui::test]
    fn toolbar_and_theme_controls_change_the_rendered_icons(cx: &mut TestAppContext) {
        setup(cx);
        let (view, cx) = cx.add_window_view(Browser::new);
        draw(cx);
        let home = |cx: &mut VisualTestContext| {
            view.update(cx, |v, cx| v.icon(Symbol::HOME, cx).asset_path())
        };
        let outlined = home(cx);
        click(cx, "rounded".into());
        assert!(home(cx).starts_with("material-symbols/rounded/"));
        click(cx, "fill".into());
        let filled = home(cx);
        assert!(filled.contains("/home/1/"), "{filled}");
        assert_ne!(outlined, filled);

        let dark = view.update(cx, |v, cx| v.palette(cx) == PaletteId::Ink);
        click(cx, "theme".into());
        view.update(cx, |v, cx| {
            assert_ne!(v.palette(cx) == PaletteId::Ink, dark);
            assert!(v.theme != Appearance::System);
        });
    }
}
