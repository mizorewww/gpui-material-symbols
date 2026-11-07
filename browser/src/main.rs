//! A native Material Symbols browser built with gpui-kit-material components.
//!
//! `cargo run -p gpui-material-symbols-browser --bin material_symbols`
//!
//! The `web` crate builds the same browser for the web with `run_web`.
#[cfg(not(target_family = "wasm"))]
extern crate gpui_kit as gpui;

mod catalog;
mod chrome;
mod export;
use catalog::{Entry, Sort};
use gpui::{prelude::*, *};
use gpui_base::{
    Placement,
    input::{InputEvent, InputState},
    slider::{SliderEvent, SliderState},
};
use gpui_kit_material::{
    color::Scheme,
    icons::{Assets, Icon as MaterialIcon},
    prelude::*,
    theme,
};
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
/// The details sheet's content inset, on its header's 24dp text ruler.
const SHEET_INSET: f32 = 24.;
/// Linked from the web build's app bar.
const REPOSITORY: &str = "https://github.com/mizorewww/gpui-material-symbols";
/// The GitHub mark (Primer Octicons, MIT) for that link.
const GITHUB_MARK: &str = "github-mark.svg";

/// The app's own files after the Material Symbols, which Material serves.
struct AppAssets;
impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok((path == GITHUB_MARK)
            .then(|| Cow::Borrowed(&include_bytes!("assets/github-mark.svg")[..])))
    }
    fn list(&self, _: &str) -> Result<Vec<SharedString>> {
        Ok(vec![])
    }
}

/// Material's asset source, which on the web also serves the few Rounded glyphs
/// Material's own controls draw before the Rounded font is downloaded.
struct BrowserAssets(Assets<AppAssets>);
impl AssetSource for BrowserAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(svg) = chrome::glyph(path) {
            return Ok(Some(Cow::Borrowed(svg.as_bytes())));
        }
        self.0.load(path)
    }
    fn list(&self, prefix: &str) -> Result<Vec<SharedString>> {
        self.0.list(prefix)
    }
}

fn assets() -> BrowserAssets {
    BrowserAssets(Assets::with(AppAssets))
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

/// A control's symbol in the Outlined style, whose font the web build
/// downloads first, at `size` design px.
fn ui_icon(symbol: Symbol, size: f32) -> MaterialIcon {
    let icon = symbol.icon();
    loader::ready(icon.required_asset());
    MaterialIcon::from(icon).size(rem(size))
}

/// `element` set in a Material type role.
fn typed<E: Styled>(element: E, role: TypeRole) -> E {
    let scale = role.scale();
    element
        .font_family(scale.font_family)
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
}

/// A plain tooltip naming an icon-only control in the app bar, below it.
fn named(id: &'static str, label: &'static str, control: impl IntoElement) -> impl IntoElement {
    title_bar_control(
        div().child(
            MaterialTooltip::plain(SharedString::from(format!("{id}-tooltip")), label)
                .placement(Placement::Bottom)
                .trigger(control),
        ),
    )
}

/// A labelled control with its current value, for the toolbar.
fn labelled(label: &'static str, value: String, control: impl IntoElement) -> Div {
    div()
        .flex()
        .items_center()
        .gap(rem(12.))
        .child(
            typed(div(), TypeRole::BodyMedium)
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
    /// Each entry's place among the shown icons, in grid order, which
    /// assistive technology announces with the selection.
    positions: Vec<usize>,
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
    /// A palette transition has asked for its next frame.
    theme_frame_pending: bool,
    sort: Sort,
    selected: Option<usize>,
    detail_open: bool,
    /// The compact layout's style and axis controls are shown.
    controls_open: bool,
    preview_size: f32,
    /// The snackbar's message, kept while it leaves after a dismissal.
    status: SharedString,
    status_open: bool,
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
            cx.subscribe(&color_input, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
            // Follow the system between light and dark until the user chooses.
            cx.observe_window_appearance(window, |this, window, cx| this.advance_theme(window, cx)),
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
            positions: vec![],
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
            theme_frame_pending: false,
            sort: Sort::Popular,
            selected: None,
            detail_open: false,
            controls_open: false,
            preview_size: 48.,
            status: "".into(),
            status_open: false,
            _subscriptions: subscriptions,
        };
        this.rebuild(cx);
        // The first palette paints without a transition; the text scale never
        // changes, so the first advance also sets it for good.
        let scheme = Scheme::from_palette(this.palette(cx));
        theme::init_component_theme(scheme, 1., cx);
        theme::advance_component_theme(scheme, 1., window, cx);
        this
    }

    /// Step the palette toward the chosen theme, frame by frame until its
    /// transition settles. Material's theme refreshes the window itself.
    fn advance_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let scheme = Scheme::from_palette(self.palette(cx));
        let running = theme::advance_component_theme(scheme, 1., window, cx);
        if running && !self.theme_frame_pending {
            self.theme_frame_pending = true;
            let view = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                let _ = view.update(cx, |this, cx| {
                    this.theme_frame_pending = false;
                    this.advance_theme(window, cx);
                });
            });
        }
    }

    /// Group the matching entries by category into rows of `columns` tiles.
    fn rebuild(&mut self, cx: &App) {
        let indices = catalog::filtered(&self.entries, &self.search.read(cx).value(), self.sort);
        self.shown = indices.len();
        self.positions.resize(self.entries.len(), 0);
        let mut position = 0;
        self.rows.clear();
        for category in catalog::CATEGORIES {
            let group: Vec<_> = indices
                .iter()
                .copied()
                .filter(|i| self.entries[*i].category == *category)
                .collect();
            if !group.is_empty() {
                for &index in &group {
                    self.positions[index] = position;
                    position += 1;
                }
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

    fn set_status(&mut self, status: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.status = status.into();
        self.status_open = true;
        cx.notify();
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

    /// The app bar is the window's title bar, drawn by Material's `TitleBar`
    /// on the same surface as the content.
    fn app_bar(&self, compact: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let dark = self.palette(cx) == PaletteId::Ink;
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
        .when(!compact, |bar| {
            bar.child(ui_icon(Symbol::INTERESTS, 24.).color(color(ColorRole::ON_SURFACE)))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_shrink_0()
                        .child(typed(div(), TypeRole::TitleMedium).child("Material Symbols"))
                        .child(
                            typed(div(), TypeRole::BodySmall)
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
                    .leading(ui_icon(Symbol::SEARCH, 24.).color(color(ColorRole::ON_SURFACE)))
                    .trailing(
                        IconButton::new(
                            "clear-search",
                            ui_icon(Symbol::CLOSE, 24.),
                            "Clear search",
                        )
                        .on_click(cx.listener(|this, _, w, cx| {
                            this.search.update(cx, |s, cx| s.set_value("", w, cx));
                            this.rebuild(cx);
                            cx.notify();
                        })),
                    ),
            ),
        )
        .when(!compact, |bar| {
            bar.child(title_bar_control(div().child(self.sort_group(cx))))
        })
        .when(compact, |bar| {
            bar.child(named(
                "controls",
                "Style and axes",
                IconButton::new("controls", ui_icon(Symbol::TUNE, 24.), "Style and axes")
                    .toggle(true)
                    .selected(self.controls_open)
                    .on_change({
                        let view = cx.weak_entity();
                        move |open, _, _, cx| {
                            let _ = view.update(cx, |this, cx| {
                                this.controls_open = open;
                                cx.notify();
                            });
                        }
                    }),
            ))
        })
        .child(named(
            "theme",
            if dark {
                "Use light theme"
            } else {
                "Use dark theme"
            },
            IconButton::new(
                "theme",
                ui_icon(
                    if dark {
                        Symbol::LIGHT_MODE
                    } else {
                        Symbol::DARK_MODE
                    },
                    24.,
                ),
                if dark {
                    "Use light theme"
                } else {
                    "Use dark theme"
                },
            )
            .on_click(cx.listener(|this, _, window, cx| {
                // An explicit choice overrides the system appearance.
                this.theme = if this.palette(cx) == PaletteId::Ink {
                    Appearance::Light
                } else {
                    Appearance::Dark
                };
                // Material's theme refreshes the window, this view included.
                this.advance_theme(window, cx);
            })),
        ))
        .when(cfg!(target_family = "wasm"), |bar| {
            let label = "Source code on GitHub";
            let mark = || MaterialIcon::new(GITHUB_MARK).size(rem(18.));
            let open = |_: &ClickEvent, _: &mut Window, cx: &mut App| cx.open_url(REPOSITORY);
            if compact {
                bar.child(named(
                    "github",
                    label,
                    IconButton::new("github", mark(), label)
                        .variant(IconButtonVariant::Tonal)
                        .on_click(open),
                ))
            } else {
                bar.child(title_bar_control(
                    material_button("github", ButtonVariant::Tonal)
                        .accessibility_label(format!("{label}: {REPOSITORY}"))
                        .icon(mark())
                        .child("GitHub")
                        .on_click(open),
                ))
            }
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
        let axis = |slider: MaterialSlider, width: f32| {
            div()
                .w(rem(width))
                .child(slider.appearance(SliderAppearance::Classic))
        };
        // Grade and optical size step through their official values.
        let stepped = |state: &Entity<SliderState>, label: &'static str, values: &'static [f32]| {
            MaterialSlider::new(state, label)
                .stops(true)
                .format(move |index| {
                    let value = values[(index.round() as usize).min(values.len() - 1)];
                    format!("{value}").into()
                })
        };
        let weight = self.weight.read(cx).value().end();
        let grade = GRADES[self.grade.read(cx).value().end() as usize];
        let optical = OPTICAL_SIZES[self.optical.read(cx).value().end() as usize];
        let controls = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_x(rem(if compact { 16. } else { 24. }))
            .gap_y(rem(12.))
            .px(px(inset(compact)))
            .py(rem(12.))
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
                axis(MaterialSlider::new(&self.weight, "Weight"), 160.),
            ))
            .child(labelled(
                "Grade",
                format!("{grade}"),
                axis(stepped(&self.grade, "Grade", &GRADES), 96.),
            ))
            .child(labelled(
                "Optical size",
                format!("{optical}"),
                axis(stepped(&self.optical, "Optical size", &OPTICAL_SIZES), 112.),
            ))
            .child(
                material_button("reset", ButtonVariant::Text)
                    .icon(ui_icon(Symbol::REFRESH, 20.))
                    .child("Reset")
                    .on_click(cx.listener(|this, _, window, cx| this.reset(window, cx))),
            );
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .child(controls)
            .child(MaterialDivider::new("toolbar-divider"))
    }

    fn row(&mut self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        match self.rows[index].clone() {
            Row::Heading(category) => list_subheader(category, category)
                .px_0()
                .pt(rem(8.))
                .into_any_element(),
            Row::Empty => typed(div(), TypeRole::BodyLarge)
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

    /// One icon as a selectable card; the selected card is the one the
    /// details sheet shows.
    fn tile(&self, index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let entry = &self.entries[index];
        let icon = self.icon(entry.symbol, cx);
        let selected = self.selected == Some(index) && self.detail_open;
        let name = entry.label();
        MaterialCard::new(("icon", index), CardVariant::Surface)
            .w(px(TILE))
            .flex_shrink_0()
            .accessibility_label(name.clone())
            // One choice of the shown icons; the details sheet shows it.
            .selected(selected)
            .selection_role(CardSelection::Radio {
                index: self.positions[index],
                count: self.shown,
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.selected = Some(index);
                this.detail_open = true;
                cx.notify();
            }))
            .child(
                div()
                    .h(px(TILE))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(rem(10.))
                    .px(rem(6.))
                    .child(if icon.is_supported() {
                        symbol_svg(icon)
                            .size(rem(36.))
                            .text_color(color(ColorRole::ON_SURFACE))
                            .flex_shrink_0()
                            .into_any_element()
                    } else {
                        ui_icon(Symbol::BLOCK, 28.)
                            .color(color(ColorRole::ON_SURFACE_VARIANT))
                            .into_any_element()
                    })
                    .child(
                        typed(div(), TypeRole::BodySmall)
                            .h(rem(32.))
                            .text_align(TextAlign::Center)
                            .whitespace_normal()
                            .overflow_hidden()
                            .child(name),
                    ),
            )
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
        let parsed_color = export::parse_hex(&self.color_input.read(cx).value());
        let preview_color = rgb(parsed_color.unwrap_or(0x1f1f1f));
        let codepoint = entry
            .symbol
            .codepoint(self.style)
            .map(|c| format!("U+{:04X}", c as u32))
            .unwrap_or_else(|| "SVG only".into());
        let fact = |id: &'static str, label: &'static str, value: String| {
            ListItem::new(id, value)
                .overline(label)
                .density(ListDensity::Compact)
        };
        let export_button = |id: &'static str, label: &'static str, png: bool| {
            material_button(id, ButtonVariant::Filled)
                .flex_1()
                .disabled(!supported)
                .icon(ui_icon(Symbol::DOWNLOAD, 18.))
                .child(label)
                .on_click(cx.listener(move |this, _, _, cx| this.export(icon, png, cx)))
        };
        let size = self.preview_size;
        let content = div()
            .flex()
            .flex_col()
            .gap(rem(20.))
            .pb(rem(SHEET_INSET))
            .child(
                // A fixed canvas, so the export color reads in either theme.
                CardSurface::new(CardVariant::Outlined)
                    .mx(rem(SHEET_INSET))
                    .h(rem(176.))
                    .items_center()
                    .justify_center()
                    .bg(rgb(0xffffff))
                    .child(if supported {
                        symbol_svg(icon)
                            .size(px(size))
                            .text_color(preview_color)
                            .into_any_element()
                    } else {
                        typed(div(), TypeRole::BodyMedium)
                            .px(rem(24.))
                            .text_align(TextAlign::Center)
                            .text_color(color(ColorRole::ERROR))
                            .child("This SVG-only symbol has no official variant for these axes.")
                            .into_any_element()
                    }),
            )
            .child(
                // List items keep 16dp of their own padding.
                div()
                    .flex()
                    .flex_col()
                    .px(rem(SHEET_INSET - 16.))
                    .child(fact("fact-name", "Name", entry.symbol.name().to_owned()))
                    .child(fact("fact-codepoint", "Code point", codepoint))
                    .child(fact("fact-category", "Category", entry.category.to_owned()))
                    .when(!entry.tags.is_empty(), |facts| {
                        facts.child(
                            typed(div(), TypeRole::BodySmall)
                                .px(rem(16.))
                                .pt(rem(4.))
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
                    .gap(rem(8.))
                    .px(rem(SHEET_INSET))
                    .child(
                        IconButton::new(
                            "size-down",
                            ui_icon(Symbol::REMOVE, 24.),
                            "Decrease export size",
                        )
                        .disabled(size <= 16.)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.preview_size = (this.preview_size - 8.).max(16.);
                            cx.notify();
                        })),
                    )
                    .child(
                        typed(div(), TypeRole::BodyLarge)
                            .flex_shrink_0()
                            .child(format!("{size} px")),
                    )
                    .child(
                        IconButton::new(
                            "size-up",
                            ui_icon(Symbol::ADD, 24.),
                            "Increase export size",
                        )
                        .disabled(size >= 96.)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.preview_size = (this.preview_size + 8.).min(96.);
                            cx.notify();
                        })),
                    )
                    .child(
                        div().flex_1().min_w_0().child(
                            TextField::new("export-color", &self.color_input)
                                .outlined()
                                .w_full()
                                .label("Color")
                                .error(parsed_color.is_none()),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap(rem(12.))
                    .px(rem(SHEET_INSET))
                    .child(export_button("save-svg", "SVG", false))
                    .child(export_button("save-png", "PNG", true)),
            )
            .child(
                // The code card, as the Material Gallery shows a component's Rust.
                CardSurface::new(CardVariant::Filled)
                    .mx(rem(SHEET_INSET))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(rem(8.))
                            .min_h(rem(56.))
                            .py(rem(8.))
                            .pl(rem(16.))
                            .pr(rem(8.))
                            .child(typed(div(), TypeRole::TitleSmall).child("Rust"))
                            .child(
                                material_button("copy-code", ButtonVariant::Text)
                                    .button_size(ButtonSize::ExtraSmall)
                                    .icon(ui_icon(Symbol::CONTENT_COPY, 20.))
                                    .child("Copy")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            code.clone(),
                                        ));
                                        this.set_status("Code copied", cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .px(rem(16.))
                            .pb(rem(16.))
                            .font_family("Menlo")
                            .text_size(rem(12.))
                            .whitespace_normal()
                            .child(code_text),
                    ),
            );
        div()
            .id("details")
            .size_full()
            .overflow_y_scroll()
            .child(content)
            .into_any_element()
    }

    fn export(&mut self, icon: SymbolIcon, png: bool, cx: &mut Context<Self>) {
        let color = self.color_input.read(cx).value();
        if export::parse_hex(&color).is_none() {
            self.set_status("Enter the color as #RRGGBB", cx);
            return;
        }
        let Ok(bytes) = export::encode(icon, self.preview_size as u32, &color, png) else {
            let status = if loader::ready(icon.required_asset()) {
                "Could not render icon"
            } else {
                "The icon is still loading"
            };
            self.set_status(status, cx);
            return;
        };
        let extension = if png { "png" } else { "svg" };
        let name = format!("{}.{extension}", icon.symbol().name());
        #[cfg(target_family = "wasm")]
        {
            let mime = if png { "image/png" } else { "image/svg+xml" };
            web::download(&name, mime, &bytes);
            self.set_status("Icon downloaded", cx);
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
                    let status = if result.is_ok() {
                        "Icon saved"
                    } else {
                        "Could not save icon"
                    };
                    this.set_status(status, cx);
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
            self.status_open = true;
        }
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
        let headline = self.selected.map(|index| self.entries[index].label());
        div()
            .id("icons-browser")
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .text_size(rem(14.))
            .bg(color(ColorRole::SURFACE))
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
                            // Material's `rem` takes 14px-based design units.
                            .width(rem(if compact {
                                SHEET * window_width / sheet_width
                            } else {
                                SHEET
                            }))
                            .when_some(headline, |sheet, headline| sheet.headline(headline))
                            .close_button("Close details")
                            .on_dismiss({
                                let view = cx.weak_entity();
                                move |_, cx| {
                                    let _ = view.update(cx, |this, cx| {
                                        this.detail_open = false;
                                        cx.notify();
                                    });
                                }
                            })
                            .child(self.detail(cx)),
                    )
                    // The web build's downloads of the other styles and the
                    // SVG-only symbols, while they last.
                    .when(loader::loading(), |body| {
                        body.child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .right_0()
                                .child(LinearProgress::new("downloads", "Downloading icons")),
                        )
                    })
                    .when(!self.status.is_empty(), |body| {
                        body.child(
                            Snackbar::new("status", self.status.clone())
                                .open(self.status_open)
                                .position(SnackbarPosition::Start)
                                .close(
                                    "Dismiss",
                                    cx.listener(|this, _, _, cx| {
                                        this.status_open = false;
                                        cx.notify();
                                    }),
                                )
                                // Escape inside it.
                                .on_dismiss({
                                    let view = cx.weak_entity();
                                    move |_, cx| {
                                        let _ = view.update(cx, |this, cx| {
                                            this.status_open = false;
                                            cx.notify();
                                        });
                                    }
                                }),
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

    /// Whether a download is under way; never, natively.
    pub fn loading() -> bool {
        false
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
        /// Downloads asked of the page and not yet answered.
        in_flight: usize,
        failure: Option<SharedString>,
    }

    thread_local! {
        static LOADER: RefCell<Option<Loader>> = const { RefCell::new(None) };
    }

    pub fn init(cx: &gpui::App) {
        LOADER.set(Some(Loader {
            app: cx.to_async(),
            requested: Vec::new(),
            in_flight: 0,
            failure: None,
        }));
    }

    /// A download failure not yet shown to the user.
    pub fn take_failure() -> Option<SharedString> {
        LOADER.with_borrow_mut(|loader| loader.as_mut()?.failure.take())
    }

    /// Whether the page is downloading an asset for the browser.
    pub fn loading() -> bool {
        LOADER.with_borrow(|loader| loader.as_ref().is_some_and(|l| l.in_flight > 0))
    }

    /// Records the end of one download, with the failure to report, if any.
    fn settle(failure: Option<&str>) {
        LOADER.with_borrow_mut(|loader| {
            if let Some(loader) = loader {
                loader.in_flight = loader.in_flight.saturating_sub(1);
                if let Some(failure) = failure {
                    loader.failure = Some(SharedString::from(failure.to_owned()));
                }
            }
        });
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
            loader.in_flight += 1;
            let app = loader.app.clone();
            app.foreground_executor()
                .spawn({
                    let app = app.clone();
                    async move {
                        // Asked while the grid lays out, after this frame read
                        // `loading`: show the progress indicator from the next.
                        app.update(|cx| cx.refresh_windows());
                        let result = match super::web::fetch_asset(asset.file_name()).await {
                            Ok(bytes) => asset.provide(bytes).map_err(|e| e.to_string()),
                            Err(error) => Err(error),
                        };
                        if let Err(error) = &result
                            && error.starts_with("reload:")
                        {
                            // The site was redeployed; retrying cannot help.
                            log::error!("{}: {error}", asset.file_name());
                            settle(Some(
                                "The site was updated. Reload the page to see these icons.",
                            ));
                            app.update(|cx| cx.refresh_windows());
                            return;
                        }
                        if let Err(error) = result {
                            log::error!("{}: {error}", asset.file_name());
                            settle(Some("Could not download icons; retrying"));
                            app.update(|cx| cx.refresh_windows());
                            app.background_executor().timer(RETRY).await;
                            LOADER.with_borrow_mut(|loader| {
                                if let Some(loader) = loader {
                                    loader.requested.retain(|a| *a != asset);
                                }
                            });
                        } else {
                            settle(None);
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
    start(gpui_kit::application(), |_| {});
}

/// Opens the browser on the web, which asks the page for each asset when an
/// icon first needs it.
#[cfg(target_family = "wasm")]
pub fn run_web(application: Application) {
    start(application, |cx| loader::init(cx));
}

fn start(application: Application, setup: impl FnOnce(&mut App) + 'static) {
    application.with_assets(assets()).run(|cx| {
        setup(cx);
        // The web has no system fonts; this is its default UI family.
        #[cfg(target_family = "wasm")]
        cx.text_system()
            .add_fonts(vec![Cow::Borrowed(include_bytes!(
                "../../web/fonts/IBMPlexSans-Regular.ttf"
            ))])
            .expect("load the UI font");
        gpui_kit::init(cx);
        gpui_kit_material::init(cx);
        gpui_kit::open_window(window_options(cx), cx, |window, cx| {
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
        Arc::new(assets()),
        gpui_kit::platform::current_headless_renderer,
    );
    app.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit_material::init(cx);
        cx.set_reduce_motion(true);
    });
    let mut browser = None;
    let window = app
        .open_window(size(px(1280.), px(860.)), |w, cx| {
            let view = cx.new(|cx| Browser::new(w, cx));
            browser = Some(view.clone());
            cx.new(|cx| gpui_base::Root::new(view, w, cx))
        })
        .unwrap();
    let browser = browser.expect("the window built the browser");
    for state in ["light", "dark", "detail", "detail-dark", "search"] {
        app.update_window(window.into(), |_, w, cx| {
            browser.update(cx, |view, cx| {
                view.theme = if state.ends_with("dark") {
                    Appearance::Dark
                } else {
                    Appearance::Light
                };
                view.advance_theme(w, cx);
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
            gpui_kit::init(cx);
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
    /// The button of the icon card for entry `index`.
    fn tile(index: usize) -> ElementId {
        (ElementId::from(("icon", index)), "card-button").into()
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
        click(cx, tile(index));
        view.read_with(cx, |v, _| {
            assert!(v.detail_open);
            assert_eq!(v.selected, Some(index));
        });
        click(cx, "copy-code".into());
        view.read_with(cx, |v, _| {
            assert!(v.status_open);
            assert_eq!(v.status.as_ref(), "Code copied");
        });
        let copied = cx.update(|_, cx| cx.read_from_clipboard().and_then(|c| c.text()));
        assert!(copied.is_some_and(|code| code.contains("Symbol::SEARCH")));
        click(cx, "sheet-close".into());
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
    fn compact_layout_folds_the_controls_behind_the_tune_button(cx: &mut TestAppContext) {
        setup(cx);
        let (view, cx) = cx.add_window_view(Browser::new);
        cx.simulate_resize(size(px(600.), px(800.)));
        draw(cx);
        let style_shown = |cx: &mut VisualTestContext| {
            cx.update(|w, _| gpui_base::test_support::find(w, &[], &"rounded".into()).is_some())
        };
        assert!(!style_shown(cx), "the compact toolbar starts folded");
        click(cx, "controls".into());
        view.read_with(cx, |v, _| assert!(v.controls_open));
        assert!(style_shown(cx));
        // Details cover the whole width instead of taking columns.
        let columns = view.read_with(cx, |v, _| v.columns);
        let first = view.read_with(cx, |v, _| match v.rows.get(1) {
            Some(Row::Icons(row)) => row[0],
            _ => panic!("expected an icon row after the first heading"),
        });
        click(cx, tile(first));
        view.read_with(cx, |v, _| {
            assert!(v.detail_open);
            assert_eq!(v.columns, columns);
        });
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
        click(cx, tile(first));
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
        let painted = theme::is_dark();
        click(cx, "theme".into());
        // Reduced motion settles the painted scheme at once.
        assert_ne!(theme::is_dark(), painted, "the click repaints the theme");
        view.update(cx, |v, cx| {
            assert_ne!(v.palette(cx) == PaletteId::Ink, dark);
            assert!(v.theme != Appearance::System);
        });
    }
}
