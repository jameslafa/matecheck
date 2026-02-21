use anyhow::Result;
use firestore::FirestoreDb;

use super::types::StatusReport;

/// Repository for managing the status report in Firestore
pub struct StatusRepository<'a> {
    db: &'a FirestoreDb,
}

impl<'a> StatusRepository<'a> {
    pub fn new(db: &'a FirestoreDb) -> Self {
        Self { db }
    }

    /// Write the full status report to Firestore as `status/latest`
    pub async fn write_report(&self, report: &StatusReport) -> Result<()> {
        self.db
            .fluent()
            .update()
            .in_col("status")
            .document_id("latest")
            .object(report)
            .execute::<()>()
            .await?;

        Ok(())
    }
}
