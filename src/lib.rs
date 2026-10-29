//! Complete Google Material Symbols, rendered from the official variable fonts,
//! plus Google's discrete SVG variants for the names that have no font glyph.
//!
//! [`Symbol`] provides generated constants and name lookup; [`Icon`] chooses a
//! style and continuous variation axes. With the default `gpui-kit` feature,
//! [`MaterialSymbols`] serves SVGs to GPUI and [`Icon::component`] integrates with Longbridge GPUI Kit.

use std::{fmt, fmt::Write, str::FromStr};
use ttf_parser::{Face, OutlineBuilder, Tag};

#[cfg(feature = "gpui-kit")]
mod integration;
/// The exact Longbridge GPUI Kit facade used by this integration.
#[cfg(feature = "gpui-kit")]
pub use gpui_kit as kit;
#[cfg(feature = "gpui-kit")]
pub use integration::{MaterialSymbols, WithMaterialSymbols};

/// Compile and run the README's examples as doctests.
#[cfg(all(doctest, feature = "gpui-kit"))]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

/// A validated symbol from Google's complete, pinned catalog, including aliases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Symbol(usize);
include!(concat!(env!("OUT_DIR"), "/catalog.rs"));

impl Symbol {
    /// Look up an exact upstream snake_case name.
    pub fn named(name: &str) -> Option<Self> {
        CATALOG
            .binary_search_by_key(&name, |(name, _)| *name)
            .ok()
            .map(Self)
    }
    /// All symbols, in stable alphabetical order.
    pub fn all() -> impl ExactSizeIterator<Item = Self> + DoubleEndedIterator {
        (0..CATALOG.len()).map(Self)
    }
    /// The official upstream name.
    pub fn name(self) -> &'static str {
        CATALOG[self.0].0
    }
    /// The Unicode codepoint for a particular visual style.
    pub fn codepoint(self, style: Style) -> Option<char> {
        let cp = CATALOG[self.0].1[style as usize];
        if cp == 0 { None } else { char::from_u32(cp) }
    }
    /// Whether this symbol supports continuous variable-font axes.
    /// SVG-only symbols expose their exact official variants via `available_variants`.
    pub fn is_variable(self) -> bool {
        self.codepoint(Style::Outlined).is_some()
    }
    /// All official discrete variants for SVG-only symbols, empty for variable symbols.
    pub fn available_variants(self) -> impl Iterator<Item = Icon> {
        SUPPLEMENTAL.iter().filter_map(move |(path, _, _)| {
            let (name, tail) = path.split_once('/')?;
            if name != self.name() {
                return None;
            }
            let (style, file) = tail.split_once('/')?;
            let style = Style::ALL
                .into_iter()
                .find(|s| style == format!("materialsymbols{}", s.name()))?;
            let suffix = file.strip_prefix(name)?.strip_suffix("px.svg")?;
            let (settings, size) = suffix.rsplit_once('_')?;
            let mut weight = 400.;
            let mut grade = 0.;
            let mut fill = 0.;
            let mut settings = settings.trim_start_matches('_');
            if let Some(rest) = settings.strip_prefix("wght") {
                let end = rest
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(rest.len());
                weight = rest[..end].parse().ok()?;
                settings = &rest[end..];
            }
            if let Some(rest) = settings.strip_prefix("grad") {
                let end = rest.find("fill").unwrap_or(rest.len());
                grade = rest[..end].replace('N', "-").parse().ok()?;
                settings = &rest[end..];
            }
            if settings == "fill1" {
                fill = 1.;
            } else if !settings.is_empty() {
                return None;
            }
            Some(
                self.icon()
                    .style(style)
                    .axes(Axes::new(fill, weight, grade, size.parse().ok()?).ok()?),
            )
        })
    }
    /// Configure this symbol, initially outlined at 24 optical pixels and weight 400.
    pub fn icon(self) -> Icon {
        Icon::new(self)
    }
}
impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
impl FromStr for Symbol {
    type Err = Error;
    fn from_str(name: &str) -> Result<Self, Error> {
        Self::named(name).ok_or(Error::UnknownSymbol)
    }
}

/// Google's three independently designed visual styles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum Style {
    /// Outlined contours (the default).
    #[default]
    Outlined,
    /// Rounded corners and terminals.
    Rounded,
    /// Angular corners and terminals.
    Sharp,
}
impl Style {
    /// All available styles.
    pub const ALL: [Self; 3] = [Self::Outlined, Self::Rounded, Self::Sharp];
    /// Stable lowercase name used in asset paths.
    pub fn name(self) -> &'static str {
        match self {
            Self::Outlined => "outlined",
            Self::Rounded => "rounded",
            Self::Sharp => "sharp",
        }
    }
    /// Complete embedded official variable font; never fetched at runtime.
    pub fn font_bytes(self) -> &'static [u8] {
        match self {
            Self::Outlined => include_bytes!("../assets/MaterialSymbolsOutlined.ttf"),
            Self::Rounded => include_bytes!("../assets/MaterialSymbolsRounded.ttf"),
            Self::Sharp => include_bytes!("../assets/MaterialSymbolsSharp.ttf"),
        }
    }
}

/// Continuous variation coordinates. Construction rejects invalid or non-finite values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Axes {
    fill: f32,
    weight: f32,
    grade: f32,
    optical_size: f32,
}
impl Default for Axes {
    fn default() -> Self {
        Self {
            fill: 0.0,
            weight: 400.0,
            grade: 0.0,
            optical_size: 24.0,
        }
    }
}
impl Axes {
    /// Construct FILL (0–1), wght (100–700), GRAD (-50–200), opsz (20–48).
    pub fn new(fill: f32, weight: f32, grade: f32, optical_size: f32) -> Result<Self, Error> {
        for (name, value, min, max) in [
            ("FILL", fill, 0., 1.),
            ("wght", weight, 100., 700.),
            ("GRAD", grade, -50., 200.),
            ("opsz", optical_size, 20., 48.),
        ] {
            if !value.is_finite() || !(min..=max).contains(&value) {
                return Err(Error::InvalidAxis(name));
            }
        }
        // Canonicalize negative zero so equivalent coordinates share the same asset key.
        let zero = |v: f32| if v == 0. { 0. } else { v };
        Ok(Self {
            fill: zero(fill),
            weight,
            grade: zero(grade),
            optical_size,
        })
    }
    /// Fill interpolation, from outline to solid.
    pub fn fill(self) -> f32 {
        self.fill
    }
    /// Stroke weight.
    pub fn weight(self) -> f32 {
        self.weight
    }
    /// Fine stroke adjustment.
    pub fn grade(self) -> f32 {
        self.grade
    }
    /// Optical size, independent of the element's display size.
    pub fn optical_size(self) -> f32 {
        self.optical_size
    }
}

/// A symbol, visual style, and validated set of variation coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Icon {
    symbol: Symbol,
    style: Style,
    axes: Axes,
}
impl Icon {
    /// Create an outlined symbol with default axes.
    pub fn new(symbol: Symbol) -> Self {
        Self {
            symbol,
            style: Style::default(),
            axes: Axes::default(),
        }
    }
    /// Choose a visual style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    /// Set all variable axes.
    pub fn axes(mut self, axes: Axes) -> Self {
        self.axes = axes;
        self
    }
    /// Select the filled or unfilled state without changing other axes.
    pub fn filled(mut self, filled: bool) -> Self {
        self.axes.fill = if filled { 1. } else { 0. };
        self
    }
    /// The selected symbol.
    pub fn symbol(self) -> Symbol {
        self.symbol
    }
    /// The selected visual style.
    pub fn visual_style(self) -> Style {
        self.style
    }
    /// The selected axes.
    pub fn variations(self) -> Axes {
        self.axes
    }
    /// A deterministic, namespaced asset path including every variation coordinate.
    pub fn asset_path(self) -> String {
        format!(
            "material-symbols/{}/{}/{}/{}/{}/{}.svg",
            self.style.name(),
            self.symbol.name(),
            self.axes.fill,
            self.axes.weight,
            self.axes.grade,
            self.axes.optical_size
        )
    }
    /// Decode an asset path. Noncanonical and malformed paths are rejected.
    pub fn from_asset_path(path: &str) -> Result<Self, Error> {
        let fields: Vec<_> = path
            .strip_prefix("material-symbols/")
            .and_then(|p| p.strip_suffix(".svg"))
            .ok_or(Error::InvalidPath)?
            .split('/')
            .collect();
        if fields.len() != 6 {
            return Err(Error::InvalidPath);
        }
        let style = Style::ALL
            .into_iter()
            .find(|s| s.name() == fields[0])
            .ok_or(Error::InvalidPath)?;
        let parse = |s: &str| s.parse::<f32>().map_err(|_| Error::InvalidPath);
        let axes = Axes::new(
            parse(fields[2])?,
            parse(fields[3])?,
            parse(fields[4])?,
            parse(fields[5])?,
        )?;
        let icon = Self::new(fields[1].parse()?).style(style).axes(axes);
        if icon.asset_path() != path {
            return Err(Error::InvalidPath);
        }
        Ok(icon)
    }
    /// Whether this exact style and axis combination is available upstream.
    pub fn is_supported(self) -> bool {
        self.symbol.is_variable() || self.supplemental_entry().is_some()
    }
    fn supplemental_entry(self) -> Option<(usize, usize)> {
        let a = self.axes;
        let mut variant = String::new();
        if a.weight != 400. {
            write!(variant, "wght{}", a.weight).ok()?;
        }
        if a.grade != 0. {
            if a.grade < 0. {
                write!(variant, "gradN{}", -a.grade).ok()?;
            } else {
                write!(variant, "grad{}", a.grade).ok()?;
            }
        }
        if a.fill != 0. {
            write!(variant, "fill{}", a.fill).ok()?;
        }
        let name = self.symbol.name();
        let mid = if variant.is_empty() {
            String::new()
        } else {
            format!("_{variant}")
        };
        let path = format!(
            "{name}/materialsymbols{}/{name}{mid}_{}px.svg",
            self.style.name(),
            a.optical_size
        );
        let index = SUPPLEMENTAL
            .binary_search_by_key(&path.as_str(), |(p, _, _)| *p)
            .ok()?;
        Some((SUPPLEMENTAL[index].1, SUPPLEMENTAL[index].2))
    }
    fn supplemental_svg(self) -> Result<String, Error> {
        let (offset, len) = self.supplemental_entry().ok_or(Error::UnsupportedVariant)?;
        let pack = include_bytes!("../assets/supplemental.bin");
        Ok(std::str::from_utf8(&pack[offset..offset + len])
            .map_err(|_| Error::InvalidFont)?
            .to_owned())
    }
    /// Convert the variable glyph outline to an SVG, preserving the full em square.
    ///
    /// The SVG uses black geometry so GPUI's monochrome SVG tinting works naturally.
    /// No OS font registration, network access, or filesystem access is needed.
    pub fn to_svg(self) -> Result<String, Error> {
        if !self.symbol.is_variable() {
            return self.supplemental_svg();
        }
        let mut face = Face::parse(self.style.font_bytes(), 0).map_err(|_| Error::InvalidFont)?;
        for (tag, value) in [
            (b"FILL", self.axes.fill),
            (b"wght", self.axes.weight),
            (b"GRAD", self.axes.grade),
            (b"opsz", self.axes.optical_size),
        ] {
            face.set_variation(Tag::from_bytes(tag), value)
                .ok_or(Error::InvalidFont)?;
        }
        let glyph = face
            .glyph_index(
                self.symbol
                    .codepoint(self.style)
                    .ok_or(Error::MissingGlyph)?,
            )
            .ok_or(Error::MissingGlyph)?;
        let mut path = SvgPath(String::new());
        face.outline_glyph(glyph, &mut path)
            .ok_or(Error::MissingGlyph)?;
        let em = face.units_per_em();
        // Font ascenders include typographic line spacing (1056 vs a 960 em).
        // Icons use the design em square, not the text line box.
        Ok(format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {em} {em}"><path fill="black" transform="translate(0 {em}) scale(1 -1)" d="{}"/></svg>"#,
            path.0
        ))
    }
}

struct SvgPath(String);
impl OutlineBuilder for SvgPath {
    fn move_to(&mut self, x: f32, y: f32) {
        write!(self.0, "M{x} {y}").unwrap();
    }
    fn line_to(&mut self, x: f32, y: f32) {
        write!(self.0, "L{x} {y}").unwrap();
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        write!(self.0, "Q{x1} {y1} {x} {y}").unwrap();
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        write!(self.0, "C{x1} {y1} {x2} {y2} {x} {y}").unwrap();
    }
    fn close(&mut self) {
        self.0.push('Z');
    }
}

/// Errors are explicit; invalid names never silently become a missing-glyph box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// No upstream symbol has this name.
    UnknownSymbol,
    /// This SVG-only symbol does not have the requested discrete upstream variant.
    UnsupportedVariant,
    /// The named axis is non-finite or outside the supported range.
    InvalidAxis(&'static str),
    /// A malformed or noncanonical asset path.
    InvalidPath,
    /// Embedded font could not be parsed or lacks a required axis.
    InvalidFont,
    /// The selected codepoint has no drawable outline.
    MissingGlyph,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSymbol => f.write_str("unknown Material Symbol"),
            Self::UnsupportedVariant => f.write_str("this SVG-only symbol has no exact upstream variant for these axes; use Symbol::available_variants"),
            Self::InvalidAxis(axis) => write!(f, "invalid Material Symbols {axis} coordinate"),
            Self::InvalidPath => f.write_str("invalid Material Symbols asset path"),
            Self::InvalidFont => f.write_str("invalid Material Symbols font"),
            Self::MissingGlyph => f.write_str("Material Symbol has no drawable glyph"),
        }
    }
}
impl std::error::Error for Error {}
