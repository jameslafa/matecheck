use anyhow::Result;
use firestore::FirestoreDb;

use crate::config::Friend;

/// Repository for managing friends in Firestore
pub struct FriendsRepository<'a> {
    db: &'a FirestoreDb,
}

impl<'a> FriendsRepository<'a> {
    /// Creates a new friends repository
    pub fn new(db: &'a FirestoreDb) -> Self {
        Self { db }
    }

    /// Get all friends from Firestore
    ///
    /// # Errors
    /// Returns error if Firestore query fails
    pub async fn get_all(&self) -> Result<Vec<Friend>> {
        let friends: Vec<Friend> = self
            .db
            .fluent()
            .select()
            .from("friends")
            .obj()
            .query()
            .await?;

        Ok(friends)
    }

    /// Get a single friend by ID
    ///
    /// # Arguments
    /// * `id` - The unique ID of the friend to retrieve
    ///
    /// # Errors
    /// Returns error if Firestore query fails or friend not found
    pub async fn get(&self, id: &str) -> Result<Option<Friend>> {
        let friend: Option<Friend> = self
            .db
            .fluent()
            .select()
            .by_id_in("friends")
            .obj()
            .one(id)
            .await?;

        Ok(friend)
    }

    /// Create or update a friend
    ///
    /// # Arguments
    /// * `friend` - The friend object to upsert
    ///
    /// # Errors
    /// Returns error if Firestore write fails
    pub async fn upsert(&self, friend: &Friend) -> Result<()> {
        self.db
            .fluent()
            .insert()
            .into("friends")
            .document_id(&friend.id)
            .object(friend)
            .execute::<()>()
            .await?;

        Ok(())
    }

    /// Delete a friend by ID
    ///
    /// # Arguments
    /// * `id` - The unique ID of the friend to delete
    ///
    /// # Errors
    /// Returns error if Firestore delete fails
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.db
            .fluent()
            .delete()
            .from("friends")
            .document_id(id)
            .execute()
            .await?;

        Ok(())
    }
}
