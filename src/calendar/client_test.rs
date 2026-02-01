use super::GoogleCalendarClient;
use crate::calendar::types::Event;
use chrono::{DateTime, TimeZone, Utc};
use google_calendar3::api::{Event as GoogleEvent, EventAttendee, EventDateTime};
use std::fs;

/// Helper function to create a mock Google Calendar event
fn mock_google_event(
    title: Option<String>,
    attendee_emails: Vec<String>,
    start: DateTime<Utc>,
    end: Option<DateTime<Utc>>,
) -> GoogleEvent {
    let attendees = if attendee_emails.is_empty() {
        None
    } else {
        Some(
            attendee_emails
                .into_iter()
                .map(|email| EventAttendee {
                    email: Some(email),
                    ..Default::default()
                })
                .collect(),
        )
    };

    GoogleEvent {
        summary: title,
        attendees,
        start: Some(EventDateTime {
            date_time: Some(start),
            ..Default::default()
        }),
        end: end.map(|e| EventDateTime {
            date_time: Some(e),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Loads calendar events from fixture file
    ///
    /// Tries to load from calendar_events.json (real data) first,
    /// falls back to calendar_events.example.json (sanitized example data)
    fn load_fixture_events() -> Vec<Event> {
        let real_fixture = "tests/fixtures/calendar_events.json";
        let example_fixture = "tests/fixtures/calendar_events.example.json";

        let json = fs::read_to_string(real_fixture)
            .or_else(|_| fs::read_to_string(example_fixture))
            .expect("No fixture found. Run: cargo run --bin record_calendar_response");

        serde_json::from_str(&json).expect("Failed to parse fixture JSON")
    }

    #[test]
    fn test_fixture_loads() {
        let events = load_fixture_events();
        assert!(!events.is_empty(), "Fixture should contain events");

        // Verify first event has expected structure
        let first = &events[0];
        assert!(!first.title.is_empty());
        // Should have either attendees or be a solo event
        // Just verify the structure is valid
    }

    #[test]
    fn test_fixture_event_structure() {
        let events = load_fixture_events();

        for event in &events {
            // All events must have a title
            assert!(!event.title.is_empty());

            // All events must have a start time
            // (this is guaranteed by our Event type)

            // Attendees can be empty (solo events)
            // End can be None (all-day events)
        }
    }

    #[test]
    fn test_convert_event_with_attendees() {
        let start = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 15, 11, 0, 0).unwrap();

        let google_event = mock_google_event(
            Some("Coffee with Alice".to_string()),
            vec!["alice@example.com".to_string(), "bob@example.com".to_string()],
            start,
            Some(end),
        );

        let result = GoogleCalendarClient::convert_event(&google_event);

        assert!(result.is_ok());
        let event = result.unwrap();

        assert_eq!(event.title, "Coffee with Alice");
        assert_eq!(event.attendees.len(), 2);
        assert!(event.attendees.contains(&"alice@example.com".to_string()));
        assert!(event.attendees.contains(&"bob@example.com".to_string()));
        assert_eq!(event.start, start);
        assert_eq!(event.end, Some(end));
    }

    #[test]
    fn test_convert_event_without_attendees() {
        let start = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let google_event = mock_google_event(
            Some("Solo work".to_string()),
            vec![],
            start,
            None,
        );

        let result = GoogleCalendarClient::convert_event(&google_event);

        assert!(result.is_ok());
        let event = result.unwrap();

        assert_eq!(event.title, "Solo work");
        assert_eq!(event.attendees.len(), 0);
        assert_eq!(event.end, None);
    }

    #[test]
    fn test_convert_event_without_title() {
        let start = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let google_event = mock_google_event(
            None,  // No title
            vec!["alice@example.com".to_string()],
            start,
            None,
        );

        let result = GoogleCalendarClient::convert_event(&google_event);

        assert!(result.is_ok());
        let event = result.unwrap();

        // Should use default title
        assert_eq!(event.title, "Untitled event");
    }

    #[test]
    fn test_convert_event_missing_start_time() {
        // Event with no start time should fail
        let google_event = GoogleEvent {
            summary: Some("Invalid event".to_string()),
            start: None,  // Missing start!
            ..Default::default()
        };

        let result = GoogleCalendarClient::convert_event(&google_event);

        // Should return an error
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_event_with_empty_email_in_attendees() {
        let start = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        // Create event with one valid email and one None
        let google_event = GoogleEvent {
            summary: Some("Meeting".to_string()),
            attendees: Some(vec![
                EventAttendee {
                    email: Some("alice@example.com".to_string()),
                    ..Default::default()
                },
                EventAttendee {
                    email: None,  // No email
                    ..Default::default()
                },
            ]),
            start: Some(EventDateTime {
                date_time: Some(start),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = GoogleCalendarClient::convert_event(&google_event);

        assert!(result.is_ok());
        let event = result.unwrap();

        // Should filter out the None email
        assert_eq!(event.attendees.len(), 1);
        assert_eq!(event.attendees[0], "alice@example.com");
    }
}
