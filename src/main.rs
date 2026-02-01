use calendar::client::{CalendarClient, GoogleCalendarClient};
use chrono::{Duration, Utc};
use clap::Parser;

// Declare modules
mod calendar;
mod config;

/// MateCheck - Track when you last met your friends
///
/// This doc comment becomes the program description in --help!
#[derive(Parser, Debug)]
#[command(name = "matecheck")]
#[command(about = "Track friend meetings from Google Calendar", long_about = None)]
struct Args {
    /// Path to friends configuration file
    ///
    /// This doc comment becomes the help text for --config
    #[arg(short, long, default_value = "friends.yaml")]
    config: String,

    /// Enable debug mode with verbose output
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

#[tokio::main]
async fn main() {
    // Initialize rustls crypto provider (required for TLS)
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Parse command-line arguments
    // This automatically handles --help and --version!
    let args = Args::parse();

    if args.debug {
        println!("[DEBUG] Running in debug mode");
        println!("[DEBUG] Config path: {}", args.config);
    }

    // Load config using the path from CLI args
    let _config = match config::Config::load(&args.config) {
        Ok(config) => {
            println!("✓ Config loaded successfully from: {}", args.config);
            println!("Found {} friends:", config.friends.len());

            for friend in &config.friends {
                if args.debug {
                    // In debug mode, show more details
                    let email = friend.email.as_ref().map_or("no email", |e| e.as_str());
                    let tg = friend
                        .telegram_username
                        .as_ref()
                        .map_or("no username", |u| u.as_str());
                    println!(
                        "  - [{}] {} ({}) - @{} - meet every {} days",
                        friend.id, friend.name, email, tg, friend.frequency_days
                    );
                } else {
                    println!(
                        "  - {} - meet every {} days",
                        friend.name, friend.frequency_days
                    );
                }
            }
            config
        }
        Err(error) => {
            eprintln!("✗ Error loading config: {}", error);
            if args.debug {
                eprintln!("[DEBUG] Full error: {:?}", error);
            }
            std::process::exit(1);
        }
    };

    // TEST: Calendar API integration
    println!("\n🔄 Testing Google Calendar connection...");

    match GoogleCalendarClient::new().await {
        Ok(client) => {
            println!("✓ Calendar client created successfully");
            println!("  (On first run, a browser will open for OAuth authorization)");

            // Fetch events from last 30 days
            let start = Utc::now() - Duration::days(30);
            let end = Utc::now();

            println!(
                "\n📅 Fetching events from {} to {}...",
                start.format("%Y-%m-%d"),
                end.format("%Y-%m-%d")
            );

            match client.fetch_events(start, end).await {
                Ok(events) => {
                    println!("✓ Fetched {} events", events.len());

                    if args.debug {
                        for event in events.iter().take(5) {
                            let attendee_list = if event.attendees.is_empty() {
                                "no attendees".to_string()
                            } else {
                                event.attendees.join(", ")
                            };

                            println!(
                                "  - {} | {} | {}",
                                event.title,
                                attendee_list,
                                event.start.format("%Y-%m-%d %H:%M")
                            );
                        }
                        if events.len() > 5 {
                            println!("  ... and {} more", events.len() - 5);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("✗ Failed to fetch events: {}", error);
                    if args.debug {
                        eprintln!("[DEBUG] Full error: {:?}", error);
                    }
                }
            }
        }
        Err(error) => {
            eprintln!("✗ Failed to create calendar client: {}", error);
            if args.debug {
                eprintln!("[DEBUG] Full error: {:?}", error);
            }
            std::process::exit(1);
        }
    }
}
