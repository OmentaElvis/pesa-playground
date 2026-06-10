use anyhow::Context;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigratorTrait;
use sqlx::Acquire;

use crate::{
    AppContext, app::TransactionManagerStats, migrations::Migrator, sandboxes, transaction_costs,
};

async fn reset_database(db: &DatabaseConnection) -> anyhow::Result<()> {
    // === Intentionally manually purge the database
    // this is because the Migrator does not seem to properly handle connection pool
    // with more than one connection.
    // This fix will obtain an explicit connection to do everything on one pass.
    use sqlx::Executor;
    let mut conn = db.get_sqlite_connection_pool().acquire().await?;
    conn.execute("PRAGMA foreign_keys = OFF").await?;

    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name != 'sqlite_sequence';",
    )
    .fetch_all(&mut *conn)
    .await?;

    let mut txn = conn.begin().await?;
    println!("Dropping {} tables.", tables.len());
    for name in &tables {
        println!("Dropping: {name}");
        sqlx::query(&format!("DROP TABLE IF EXISTS \"{}\";", name))
            .execute(&mut *txn)
            .await?;
    }

    txn.commit().await?;
    println!("Dropped all {} tables.", tables.len());

    conn.execute("PRAGMA foreign_keys = ON").await?;
    drop(conn);
    // ================================================

    // Re run the migrations
    Migrator::up(db, None)
        .await
        .context("Failed to reset migrations")?;

    transaction_costs::init_default_costs(db)
        .await
        .context("Failed to re-seed default data after clearing")?;

    Ok(())
}

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

    reset_database(&context.db)
        .await
        .context("Failed to reset database")?;

    Ok(())
}

pub async fn get_transaction_manager_stats(
    context: &AppContext,
) -> anyhow::Result<TransactionManagerStats> {
    match context.stats.read() {
        Ok(stats) => Ok(stats.clone()),
        Err(err) => Err(anyhow::anyhow!(
            "Failed to acquire read lock on TransactionManagerStats: {err}"
        )),
    }
}
