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
  - id: "alice"                      # Unique identifier (required)
    name: "Alice"
    email: "alice@example.com"       # Optional - for calendar matching
    telegram_username: "alice_tg"    # Optional - for sending reminders
    frequency_days: 30

  - id: "bob_jones"
    name: "Bob Jones"
    email: ~                         # Optional - can be omitted
    telegram_username: ~             # Optional - can be omitted
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

7. **Step 3.1: Matcher Module** ✅ COMPLETE
   - Created `src/matcher.rs` (flat structure, not nested modules)
   - Implemented email-based matching (handles optional emails)
   - Implemented title-based matching (case-insensitive fallback)
   - Optimized `find_matches()` to skip email checks when no attendees
   - Refactored `Friend` struct: added `id` field, made email/telegram optional
   - Added ID uniqueness validation in `Config::load()`
   - Learning: Option types, HashSet, pattern matching, optimization

8. **Step 3.2: Last Meeting Tracker** ✅ COMPLETE
   - Implemented `find_last_meetings()` - tracks most recent event per friend
   - Used friend ID as HashMap key (handles optional emails correctly)
   - Returns `HashMap<String, Option<Event>>` (key = friend.id)
   - Implemented `days_since()` - calculates days between date and now
   - Implemented `days_since_last_meeting()` - convenience wrapper
   - All tests passing (17 tests total in matcher module)
   - Learning: HashMap operations, Option<&T> vs &Option<T>, mutable references, pattern matching

### Phase 4: Reminder Logic
**Goal**: Determine who needs reminders

9. **Step 4.1: Reminder Engine** ✅ COMPLETE
   - Created `src/reminder/mod.rs` and `engine.rs`
   - Implemented `find_friends_needing_reminders()` with business logic
   - Returns `Vec<ReminderInfo>` with friend, days_since, and days_overdue
   - Handles friends never met (days_since = None, always remind)
   - Handles friends overdue (days_since > frequency_days)
   - All tests passing (5 tests in reminder engine)
   - Learning: Pattern matching with guards, HashMap lookups, business logic, struct construction

### Phase 5: Telegram Integration
**Goal**: Send notifications

10. **Step 5.1: Telegram Bot Setup** 🚧 NEXT
    - Create bot via BotFather
    - Store bot token (local + GitHub secret)
    - Learning: External service integration

11. **Step 5.2: Telegram Client**
    - Create `src/telegram/mod.rs` and `client.rs`
    - Implement message sending
    - Format reminder message with clickable links
    - Learning: HTTP APIs, string formatting

12. **Step 5.3: Message Formatting**
    - Create list of friends with Telegram deep links
    - Format: "👤 Alice - last seen 45 days ago [Message](tg://resolve?domain=alice_tg)"
    - Learning: String formatting, markdown

### Phase 6: Integration & Testing ✅ COMPLETE
**Goal**: Make it production-ready locally

13. **Step 6.1: End-to-End Testing** ✅
    - Test full flow locally
    - Add error handling throughout
    - Learning: Error propagation, debugging

14. **Step 6.2: Documentation**
    - Write README.md
    - Document setup process
    - Add code comments for learning

### Phase 7: Smart Reminder Enhancements

**15. Smart Reminder Logic Enhancements**

**a) Future Meeting Check** ✅ COMPLETE
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
- **Logic**: Remind at (frequency_days - buffer_days)
- **Example**:
  - Matilda: frequency = 10 days, buffer = 2 days
  - Last meeting: 8 days ago
  - Result: Send reminder now (before hitting 10 days)
- **Configuration**: Add to friends.yaml or global settings
  ```yaml
  settings:
    reminder_buffer_days: 2  # Default buffer

  friends:
    - id: "matilda"
      frequency_days: 10
      # Uses default buffer of 2 days
  ```

**c) Filter Recurring Events**
- Skip recurring events (birthdays, anniversaries) when matching
- **Why**: Recurring events don't represent actual plans to meet
- **Implementation**: Filter in `calendar/client.rs` during API conversion
- **Logic**: Skip events where `recurring_event_id.is_some()`
- **Result**: Recurring events simply don't exist in our internal Event list

**d) Friend Name Aliases**
- Support multiple names/aliases for the same friend
- **Why**: Friends may be called by different names (Lou/Louise, Mike/Michael, nicknames)
- **Example**:
  ```yaml
  friends:
    - id: "louise"
      name: "Louise"
      aliases: ["Lou", "Loulou"]
      email: "louise@example.com"
      frequency_days: 14
  ```
- **Implementation**:
  - Add optional `aliases: Vec<String>` field to Friend struct
  - Update title matching in `matcher.rs` to check aliases
  - Match if event title contains friend name OR any alias
- **Use Case**: Calendar event "Coffee with Lou" matches friend "Louise"

**16. Other Future Ideas**
- Firebase integration for state persistence
- Multiple calendar support
- Configurable reminder messages
- Web dashboard

### Phase 8: Production Deployment (When Ready)

**17. GitHub Actions Automated Deployment**
- Create `.github/workflows/daily-check.yml`
- Set up secrets (Google OAuth token, Telegram token, chat ID)
- Schedule cron job (e.g., daily at 9 AM)
- Test workflow manually before enabling schedule
- Learning: CI/CD, secrets management, cron expressions
- **Note**: Only do this when you're confident the app works reliably

## Rust Concepts to Learn (Mapped from Go)

| Rust Concept | Go Equivalent | When We'll Learn It |
|--------------|---------------|---------------------|
| `Result<T, E>` | `error` return value | Step 1.2 |
| `Option<T>` | Pointer nilability | Step 3.1 |
| Ownership & Borrowing | Explicit copying/references | Step 1.2 |
| Traits | Interfaces | Step 1.2 |
| `async/await` | Goroutines/channels | Step 1.3 |
| Pattern matching | Switch statements | Step 3.1 |
| Iterators | Range loops | Step 3.2 |
| Modules | Packages | Step 2.1 |
| Cargo | go mod | Step 1.1 |
| Macros | (no direct equivalent) | Throughout |

## Dependencies (Cargo.toml)

```toml
[dependencies]
# Config & Serialization
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"

# Date/Time
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.8"

# Async Runtime
tokio = { version = "1", features = ["full"] }

# HTTP Client
reqwest = { version = "0.11", features = ["json"] }

# OAuth2
oauth2 = "4.4"

# CLI
clap = { version = "4", features = ["derive"] }

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

# Google API
google-calendar3 = "5.0"
yup-oauth2 = "8.0"
```

## Current Status

- [x] Initial Cargo project created
- [ ] Step 1.1: Project setup and dependencies
- [ ] Step 1.2: Configuration module
- [ ] ... (to be updated as we progress)

## Notes & Decisions

- **Public Repo**: friends.yaml and sensitive data must be gitignored
- **Stateless v1**: No persistence initially, recalculate from calendar each run
- **Day Granularity**: No need for hour/minute precision
- **Telegram Links**: Use tg://resolve?domain=username format for deep links
