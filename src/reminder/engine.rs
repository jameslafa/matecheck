use std::i64;

use crate::calendar::types::Event;
use crate::config::{Config, Friend};
use crate::matcher;
use chrono::Utc;

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
/// - Reminds when `days_since >= (frequency_days - buffer_days)`
/// - Buffer is automatically calculated as 15% of frequency (proportional early warning)
/// - If never met (no events), always send reminder
/// - **SKIP** reminder if meeting already scheduled within frequency window
///
/// **Automatic buffer (15% of frequency):**
/// - frequency=10 → buffer=2 → remind at day 8
/// - frequency=30 → buffer=5 → remind at day 25
/// - frequency=45 → buffer=7 → remind at day 38
///
/// **Future meeting check:**
/// - If friend would get reminder BUT has meeting scheduled within frequency_days, skip
/// - Example: frequency=10, last_met=9 days ago, future_meeting=in 2 days → no reminder
pub fn find_friends_needing_reminders(events: &[Event], config: &Config) -> Vec<ReminderInfo> {
    let friends = &config.friends;
    let last_meetings_by_friend = matcher::find_last_meetings(events, friends);
    let next_meetings_by_friend = matcher::find_next_meetings(events, friends);
    let mut reminders = Vec::new();

    for friend in friends {
        let last_meeting = last_meetings_by_friend.get(&friend.id);
        let days_since = matcher::days_since_last_meeting(last_meeting.as_ref().unwrap());

        // Check if they have an upcoming meeting
        let has_upcoming_meeting = next_meetings_by_friend
            .get(&friend.id)
            .and_then(|opt_event| opt_event.as_ref())
            .and_then(|event| {
                let days_until = (event.start - Utc::now()).num_days();
                if days_until >= 0 && days_until <= friend.frequency_days as i64 {
                    Some(days_until)
                } else {
                    None
                }
            });

        // Calculate reminder threshold with automatic 15% buffer
        let buffer = friend.buffer_days();
        let threshold = friend.frequency_days.saturating_sub(buffer) as i64;

        // Determine if friend needs reminder based on automatic buffer
        let needs_reminder = match days_since {
            None => {
                // Friend never met - always needs reminder (unless meeting scheduled)
                has_upcoming_meeting.is_none()
            }
            Some(days) => {
                // Buffer is always > 0 (minimum 1), so remind at or after threshold
                // Example: freq=10, buffer=2 → threshold=8, remind at day 8+
                let should_remind = days >= threshold;
                should_remind && has_upcoming_meeting.is_none()
            }
        };

        if needs_reminder {
            match days_since {
                None => {
                    reminders.push(ReminderInfo {
                        friend: friend.clone(),
                        days_since_last_meeting: None,
                        days_overdue: i64::MAX,
                    });
                }
                Some(days) => {
                    let days_overdue = days - friend.frequency_days as i64;
                    reminders.push(ReminderInfo {
                        friend: friend.clone(),
                        days_since_last_meeting: Some(days),
                        days_overdue,
                    });
                }
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
            aliases: vec![],
            frequency_days,
        }
    }

    fn mock_config(friends: Vec<Friend>) -> Config {
        Config {
            friends,
        }
    }

    fn mock_event(title: &str, attendees: Vec<String>, days_ago: i64) -> Event {
        let start = Utc::now() - Duration::days(days_ago);
        Event { title: title.to_string(), attendees, start, end: None }
    }

    #[test]
    fn test_friend_overdue_needs_reminder() {
        let alice = mock_friend("alice", "Alice", 10); // wants to meet every 10 days
        let friends = vec![alice];

        // Last meeting was 15 days ago (overdue by 5 days)
        let event = mock_event("Coffee", vec!["alice@example.com".to_string()], 15);
        let events = vec![event];

        let reminders = find_friends_needing_reminders(&events, &mock_config(friends));

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

        let reminders = find_friends_needing_reminders(&events, &mock_config(friends));

        assert_eq!(reminders.len(), 0);
    }

    #[test]
    fn test_friend_never_met_needs_reminder() {
        let bob = mock_friend("bob", "Bob", 10);
        let friends = vec![bob];

        let events = vec![]; // No events

        let reminders = find_friends_needing_reminders(&events, &mock_config(friends));

        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].friend.id, "bob");
        assert!(reminders[0].days_since_last_meeting.is_none());
        // days_overdue should be very high (or i64::MAX)
        assert!(reminders[0].days_overdue > 1000);
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

        let reminders = find_friends_needing_reminders(&events, &mock_config(friends));

        assert_eq!(reminders.len(), 2); // Alice and Charlie
        let ids: Vec<&str> = reminders.iter().map(|r| r.friend.id.as_str()).collect();
        assert!(ids.contains(&"alice"));
        assert!(ids.contains(&"charlie"));
        assert!(!ids.contains(&"bob"));
    }

    #[test]
    fn test_overdue_but_has_upcoming_meeting_no_reminder() {
        let alice = mock_friend("alice", "Alice", 10);
        let friends = vec![alice];

        // Last meeting was 12 days ago (overdue by 2)
        let past_event = mock_event("Past coffee", vec!["alice@example.com".to_string()], 12);

        // But has a meeting scheduled in 3 days (within frequency window)
        let future_start = Utc::now() + Duration::days(3);
        let future_event = Event {
            title: "Upcoming lunch".to_string(),
            attendees: vec!["alice@example.com".to_string()],
            start: future_start,
            end: None,
        };

        let events = vec![past_event, future_event];

        let reminders = find_friends_needing_reminders(&events, &mock_config(friends));

        // Should NOT remind because meeting is already scheduled
        assert_eq!(reminders.len(), 0);
    }

    #[test]
    fn test_overdue_with_far_future_meeting_still_reminds() {
        let alice = mock_friend("alice", "Alice", 10);
        let friends = vec![alice];

        // Last meeting was 12 days ago (overdue by 2)
        let past_event = mock_event("Past coffee", vec!["alice@example.com".to_string()], 12);

        // Has a meeting scheduled in 20 days (beyond frequency window)
        let future_start = Utc::now() + Duration::days(20);
        let future_event = Event {
            title: "Far future lunch".to_string(),
            attendees: vec!["alice@example.com".to_string()],
            start: future_start,
            end: None,
        };

        let events = vec![past_event, future_event];

        let reminders = find_friends_needing_reminders(&events, &mock_config(friends));

        // SHOULD remind because future meeting is too far away
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].friend.id, "alice");
    }

    #[test]
    fn test_never_met_with_upcoming_meeting_no_reminder() {
        let alice = mock_friend("alice", "Alice", 10);
        let friends = vec![alice];

        // Never met before, but has upcoming meeting in 5 days
        let future_start = Utc::now() + Duration::days(5);
        let future_event = Event {
            title: "First meeting".to_string(),
            attendees: vec!["alice@example.com".to_string()],
            start: future_start,
            end: None,
        };

        let events = vec![future_event];

        let reminders = find_friends_needing_reminders(&events, &mock_config(friends));

        // Should NOT remind because meeting is scheduled
        assert_eq!(reminders.len(), 0);
    }

    #[test]
    fn test_automatic_buffer_15_percent() {
        // Test that buffer is automatically calculated as 15% of frequency
        let alice = mock_friend("alice", "Alice", 10);  // buffer = round(10 * 0.15) = 2
        let bob = mock_friend("bob", "Bob", 30);        // buffer = round(30 * 0.15) = 5
        let charlie = mock_friend("charlie", "Charlie", 45); // buffer = round(45 * 0.15) = 7

        assert_eq!(alice.buffer_days(), 2);
        assert_eq!(bob.buffer_days(), 5);
        assert_eq!(charlie.buffer_days(), 7);
    }

    #[test]
    fn test_early_reminder_with_automatic_buffer() {
        let alice = mock_friend("alice", "Alice", 10);  // buffer=2, threshold=8
        let friends = vec![alice];

        // Last meeting was 8 days ago (at threshold)
        let event = mock_event("Coffee", vec!["alice@example.com".to_string()], 8);
        let events = vec![event];

        let reminders = find_friends_needing_reminders(&events, &mock_config(friends));

        // SHOULD remind because days_since (8) >= threshold (10-2=8)
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].friend.id, "alice");
        assert_eq!(reminders[0].days_since_last_meeting, Some(8));
    }

    #[test]
    fn test_no_early_reminder_before_automatic_threshold() {
        let alice = mock_friend("alice", "Alice", 10);  // buffer=2, threshold=8
        let friends = vec![alice];

        // Last meeting was 7 days ago (before threshold)
        let event = mock_event("Coffee", vec!["alice@example.com".to_string()], 7);
        let events = vec![event];

        let reminders = find_friends_needing_reminders(&events, &mock_config(friends));

        // Should NOT remind (7 < 8)
        assert_eq!(reminders.len(), 0);
    }
}
