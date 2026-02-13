use crate::migrations::sqlite_helpers::{AddForeignKeyBuilder, DropForeignKeyBuilder};
use sea_orm_migration::prelude::*;

use super::create_test_table::{PocTable, PocTable2};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        AddForeignKeyBuilder::create()
            .from(PocTable::Table)
            .to(PocTable2::Table)
            .with_column(PocTable::PocTable2Id, PocTable2::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .on_update(ForeignKeyAction::Cascade)
            .execute(manager, PocTable::Table)
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        DropForeignKeyBuilder::create()
            .from_table(PocTable::Table)
            .to(PocTable2::Table)
            .with_column(PocTable::PocTable2Id, PocTable2::Id)
            .execute(manager, PocTable::Table)
            .await
    }
}
