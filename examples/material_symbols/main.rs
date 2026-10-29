//! A native Material Symbols browser built with gpui-kit-material components.
//!
//! `cargo run --no-default-features --features material-browser --example material_symbols`
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
use std::{borrow::Cow, path::Path};

/// The app bar doubles as the window title bar.
const APP_BAR: f32 = 64.;
/// macOS draws its traffic lights inside the app bar, vertically centred.
const TRAFFIC_LIGHTS: Point<Pixels> = point(px(20.), px(25.));
const TRAFFIC_LIGHTS_WIDTH: f32 = 72.;
const TILE: f32 = 104.;
const TILE_GAP: f32 = 8.;
const GRID_INSET: f32 = 24.;
const GRADES: [f32; 3] = [-25., 0., 200.];
const OPTICAL_SIZES: [f32; 4] = [20., 24., 40., 48.];
const DEFAULT_COLOR: &str = "#1F1F1F";

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

fn icon_button(id: impl Into<ElementId>, symbol: Symbol, label: &str) -> gpui_base::Button {
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
                .child(div().text_color(color(ColorRole::GRAY)).child(value)),
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
    dark: bool,
    sort: Sort,
    selected: Option<usize>,
    detail_open: bool,
    preview_size: f32,
    status: SharedString,
    should_move: bool,
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
            dark: theme::system_dark(cx),
            sort: Sort::Popular,
            selected: None,
            detail_open: false,
            preview_size: 48.,
            status: "".into(),
            should_move: false,
            _subscriptions: subscriptions,
        };
        this.rebuild(cx);
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
        self.search.update(cx, |s, cx| s.set_value("", window, cx));
        for (s, v) in [(&self.weight, 400.), (&self.grade, 1.), (&self.optical, 1.)] {
            s.update(cx, |s, cx| s.set_value(v, window, cx));
        }
        self.rebuild(cx);
        cx.notify();
    }

    /// The app bar is the window's title bar: same surface as the content,
    /// draggable, with the platform's own window controls.
    fn app_bar(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let leading = if cfg!(target_os = "macos") && !window.is_fullscreen() {
            TRAFFIC_LIGHTS_WIDTH + 16.
        } else {
            20.
        };
        div()
            .id("app-bar")
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap(rem(16.))
            .h(px(APP_BAR))
            .pl(px(leading))
            .pr(px(CaptionButtons::trailing(window)))
            .window_control_area(WindowControlArea::Drag)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.should_move = true),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.should_move = false),
            )
            .on_mouse_move(cx.listener(|this, _, window, _| {
                // Windows moves the window through the drag control area.
                if this.should_move && !cfg!(target_os = "windows") {
                    this.should_move = false;
                    window.start_window_move();
                }
            }))
            .on_click(|event, window, _| {
                // Windows zooms on a double click in the drag area by itself.
                if event.click_count() == 2 {
                    if cfg!(target_os = "macos") {
                        window.titlebar_double_click();
                    } else if !cfg!(target_os = "windows") {
                        window.zoom_window();
                    }
                }
            })
            .child(glyph(Symbol::INTERESTS, 24.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_shrink_0()
                    .child(div().text_size(rem(16.)).child("Material Symbols"))
                    .child(
                        div()
                            .text_size(rem(12.))
                            .text_color(color(ColorRole::GRAY))
                            .child(format!("{} of {} icons", self.shown, self.entries.len())),
                    ),
            )
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
            .child(title_bar_control(
                div().child(
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
                        })),
                ),
            ))
            .child(title_bar_control(
                icon_button(
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
            ))
    }

    /// Style, fill, the three numeric axes and reset.
    fn toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
            .gap_x(rem(24.))
            .gap_y(rem(12.))
            .px(px(GRID_INSET))
            .py(rem(12.))
            .border_b_1()
            .border_color(color(ColorRole::HAIRLINE))
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
                .text_color(color(ColorRole::GRAY))
                .child(category)
                .into_any_element(),
            Row::Empty => div()
                .py(rem(48.))
                .flex()
                .justify_center()
                .text_color(color(ColorRole::GRAY))
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
                svg()
                    .path(icon.asset_path())
                    .size(rem(36.))
                    .text_color(color(ColorRole::INK))
                    .into_any_element()
            } else {
                glyph(Symbol::BLOCK, 28.)
                    .text_color(color(ColorRole::GRAY))
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
                .child(div().text_color(color(ColorRole::GRAY)).child(label))
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
                    .bg(color(ColorRole::SURFACE))
                    .child(if supported {
                        svg()
                            .path(icon.asset_path())
                            .size(px(self.preview_size))
                            .text_color(preview_color)
                            .into_any_element()
                    } else {
                        div()
                            .px(rem(24.))
                            .text_align(TextAlign::Center)
                            .text_color(color(ColorRole::DANGER))
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
                                .text_color(color(ColorRole::GRAY))
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
                    .bg(color(ColorRole::SURFACE_HIGH))
                    .font_family("Menlo")
                    .text_size(rem(12.))
                    .whitespace_normal()
                    .child(catalog::api(icon)),
            );
        div()
            .id("details")
            .size_full()
            .overflow_y_scroll()
            .bg(color(ColorRole::CANVAS))
            .border_l_1()
            .border_color(color(ColorRole::HAIRLINE))
            .child(content)
            .into_any_element()
    }

    fn export(&mut self, icon: SymbolIcon, png: bool, cx: &mut Context<Self>) {
        let color = self.color_input.read(cx).value();
        let Ok(bytes) = export::encode(icon, self.preview_size as u32, &color, png) else {
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

fn title_case(name: &str) -> String {
    let mut chars = name.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

impl Render for Browser {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        theme::apply_palette(
            if self.dark {
                PaletteId::Ink
            } else {
                PaletteId::Paper
            },
            window,
            cx,
        );
        theme::apply_scale(1., window);
        let width = f32::from(window.viewport_size().width);
        let columns = ((width - 2. * GRID_INSET + TILE_GAP) / (TILE + TILE_GAP))
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
            .bg(color(ColorRole::CANVAS))
            .text_color(color(ColorRole::INK))
            .child(self.app_bar(window, cx))
            .child(self.toolbar(cx))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .child(
                        div()
                            .size_full()
                            .px(px(GRID_INSET))
                            .pb(rem(16.))
                            .child(grid),
                    )
                    .child(
                        MaterialSideSheet::new("details-sheet")
                            .open(self.detail_open)
                            .width(rem(360.))
                            .child(self.detail(cx)),
                    )
                    .when(!self.status.is_empty(), |body| {
                        body.child(
                            div()
                                .absolute()
                                .bottom(rem(24.))
                                .left(px(GRID_INSET))
                                .flex()
                                .items_center()
                                .gap(rem(12.))
                                .pl(rem(20.))
                                .pr(rem(8.))
                                .py(rem(8.))
                                .rounded(rem(8.))
                                .bg(color(ColorRole::INK))
                                .text_color(color(ColorRole::CANVAS))
                                .child(self.status.clone())
                                .child(
                                    material_button("dismiss-status", ButtonVariant::Text)
                                        .text_color(color(ColorRole::CANVAS))
                                        .child("Dismiss")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.status = "".into();
                                            cx.notify();
                                        })),
                                ),
                        )
                    }),
            )
            .child(CaptionButtons::new(px(APP_BAR)))
    }
}

fn window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(1280.), px(860.)),
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("Material Symbols".into()),
            appears_transparent: true,
            traffic_light_position: Some(TRAFFIC_LIGHTS),
        }),
        // The app bar moves the window itself; without this AppKit also treats
        // it as a native title bar, zooming twice and delaying clicks in it.
        app_owns_titlebar_drag: true,
        window_decorations: Some(WindowDecorations::Client),
        window_min_size: Some(size(px(720.), px(480.))),
        ..Default::default()
    }
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--capture") {
        let directory = std::env::args()
            .nth(2)
            .expect("usage: --capture <directory>");
        capture(&directory);
        return;
    }
    gpui_platform::application()
        .with_assets(Assets::with(SymbolsSource))
        .run(|cx| {
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
    });
    let window = app
        .open_window(size(px(1280.), px(860.)), |w, cx| {
            cx.new(|cx| Browser::new(w, cx))
        })
        .unwrap();
    for state in ["light", "dark", "detail", "search"] {
        window
            .update(&mut app, |view, w, cx| {
                view.dark = state == "dark";
                view.detail_open = state == "detail";
                view.selected = (state == "detail")
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
    println!("Captured four native GPUI states to {directory}");
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

        view.update(cx, |v, cx| {
            v.sort = Sort::Name;
            v.rebuild(cx);
        });
        view.read_with(cx, |v, _| {
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
}
