use crate::reminder::ReminderInfo;

/// Formats a reminder message for Telegram
///
/// Creates a human-readable message listing friends who need to be contacted,
/// including how long since last meeting and clickable Telegram links.
///
/// # Arguments
/// * `reminders` - List of friends needing reminders with metadata
///
/// # Returns
/// A formatted Markdown message ready to send via Telegram
pub fn format_reminder_message(reminders: &[ReminderInfo]) -> String {
    let mut message = String::from("🔔 Time to reach out to your friends!\n\n");

    for reminder in reminders {
        // Format the "last seen" part
        let last_seen = match reminder.days_since_last_meeting {
            Some(days) => format!(
                "last seen {} days ago ({} days overdue)",
                days, reminder.days_overdue
            ),
            None => "never met".to_string(),
        };

        // Make the friend's name a clickable link if they have Telegram
        let friend_name = if let Some(ref username) = reminder.friend.telegram_username {
            format!("[{}](https://t.me/{})", reminder.friend.name, username)
        } else {
            reminder.friend.name.clone()
        };

        // Single line per friend
        message.push_str(&format!("👤 {} - {}\n", friend_name, last_seen));
    }

    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Friend;
    use crate::reminder::ReminderInfo;

    #[test]
    fn test_format_single_reminder() {
        let friend = Friend {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: Some("alice@example.com".to_string()),
            telegram_username: Some("alice_tg".to_string()),
            frequency_days: 30,
        };

        let reminder = ReminderInfo {
            friend,
            days_since_last_meeting: Some(45),
            days_overdue: 15,
        };

        let message = format_reminder_message(&[reminder]);

        assert!(message.contains("Alice"));
        assert!(message.contains("45 days"));
        assert!(message.contains("alice_tg"));
    }

    #[test]
    fn test_format_never_met_friend() {
        let friend = Friend {
            id: "bob".to_string(),
            name: "Bob".to_string(),
            email: None,
            telegram_username: Some("bob_tg".to_string()),
            frequency_days: 30,
        };

        let reminder = ReminderInfo {
            friend,
            days_since_last_meeting: None,
            days_overdue: 30,
        };

        let message = format_reminder_message(&[reminder]);

        assert!(message.contains("Bob"));
        assert!(message.contains("never met") || message.contains("no meetings"));
    }

    #[test]
    fn test_format_multiple_reminders() {
        let friend1 = Friend {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            telegram_username: Some("alice_tg".to_string()),
            frequency_days: 30,
        };

        let friend2 = Friend {
            id: "bob".to_string(),
            name: "Bob".to_string(),
            email: None,
            telegram_username: None,
            frequency_days: 14,
        };

        let reminders = vec![
            ReminderInfo {
                friend: friend1,
                days_since_last_meeting: Some(45),
                days_overdue: 15,
            },
            ReminderInfo {
                friend: friend2,
                days_since_last_meeting: Some(20),
                days_overdue: 6,
            },
        ];

        let message = format_reminder_message(&reminders);

        assert!(message.contains("Alice"));
        assert!(message.contains("Bob"));
        assert!(message.contains("45 days"));
        assert!(message.contains("20 days"));
    }
}
