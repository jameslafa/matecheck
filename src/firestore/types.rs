use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
