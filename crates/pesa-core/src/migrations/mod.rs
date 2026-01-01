pub use sea_orm_migration::prelude::*;

mod m20251227_183827_initial_schema;
mod m20251228_082822_apply_schema_changes;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251227_183827_initial_schema::Migration),
            Box::new(m20251228_082822_apply_schema_changes::Migration),
        ]
    }
}
