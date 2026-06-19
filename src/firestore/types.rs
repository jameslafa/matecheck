use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of a friend relationship
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FriendStatusValue {
    OnTrack,
    DueSoon,
    Overdue,
    NeverMet,
}

/// Status snapshot for a single friend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendStatus {
    pub friend_id: String,
    pub friend_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_planned_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_planned_event: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub planned_dates: Vec<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days_since_last_seen: Option<i64>,
    pub frequency_days: u32,
    pub days_overdue: i64,
    pub status: FriendStatusValue,
    pub snoozed: bool,
}

/// Full status report stored as a single Firestore document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusReport {
    pub updated_at: DateTime<Utc>,
    pub friends: Vec<FriendStatus>,
    pub should_notify: bool,
}

/// Represents a snooze record in Firestore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snooze {
    /// The friend ID being snoozed
    pub friend_id: String,

    /// When the snooze expires (day-granularity)
    pub snoozed_until: DateTime<Utc>,

    /// When the snooze was created
    pub snoozed_at: DateTime<Utc>,

    /// Optional reason for the snooze (for future UI)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
