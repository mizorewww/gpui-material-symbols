//! cargo run --features showcase --example showcase
//! cargo run --features showcase-capture --example showcase -- --capture showcase.png
extern crate gpui_kit as gpui;

use gpui_kit::component::{ActiveTheme, button::Button};
use gpui_kit::{prelude::*, *};
use gpui_material_symbols::{Axes, MaterialSymbols, Style, Symbol};

struct Showcase;
impl Render for Showcase {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_6()
            .p_8()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child("Material Symbols · Longbridge GPUI Kit")
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
                            Button::new((style.name(), index)).icon(symbol.icon().style(style))
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
                    .component()
                    .size(px(36.))
            })))
            .child(
                Button::new("embedded")
                    .label("Embedded SVG")
                    .icon(Symbol::HOME.icon().embedded().unwrap()),
            )
    }
}
fn main() {
    #[cfg(feature = "showcase-capture")]
    if std::env::args().nth(1).as_deref() == Some("--capture") {
        let path = std::env::args()
            .nth(2)
            .expect("usage: --capture <file.png>");
        capture(&path);
        return;
    }
    gpui_kit::application()
        .with_assets(MaterialSymbols.with_fallback(gpui_kit::assets::Assets))
        .run(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(620.), px(560.)),
                        cx,
                    ))),
                    ..Default::default()
                },
                cx,
                |_, cx| cx.new(|_| Showcase),
            )
            .expect("open showcase window");
            cx.activate(true);
        });
}

/// Render through the native headless renderer (Metal on macOS) into a PNG.
#[cfg(feature = "showcase-capture")]
fn capture(path: &str) {
    let mut app = HeadlessAppContext::with_platform(
        platform::current_platform(true).text_system(),
        std::sync::Arc::new(MaterialSymbols.with_fallback(gpui_kit::assets::Assets)),
        platform::current_headless_renderer,
    );
    app.update(gpui_kit::init);
    let window = app
        .open_window(size(px(620.), px(560.)), |window, cx| {
            let view = cx.new(|_| Showcase);
            cx.new(|cx| base::Root::new(view, window, cx))
        })
        .expect("open headless showcase window");
    // The first frame requests SVG assets; later frames paint them.
    for _ in 0..2 {
        app.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        app.run_until_parked();
    }
    app.capture_screenshot(window.into())
        .expect("capture showcase")
        .save(path)
        .expect("write showcase PNG");
    println!("Captured showcase to {path}");
}
