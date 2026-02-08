use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a calendar event from Google Calendar
///
/// This is a simplified version - we only extract the fields we need
/// for matching events to friends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event title/summary (e.g., "Coffee with Alice")
    pub title: String,

    /// List of attendee email addresses
    pub attendees: Vec<String>,

    /// When the event starts
    pub start: DateTime<Utc>,

    /// When the event ends (optional - some events are all-day)
    pub end: Option<DateTime<Utc>>,

    /// Whether this is an all-day event (vs a timed event)
    pub is_all_day: bool,
}

impl Event {
    /// Checks if this event has any attendees
    pub fn has_attendees(&self) -> bool {
        !self.attendees.is_empty()
    }

    /// Checks if a specific email is in the attendees list
    pub fn has_attendee(&self, email: &str) -> bool {
        self.attendees.iter().any(|a| a == email)
    }
}

// Tests to help you verify your implementation!
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_event_creation() {
        let start = Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 15, 11, 0, 0).unwrap();

        let event = Event {
            title: "Coffee with Alice".to_string(),
            attendees: vec!["alice@example.com".to_string()],
            start,
            end: Some(end),
            is_all_day: false,
        };

        assert_eq!(event.title, "Coffee with Alice");
        assert_eq!(event.attendees.len(), 1);
    }

    #[test]
    fn test_has_attendees() {
        let start = Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap();

        let event_with = Event {
            title: "Meeting".to_string(),
            attendees: vec!["alice@example.com".to_string()],
            start,
            end: None,
            is_all_day: false,
        };

        let event_without = Event {
            title: "Solo work".to_string(),
            attendees: vec![],
            start,
            end: None,
            is_all_day: false,
        };

        assert!(event_with.has_attendees());
        assert!(!event_without.has_attendees());
    }

    #[test]
    fn test_has_attendee() {
        let start = Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap();

        let event = Event {
            title: "Team meeting".to_string(),
            attendees: vec![
                "alice@example.com".to_string(),
                "bob@example.com".to_string(),
            ],
            start,
            end: None,
            is_all_day: false,
        };

        assert!(event.has_attendee("alice@example.com"));
        assert!(event.has_attendee("bob@example.com"));
        assert!(!event.has_attendee("charlie@example.com"));
    }
}
