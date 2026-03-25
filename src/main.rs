use calendar::client::{CalendarClient, GoogleCalendarClient};
use chrono::{Duration, Utc};
use clap::Parser;
use std::collections::HashSet;

// Declare modules
mod calendar;
mod config;
mod firestore;
mod matcher;
mod reminder;
mod telegram;

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

    /// Test Telegram bot by sending a message (uses TELEGRAM_CHAT_ID from .env, or specify chat_id)
    #[arg(long)]
    test_telegram: Option<Option<String>>,

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

    // Handle --test-telegram flag early
    if let Some(chat_id_override) = args.test_telegram {
        println!("🤖 Testing Telegram bot...");

        // Load .env file
        dotenvy::dotenv().ok(); // Don't fail if .env doesn't exist

        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
            .expect("TELEGRAM_BOT_TOKEN not set in .env file");

        // Get chat_id from command line or from .env
        let chat_id = chat_id_override.unwrap_or_else(|| {
            std::env::var("TELEGRAM_CHAT_ID")
                .expect("TELEGRAM_CHAT_ID not set in .env file and no chat_id provided")
        });

        let client = telegram::TelegramClient::new(bot_token);
        let message = "🎉 Test message from MateCheck! The Telegram integration is working.";

        println!("📤 Sending message to chat_id: {}", chat_id);

        match client.send_message(&chat_id, message, false).await {
            Ok(()) => {
                println!("✅ Message sent successfully!");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("❌ Failed to send message: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Load environment variables for Telegram
    dotenvy::dotenv().ok();

    if args.debug {
        println!("[DEBUG] Running in debug mode");
        println!("[DEBUG] Config path: {}", args.config);
    }

    // Initialize Firestore client (fail-open on error)
    let firestore = match firestore::FirestoreClient::new().await {
        Ok(client) => {
            if args.debug {
                println!("✓ Firestore connected");
            }
            Some(client)
        }
        Err(e) => {
            eprintln!("⚠️  Firestore unavailable: {}. Snooze feature disabled.", e);
            None
        }
    };

    // Load config (Firestore first, YAML fallback)
    let config = match config::Config::load_auto(firestore.as_ref(), &args.config, args.debug)
        .await
    {
        Ok(config) => {
            if args.debug {
                for friend in &config.friends {
                    let email = friend.email.as_ref().map_or("no email", |e| e.as_str());
                    let tg = friend
                        .telegram_username
                        .as_ref()
                        .map_or("no telegram", |u| u.as_str());
                    let aliases = if friend.aliases.is_empty() {
                        "no aliases".to_string()
                    } else {
                        friend.aliases.join(", ")
                    };
                    println!(
                        "  - [{}] {} ({}) - @{} - aliases: [{}] - every {} days",
                        friend.id, friend.name, email, tg, aliases, friend.frequency_days
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

    // Get active snoozes (empty set if Firestore unavailable)
    let snoozed_friends = match &firestore {
        Some(client) => client
            .snoozes()
            .get_active_snoozes()
            .await
            .unwrap_or_else(|e| {
                eprintln!("⚠️  Could not load snoozes: {}. Ignoring snoozes.", e);
                HashSet::new()
            }),
        None => HashSet::new(),
    };

    if !snoozed_friends.is_empty() && args.debug {
        println!(
            "📵 {} friend(s) currently snoozed: {:?}",
            snoozed_friends.len(),
            snoozed_friends
        );
    }

    // Connect to Google Calendar
    if args.debug {
        println!("\n🔄 Connecting to Google Calendar...");
    }

    match GoogleCalendarClient::new().await {
        Ok(client) => {
            if args.debug {
                println!("✓ Calendar client connected");
            }

            // Fetch events from last 90 days AND future events
            // Future events help us avoid reminding when meeting is already scheduled
            let max_frequency = config
                .friends
                .iter()
                .map(|f| f.frequency_days)
                .max()
                .unwrap_or(30);

            let start = Utc::now() - Duration::days(90);
            let end = Utc::now() + Duration::days(max_frequency as i64);

            if args.debug {
                println!(
                    "📅 Fetching events: {} to {}",
                    start.format("%Y-%m-%d"),
                    end.format("%Y-%m-%d")
                );
            }

            match client.fetch_events(start, end).await {
                Ok(events) => {
                    if args.debug {
                        println!("✓ Fetched {} events", events.len());
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

                    // Compute status for all friends
                    let last_meetings = matcher::find_last_meetings(&events, &config.friends);
                    let next_meetings = matcher::find_next_meetings(&events, &config.friends);

                    let friend_statuses: Vec<firestore::types::FriendStatus> = config.friends.iter().map(|friend| {
                        let last = last_meetings.get(&friend.id).and_then(|e| e.as_ref());
                        let next = next_meetings.get(&friend.id).and_then(|e| e.as_ref());
                        let days_since = last.and_then(|e| matcher::days_since(e.start));
                        let is_snoozed = snoozed_friends.contains(&friend.id);

                        let days_overdue = match days_since {
                            Some(d) => d - friend.frequency_days as i64,
                            None => 0,
                        };

                        let upcoming = matcher::days_until_next_meeting_within_window(next, friend.frequency_days);

                        let status = if upcoming.is_some() {
                            firestore::types::FriendStatusValue::OnTrack
                        } else if days_since.is_none() && last.is_none() {
                            firestore::types::FriendStatusValue::NeverMet
                        } else if days_overdue > 0 {
                            firestore::types::FriendStatusValue::Overdue
                        } else if days_overdue > -(friend.buffer_days() as i64) {
                            firestore::types::FriendStatusValue::DueSoon
                        } else {
                            firestore::types::FriendStatusValue::OnTrack
                        };

                        firestore::types::FriendStatus {
                            friend_id: friend.id.clone(),
                            friend_name: friend.name.clone(),
                            last_seen_date: last.map(|e| e.start),
                            last_seen_event: last.map(|e| e.title.clone()),
                            next_planned_date: next.map(|e| e.start),
                            next_planned_event: next.map(|e| e.title.clone()),
                            days_since_last_seen: days_since,
                            frequency_days: friend.frequency_days,
                            days_overdue,
                            status,
                            snoozed: is_snoozed,
                        }
                    }).collect();

                    // Check Do Not Disturb mode
                    let dnd_active = calendar::dnd::is_dnd_active(&events, Utc::now());
                    if let Some(ref dnd_event_title) = dnd_active {
                        if args.debug {
                            println!("🔕 Do Not Disturb is active: \"{}\"", dnd_event_title);
                            println!("   Skipping all reminders.");
                        }
                    }

                    if args.debug {
                        println!("\n🔔 Checking who needs reminders...");
                    }

                    let reminders =
                        reminder::find_friends_needing_reminders(&events, &config, &snoozed_friends);

                    // Write status report to Firestore (should_notify triggers TypeScript notification)
                    if let Some(client) = &firestore {
                        let report = firestore::types::StatusReport {
                            updated_at: Utc::now(),
                            friends: friend_statuses.clone(),
                            should_notify: dnd_active.is_none() && !reminders.is_empty(),
                        };

                        match client.status().write_report(&report).await {
                            Ok(()) => {
                                if args.debug {
                                    println!("✓ Status report written to Firestore");
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠️  Failed to write status report: {}", e);
                            }
                        }
                    }

                    if dnd_active.is_some() {
                        println!("🔕 Do Not Disturb mode active - no reminders today.");
                        return;
                    }

                    if reminders.is_empty() {
                        println!("✅ Everyone is up to date! No reminders needed.");
                    } else {
                        if args.debug {
                            println!("📋 Found {} friend(s) who need reminders:", reminders.len());
                            for reminder_info in &reminders {
                                let days_since_str = match reminder_info.days_since_last_meeting {
                                    Some(days) => format!("{} days ago", days),
                                    None => "never met".to_string(),
                                };

                                println!("  📌 {} ({})", reminder_info.friend.name, reminder_info.friend.id);
                                println!("     Last meeting: {}", days_since_str);
                                println!("     Target frequency: {} days", reminder_info.friend.frequency_days);
                                println!("     Days overdue: {}", reminder_info.days_overdue);

                                if let Some(email) = &reminder_info.friend.email {
                                    println!("     Email: {}", email);
                                }
                                if let Some(tg) = &reminder_info.friend.telegram_username {
                                    println!("     Telegram: @{}", tg);
                                }
                                println!();
                            }
                        }
                        println!("✅ Notification queued for {} friend(s) via Firestore trigger", reminders.len());
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
