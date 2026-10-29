//! cargo run --features showcase --example showcase
use gpui::{prelude::*, *};
use gpui_material_symbols::{Axes, MaterialSymbols, Style, Symbol};
use gpuikit::elements::icon_button::icon_button;

struct Showcase;
impl Render for Showcase {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_6()
            .p_8()
            .bg(rgb(0x17191f))
            .text_color(rgb(0xeeeeee))
            .child("Material Symbols · GPUI + gpuikit")
            .children(Style::ALL.map(|style| {
                div().flex().flex_col().gap_3().child(style.name()).child(
                    div().flex().gap_3().children(
                        [
                            Symbol::HOME,
                            Symbol::FAVORITE,
                            Symbol::SETTINGS,
                            Symbol::SEARCH,
                            Symbol::APP_SPARK,
                        ]
                        .into_iter()
                        .enumerate()
                        .map(move |(index, symbol)| {
                            icon_button((style.name(), index), symbol.icon().style(style).svg())
                        }),
                    ),
                )
            }))
            .child("Variable fill · weight 400 · optical size 24")
            .child(div().flex().gap_3().children((0..=4).map(|step| {
                Symbol::FAVORITE
                    .icon()
                    .style(Style::Rounded)
                    .axes(Axes::new(step as f32 / 4., 400., 0., 24.).unwrap())
                    .svg()
                    .size(px(36.))
            })))
    }
}
fn main() {
    Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(MaterialSymbols.with_fallback(gpuikit::assets()))
        .run(|cx| {
            gpuikit::init(cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(620.), px(520.)),
                        cx,
                    ))),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| Showcase),
            )
            .expect("open showcase window");
            cx.activate(true);
        });
}
