use include_bytes_plus::include_bytes;

pub const DEFAULT_MENTION: &str = "@ThatDraenGuy";
pub const DEFAULT_LOCALE: &str = "ru";
pub const DEFAULT_TOP_LIMIT: u64 = 5;

pub const SHIPU_STICKER: [u8; 36758] = include_bytes!("resources/shipu.webp");
