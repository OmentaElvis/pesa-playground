pub use sea_orm_migration::prelude::*;

pub mod sqlite_helpers;
pub use sqlite_helpers::{AddForeignKeyBuilder, DropForeignKeyBuilder};

mod m20251227_183827_initial_schema;
mod m20251228_082822_apply_schema_changes;
mod m20260114_154216_transaction_jobs;
mod m20260119_203606_requests_table;
mod m20260119_203640_request_ids_table;
mod m20260119_205721_add_request_id_columns;

pub struct Migrator;
#[cfg(test)]
pub mod tests;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251227_183827_initial_schema::Migration),
            Box::new(m20251228_082822_apply_schema_changes::Migration),
            Box::new(m20260114_154216_transaction_jobs::Migration),
            Box::new(m20260119_203606_requests_table::Migration),
            Box::new(m20260119_203640_request_ids_table::Migration),
            Box::new(m20260119_205721_add_request_id_columns::Migration),
        ]
    }
}

#[cfg(test)]
mod migration_tests {
    use super::Migrator;
    use sea_orm::{Database, DbErr, TransactionTrait};
    use sea_orm_migration::MigratorTrait;

    #[tokio::test]
    async fn test_full_up_and_down() -> Result<(), DbErr> {
        // Setup in-memory SQLite database
        let mut opts = sea_orm::ConnectOptions::from("sqlite::memory:");
        opts.max_connections(10)
            .connect_timeout(std::time::Duration::from_secs(60));
        let db = Database::connect(opts).await?;

        let txn = db.begin().await?;
        Migrator::up(&txn, None).await?;
        txn.commit().await?;

        let txn = db.begin().await?;
        Migrator::down(&txn, None).await?;
        txn.commit().await?;

        Ok(())
    }
}
