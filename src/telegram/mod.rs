// Telegram integration module
// This module handles sending reminder messages via Telegram Bot API

mod client;
mod formatter;

pub use client::TelegramClient;
pub use formatter::{format_morning_with_buttons, format_reminder_message};
