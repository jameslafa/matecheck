use anyhow::Result;
use serde::Serialize;

use super::formatter::InlineKeyboardMarkup;

/// Telegram Bot API client
///
/// Responsible for sending messages to users via Telegram.
/// Uses the Telegram Bot API: https://core.telegram.org/bots/api
pub struct TelegramClient {
    bot_token: String,
    http_client: reqwest::Client,
}

/// Request body for sending a message
///
/// This matches the Telegram sendMessage API format.
/// See: https://core.telegram.org/bots/api#sendmessage
#[derive(Serialize)]
struct SendMessageRequest {
    /// The chat ID - can be a username (@username) or numeric ID
    chat_id: String,
    /// The message text to send
    text: String,
    /// Parse mode for formatting (e.g., "Markdown" or "HTML")
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<String>,
    /// Disable link previews in the message
    #[serde(skip_serializing_if = "Option::is_none")]
    disable_web_page_preview: Option<bool>,
    /// Inline keyboard attached to the message
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_markup: Option<InlineKeyboardMarkup>,
}

impl TelegramClient {
    /// Creates a new Telegram client with the given bot token
    ///
    /// # Arguments
    /// * `bot_token` - The bot token from @BotFather (format: "123456:ABC-DEF...")
    ///
    /// # Example
    /// ```
    /// use matecheck::telegram::TelegramClient;
    /// let client = TelegramClient::new("123456:ABC-DEF".to_string());
    /// ```
    pub fn new(bot_token: String) -> Self {
        TelegramClient {
            bot_token,
            http_client: reqwest::Client::new(),
        }
    }

    /// Sends a message to a Telegram user
    ///
    /// # Arguments
    /// * `username` - The recipient's Telegram username (without @) or numeric chat ID
    /// * `message` - The message text to send
    /// * `use_markdown` - Whether to parse the message as Markdown
    ///
    /// # Returns
    /// * `Ok(())` if the message was sent successfully
    /// * `Err` if the API call failed
    pub async fn send_message(
        &self,
        username: &str,
        message: &str,
        use_markdown: bool,
    ) -> Result<()> {
        self.send_message_with_buttons(username, message, use_markdown, None)
            .await
    }

    /// Sends a message with optional inline keyboard buttons
    ///
    /// # Arguments
    /// * `username` - The recipient's Telegram username (without @) or numeric chat ID
    /// * `message` - The message text to send
    /// * `use_markdown` - Whether to parse the message as Markdown
    /// * `buttons` - Optional inline keyboard markup
    ///
    /// # Returns
    /// * `Ok(())` if the message was sent successfully
    /// * `Err` if the API call failed
    pub async fn send_message_with_buttons(
        &self,
        username: &str,
        message: &str,
        use_markdown: bool,
        buttons: Option<InlineKeyboardMarkup>,
    ) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );

        // Format chat_id: add @ prefix for usernames, but not for numeric IDs
        let chat_id = if username.starts_with('@') || username.chars().all(|c| c.is_numeric()) {
            username.to_string()
        } else {
            format!("@{}", username)
        };

        let request_body = SendMessageRequest {
            chat_id,
            text: message.to_string(),
            parse_mode: if use_markdown {
                Some("Markdown".to_string())
            } else {
                None
            },
            disable_web_page_preview: Some(true),
            reply_markup: buttons,
        };

        let response = self
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read error body".to_string());
            Err(anyhow::anyhow!(
                "Failed to send message to {}: HTTP {}\nResponse: {}",
                username,
                status,
                error_body
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_client_creation() {
        let client = TelegramClient::new("test_token".to_string());
        assert_eq!(client.bot_token, "test_token");
    }
}
