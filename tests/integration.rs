#![cfg(feature = "gpui-kit")]
use gpui_kit::{AssetSource, SharedString};
use gpui_material_symbols::{MaterialSymbols, Symbol};
use std::borrow::Cow;

struct ExistingAssets;
impl AssetSource for ExistingAssets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        Ok((path == "app/logo.svg").then_some(Cow::Borrowed(b"existing")))
    }
    fn list(&self, prefix: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(if "app/logo.svg".starts_with(prefix) {
            vec!["app/logo.svg".into()]
        } else {
            vec![]
        })
    }
}
#[test]
fn composes_with_existing_assets_and_preserves_errors() {
    let source = MaterialSymbols.with_fallback(ExistingAssets);
    assert_eq!(
        source.load("app/logo.svg").unwrap().unwrap().as_ref(),
        b"existing"
    );
    assert!(source.load("app/missing.svg").unwrap().is_none());
    assert!(source.load("material-symbols/malformed").is_err());
    for symbol in [Symbol::HOME, Symbol::APP_SPARK] {
        let icon = symbol.icon();
        assert_eq!(
            source.load(&icon.asset_path()).unwrap().unwrap().as_ref(),
            icon.to_svg().unwrap().as_bytes()
        );
    }
    assert_eq!(source.list("app/").unwrap().len(), 1);
    assert_eq!(
        source
            .list("material-symbols/outlined/home/")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        MaterialSymbols.list("").unwrap().len(),
        Symbol::all().len() * 3
    );
    assert_eq!(
        MaterialSymbols.list("material").unwrap().len(),
        Symbol::all().len() * 3
    );
    assert!(MaterialSymbols.list("app/").unwrap().is_empty());
    let _: gpui_kit::Svg = Symbol::HOME.icon().svg();
}

#[test]
fn longbridge_component_slots_accept_symbols_and_configured_icons() {
    use gpui_kit::component::{Icon as KitIcon, IconNamed, button::Button, menu::PopupMenuItem};
    use gpui_material_symbols::Style;
    let configured = Symbol::FAVORITE.icon().style(Style::Rounded).filled(true);
    assert_eq!(
        IconNamed::path(configured).as_ref(),
        configured.asset_path()
    );
    assert_eq!(
        IconNamed::path(Symbol::HOME).as_ref(),
        Symbol::HOME.icon().asset_path()
    );
    let _: KitIcon = Symbol::HOME.into();
    let _: KitIcon = configured.into();
    let _ = Button::new("home").icon(Symbol::HOME);
    let _ = Button::new("favorite").icon(configured);
    let _ = PopupMenuItem::new("Favorite").icon(configured);
    let _ = configured.component();
    let _ = configured.embedded().unwrap();
    let source = MaterialSymbols.with_fallback(gpui_kit::assets::Assets);
    assert!(source.load("icons/check.svg").unwrap().is_some());
    assert!(source.load(&configured.asset_path()).unwrap().is_some());
}
