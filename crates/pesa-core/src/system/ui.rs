use anyhow::Context;
use sea_orm::{ConnectionTrait, EntityName, TransactionTrait};
use sea_orm_migration::MigratorTrait;
use sea_query::{Alias, IntoIden};

use crate::{
    AppContext, accounts, api_keys, api_logs, business, business_operators, callbacks,
    migrations::Migrator,
    projects, sandboxes,
    server::{self},
    transaction_costs, transactions, transactions_log,
};

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

    let txn = context.db.begin().await?;

    // Execute PRAGMA to disable foreign key checks for this connection
    txn.execute_unprepared("PRAGMA foreign_keys = OFF;").await?;

    // 2. Drop all tables. The order does not matter now.
    let tables_to_drop = vec![
        transaction_costs::db::Entity.table_name().to_string(),
        transactions_log::db::Entity.table_name().to_string(),
        transactions::db::Entity.table_name().to_string(),
        server::access_token::db::Entity.table_name().to_string(),
        projects::db::Entity.table_name().to_string(),
        callbacks::db::Entity.table_name().to_string(),
        business_operators::db::Entity.table_name().to_string(),
        business::db::Entity.table_name().to_string(),
        api_logs::db::Entity.table_name().to_string(),
        api_keys::db::Entity.table_name().to_string(),
        accounts::user_profiles::db::Entity.table_name().to_string(),
        accounts::till_accounts::db::Entity.table_name().to_string(),
        accounts::mmf_accounts::db::Entity.table_name().to_string(),
        accounts::utility_accounts::db::Entity
            .table_name()
            .to_string(),
        accounts::paybill_accounts::db::Entity
            .table_name()
            .to_string(),
        accounts::db::Entity.table_name().to_string(),
        "seaql_migrations".to_string(),
    ];

    for table in tables_to_drop {
        txn.execute(
            txn.get_database_backend().build(
                sea_query::Table::drop()
                    .table(sea_orm::sea_query::TableRef::Table(
                        Alias::new(&table).into_iden(),
                    ))
                    .if_exists(),
            ),
        )
        .await
        .with_context(|| format!("Failed to drop table {}", table))?;
    }

    // Re-enable foreign key checks
    txn.execute_unprepared("PRAGMA foreign_keys = ON;").await?;

    // 3. Re-run migrations to create a fresh set of tables
    Migrator::up(&txn, None)
        .await
        .context("Failed to run migrations after clearing data")?;

    // 4. Re-seed default data
    transaction_costs::init_default_costs(&txn)
        .await
        .context("Failed to re-seed default data after clearing")?;

    txn.commit().await?;

    Ok(())
}
