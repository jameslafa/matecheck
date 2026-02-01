/// Helper binary to fetch real calendar events and save the response to JSON
///
/// Usage: cargo run --bin record_calendar_response
///
/// This will:
/// 1. Authenticate with Google Calendar
/// 2. Fetch events from the last 30 days
/// 3. Save the raw API response to tests/fixtures/calendar_events.json

use anyhow::Result;
use chrono::{Duration, Utc};
use matecheck::calendar::client::{CalendarClient, GoogleCalendarClient};
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize rustls crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    println!("🔄 Fetching calendar events from Google Calendar API...");

    // Create calendar client (will use existing token or prompt for auth)
    let client = GoogleCalendarClient::new().await?;

    // Fetch events from last 30 days
    let start = Utc::now() - Duration::days(30);
    let end = Utc::now();

    println!("📅 Fetching events from {} to {}", start.format("%Y-%m-%d"), end.format("%Y-%m-%d"));

    let events = client.fetch_events(start, end).await?;

    println!("✓ Fetched {} events", events.len());

    // Create fixtures directory if it doesn't exist
    let fixtures_dir = Path::new("tests/fixtures");
    fs::create_dir_all(fixtures_dir)?;

    // Serialize events to JSON
    let json = serde_json::to_string_pretty(&events)?;

    // Save to file
    let output_path = fixtures_dir.join("calendar_events.json");
    fs::write(&output_path, json)?;

    println!("✓ Saved API response to {}", output_path.display());
    println!("\nYou can now use this fixture in tests!");

    Ok(())
}
