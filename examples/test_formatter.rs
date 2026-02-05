/// Quick example to show formatted reminder messages

use matecheck::config::Friend;
use matecheck::reminder::ReminderInfo;
use matecheck::telegram::format_reminder_message;

fn main() {
    let friend1 = Friend {
        id: "alice".to_string(),
        name: "Alice".to_string(),
        email: Some("alice@example.com".to_string()),
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

    let friend3 = Friend {
        id: "charlie".to_string(),
        name: "Charlie".to_string(),
        email: None,
        telegram_username: Some("charlie_tg".to_string()),
        frequency_days: 7,
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
        ReminderInfo {
            friend: friend3,
            days_since_last_meeting: None,
            days_overdue: 7,
        },
    ];

    let message = format_reminder_message(&reminders);
    println!("{}", message);
}
