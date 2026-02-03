# MateCheck 🤝

A Rust application that helps you stay in touch with friends by tracking calendar meetings and sending reminders when it's been too long.

**Status:** 🚧 Work in Progress (Phase 4/6 complete)

## What It Does

MateCheck connects to your Google Calendar, identifies meetings with friends, and reminds you when you haven't seen someone in a while based on your configured frequency preferences.

### Current Features ✅

- **Google Calendar Integration** - Fetches events from the last 90 days via OAuth 2.0
- **Smart Friend Matching** - Matches events to friends by:
  - Email addresses (primary method)
  - Event titles (fallback for friends without emails)
- **Last Meeting Tracking** - Finds the most recent meeting with each friend
- **Reminder Engine** - Identifies friends who are overdue based on your frequency preferences
- **Configurable Frequencies** - Set how often you want to see each friend (in days)

### Coming Soon 🚧

- **Telegram Notifications** - Send reminders via Telegram bot (Phase 5)
- **GitHub Actions Automation** - Run daily checks automatically (Phase 6)
- **Future Enhancements** (Phase 7):
  - Check future calendar for scheduled meetings
  - Early reminder threshold (remind before overdue)
  - Filter recurring events (birthdays, anniversaries)

## Tech Stack

- **Language:** Rust 🦀
- **APIs:** Google Calendar API v3, Telegram Bot API
- **Key Crates:** tokio (async), serde (serialization), chrono (dates), clap (CLI)
- **Auth:** OAuth 2.0 for Google Calendar

## Project Structure

```
matecheck/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── config.rs            # Friend configuration loader
│   ├── calendar/            # Google Calendar integration
│   ├── matcher.rs           # Event-to-friend matching logic
│   └── reminder/            # Reminder engine
├── friends.yaml             # Your friends config (gitignored)
├── friends.example.yaml     # Example configuration
└── Cargo.toml              # Rust dependencies
```

## Setup (for Development)

### Prerequisites

- Rust (latest stable)
- Google Calendar API credentials
- (Optional) Telegram bot token

### Quick Start

1. **Clone and configure:**
   ```bash
   git clone <your-repo>
   cd matecheck
   cp friends.example.yaml friends.yaml
   # Edit friends.yaml with your friends
   ```

2. **Set up Google Calendar API:**
   - Create a project in Google Cloud Console
   - Enable Google Calendar API
   - Create OAuth 2.0 credentials
   - Download as `credentials.json` in project root

3. **Run:**
   ```bash
   cargo run --bin matecheck -- --debug
   ```

   On first run, a browser will open for Google OAuth authorization.

## Configuration

Create `friends.yaml` with your friends:

```yaml
friends:
  - id: "alice"                    # Unique identifier
    name: "Alice"
    email: "alice@example.com"     # For email matching
    telegram_username: "alice_tg"  # For reminders (no @ prefix)
    frequency_days: 14             # Want to see every 2 weeks

  - id: "bob"
    name: "Bob"
    email: ~                       # No email = match by title only
    telegram_username: ~           # Optional
    frequency_days: 30
```

## Development

This is a learning project built step-by-step following a didactic approach. See `plan.md` for the full implementation roadmap.

### Current Status

- ✅ Phase 1: Foundation (config loading, CLI)
- ✅ Phase 2: Google Calendar Integration
- ✅ Phase 3: Friend Matching Logic
- ✅ Phase 4: Reminder Logic
- 🚧 Phase 5: Telegram Integration (next)
- 🚧 Phase 6: Deployment (GitHub Actions)

## Learning Goals

This project is a tutorial for learning Rust, coming from a Go background, with emphasis on:
- Rust ownership and borrowing
- Error handling with Result types
- Async/await patterns
- Trait-based design
- Testing and modular architecture

## License

MIT License - feel free to use, modify, and distribute!

---

Built with 🦀 Rust and ☕ coffee
