use crate::calendar::types::Event;
use crate::config::Friend;

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
        Event::new(title.to_string(), attendees, Utc::now(), None)
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
}
