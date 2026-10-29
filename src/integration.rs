use crate::{Icon, Style, Symbol};
use gpui::{AssetSource, SharedString, Svg, svg};
use std::borrow::Cow;

impl Icon {
    /// Return a GPUI SVG accepted directly by gpuikit buttons, alerts and menus.
    /// Register [`MaterialSymbols`] on the application before rendering.
    pub fn svg(self) -> Svg {
        svg().path(self.asset_path())
    }
}

/// Embedded SVG asset source. GPUI caches rasterized SVGs by their full path.
///
/// Use [`Self::with_fallback`] to preserve your application's existing resources.
#[derive(Clone, Copy, Debug, Default)]
pub struct MaterialSymbols;
impl MaterialSymbols {
    /// Compose with an existing source, including gpuikit's `ResourceSource`.
    pub fn with_fallback<A: AssetSource>(self, fallback: A) -> WithMaterialSymbols<A> {
        WithMaterialSymbols { fallback }
    }
}
impl AssetSource for MaterialSymbols {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        if !path.starts_with("material-symbols/") {
            return Ok(None);
        }
        Ok(Some(Cow::Owned(
            Icon::from_asset_path(path)?.to_svg()?.into_bytes(),
        )))
    }
    fn list(&self, prefix: &str) -> anyhow::Result<Vec<SharedString>> {
        // Continuous axes cannot be enumerated. List default coordinates in every style.
        Ok(Symbol::all()
            .flat_map(|symbol| Style::ALL.map(move |style| symbol.icon().style(style).asset_path()))
            .filter(|path| path.starts_with(prefix))
            .map(Into::into)
            .collect())
    }
}

/// Asset source combining Material Symbols with existing application resources.
pub struct WithMaterialSymbols<A> {
    fallback: A,
}
impl<A: AssetSource> AssetSource for WithMaterialSymbols<A> {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        if path.starts_with("material-symbols/") {
            MaterialSymbols.load(path)
        } else {
            self.fallback.load(path)
        }
    }
    fn list(&self, prefix: &str) -> anyhow::Result<Vec<SharedString>> {
        let mut paths = MaterialSymbols.list(prefix)?;
        paths.extend(self.fallback.list(prefix)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}
