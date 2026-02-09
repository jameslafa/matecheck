use anyhow::Result;
use firestore::FirestoreDb;

use super::snoozes::SnoozesRepository;

/// Firestore client for managing database connections
pub struct FirestoreClient {
    db: FirestoreDb,
}

impl FirestoreClient {
    /// Creates a new Firestore client
    ///
    /// Uses GOOGLE_APPLICATION_CREDENTIALS environment variable for authentication.
    /// Defaults to "matecheck-prod" project unless FIREBASE_PROJECT_ID is set.
    ///
    /// # Errors
    /// Returns error if Firestore connection fails or credentials are invalid
    pub async fn new() -> Result<Self> {
        let project_id = std::env::var("FIREBASE_PROJECT_ID")
            .unwrap_or_else(|_| "matecheck-prod".to_string());

        // Automatically uses GOOGLE_APPLICATION_CREDENTIALS environment variable
        let db = FirestoreDb::new(&project_id).await?;

        Ok(Self { db })
    }

    /// Returns a repository for managing snoozes
    pub fn snoozes(&self) -> SnoozesRepository<'_> {
        SnoozesRepository::new(&self.db)
    }
}
