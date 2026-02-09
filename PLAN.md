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
│   ├── config.rs               # Load friends config
│   ├── calendar/
│   │   ├── mod.rs              # Module declaration
│   │   ├── client.rs           # Google Calendar API client
│   │   └── types.rs            # Calendar event types
│   ├── matcher.rs              # Logic to match events to friends
│   ├── reminder/
│   │   ├── mod.rs
│   │   └── engine.rs           # Determine who needs reminders
│   └── telegram/
│       ├── mod.rs
│       └── client.rs           # Telegram bot client
├── Cargo.toml                  # Dependencies
├── .gitignore                  # Exclude sensitive files
├── friends.example.yaml        # Example config (checked in)
├── friends.yaml                # Actual config (gitignored)
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

**a) Future Meeting Check**

- Check if meeting already scheduled within frequency window
- **Logic**: If friend has event scheduled in next N days (where N = frequency_days), skip reminder
- **Example**:
  - Matilda: frequency = 10 days
  - Last meeting: 9 days ago (almost overdue)
  - Future meeting: scheduled in 2 days
  - Result: No reminder (total gap = 11 days, within acceptable range)
- **Note**: This means max gap could be 2N days, but that's intentional and user-controlled
- **Implementation**: Fetch future events separately, check in reminder engine

**b) Early Reminder Threshold**

- Send reminders BEFORE hitting the target, not after
- **Goal**: Give time to schedule meeting before going overdue
- **Logic**: Automatic 15% buffer calculation
- **Implementation**: `buffer = round(frequency * 0.15).max(1)`
- **Results**:
  - 10 days → 2 day buffer (remind at day 8)
  - 30 days → 5 day buffer (remind at day 25)
  - 45 days → 7 day buffer (remind at day 38)
- **Configuration**: Zero-config! Automatically calculated, no setup needed
- **Learning**: Sub-linear scaling, automatic calculation, zero-config UX design

**c) Filter Recurring Events**

- Skip recurring events (birthdays, anniversaries) when matching
- **Why**: Recurring events don't represent actual plans to meet
- **Implementation**: Filter in `calendar/client.rs` during event fetching
- **Logic**: Skip events where `recurring_event_id.is_some()`
- **Result**: Recurring events simply don't exist in our internal Event list
- **Code**: Added `.filter(|e| e.recurring_event_id.is_none())` before conversion
- **Learning**: Iterator chaining, Google Calendar API recurring event detection

**d) Friend Name Aliases**

- Support multiple names/aliases for the same friend
- **Why**: Friends may be called by different names (Lou/Louise, Mike/Michael, nicknames)
- **Example**:
  ```yaml
  friends:
    - id: "louise"
      name: "Louise"
      aliases: ["Lou", "Loulou"] # Don't repeat main name
      email: "louise@example.com"
      frequency_days: 14
  ```
- **Implementation**:
  - Added optional `aliases: Vec<String>` field to Friend struct with `#[serde(default)]`
  - Updated `match_by_title()` in `matcher.rs` to check name OR any alias (case-insensitive)
  - Added tests for alias matching (all 48 tests passing)
- **Use Case**: Calendar event "Coffee with Lou" now matches friend "Louise"
- **Learning**: Default field values with serde, iterator methods

**e) Do Not Disturb Mode**

- Automatically pause ALL friend reminders during specific periods
- **Why**: Need breaks during vacations, deep work, or personal time without manual intervention
- **How It Works**:
  - Create all-day calendar events with 🔕 emoji OR [DND] text (case-insensitive)
  - Examples: "🔕 Vacation in Paris", "[DND] Focus Week", "[dnd] personal time"
  - Only all-day events count (timed events like "🔕 Meeting" are ignored)
  - Works with single-day and multi-day events
- **Implementation**:
  - Added `is_all_day: bool` field to Event struct for explicit event type tracking
  - Updated `convert_event()` in `calendar/client.rs` to detect all-day vs timed events
  - Created new `calendar/dnd.rs` module with DND detection logic
  - Added DND check in `main.rs` after event fetching, before reminder calculation
  - Early exit with appropriate debug/user messages when DND is active
  - Comprehensive test suite: 27 tests covering all DND scenarios and edge cases
  - Updated all existing tests to include new `is_all_day` field (75 tests total)
- **Result**: Calendar-based, stateless DND that requires no configuration or database
- **Learning**: Pattern matching with filters, Option handling, early return patterns

**16. Other Future Ideas**

- Firebase integration for state persistence
- Multiple calendar support
- Configurable reminder messages
- Web dashboard

### Phase 8: Production Deployment

**17. GitHub Actions Automated Deployment**

- Created `.github/workflows/daily-check.yml` with timezone-aware scheduling
- Set up GitHub secrets for credentials (Google, Telegram, friends config)
- Schedule: Weekdays 8:00 AM, Weekends 9:30 AM Berlin time
- Tested workflow manually - working perfectly
- Runs automatically daily
- Learning: CI/CD, secrets management, cron expressions, GitHub Actions

### Phase 9: Firebase Integration & Web UI

**Goal**: Add state persistence for snooze functionality and enable online config editing

**18. Phase 9a: Snooze Functionality with Firebase**

- Set up Firebase project and Firestore database
- Add Firebase/Firestore Rust crate dependencies
- Create `snoozes` collection in Firestore
- Implement snooze logic:
  - Store: `snoozes/{friend_id} → { snoozed_until: date }`
  - Query: Get all active snoozes (where snoozed_until > today)
  - Check: Skip reminder if friend is currently snoozed
- Add Firebase credentials to GitHub Actions secrets
- Test snooze functionality locally and in GitHub Actions
- Learning: Firestore SDK, state management, date comparisons

**19. Phase 9b: Move Friends Config to Firebase**

- Migrate friends.yaml structure to Firestore
- Create `friends` collection: `friends/{friend_id} → { name, email, telegram_username, whatsapp_phone, aliases, frequency_days }`
- Update config loading to read from Firestore instead of YAML
- Keep friends.example.yaml for documentation
- Edit friends via Firebase Console (requires Google account login)
- Remove FRIENDS_CONFIG from GitHub secrets (read from Firestore instead)
- Learning: Cloud-based configuration, Firestore queries

**20. Phase 9c: Web UI for Friend Management (Optional)**

- Build single-page web app for managing friends
- Features:
  - List all friends with their configs
  - Add/edit/delete friends via form
  - View last meeting date and snooze status
  - Snooze/unsnooze friends
- Security:
  - Google Sign-In authentication
  - Firestore security rules (only your email can access)
  - No public access to data
- Deploy to GitHub Pages (free hosting)
- Learning: Firebase JS SDK, web authentication, static site hosting

## Current Status

**✅ Production Ready - Fully Automated with Firebase State**

All core phases (1-8) and Phase 9a complete! MateCheck is deployed and running automatically via GitHub Actions with Firebase-powered snooze functionality.

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
- ✅ **Phase 9a: Snooze functionality** (Firebase + Cloud Functions + Telegram inline buttons)
  - Firestore for state persistence
  - Inline snooze buttons in Telegram (3d/1w/2w)
  - Firebase Cloud Function webhook for button handling
  - Fail-open design (degrades gracefully if Firebase unavailable)
- ✅ Production-ready README

### Next Up: Phase 9b & 9c - Firebase Config & Web UI (Optional)

- 📋 **Phase 9b**: Move friends config to Firebase (edit via console instead of YAML)
- 🌐 **Phase 9c**: Optional web UI for easy editing

### Session Info

- **Last Updated**: 2026-02-09
- **Total Commits**: 11+
- **Test Suite**: 79 passing tests (Rust) + TypeScript webhook
- **Lines of Code**: ~2500 (src/ + functions/)
- **Learning Progress**: Completed Rust fundamentals, async/await, OAuth, CI/CD, Firebase, Cloud Functions, TypeScript

## Notes & Decisions

- **Public Repo**: friends.yaml and sensitive data must be gitignored
- **Stateless v1**: No persistence initially, recalculate from calendar each run
- **Day Granularity**: No need for hour/minute precision
