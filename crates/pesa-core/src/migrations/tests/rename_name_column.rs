use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PocTable::Table)
                    .rename_column(PocTable::Name, PocTable::PocName)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PocTable::Table)
                    .rename_column(PocTable::PocName, PocTable::Name)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum PocTable {
    Table,
    Name,
    PocName,
}
