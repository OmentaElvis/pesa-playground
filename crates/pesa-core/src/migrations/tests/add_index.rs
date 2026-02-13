use sea_orm_migration::prelude::*;

use super::create_test_table::PocTable;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx-poc_table-poc_name")
                    .table(PocTable::Table)
                    .col(PocNameOnly::PocName)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(PocTable::Table)
                    .name("idx-poc_table-poc_name")
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum PocNameOnly {
    PocName,
}
