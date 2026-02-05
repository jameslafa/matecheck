/// Simple test program to verify Telegram bot works
///
/// Usage: cargo run --example test_telegram <your_telegram_username>
/// Example: cargo run --example test_telegram jaylcr

use matecheck::telegram::TelegramClient;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().expect("Failed to load .env file");

    // Get bot token from environment
    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
        .expect("TELEGRAM_BOT_TOKEN not set in .env file");

    // Get username from command line
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --example test_telegram <your_telegram_username>");
        eprintln!("Example: cargo run --example test_telegram jaylcr");
        std::process::exit(1);
    }
    let username = &args[1];

    println!("🤖 Creating Telegram client...");
    let client = TelegramClient::new(bot_token);

    println!("📤 Sending test message to @{}...", username);
    let message = "🎉 Hello from MateCheck! Your Telegram bot is working correctly!";

    match client.send_message(username, message, false).await {
        Ok(()) => {
            println!("✅ Message sent successfully!");
            println!("Check your Telegram to see the message.");
        }
        Err(e) => {
            eprintln!("❌ Failed to send message: {}", e);
            eprintln!();
            eprintln!("Troubleshooting:");
            eprintln!("1. Make sure you've started a chat with your bot on Telegram");
            eprintln!("2. Send /start to your bot first");
            eprintln!("3. Check that your username is correct (without @)");
            std::process::exit(1);
        }
    }
}
