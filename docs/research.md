# Upstream investigation

Investigated on 2026-10-29; all counts are snapshot counts, not promises about future Google releases.

- GPUiKit: https://github.com/iamnbutler/gpuikit at `03006cfbfacdecbc1804e09048ffcd2ecf21f09e`, package 0.9.0. Its icon buttons, alert and empty states accept `gpui::Svg`; menus accept closures returning that same type. It depends on the `gpui-unofficial` distribution. A separate custom UI element or string-font renderer would lose this compatibility.
- Google: https://github.com/google/material-design-icons at `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`. Downloaded Git objects and the complete three TTF distributions with their codepoint catalogs; sparse checkout avoids expanding approximately 4.87 million files.
- Official guide: https://developers.google.com/fonts/docs/material_symbols . Three styles and four conventional axes. The online guide also mentions ROND; the pinned repository fonts each expose exactly FILL, GRAD, opsz and wght. The test suite reads actual fvar metadata to enforce this contract.
- ttf-parser: https://github.com/harfbuzz/ttf-parser at `0c7291223fe9e0bc2808f0255cb991066eedd796`; source inspection verified `set_variation` and `outline_glyph`. The selected published version is 0.25.1.
- Context7 resolved GPUI and provided AssetSource/Svg documentation. Its ttf-parser search returned unrelated libraries; official cloned source was used instead of accepting the unrelated matches.

## Coverage evidence

All three font catalogs have 4,304 names (including aliases). The official `symbols/web` tree has 4,157 names, of which 55 are absent from the fonts and 202 font names are absent from the SVG tree. Therefore the union is 4,359, not 4,304 or 4,157. The 55 supplemental names have 27,720 SVG files covering 3 styles × 7 weights × 3 grades × 2 fills × 4 optical sizes. All are embedded, byte-for-byte, in a deduplicated binary pack with a sorted offset index. `assets/svg-names.txt` captures the independent upstream directory list for coverage tests.

The continuous font path emits quadratic/cubic SVG outlines at the requested coordinates. The SVG path uses the design em square (960), not the font ascender (1056); using the ascender introduces vertical drift. Some icons (for example home) intentionally have identical default outlines in two styles, so style tests check for actual differences across several symbols rather than incorrectly requiring every icon to differ.

## Tradeoffs

- Full fidelity and offline use require roughly 51 MB of raw embedded font/SVG data. Per-icon or per-style subsetting is not implemented.
- Variable glyphs support continuous axes; supplemental SVGs support only upstream-provided discrete variants. Unsupported combinations are explicit errors; there is no hidden rounding or simulated interpolation.
- No mutable global font registration or unbounded library cache. Each SVG generation parses borrowed static font data; GPUI owns its usual SVG rendering cache.
- No claim of continuous ROND support, legacy Material Icons Two Tone support, or automatic coverage of future additions.
- The core SVG library is platform independent. The native showcase demonstrates the GPUiKit type integration; platform backends remain GPUI's responsibility.

## Dependency compatibility finding

GPUiKit 0.9.0 declares a broad `gpui-unofficial = "1.18"` dependency. Fresh resolution selected 1.25.3, where `Window::blur` requires an App argument; GPUiKit 0.9.0 still calls the old method. Its own checked-in lockfile uses 1.18.0. This crate therefore pins GPUI and the showcase platform to `=1.18.0`, rather than shipping an example known not to compile. The six official SVG fixtures test rasterized geometry against the generated font outlines, including their placement within the square.
