use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create PocTable
        manager
            .create_table(
                Table::create()
                    .table(PocTable::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PocTable::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PocTable::Name).string().not_null())
                    .col(ColumnDef::new(PocTable::PocTable2Id).integer().null())
                    .to_owned(),
            )
            .await?;

        // Create PocTable2
        manager
            .create_table(
                Table::create()
                    .table(PocTable2::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PocTable2::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PocTable2::Data).string().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PocTable::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PocTable2::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum PocTable {
    Table,
    Id,
    Name,
    PocTable2Id,
}

#[derive(Iden)]
pub enum PocTable2 {
    Table,
    Id,
    Data,
}