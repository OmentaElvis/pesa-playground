pub use sea_orm_migration::prelude::*;

mod m20251227_183827_initial_schema;
mod m20251228_082822_apply_schema_changes;
mod m20260114_154216_transaction_jobs;
mod m20260119_203606_requests_table;
mod m20260119_203640_request_ids_table;
mod m20260119_205721_add_request_id_columns;

pub struct Migrator;

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
