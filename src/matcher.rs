use crate::calendar::types::Event;
use crate::config::Friend;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Matches calendar events with friends from config
/// Uses two strategies: email matching and title matching

/// Check if a friend attended an event by matching their email with attendees
///
/// Returns false if the friend has no email configured
pub fn match_by_email(event: &Event, friend: &Friend) -> bool {
    friend
        .email
        .as_ref()
        .map_or(false, |email| event.has_attendee(email))
}

/// Check if a friend is mentioned in an event title (case-insensitive)
///
/// Example: "Coffee with Alice" matches friend named "Alice"
pub fn match_by_title(event: &Event, friend: &Friend) -> bool {
    event
        .title
        .to_lowercase()
        .contains(&friend.name.to_lowercase())
}

/// Find all friends who match an event (either by email or title)
pub fn find_matches<'a>(event: &Event, friends: &'a [Friend]) -> Vec<&'a Friend> {
    let has_attendees = event.has_attendees();
    friends
        .iter()
        .filter(|f| (has_attendees && match_by_email(event, f)) || match_by_title(event, f))
        .collect()
}

/// Find the most recent event for each friend
///
/// Returns a HashMap where:
/// - Key: friend.id
/// - Value: Option<Event> - Some(event) if any meeting was found, None if no meetings
///
/// Events are matched to friends using both email and title matching.
/// When multiple events match a friend, only the most recent is kept.
pub fn find_last_meetings(events: &[Event], friends: &[Friend]) -> HashMap<String, Option<Event>> {
    let mut last_event_by_friend: HashMap<String, Option<Event>> = HashMap::new();
    for friend in friends {
        last_event_by_friend.insert(friend.id.clone(), None);
    }
    for event in events {
        for matched_friend in find_matches(&event, friends) {
            let current = last_event_by_friend.get_mut(&matched_friend.id).unwrap();

            // Update if None or if this event is more recent
            if current.is_none() || event.start > current.as_ref().unwrap().start {
                *current = Some(event.clone());
            }
        }
    }
    last_event_by_friend
}

/// Calculate the number of days between a date and now
///
/// Returns None if the event is in the future.
pub fn days_since(event_time: DateTime<Utc>) -> Option<i64> {
    let time_difference = Utc::now() - event_time;
    let num_days = time_difference.num_days();
    if num_days < 0 {
        return None;
    }
    Some(num_days)
}

/// Calculate days since the last meeting with a friend
///
/// Returns None if the friend has no recorded meetings or if the meeting is in the future.
pub fn days_since_last_meeting(last_meeting: &Option<Event>) -> Option<i64> {
    match last_meeting {
        None => None,
        Some(event) => days_since(event.start),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn mock_friend(id: &str, name: &str, email: Option<&str>) -> Friend {
        Friend {
            id: id.to_string(),
            name: name.to_string(),
            email: email.map(|e| e.to_string()),
            telegram_username: Some("test".to_string()),
            frequency_days: 30,
        }
    }

    fn mock_event(title: &str, attendees: Vec<String>) -> Event {
        Event { title: title.to_string(), attendees, start: Utc::now(), end: None }
    }

    #[test]
    fn test_match_by_email() {
        let friend = mock_friend("alice", "Alice", Some("alice@example.com"));
        let event = mock_event("Meeting", vec!["alice@example.com".to_string()]);

        assert!(match_by_email(&event, &friend));
    }

    #[test]
    fn test_match_by_email_no_match() {
        let friend = mock_friend("alice", "Alice", Some("alice@example.com"));
        let event = mock_event("Meeting", vec!["bob@example.com".to_string()]);

        assert!(!match_by_email(&event, &friend));
    }

    #[test]
    fn test_match_by_email_when_friend_has_no_email() {
        let friend = mock_friend("alice", "Alice", None);
        let event = mock_event("Meeting", vec!["alice@example.com".to_string()]);

        // Should return false since friend has no email to match
        assert!(!match_by_email(&event, &friend));
    }

    #[test]
    fn test_match_by_title() {
        let friend = mock_friend("alice", "Alice", Some("alice@example.com"));
        let event = mock_event("Coffee with Alice", vec![]);

        assert!(match_by_title(&event, &friend));
    }

    #[test]
    fn test_match_by_title_case_insensitive() {
        let friend = mock_friend("alice", "Alice", Some("alice@example.com"));
        let event = mock_event("coffee with ALICE", vec![]);

        assert!(match_by_title(&event, &friend));
    }

    #[test]
    fn test_match_by_title_no_match() {
        let friend = mock_friend("alice", "Alice", Some("alice@example.com"));
        let event = mock_event("Meeting with Bob", vec![]);

        assert!(!match_by_title(&event, &friend));
    }

    #[test]
    fn test_find_matches() {
        let alice = mock_friend("alice", "Alice", Some("alice@example.com"));
        let bob = mock_friend("bob", "Bob", Some("bob@example.com"));
        let charlie = mock_friend("charlie", "Charlie", Some("charlie@example.com"));
        let friends = vec![alice, bob, charlie];

        let event = mock_event(
            "Meeting",
            vec![
                "alice@example.com".to_string(),
                "bob@example.com".to_string(),
            ],
        );

        let matches = find_matches(&event, &friends);

        assert_eq!(matches.len(), 2);
        assert!(matches.iter().any(|f| f.name == "Alice"));
        assert!(matches.iter().any(|f| f.name == "Bob"));
    }

    #[test]
    fn test_find_matches_by_title() {
        let alice = mock_friend("alice", "Alice", Some("alice@example.com"));
        let bob = mock_friend("bob", "Bob", Some("bob@example.com"));
        let friends = vec![alice, bob];

        let event = mock_event("Lunch with Alice", vec![]);

        let matches = find_matches(&event, &friends);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "Alice");
    }

    #[test]
    fn test_find_matches_no_match() {
        let alice = mock_friend("alice", "Alice", Some("alice@example.com"));
        let friends = vec![alice];

        let event = mock_event("Meeting with Bob", vec!["bob@example.com".to_string()]);

        let matches = find_matches(&event, &friends);

        assert!(matches.is_empty());
    }

    #[test]
    fn test_days_since_past_event() {
        use chrono::Duration;

        // Create a timestamp from 10 days ago
        let ten_days_ago = Utc::now() - Duration::days(10);

        let days = days_since(ten_days_ago);

        // Should be around 10 days (within 1 day for test timing tolerance)
        assert!(days.is_some());
        let days = days.unwrap();
        assert!(days >= 9 && days <= 11, "Expected ~10 days, got {}", days);
    }

    #[test]
    fn test_days_since_future_event() {
        use chrono::Duration;

        // Create a timestamp 5 days in the future
        let future = Utc::now() + Duration::days(5);

        let days = days_since(future);

        // Should return None for future events
        assert!(days.is_none());
    }

    #[test]
    fn test_days_since_last_meeting_with_event() {
        use chrono::Duration;

        let past = Utc::now() - Duration::days(7);
        let event = mock_event_at("Meeting", vec![], past, None);

        let days = days_since_last_meeting(&Some(event));

        assert!(days.is_some());
        let days = days.unwrap();
        assert!(days >= 6 && days <= 8, "Expected ~7 days, got {}", days);
    }

    #[test]
    fn test_days_since_last_meeting_no_event() {
        let days = days_since_last_meeting(&None);

        // Should return None when there's no meeting
        assert!(days.is_none());
    }

    #[test]
    fn test_find_last_meetings_single_friend_single_event() {
        use chrono::Duration;

        let alice = mock_friend("alice", "Alice", Some("alice@example.com"));
        let friends = vec![alice];

        let past = Utc::now() - Duration::days(5);
        let event = mock_event_at("Meeting", vec!["alice@example.com".to_string()], past, None);
        let events = vec![event];

        let last_meetings = find_last_meetings(&events, &friends);

        assert_eq!(last_meetings.len(), 1);
        assert!(last_meetings.contains_key("alice"));
        assert!(last_meetings.get("alice").unwrap().is_some());
    }

    #[test]
    fn test_find_last_meetings_multiple_events_picks_most_recent() {
        use chrono::Duration;

        let alice = mock_friend("alice", "Alice", Some("alice@example.com"));
        let friends = vec![alice];

        let old_event = mock_event_at(
            "Old meeting",
            vec!["alice@example.com".to_string()],
            Utc::now() - Duration::days(30),
            None,
        );
        let recent_event = mock_event_at(
            "Recent meeting",
            vec!["alice@example.com".to_string()],
            Utc::now() - Duration::days(5),
            None,
        );

        // Events in random order to test comparison logic
        let events = vec![old_event, recent_event.clone()];

        let last_meetings = find_last_meetings(&events, &friends);

        let alice_meeting = last_meetings.get("alice").unwrap().as_ref().unwrap();
        assert_eq!(alice_meeting.title, "Recent meeting");
    }

    #[test]
    fn test_find_last_meetings_friend_with_no_events() {
        use chrono::Duration;

        let alice = mock_friend("alice", "Alice", Some("alice@example.com"));
        let bob = mock_friend("bob", "Bob", Some("bob@example.com"));
        let friends = vec![alice, bob];

        // Only Alice has a meeting
        let past = Utc::now() - Duration::days(5);
        let event = mock_event_at(
            "Alice meeting",
            vec!["alice@example.com".to_string()],
            past,
            None,
        );
        let events = vec![event];

        let last_meetings = find_last_meetings(&events, &friends);

        // Both friends should be in the map
        assert_eq!(last_meetings.len(), 2);
        assert!(last_meetings.get("alice").unwrap().is_some());
        assert!(last_meetings.get("bob").unwrap().is_none()); // Bob has no meetings
    }

    #[test]
    fn test_find_last_meetings_title_matching() {
        use chrono::Duration;

        // Friend without email - relies on title matching
        let alice = mock_friend("alice", "Alice", None);
        let friends = vec![alice];

        let event = mock_event_at(
            "Lunch with Alice",
            vec![],
            Utc::now() - Duration::days(3),
            None,
        );
        let events = vec![event];

        let last_meetings = find_last_meetings(&events, &friends);

        assert!(last_meetings.get("alice").unwrap().is_some());
    }

    // Helper for creating events with specific timestamp
    fn mock_event_at(
        title: &str,
        attendees: Vec<String>,
        start: DateTime<Utc>,
        end: Option<DateTime<Utc>>,
    ) -> Event {
        Event { title: title.to_string(), attendees, start, end }
    }
}
