use serde::Serialize;

/// Inline keyboard button for Telegram
#[derive(Serialize, Clone, Debug)]
pub struct InlineKeyboardButton {
    /// Display text on the button
    pub text: String,
    /// Callback data sent when button is pressed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_data: Option<String>,
}

/// Inline keyboard markup for Telegram messages
#[derive(Serialize, Clone, Debug)]
pub struct InlineKeyboardMarkup {
    /// Grid of buttons (each Vec is a row)
    pub inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
}
