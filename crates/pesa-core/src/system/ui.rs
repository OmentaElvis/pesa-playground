use anyhow::Context;
use sea_orm_migration::MigratorTrait;

use crate::{AppContext, migrations::Migrator, sandboxes, transaction_costs};

pub async fn clear_all_data(context: &AppContext) -> anyhow::Result<()> {
    // 1. Stop all running sandboxes to release any file locks or running processes
    let running_sandboxes = sandboxes::ui::list_running_sandboxes(context).await?;
    for sandbox in running_sandboxes {
        sandboxes::ui::stop_sandbox(context, sandbox.project_id)
            .await
            .with_context(|| {
                format!("Failed to stop sandbox for project {}", sandbox.project_id)
            })?;
    }

    // Execute PRAGMA to disable foreign key checks for this connection
    Migrator::fresh(&context.db).await?;

    transaction_costs::init_default_costs(&context.db)
        .await
        .context("Failed to re-seed default data after clearing")?;

    Ok(())
}
