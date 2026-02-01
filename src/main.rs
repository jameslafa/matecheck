use clap::Parser;

// Declare the config module (tells Rust to look for src/config.rs)
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

fn main() {
    // Parse command-line arguments
    // This automatically handles --help and --version!
    let args = Args::parse();

    if args.debug {
        println!("[DEBUG] Running in debug mode");
        println!("[DEBUG] Config path: {}", args.config);
    }

    // Load config using the path from CLI args
    match config::Config::load(&args.config) {
        Ok(config) => {
            println!("✓ Config loaded successfully from: {}", args.config);
            println!("Found {} friends:", config.friends.len());

            for friend in &config.friends {
                if args.debug {
                    // In debug mode, show more details
                    println!(
                        "  - {} ({}) - @{} - meet every {} days",
                        friend.name,
                        friend.email,
                        friend.telegram_username,
                        friend.frequency_days
                    );
                } else {
                    println!(
                        "  - {} - meet every {} days",
                        friend.name, friend.frequency_days
                    );
                }
            }
        }
        Err(error) => {
            eprintln!("✗ Error loading config: {}", error);
            if args.debug {
                eprintln!("[DEBUG] Full error: {:?}", error);
            }
            std::process::exit(1);
        }
    }
}
