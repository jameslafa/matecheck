# MateCheck - Friend Meeting Reminder System

## Project Overview

A Rust application that connects to Google Calendar, tracks meetings with friends, and sends Telegram reminders when it's been too long since the last meeting.

**Learning Goal**: Tutorial project to learn Rust, coming from Go background, with emphasis on understanding each step and good separation of concerns.

## Development Approach

**IMPORTANT: Read this section at the start of every session to maintain consistency.**

### Learning Style

- **Step-by-step, didactic approach**: Explain concepts thoroughly as we build
- **Coming from Go background**: Relate Rust concepts to Go equivalents when helpful
- **Hands-on implementation**: User writes the code to learn effectively

### Scaffolding Method

1. **Claude creates structure**: File organization, function signatures, type definitions
2. **Claude adds detailed comments**: Hints, sequencing, and guidance inside functions
3. **User implements**: User writes the actual function bodies based on hints
4. **Claude guides**: Answer questions, explain concepts, help when stuck
5. **Clean up after**: Remove tutorial comments once implementation is complete and working

### Code Quality

- Keep code **clean and production-ready**
- Remove TODO comments and tutorial hints after implementation
- Maintain proper documentation (doc comments with ///)
- Write tests for new functionality

### Dependency Management

- **Always check for latest compatible versions** before adding dependencies
- Use `cargo search <crate-name>` to find the latest version
- Verify version compatibility between related crates (e.g., google-calendar3 + yup-oauth2)
- Don't add arbitrary version numbers without checking

### Git Workflow

- **NEVER commit automatically** - always ask the user first
- **ALWAYS update "Current Status" section** before committing - add completed features, update phase progress, increment session info
- **Claude can suggest** when it's a good time to commit (e.g., after completing a phase/step)
- User makes the final decision on when to commit

## Core Functionality

1. Read a configurable list of friends (YAML/JSON)
2. Connect to Google Calendar via OAuth 2.0
3. Match calendar events to friends based on:
   - Invitee email addresses (primary method)
   - Event titles (fallback when no invitees)
4. Calculate time since last meeting for each friend
5. Send Telegram reminders for friends not seen within their configured frequency
6. Provide CLI debug mode for development
7. Run as GitHub Actions cron job for production

## Architecture Decisions

### Technology Choices

- **Language**: Rust (learning from Go background)
- **Config Format**: YAML (human-readable, not tracked in git)
- **Calendar API**: Google Calendar API v3 with OAuth 2.0
- **Notifications**: Telegram Bot API
- **Persistence**: Stateless for v1 (query calendar each run), Firebase for v2
- **Execution**: GitHub Actions cron + local CLI

### Key Design Principles

- **Separation of Concerns**: Each module has single responsibility
- **Trait-based Design**: Use traits (like Go interfaces) for testability
- **Explicit Error Handling**: Use Result<T, E> types throughout
- **Configuration Over Code**: Externalize friend list and settings

### Timezone & Granularity

- **Timezone**: Europe/Berlin (hardcoded)
- **Granularity**: Day-level only (no hour precision needed)

## Project Structure

```
matecheck/
├── src/
│   ├── main.rs                 # CLI entry point, orchestration
│   ├── config.rs               # Load friends config (Firestore + YAML fallback)
│   ├── calendar/
│   │   ├── mod.rs              # Module declaration
│   │   ├── client.rs           # Google Calendar API client
│   │   ├── types.rs            # Calendar event types
│   │   └── dnd.rs              # Do Not Disturb detection
│   ├── firestore/              # Firebase Firestore integration
│   │   ├── mod.rs
│   │   ├── client.rs           # Firestore connection
│   │   ├── snoozes.rs          # Snooze repository (CRUD)
│   │   └── types.rs            # Firestore data types
│   ├── matcher.rs              # Logic to match events to friends
│   ├── reminder/
│   │   ├── mod.rs
│   │   └── engine.rs           # Determine who needs reminders
│   └── telegram/
│       ├── mod.rs
│       ├── client.rs           # Telegram bot client
│       └── formatter.rs        # Message formatting + inline buttons
├── docs/                       # Web UI (GitHub Pages)
│   ├── index.html              # Friends management interface
│   ├── README.md               # Web UI setup guide
│   └── SETUP.md                # Deployment instructions
├── functions/                  # Firebase Cloud Functions (TypeScript)
│   ├── src/
│   │   └── index.ts            # Webhook handler for button callbacks
│   ├── package.json
│   └── tsconfig.json
├── .github/
│   └── workflows/
│       └── daily-check.yml     # Automated deployment
├── Cargo.toml                  # Rust dependencies
├── .gitignore                  # Exclude sensitive files
├── friends.example.yaml        # Example config (checked in)
├── friends.yaml                # Fallback config (gitignored, optional)
├── firebase.json               # Firebase configuration
└── PLAN.md                     # This file
```

## Friends Configuration Schema

```yaml
friends:
  - id: "alice" # Unique identifier (required)
    name: "Alice"
    email: "alice@example.com" # Optional - for calendar matching
    telegram_username: "alice_tg" # Optional - for sending reminders
    frequency_days: 30

  - id: "bob_jones"
    name: "Bob Jones"
    email: ~ # Optional - can be omitted
    telegram_username: ~ # Optional - can be omitted
    frequency_days: 14
```

**Note**: All friend IDs must be unique. Validation is performed when loading config.

## Step-by-Step Implementation Plan

### Phase 1: Foundation

**Goal**: Set up project basics and configuration loading

1. **Step 1.1: Project Setup**
   - Review existing Cargo.toml
   - Add initial dependencies (serde, serde_yaml, chrono, tokio)
   - Set up .gitignore
   - Learning: Cargo basics, dependency management

2. **Step 1.2: Configuration Module**
   - Create `src/config.rs`
   - Define Friend struct with serde deserialization
   - Implement config loading from YAML
   - Create `friends.example.yaml`
   - Learning: Structs, traits (Deserialize), Result types, file I/O

3. **Step 1.3: Basic CLI**
   - Create minimal `src/main.rs`
   - Add CLI argument parsing (clap crate)
   - Add debug flag
   - Load and print config
   - Learning: main function, tokio async runtime, error handling

### Phase 2: Google Calendar Integration

**Goal**: Fetch calendar events

4. **Step 2.1: Calendar Types**
   - Create `src/calendar/mod.rs` and `types.rs`
   - Define Event struct matching Google Calendar API
   - Learning: Modules, nested modules, visibility

5. **Step 2.2: Calendar Client**
   - Create `src/calendar/client.rs`
   - Set up OAuth 2.0 flow
   - Implement calendar event fetching
   - Learning: Traits, async/await, HTTP clients, OAuth

6. **Step 2.3: Date Handling**
   - Add timezone support (Europe/Berlin)
   - Implement date comparison utilities
   - Learning: chrono crate, date/time in Rust

### Phase 3: Friend Matching Logic

**Goal**: Match calendar events to friends

7. **Step 3.1: Matcher Module**
   - Created `src/matcher.rs` (flat structure, not nested modules)
   - Implemented email-based matching (handles optional emails)
   - Implemented title-based matching (case-insensitive fallback)
   - Optimized `find_matches()` to skip email checks when no attendees
   - Refactored `Friend` struct: added `id` field, made email/telegram optional
   - Added ID uniqueness validation in `Config::load()`
   - Learning: Option types, HashSet, pattern matching, optimization

8. **Step 3.2: Last Meeting Tracker**
   - Implemented `find_last_meetings()` - tracks most recent event per friend
   - Used friend ID as HashMap key (handles optional emails correctly)
   - Returns `HashMap<String, Option<Event>>` (key = friend.id)
   - Implemented `days_since()` - calculates days between date and now
   - Implemented `days_since_last_meeting()` - convenience wrapper
   - All tests passing (17 tests total in matcher module)
   - Learning: HashMap operations, Option<&T> vs &Option<T>, mutable references, pattern matching

### Phase 4: Reminder Logic

**Goal**: Determine who needs reminders

9. **Step 4.1: Reminder Engine**
   - Created `src/reminder/mod.rs` and `engine.rs`
   - Implemented `find_friends_needing_reminders()` with business logic
   - Returns `Vec<ReminderInfo>` with friend, days_since, and days_overdue
   - Handles friends never met (days_since = None, always remind)
   - Handles friends overdue (days_since > frequency_days)
   - All tests passing (5 tests in reminder engine)
   - Learning: Pattern matching with guards, HashMap lookups, business logic, struct construction

### Phase 5: Telegram Integration

**Goal**: Send notifications

10. **Step 5.1: Telegram Bot Setup**
    - Created bot via BotFather
    - Stored bot token in .env file and GitHub secret
    - Learning: External service integration

11. **Step 5.2: Telegram Client**
    - Created `src/telegram/mod.rs` and `client.rs`
    - Implemented message sending via Bot API
    - Added clickable Telegram username links
    - Learning: HTTP APIs, async requests, reqwest

12. **Step 5.3: Message Formatting**
    - Created formatter with Telegram/WhatsApp deep links
    - Format: "👤 Alice - last seen 45 days ago" with clickable name
    - Added WhatsApp support for friends without Telegram
    - Learning: String formatting, markdown, deep link protocols

### Phase 6: Integration & Testing

**Goal**: Make it production-ready locally

13. **Step 6.1: End-to-End Testing**
    - Test full flow locally
    - Add error handling throughout
    - Learning: Error propagation, debugging

14. **Step 6.2: Documentation**
    - Write README.md
    - Document setup process
    - Add code comments for learning

### Phase 7: Smart Reminder Enhancements

**15. Smart Reminder Logic Enhancements**

- Future meeting check: Skip reminders if meeting already scheduled
- Early reminder threshold: 15% buffer (e.g., 30 days → remind at day 25)
- Filter recurring events: Skip birthdays/anniversaries
- Friend name aliases: Support multiple names per friend
- Do Not Disturb mode: Calendar-based automatic pause
- Learning: Pattern matching, iterator chaining, zero-config design

### Phase 8: Production Deployment

**16. GitHub Actions Automated Deployment**

- Create `.github/workflows/daily-check.yml` with timezone-aware scheduling
- Set up GitHub secrets for credentials
- Schedule: Weekdays 8:00 AM, Weekends 9:30 AM Berlin time
- Learning: CI/CD, secrets management, cron expressions, GitHub Actions

### Phase 9: Firebase Integration & Web UI

**17. Snooze Functionality with Firebase**

- Set up Firebase project and Firestore database
- Create `snoozes` collection in Firestore
- Implement snooze logic with fail-open design
- Add inline snooze buttons to Telegram messages
- Create Cloud Function webhook for button callbacks
- Learning: Firestore SDK, state management, Cloud Functions, TypeScript

**18. Firestore Configuration with YAML Fallback**

- Create `friends` collection in Firestore
- Update `config.rs` with `load_from_firestore()` method
- Implement automatic source selection with fallback
- Learning: Cloud-based configuration, graceful degradation patterns

**19. Web UI for Friend Management**

- Build single-page web app in `docs/index.html`
- Implement full CRUD operations (add, edit, delete friends)
- Add mobile-responsive design
- Set up Google Sign-In authentication via Firebase Auth
- Deploy to GitHub Pages
- Learning: Firebase JS SDK, web authentication, responsive CSS, GitHub Pages

## Current Status

**✅ Production Ready - Fully Automated with Web UI & Firebase**

All phases (1-9) complete! MateCheck is deployed with automated reminders, Firestore-backed state management, and a mobile-responsive web UI for friends configuration.

### Completed Features

- ✅ Google Calendar integration (OAuth 2.0)
- ✅ Smart event matching (email + title + aliases)
- ✅ All-day and timed event support
- ✅ Recurring event filtering (birthdays excluded)
- ✅ Automatic 15% early reminder buffer
- ✅ Future meeting awareness
- ✅ Do Not Disturb mode (calendar-based, automatic pause)
- ✅ Telegram notifications with clickable links
- ✅ WhatsApp link support
- ✅ GitHub Actions automation (weekdays 8am, weekends 9:30am Berlin time)
- ✅ Snooze functionality (Firebase + Cloud Functions + Telegram inline buttons)
- ✅ Firestore configuration with automatic YAML fallback
- ✅ Web UI for friends management
- ✅ Production-ready README with full documentation
