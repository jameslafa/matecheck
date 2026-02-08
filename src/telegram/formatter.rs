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

        // Make the friend's name a clickable link
        // Priority: Telegram username > WhatsApp > plain name
        let friend_name = if let Some(ref username) = reminder.friend.telegram_username {
            if !username.is_empty() {
                format!("[{}](https://t.me/{})", reminder.friend.name, username)
            } else if let Some(ref phone) = reminder.friend.whatsapp_phone {
                // Strip + and spaces from phone number for WhatsApp link
                let clean_phone = phone.replace('+', "").replace(' ', "");
                format!("[{}](https://wa.me/{})", reminder.friend.name, clean_phone)
            } else {
                reminder.friend.name.clone()
            }
        } else if let Some(ref phone) = reminder.friend.whatsapp_phone {
            // Strip + and spaces from phone number for WhatsApp link
            let clean_phone = phone.replace('+', "").replace(' ', "");
            format!("[{}](https://wa.me/{})", reminder.friend.name, clean_phone)
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
            whatsapp_phone: None,
            aliases: vec![],
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
            whatsapp_phone: None,
            aliases: vec![],
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
            whatsapp_phone: None,
            aliases: vec![],
            frequency_days: 30,
        };

        let friend2 = Friend {
            id: "bob".to_string(),
            name: "Bob".to_string(),
            email: None,
            telegram_username: None,
            whatsapp_phone: None,
            aliases: vec![],
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

    #[test]
    fn test_format_with_whatsapp() {
        let friend = Friend {
            id: "david".to_string(),
            name: "David".to_string(),
            email: None,
            telegram_username: None,
            whatsapp_phone: Some("4915734630875".to_string()),
            aliases: vec![],
            frequency_days: 30,
        };

        let reminder = ReminderInfo {
            friend,
            days_since_last_meeting: Some(35),
            days_overdue: 5,
        };

        let message = format_reminder_message(&[reminder]);

        // Should contain WhatsApp link
        assert!(message.contains("David"));
        assert!(message.contains("https://wa.me/4915734630875"));
        assert!(message.contains("35 days"));
    }

    #[test]
    fn test_empty_telegram_falls_back_to_whatsapp() {
        let friend = Friend {
            id: "annie".to_string(),
            name: "Annie".to_string(),
            email: Some("annie@example.com".to_string()),
            telegram_username: Some("".to_string()),  // Empty!
            whatsapp_phone: Some("4915734630875".to_string()),
            aliases: vec![],
            frequency_days: 60,
        };

        let reminder = ReminderInfo {
            friend,
            days_since_last_meeting: Some(65),
            days_overdue: 5,
        };

        let message = format_reminder_message(&[reminder]);

        // Should fall back to WhatsApp
        assert!(message.contains("Annie"));
        assert!(message.contains("https://wa.me/4915734630875"));
        assert!(!message.contains("t.me/")); // No broken Telegram link
    }

    #[test]
    fn test_whatsapp_strips_plus_and_spaces() {
        let friend = Friend {
            id: "sarah".to_string(),
            name: "Sarah".to_string(),
            email: None,
            telegram_username: None,
            whatsapp_phone: Some("+49 157 3463 0875".to_string()),  // With + and spaces
            aliases: vec![],
            frequency_days: 30,
        };

        let reminder = ReminderInfo {
            friend,
            days_since_last_meeting: Some(40),
            days_overdue: 10,
        };

        let message = format_reminder_message(&[reminder]);

        // Should strip + and spaces
        assert!(message.contains("Sarah"));
        assert!(message.contains("https://wa.me/4915734630875"));
        assert!(!message.contains("+")); // No + in the link
        assert!(!message.contains(" 157")); // No spaces
    }
}
