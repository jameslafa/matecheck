/// Helper script to get your Telegram chat ID
///
/// Usage:
/// 1. Send /start to your bot on Telegram
/// 2. Run: cargo run --example get_chat_id
/// 3. Use the chat_id shown to test sending messages

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Update {
    message: Option<Message>,
}

#[derive(Deserialize, Debug)]
struct Message {
    chat: Chat,
    from: User,
    text: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Chat {
    id: i64,
}

#[derive(Deserialize, Debug)]
struct User {
    id: i64,
    username: Option<String>,
    first_name: String,
}

#[derive(Deserialize, Debug)]
struct GetUpdatesResponse {
    ok: bool,
    result: Vec<Update>,
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().expect("Failed to load .env file");

    // Get bot token from environment
    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
        .expect("TELEGRAM_BOT_TOKEN not set in .env file");

    println!("🔍 Fetching recent messages sent to your bot...");
    println!();

    // Call getUpdates to get recent messages
    let url = format!("https://api.telegram.org/bot{}/getUpdates", bot_token);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to call Telegram API");

    if !response.status().is_success() {
        eprintln!("❌ API call failed: {}", response.status());
        let body = response.text().await.unwrap_or_default();
        eprintln!("Response: {}", body);
        std::process::exit(1);
    }

    let updates: GetUpdatesResponse = response
        .json()
        .await
        .expect("Failed to parse response");

    if updates.result.is_empty() {
        println!("⚠️  No messages found!");
        println!();
        println!("Please:");
        println!("1. Open Telegram and search for your bot");
        println!("2. Send /start to your bot");
        println!("3. Run this script again");
        std::process::exit(0);
    }

    println!("✅ Found {} message(s):\n", updates.result.len());

    for update in updates.result {
        if let Some(msg) = update.message {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("👤 From: {} (@{})",
                msg.from.first_name,
                msg.from.username.as_deref().unwrap_or("no username")
            );
            println!("💬 Chat ID: {}", msg.chat.id);
            println!("📝 Message: {}", msg.text.unwrap_or_default());
            println!();
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("✨ Use the Chat ID number to test sending messages:");
    println!("   cargo run --example test_telegram <chat_id>");
}
