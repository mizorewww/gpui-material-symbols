use crate::{Icon, Style, Symbol};
use gpui_kit::{AssetSource, SharedString, Svg, svg};
use std::borrow::Cow;

impl Icon {
    /// A themed Longbridge component icon, respecting Kit sizing and icon resolution.
    /// Register [`MaterialSymbols`] before rendering path-based icons.
    ///
    /// Path-based icons are not validated here: an SVG-only symbol with axes that
    /// [`Icon::is_supported`] rejects loads with an error and renders empty.
    /// Use [`Self::embedded`] to surface that error at construction time.
    pub fn component(self) -> gpui_kit::component::Icon {
        gpui_kit::component::Icon::new(self)
    }

    /// A component icon with embedded SVG data; no asset source registration needed.
    /// Invalid discrete variants fail immediately. Raw data bypasses Kit's path resolver.
    pub fn embedded(self) -> Result<gpui_kit::component::Icon, crate::Error> {
        Ok(gpui_kit::component::Icon::default().data(self.to_svg()?.as_bytes()))
    }

    /// Return the raw SVG primitive from the same backend as GPUI Kit.
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
    /// Compose with an existing source, including `gpui_kit::assets::Assets`.
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
        const ROOT: &str = "material-symbols/";
        if !(prefix.starts_with(ROOT) || ROOT.starts_with(prefix)) {
            return Ok(Vec::new());
        }
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

// Kit's blanket conversions route IconNamed into Icon and all standard icon slots.
impl gpui_kit::component::IconNamed for Icon {
    fn path(self) -> SharedString {
        self.asset_path().into()
    }
}
impl gpui_kit::component::IconNamed for Symbol {
    fn path(self) -> SharedString {
        self.icon().asset_path().into()
    }
}
