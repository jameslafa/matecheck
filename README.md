# MateCheck 🤝

A Rust application that helps you stay in touch with friends by tracking calendar meetings and sending automated reminders via Telegram and WhatsApp.

**Status:** ✅ Production Ready - Fully automated with GitHub Actions + Firebase state

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

### Configuration Management

- **Firestore Configuration** - Store friends config in Firebase Firestore
- **Web UI** - Mobile-responsive web interface for managing friends
  - Add, edit, and delete friends via intuitive UI
  - Status report dashboard with color-coded badges
  - No YAML editing required
  - Deployed on GitHub Pages
  - Google Sign-In authentication
- **Automatic YAML Fallback** - Seamlessly falls back to local friends.yaml if Firestore unavailable

### Smart Reminders

- **Automatic Early Warnings** - Reminds you 15% before your target frequency
  - 10 days → remind at day 8 (2 days early)
  - 30 days → remind at day 25 (5 days early)
  - 45 days → remind at day 38 (7 days early)
- **Future Meeting Awareness** - Skips reminders if meeting already scheduled
- **Friend Aliases** - Match calendar events with nicknames (e.g., "Lou" matches "Louise")
- **Snooze Functionality** - Temporarily pause reminders for specific friends
  - Use `/snooze <name> <days>` and `/unsnooze <name>` commands in Telegram
  - Powered by Firebase Firestore + Cloud Functions
  - Fail-open design (works even if Firebase is unavailable)
- **Status Report** - Full overview of where you stand with all friends
  - Stored as a single Firestore document, updated each cron run
  - View in the web UI with color-coded status badges
  - Statuses: on track, due soon, overdue, never met
- **Do Not Disturb Mode** - Automatically pauses ALL reminders during specific periods
  - Create all-day calendar events with 🔕 emoji or [DND] text
  - Examples: "🔕 Vacation in Paris", "[DND] Focus Week"
  - Works with single-day and multi-day events
  - Only all-day events count (timed events ignored)

### Notifications

- **Telegram Integration** - Sends formatted reminders with clickable links
- **WhatsApp Support** - Creates WhatsApp deep links for friends without Telegram
- **Smart Fallback** - Telegram username → WhatsApp → plain name

### Telegram Bot Commands

- `update` - Trigger a data refresh
- `refresh` - Re-render and update the last notification message
- `snooze` - Snooze a friend for x days
- `unsnooze` - Remove a snooze for a friend

### Automation

- **GitHub Actions** - Runs automatically on schedule
  - Weekdays: 8:00 AM Berlin time
  - Weekends: 9:30 AM Berlin time
- **Manual Testing** - Can be triggered manually for testing

## Tech Stack

- **Language:** Rust 🦀 (2021 edition) + TypeScript (Cloud Functions)
- **APIs:** Google Calendar API v3, Telegram Bot API
- **Database:** Firebase Firestore (state persistence + config storage)
- **Backend:** Firebase Cloud Functions (webhook for Telegram commands + Firestore trigger)
- **Frontend:** Vanilla JavaScript + Firebase SDK (web UI)
- **Key Crates:** tokio, serde, chrono, clap, reqwest, firestore
- **Auth:** OAuth 2.0 for Google Calendar, Firebase Service Account, Google Sign-In (web UI)
- **CI/CD:** GitHub Actions
- **Hosting:** GitHub Pages (web UI)
- **Tests:** 79 passing tests (Rust)

## Project Structure

```
matecheck/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── config.rs            # Friend configuration loader (Firestore + YAML)
│   ├── calendar/            # Google Calendar integration
│   │   ├── client.rs        # OAuth & API client
│   │   ├── types.rs         # Event types
│   │   └── dnd.rs           # Do Not Disturb detection
│   ├── firestore/           # Firebase Firestore integration
│   │   ├── client.rs        # Firestore connection
│   │   ├── snoozes.rs       # Snooze repository (CRUD)
│   │   ├── status.rs        # Status report repository
│   │   └── types.rs         # Firestore data types
│   ├── matcher.rs           # Event-to-friend matching logic
│   ├── reminder/
│   │   └── engine.rs        # Reminder calculation logic
│   └── telegram/            # Telegram integration
│       ├── client.rs        # Bot API client
│       └── formatter.rs     # Message formatting
├── docs/                    # Web UI (GitHub Pages)
│   ├── index.html           # Friends management interface
│   ├── README.md            # Web UI setup instructions
│   └── SETUP.md             # Deployment guide
├── functions/               # Firebase Cloud Functions (TypeScript)
│   ├── src/
│   │   ├── index.ts         # Webhook handler (Telegram commands) + Firestore trigger
│   │   └── formatter.ts     # Status report formatting, friend lookup
│   ├── package.json
│   └── tsconfig.json
├── .github/
│   └── workflows/
│       └── daily-check.yml  # Automated deployment
├── friends.yaml             # Fallback config (gitignored, optional)
├── friends.example.yaml     # Example configuration
├── firebase.json            # Firebase configuration
└── Cargo.toml              # Rust dependencies
```

## Setup

### Prerequisites

- Rust (latest stable)
- Node.js 22+ (for Firebase Functions)
- Google Calendar API credentials
- Telegram bot token
- Firebase project (free tier)
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

2. **Set up Firebase (optional, for snooze feature):**
   - Create Firebase project
   - Enable Firestore Database
   - Enable billing (required for Secret Manager, but stays on free tier)
   - Create service account, download as `service-account.json`
   - Deploy Cloud Function: `cd functions && npm install && firebase deploy --only functions`
   - Set Telegram webhook to Cloud Function URL

3. **Add repository secrets** (Settings → Secrets → Actions):
   - `GOOGLE_CREDENTIALS` - Content of `credentials.json`
   - `GOOGLE_OAUTH_TOKEN` - Content of `token.json` (refresh tokens last 6+ months)
   - `TELEGRAM_BOT_TOKEN` - Your bot token
   - `TELEGRAM_CHAT_ID` - Your chat ID
   - `FRIENDS_CONFIG` - Content of `friends.yaml`
   - `FIREBASE_SERVICE_ACCOUNT` - Content of `service-account.json` (if using snooze)

4. **Test workflow:**
   - Go to Actions tab
   - Select "Daily Friend Reminder Check"
   - Click "Run workflow"

5. **Done!** Reminders run automatically on schedule.

### Web UI Deployment (Optional but Recommended)

Deploy the friends management interface to GitHub Pages:

1. **Enable GitHub Pages:**
   - Go to repo Settings → Pages
   - Source: Deploy from a branch
   - Branch: `master`, Folder: `/docs`
   - Save and wait 1-2 minutes

2. **Configure Firebase for web access:**
   - Follow the detailed setup guide in `docs/SETUP.md`
   - Enable Google Sign-In authentication
   - Update Firestore security rules
   - Add your GitHub Pages domain to authorized domains

3. **Access your web UI:**
   - Visit `https://YOUR_USERNAME.github.io/matecheck/`
   - Sign in with Google
   - Manage friends through the interface

See `docs/README.md` for complete setup instructions.

## Configuration

MateCheck supports two configuration methods:

1. **Firestore (Recommended)** - Store friends in Firebase Firestore, edit via web UI
2. **YAML (Fallback)** - Local friends.yaml file (automatically used if Firestore unavailable)

### Firestore Configuration (via Web UI)

The easiest way to manage friends is through the web UI:

1. Deploy web UI to GitHub Pages (see docs/SETUP.md)
2. Visit your GitHub Pages URL
3. Sign in with Google
4. Add/edit/delete friends via the interface

Changes take effect immediately - no deployment needed!

### friends.yaml Example (Fallback)

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
    whatsapp_phone: "+1 234 567 8900" # For friends without Telegram
    aliases: ["Bobby"] # Match "Bobby" in calendar
    frequency_days: 14

  - id: "charlie"
    name: "Charlie"
    frequency_days: 60 # No contact info = plain name
```

### Field Reference

- `id` (required) - Unique identifier
- `name` (required) - Friend's display name
- `email` (optional) - For calendar matching
- `telegram_username` (optional) - Creates t.me/username link
- `whatsapp_phone` (optional) - Creates WhatsApp link (+ and spaces auto-stripped)
- `aliases` (optional) - Alternative names for calendar matching
- `frequency_days` (required) - How often you want to meet (in days)

## Architecture

The system is split across two runtimes:

### Rust (runs daily via GitHub Actions)
- Reads friend config from Firestore (falls back to `friends.yaml`)
- Reads active snoozes from Firestore
- Fetches Google Calendar events (last 90 days + future)
- Matches events to friends, computes statuses (`on_track`, `due_soon`, `overdue`, `never_met`)
- Writes the full status report to Firestore (`status/latest`) with `should_notify: true`
- Does **not** send Telegram messages directly

### Firebase Cloud Functions (TypeScript, always-on)
- **`morningNotification` trigger** — fires when Rust writes `should_notify: true` to `status/latest`; formats and sends the Telegram message; stores `last_notification_message_id`
- **`webhook`** — receives Telegram updates:
  - `/update` — triggers the GitHub Actions workflow via API
  - `/snooze <name> <days>` — writes snooze to Firestore, edits the notification message
  - `/unsnooze <name>` — removes a snooze from Firestore, edits the notification message (matches by id, name, or alias)

### Formatting (TypeScript `formatter.ts`)
All status report formatting lives in `formatter.ts`, not in Rust. This includes:
- 4-bucket grouping: Already planned / Need to catch up / Schedule soon / On track
- Berlin-time timestamp header
- Friend line format (bold name, days since last seen, snooze expiry)
- Friend lookup by id, name, or alias (case-insensitive)

## How It Works

1. **Loads Active Snoozes** - Queries Firestore for snoozed friends (fail-open if unavailable)
2. **Fetches Calendar Events** - Gets events from last 90 days + future events
3. **Writes Status Report** - Computes and stores friend status snapshot in Firestore with `should_notify: true`
4. **Checks Do Not Disturb** - Exits early if DND event detected (skips all reminders)
5. **Matches Friends** - Identifies which events involved which friends
6. **Calculates Last Meeting** - Finds most recent past meeting per friend
7. **Checks Future Meetings** - Looks for upcoming scheduled meetings
8. **Applies Smart Logic:**
   - Filters out snoozed friends
   - Reminds at 85% of target frequency (15% buffer)
   - Skips reminder if meeting already scheduled
   - Ignores recurring events (birthdays)
9. **Firestore Trigger Fires** - Cloud Function detects `should_notify: true`, formats the report in TypeScript, sends Telegram message
10. **Snooze/Unsnooze** - `/snooze` and `/unsnooze` commands update Firestore and edit the live notification message

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

### Deploy Firebase Functions

```bash
firebase deploy --only functions
```

## License

MIT License - See [LICENSE](LICENSE) file

## Learning Project

This project was built as a learning exercise to understand Rust coming from a Go background. It covers:

- Rust ownership, borrowing, and lifetimes
- Async/await with tokio
- OAuth 2.0 authentication
- API integration (Google Calendar, Telegram, Firestore)
- Firebase Cloud Functions (TypeScript)
- State persistence with Firestore
- GitHub Actions CI/CD
- Error handling with Result types
- Testing and test-driven development
- Graceful degradation (fail-open patterns)
