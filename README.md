# gpui-material-symbols

Google Material Symbols 的完整离线 Rust 图标库，直接对接 [gpuikit](https://github.com/iamnbutler/gpuikit) 的 GPUI `Svg` 接口。Apache-2.0。

## 覆盖范围

以 `google/material-design-icons` 提交 `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7` 为可复现基准，取官方字体清单与 SVG 清单的并集：

| 资源 | 覆盖 |
| --- | --- |
| 统一图标名称 | **4,359**，保留官方别名 |
| 可变字体名称 | **4,304**，每个均支持 Outlined / Rounded / Sharp |
| SVG 独有名称 | **55**，三种风格、全部 **27,720** 个官方离散变体 |
| 连续可变轴 | FILL `0..=1`、wght `100..=700`、GRAD `-50..=200`、opsz `20..=48` |

三个风格与填充状态互相独立：`Rounded + filled`、`Sharp + unfilled` 等均可。所有资源随包内嵌，构建及运行无需下载字体、注册系统字体或访问网络。默认包含完整集合，原始内嵌资源约 51 MB；未做按图标裁剪。

**SVG 独有图标的限制：** Google 未为这些名称提供可变字体。它们支持官方给出的离散坐标：填充 0/1、粗细 100/200/300/400/500/600/700、grade -25/0/200、光学尺寸 20/24/40/48。任意中间值不会被静默舍入；用 `is_supported()` 检查，或 `available_variants()` 枚举，生成不支持的组合返回 `Error::UnsupportedVariant`。

此快照的三个字体实际均有四个轴。Google Fonts 在线接口另提及 `ROND`，但此 Git 发行物没有该轴，本库不伪造它。这里的完整性指上述固定官方快照，未来上游新增资源需执行同步流程。旧版 Material Icons 的独立 Two Tone 风格不属于 Material Symbols。

## 安装

当前仓库为 private，使用拥有读取权限的 Git 凭据：

```toml
[dependencies]
gpui-material-symbols = { git = "https://github.com/mizorewww/gpui-material-symbols" }
gpuikit = "0.9"
gpui = { package = "gpui-unofficial", version = "=1.18.0" }
```

本库使用 GPUiKit 同源的 `gpui-unofficial` 包，不与 crates.io 上独立的 `gpui` 包混用。为兼容 GPUiKit 0.9，本库固定 `gpui-unofficial =1.18.0`；更新 GPUI 时需同步验证 GPUiKit。`Cargo.lock` 记录实际验证依赖。生产应用可将 Git 依赖固定到具体 `rev`。

## GPUiKit 集成

在应用启动时组合资源源，让 GPUiKit 自带的字体和 Radix 图标继续可用：

```rust,ignore
use gpui_material_symbols::MaterialSymbols;

Application::with_platform(gpui_platform::current_platform(false))
    .with_assets(MaterialSymbols.with_fallback(gpuikit::assets()))
    .run(|cx| {
        gpuikit::init(cx);
        // 创建应用窗口……
    });
```

直接把图标交给现有 GPUiKit 组件：

```rust,ignore
use gpui_material_symbols::{Axes, Style, Symbol};
use gpuikit::elements::icon_button::icon_button;

let favorite = Symbol::FAVORITE.icon()
    .style(Style::Rounded)
    .filled(true)
    .svg();
let button = icon_button("favorite", favorite);

let custom = Symbol::HOME.icon()
    .style(Style::Sharp)
    .axes(Axes::new(0.5, 550.0, 25.0, 32.0)?)
    .svg();
```

`.svg()` 返回原生 `gpui::Svg`，支持 GPUI 的 `Styled` 接口（如 `.size(px(24.))`、`.text_color(...)`）。视觉尺寸与光学尺寸独立，改变布局大小不会暗中改变图标形状。建议在按钮等父控件上设置可访问名称，而不是把图标名称当作面向用户的标签。

完整、可运行示例：

```sh
cargo run --features showcase --example showcase
```

## 查找与枚举

```rust
use gpui_material_symbols::{Style, Symbol};

let home: Symbol = "home".parse().unwrap();
assert_eq!(home, Symbol::HOME);
assert!(home.is_variable());
assert!(Symbol::named("does_not_exist").is_none());

for symbol in Symbol::all().filter(|s| s.name().contains("arrow")) {
    println!("{}", symbol.name());
}

// 名称以数字开头时常量加前导下划线，例如 Symbol::_10K。
let supplemental = Symbol::APP_SPARK;
assert!(!supplemental.is_variable());
for icon in supplemental.available_variants() {
    assert!(icon.is_supported());
}
assert!(supplemental.icon().style(Style::Rounded).to_svg().is_ok());
```

生成的常量由官方清单自动构建，不需要手工维护数千个枚举项。`codepoint(style)` 对 SVG 独有名称返回 `None`。

## 不依赖 GPUI 的 SVG 导出

```toml
[dependencies]
gpui-material-symbols = { git = "https://github.com/mizorewww/gpui-material-symbols", default-features = false }
```

```rust
use gpui_material_symbols::{Style, Symbol};
let svg = Symbol::HOME.icon().style(Style::Rounded).to_svg().unwrap();
assert!(svg.contains("<svg"));
```

```sh
cargo run --no-default-features --example export -- home rounded > home.svg
```

`asset_path()` 是确定性的命名空间路径，包含名称、风格和四个轴；可用 `Icon::from_asset_path()` 反向解析。GPUI 依据该路径缓存渲染结果。资源加载器本身不持有无限增长的变体缓存。`list()` 仅列出每种风格的默认坐标，因为连续轴的所有取值无法有限枚举。

## 验证与上游同步

```sh
python3 scripts/sync_assets.py --verify
cargo fmt --all -- --check
cargo test --no-default-features
cargo test --test integration
cargo check --features showcase --example showcase
cargo clippy --all-targets --all-features -- -D warnings
```

测试核对完整名称清单、三种字体轴、每个可变图标的四组参数（51,648 次 SVG 生成）、全部 27,720 个额外 SVG 变体、真实几何变化、坐标系、非法输入与现有资源源的组合行为。

更新资源时固定完整提交 SHA，先同步字体，再从同一上游 Git 提交抽取额外 SVG：

```sh
git clone --depth 1 --no-checkout https://github.com/google/material-design-icons .research/material-design-icons
python3 scripts/sync_assets.py --revision <full-40-character-commit-sha>
python3 scripts/sync_supplemental.py .research/material-design-icons
python3 scripts/sync_assets.py --verify
cargo test --no-default-features
```

已有 clone 时先 fetch 目标提交。无需 checkout 数百万个 SVG 文件；补充提取器直接读取 Git objects。更新后审核 `assets/manifest.json`、覆盖统计及测试中的快照计数，再提交资源变化。下载脚本仅用于维护，Cargo 构建不会运行它们。

可用的 GitHub Actions 配置保存在 [CI 模板](docs/ci.yml)。当前创建仓库的令牌没有 `workflow` scope，因此未启用远程 Actions；以后具备该权限时可将模板移至 `.github/workflows/ci.yml`。上述验收命令均可本地执行。

参见 [调查记录](docs/research.md)。本项目代码与 Google 资源均按 Apache-2.0 分发；上游许可证保存在 `assets/LICENSE`，归属信息见 `NOTICE`。
