use std::i64;

use crate::calendar::types::Event;
use crate::config::Friend;
use crate::matcher;

/// Information about a friend who needs a reminder
#[derive(Debug, Clone)]
pub struct ReminderInfo {
    /// The friend who needs a reminder
    pub friend: Friend,

    /// How many days since the last meeting (None if never met)
    pub days_since_last_meeting: Option<i64>,

    /// How many days overdue (negative if not yet due)
    /// Example: frequency=10, days_since=12 → overdue=2
    pub days_overdue: i64,
}

/// Find all friends who need reminders based on their meeting frequency
///
/// **Logic:**
/// - If `days_since_last_meeting > frequency_days`, the friend needs a reminder
/// - If never met (no events), always send reminder
///
/// **Future enhancements (Phase 7):**
/// - Early reminder threshold (remind before overdue)
/// - Future meeting check (skip if meeting already scheduled)
pub fn find_friends_needing_reminders(events: &[Event], friends: &[Friend]) -> Vec<ReminderInfo> {
    let last_meetings_by_friend = matcher::find_last_meetings(events, friends);
    let mut reminders = Vec::new();
    for friend in friends {
        let last_meeting = last_meetings_by_friend.get(&friend.id);
        let days_since = matcher::days_since_last_meeting(last_meeting.as_ref().unwrap());

        match days_since {
            None => {
                // Friend never met - always send reminder
                reminders.push(ReminderInfo {
                    friend: friend.clone(),
                    days_since_last_meeting: None,
                    days_overdue: i64::MAX,
                });
            }
            Some(days) if days > friend.frequency_days as i64 => {
                // Friend is overdue
                let days_overdue = days - friend.frequency_days as i64;
                reminders.push(ReminderInfo {
                    friend: friend.clone(),
                    days_since_last_meeting: Some(days),
                    days_overdue,
                });
            }
            Some(_) => {
                // Not overdue yet - no reminder needed
            }
        }
    }
    reminders
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn mock_friend(id: &str, name: &str, frequency_days: u32) -> Friend {
        Friend {
            id: id.to_string(),
            name: name.to_string(),
            email: Some(format!("{}@example.com", id)),
            telegram_username: Some(id.to_string()),
            frequency_days,
        }
    }

    fn mock_event(title: &str, attendees: Vec<String>, days_ago: i64) -> Event {
        let start = Utc::now() - Duration::days(days_ago);
        Event::new(title.to_string(), attendees, start, None)
    }

    #[test]
    fn test_friend_overdue_needs_reminder() {
        let alice = mock_friend("alice", "Alice", 10); // wants to meet every 10 days
        let friends = vec![alice];

        // Last meeting was 15 days ago (overdue by 5 days)
        let event = mock_event("Coffee", vec!["alice@example.com".to_string()], 15);
        let events = vec![event];

        let reminders = find_friends_needing_reminders(&events, &friends);

        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].friend.id, "alice");
        assert_eq!(reminders[0].days_since_last_meeting, Some(15));
        assert_eq!(reminders[0].days_overdue, 5);
    }

    #[test]
    fn test_friend_not_overdue_no_reminder() {
        let alice = mock_friend("alice", "Alice", 10);
        let friends = vec![alice];

        // Last meeting was 5 days ago (not overdue yet)
        let event = mock_event("Coffee", vec!["alice@example.com".to_string()], 5);
        let events = vec![event];

        let reminders = find_friends_needing_reminders(&events, &friends);

        assert_eq!(reminders.len(), 0);
    }

    #[test]
    fn test_friend_never_met_needs_reminder() {
        let bob = mock_friend("bob", "Bob", 10);
        let friends = vec![bob];

        let events = vec![]; // No events

        let reminders = find_friends_needing_reminders(&events, &friends);

        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].friend.id, "bob");
        assert!(reminders[0].days_since_last_meeting.is_none());
        // days_overdue should be very high (or i64::MAX)
        assert!(reminders[0].days_overdue > 1000);
    }

    #[test]
    fn test_exactly_at_frequency_no_reminder() {
        let alice = mock_friend("alice", "Alice", 10);
        let friends = vec![alice];

        // Last meeting was exactly 10 days ago (at threshold, not over)
        let event = mock_event("Coffee", vec!["alice@example.com".to_string()], 10);
        let events = vec![event];

        let reminders = find_friends_needing_reminders(&events, &friends);

        // Should NOT remind (> not >=)
        assert_eq!(reminders.len(), 0);
    }

    #[test]
    fn test_multiple_friends_mixed() {
        let alice = mock_friend("alice", "Alice", 10);
        let bob = mock_friend("bob", "Bob", 7);
        let charlie = mock_friend("charlie", "Charlie", 30);
        let friends = vec![alice, bob, charlie];

        let events = vec![
            mock_event("Alice meeting", vec!["alice@example.com".to_string()], 15), // overdue
            mock_event("Bob meeting", vec!["bob@example.com".to_string()], 5),      // not overdue
                                                                                    // Charlie never met
        ];

        let reminders = find_friends_needing_reminders(&events, &friends);

        assert_eq!(reminders.len(), 2); // Alice and Charlie
        let ids: Vec<&str> = reminders.iter().map(|r| r.friend.id.as_str()).collect();
        assert!(ids.contains(&"alice"));
        assert!(ids.contains(&"charlie"));
        assert!(!ids.contains(&"bob"));
    }
}
