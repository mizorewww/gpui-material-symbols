#![cfg(feature = "gpui")]
use gpui::{AssetSource, SharedString};
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
    let _: gpui::Svg = Symbol::HOME.icon().svg();
}
