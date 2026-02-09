use anyhow::Result;
use chrono::{Duration, Utc};
use firestore::FirestoreDb;
use std::collections::HashSet;

use super::types::Snooze;

/// Repository for managing friend snoozes in Firestore
pub struct SnoozesRepository<'a> {
    db: &'a FirestoreDb,
}

impl<'a> SnoozesRepository<'a> {
    /// Creates a new snoozes repository
    pub fn new(db: &'a FirestoreDb) -> Self {
        Self { db }
    }

    /// Get set of friend IDs that are currently snoozed
    ///
    /// Returns only friends whose snooze_until date is in the future.
    ///
    /// # Errors
    /// Returns error if Firestore query fails
    pub async fn get_active_snoozes(&self) -> Result<HashSet<String>> {
        let now = Utc::now();

        // Query all snoozes from Firestore
        let snoozes: Vec<Snooze> = self
            .db
            .fluent()
            .select()
            .from("snoozes")
            .obj()
            .query()
            .await?;

        // Filter for active snoozes (snoozed_until > now)
        let active_snoozes: HashSet<String> = snoozes
            .into_iter()
            .filter(|snooze| snooze.snoozed_until > now)
            .map(|snooze| snooze.friend_id)
            .collect();

        Ok(active_snoozes)
    }

    /// Snooze a friend for a specific number of days
    ///
    /// # Arguments
    /// * `friend_id` - The unique ID of the friend to snooze
    /// * `days` - Number of days to snooze for
    ///
    /// # Errors
    /// Returns error if Firestore write fails
    #[allow(dead_code)]
    pub async fn snooze_friend(&self, friend_id: &str, days: u32) -> Result<()> {
        let until = Utc::now() + Duration::days(days as i64);
        let snooze = Snooze {
            friend_id: friend_id.to_string(),
            snoozed_until: until,
            snoozed_at: Utc::now(),
            reason: None,
        };

        self.db
            .fluent()
            .insert()
            .into("snoozes")
            .document_id(friend_id)
            .object(&snooze)
            .execute::<()>()
            .await?;

        Ok(())
    }

    /// Remove snooze for a friend
    ///
    /// # Arguments
    /// * `friend_id` - The unique ID of the friend to unsnooze
    ///
    /// # Errors
    /// Returns error if Firestore delete fails
    #[allow(dead_code)]
    pub async fn unsnooze_friend(&self, friend_id: &str) -> Result<()> {
        self.db
            .fluent()
            .delete()
            .from("snoozes")
            .document_id(friend_id)
            .execute()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are unit tests that would need a mock Firestore client
    // For now, they serve as documentation of the expected behavior

    #[test]
    fn test_snooze_struct_serialization() {
        let snooze = Snooze {
            friend_id: "alice".to_string(),
            snoozed_until: Utc::now() + Duration::days(7),
            snoozed_at: Utc::now(),
            reason: Some("vacation".to_string()),
        };

        // Verify the struct can be serialized
        let json = serde_json::to_string(&snooze).unwrap();
        assert!(json.contains("alice"));
        assert!(json.contains("vacation"));
    }

    #[test]
    fn test_snooze_without_reason() {
        let snooze = Snooze {
            friend_id: "bob".to_string(),
            snoozed_until: Utc::now() + Duration::days(3),
            snoozed_at: Utc::now(),
            reason: None,
        };

        // Verify reason is optional
        let json = serde_json::to_string(&snooze).unwrap();
        assert!(json.contains("bob"));
        // With skip_serializing_if, "reason" field should not be present
        assert!(!json.contains("reason"));
    }
}
