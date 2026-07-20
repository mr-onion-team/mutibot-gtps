mod auth;
mod core;
mod shared;
pub(crate) mod ws;

pub use core::Bot;
pub use shared::{BotEventRaw, Socks5Config};
