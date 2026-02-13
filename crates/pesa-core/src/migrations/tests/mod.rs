//! This module and its contents are for a proof-of-concept test only.
//! It demonstrates an isolated migration flow to prove that `sea-orm-migration`
//! can handle advanced alterations like `rename_column` on SQLite, and that
//! more complex operations can be abstracted into helpers.
pub mod add_foreign_key_and_index;
pub mod add_index;
pub mod create_test_table;
pub mod rename_name_column;

use sea_orm_migration::prelude::*;

// Define the isolated Migrator
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(create_test_table::Migration),
            Box::new(rename_name_column::Migration),
            Box::new(add_foreign_key_and_index::Migration),
            Box::new(add_index::Migration),
        ]
    }
}

#[cfg(test)]
mod poc {
    use super::Migrator;
    use sea_orm::{ConnectionTrait, Database, DbErr, Statement};
    use sea_orm_migration::{MigratorTrait, SchemaManager};

    async fn setup_db() -> Result<sea_orm::DatabaseConnection, DbErr> {
        let mut opts = sea_orm::ConnectOptions::from("sqlite::memory:");
        opts.max_connections(10)
            .connect_timeout(std::time::Duration::from_secs(60));
        Database::connect(opts).await
    }

    #[tokio::test]
    async fn test_isolated_migration_flow() -> Result<(), DbErr> {
        let db = setup_db().await?;
        let schema_manager = SchemaManager::new(&db);

        // --- UP MIGRATIONS ---

        Migrator::up(&db, None).await?;

        // Check tables exist
        assert!(
            schema_manager.has_table("poc_table").await?,
            "UP check failed: 'poc_table' should exist."
        );
        assert!(
            schema_manager.has_table("poc_table2").await?,
            "UP check failed: 'poc_table2' should exist."
        );

        // Check that the column was renamed
        assert!(
            !schema_manager.has_column("poc_table", "name").await?,
            "UP check failed: 'name' column should have been renamed."
        );
        assert!(
            schema_manager.has_column("poc_table", "poc_name").await?,
            "UP check failed: 'poc_name' column should exist after rename."
        );

        // Check for index using PRAGMA
        let index_info = db
            .query_one(Statement::from_string(
                db.get_database_backend(),
                "PRAGMA index_list('poc_table');",
            ))
            .await?
            .expect("PRAGMA index_list should return a row for the index.");

        let index_name: String = index_info.try_get("", "name")?;
        assert_eq!(index_name, "idx-poc_table-poc_name");

        // Check for foreign key using PRAGMA
        let fk_info = db
            .query_one(Statement::from_string(
                db.get_database_backend(),
                "PRAGMA foreign_key_list('poc_table');",
            ))
            .await?
            .expect("PRAGMA foreign_key_list should return a row for the FK.");

        let fk_to_table: String = fk_info.try_get("", "table")?;
        assert_eq!(fk_to_table, "poc_table2");

        // --- DOWN MIGRATIONS ---

        Migrator::down(&db, None).await?;

        assert!(
            !schema_manager.has_table("poc_table").await?,
            "DOWN check failed: 'poc_table' should NOT exist."
        );
        assert!(
            !schema_manager.has_table("poc_table2").await?,
            "DOWN check failed: 'poc_table2' should NOT exist."
        );

        Ok(())
    }
}
