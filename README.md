# MateCheck 🤝

A Rust application that helps you stay in touch with friends by tracking calendar meetings and sending automated reminders via Telegram and WhatsApp.

**Status:** ✅ Production Ready - Fully automated with GitHub Actions

## What It Does

MateCheck connects to your Google Calendar, identifies meetings with friends, and sends you Telegram reminders when you haven't seen someone in a while. It runs automatically every day via GitHub Actions.

## Features ✅

### Core Functionality
- **Google Calendar Integration** - Fetches events with OAuth 2.0 authentication
- **Smart Friend Matching** - Matches events to friends by:
  - Email addresses (primary method)
  - Event titles with name/alias matching (fallback)
- **All-Day Event Support** - Tracks both timed and all-day calendar events
- **Recurring Event Filtering** - Automatically filters out birthdays and anniversaries

### Smart Reminders
- **Automatic Early Warnings** - Reminds you 15% before your target frequency
  - 10 days → remind at day 8 (2 days early)
  - 30 days → remind at day 25 (5 days early)
  - 45 days → remind at day 38 (7 days early)
- **Future Meeting Awareness** - Skips reminders if meeting already scheduled
- **Friend Aliases** - Match calendar events with nicknames (e.g., "Lou" matches "Louise")
- **Do Not Disturb Mode** - Automatically pauses ALL reminders during specific periods
  - Create all-day calendar events with 🔕 emoji or [DND] text
  - Examples: "🔕 Vacation in Paris", "[DND] Focus Week"
  - Works with single-day and multi-day events
  - Only all-day events count (timed events ignored)

### Notifications
- **Telegram Integration** - Sends formatted reminders with clickable links
- **WhatsApp Support** - Creates WhatsApp deep links for friends without Telegram
- **Smart Fallback** - Telegram username → WhatsApp → plain name

### Automation
- **GitHub Actions** - Runs automatically on schedule
  - Weekdays: 8:00 AM Berlin time
  - Weekends: 9:30 AM Berlin time
- **Manual Testing** - Can be triggered manually for testing

## Tech Stack

- **Language:** Rust 🦀 (2021 edition)
- **APIs:** Google Calendar API v3, Telegram Bot API
- **Key Crates:** tokio, serde, chrono, clap, reqwest
- **Auth:** OAuth 2.0 for Google Calendar
- **CI/CD:** GitHub Actions
- **Tests:** 75 passing tests

## Project Structure

```
matecheck/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── config.rs            # Friend configuration loader
│   ├── calendar/            # Google Calendar integration
│   │   ├── client.rs        # OAuth & API client
│   │   ├── types.rs         # Event types
│   │   └── dnd.rs           # Do Not Disturb detection
│   ├── matcher.rs           # Event-to-friend matching logic
│   ├── reminder/
│   │   └── engine.rs        # Reminder calculation logic
│   └── telegram/            # Telegram integration
│       ├── client.rs        # Bot API client
│       └── formatter.rs     # Message formatting
├── .github/
│   └── workflows/
│       └── daily-check.yml  # Automated deployment
├── friends.yaml             # Your friends config (gitignored)
├── friends.example.yaml     # Example configuration
└── Cargo.toml              # Rust dependencies
```

## Setup

### Prerequisites

- Rust (latest stable)
- Google Calendar API credentials
- Telegram bot token
- GitHub account (for automation)

### Local Development

1. **Clone and configure:**
   ```bash
   git clone <your-repo>
   cd matecheck
   cp friends.example.yaml friends.yaml
   # Edit friends.yaml with your friends
   ```

2. **Set up Google Calendar API:**
   - Create a project in [Google Cloud Console](https://console.cloud.google.com/)
   - Enable Google Calendar API
   - Create OAuth 2.0 credentials (Desktop app)
   - Download as `credentials.json` in project root
   - Run once locally to authenticate: `cargo run -- --debug`

3. **Set up Telegram Bot:**
   - Create bot via [@BotFather](https://t.me/botfather)
   - Get your chat ID: `cargo run --bin get_chat_id`
   - Create `.env` file:
     ```
     TELEGRAM_BOT_TOKEN=your_bot_token
     TELEGRAM_CHAT_ID=your_chat_id
     ```

4. **Run locally:**
   ```bash
   cargo run                    # Normal run
   cargo run -- --debug         # Debug mode with verbose output
   cargo run -- --test-telegram # Test Telegram integration
   ```

### GitHub Actions Deployment

1. **Push code to GitHub:**
   ```bash
   git push origin master
   ```

2. **Add repository secrets** (Settings → Secrets → Actions):
   - `GOOGLE_CREDENTIALS` - Content of `credentials.json`
   - `GOOGLE_OAUTH_TOKEN` - Content of `token.json` (refresh tokens last 6+ months)
   - `TELEGRAM_BOT_TOKEN` - Your bot token
   - `TELEGRAM_CHAT_ID` - Your chat ID
   - `FRIENDS_CONFIG` - Content of `friends.yaml`

3. **Test workflow:**
   - Go to Actions tab
   - Select "Daily Friend Reminder Check"
   - Click "Run workflow"

4. **Done!** Reminders run automatically on schedule.

## Configuration

### friends.yaml Example

```yaml
friends:
  - id: "alice"
    name: "Alice Smith"
    email: "alice@example.com"
    telegram_username: "alice_tg"
    frequency_days: 30

  - id: "bob"
    name: "Bob Johnson"
    email: "bob@example.com"
    whatsapp_phone: "+1 234 567 8900"  # For friends without Telegram
    aliases: ["Bobby"]                  # Match "Bobby" in calendar
    frequency_days: 14

  - id: "charlie"
    name: "Charlie"
    frequency_days: 60                  # No contact info = plain name
```

### Field Reference

- `id` (required) - Unique identifier
- `name` (required) - Friend's display name
- `email` (optional) - For calendar matching
- `telegram_username` (optional) - Creates t.me/username link
- `whatsapp_phone` (optional) - Creates WhatsApp link (+ and spaces auto-stripped)
- `aliases` (optional) - Alternative names for calendar matching
- `frequency_days` (required) - How often you want to meet (in days)

## How It Works

1. **Fetches Calendar Events** - Gets events from last 90 days + future events
2. **Checks Do Not Disturb** - Exits early if DND event detected (skips all reminders)
3. **Matches Friends** - Identifies which events involved which friends
4. **Calculates Last Meeting** - Finds most recent past meeting per friend
5. **Checks Future Meetings** - Looks for upcoming scheduled meetings
6. **Applies Smart Logic:**
   - Reminds at 85% of target frequency (15% buffer)
   - Skips reminder if meeting already scheduled
   - Ignores recurring events (birthdays)
7. **Sends Telegram Message** - Formatted list with clickable links

## Development

### Run Tests
```bash
cargo test                    # All tests
cargo test --lib              # Library tests only
cargo test test_name          # Specific test
```

### Utilities
```bash
cargo run --bin get_chat_id   # Get your Telegram chat ID
```

## License

MIT License - See [LICENSE](LICENSE) file

## Learning Project

This project was built as a learning exercise to understand Rust coming from a Go background. It covers:
- Rust ownership, borrowing, and lifetimes
- Async/await with tokio
- OAuth 2.0 authentication
- API integration (Google Calendar, Telegram)
- GitHub Actions CI/CD
- Error handling with Result types
- Testing and test-driven development

Built with assistance from Claude Code.
