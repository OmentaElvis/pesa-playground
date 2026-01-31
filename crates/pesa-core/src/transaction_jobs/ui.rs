use super::TransactionJob;
use crate::AppContext;
use anyhow::{Context, Result};

/// Public-facing function to get the status and details of a job.
/// This function returns the high-level, serializable TransactionJob struct.
pub async fn get_job_status(ctx: &AppContext, id: &str) -> Result<Option<TransactionJob>> {
    // Simple, clean call to the abstraction in mod.rs
    TransactionJob::get_by_id(&ctx.db, id)
        .await
        .context("Database error while fetching transaction job status.")
}
