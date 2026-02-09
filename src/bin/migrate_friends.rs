use anyhow::Result;
use matecheck::config::Config;
use matecheck::firestore::FirestoreClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    println!("🔄 Starting friends migration to Firestore...\n");

    // Load friends from YAML
    let yaml_path = std::env::var("FRIENDS_YAML_PATH")
        .unwrap_or_else(|_| "friends.yaml".to_string());

    println!("📄 Loading friends from: {}", yaml_path);
    let config = Config::load(&yaml_path)?;
    println!("✓ Loaded {} friends from YAML\n", config.friends.len());

    // Connect to Firestore
    println!("🔥 Connecting to Firestore...");
    let client = FirestoreClient::new().await?;
    println!("✓ Connected to Firestore\n");

    // Migrate each friend
    println!("📤 Migrating friends to Firestore:");
    for friend in &config.friends {
        client.friends().upsert(friend).await?;
        println!("  ✓ Migrated: {} ({})", friend.name, friend.id);
    }

    println!(
        "\n✅ Migration complete! {} friends migrated.",
        config.friends.len()
    );
    println!("\n💡 Next steps:");
    println!("  1. Verify in Firebase Console: Firestore Database > friends collection");
    println!("  2. Test loading: cargo run -- --debug");
    println!("  3. Check logs for: '✓ Config loaded from Firestore'");

    Ok(())
}
