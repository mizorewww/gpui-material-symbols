//! cargo run --no-default-features --example export -- home rounded > home.svg
use gpui_material_symbols::{Style, Symbol};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| "home".into());
    let style_name = args.next().unwrap_or_else(|| "outlined".into());
    let style = Style::ALL
        .into_iter()
        .find(|s| s.name() == style_name)
        .ok_or("unknown style")?;
    println!("{}", name.parse::<Symbol>()?.icon().style(style).to_svg()?);
    Ok(())
}
