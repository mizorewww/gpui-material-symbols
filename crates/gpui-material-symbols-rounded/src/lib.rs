#![doc = include_str!("../README.md")]
#![no_std]

/// The file's name in Google's distribution and in `gpui-material-symbols`.
pub const FILE_NAME: &str = "MaterialSymbolsRounded.ttf";

/// The file's bytes, exactly as recorded in the README.
pub static BYTES: &[u8] = include_bytes!("../MaterialSymbolsRounded.ttf");
