use sea_orm_migration::prelude::*;

#[derive(Iden)]
enum RequestIds {
    Table,
    Id,
    RequestId,
    IdType,
    ExternalId,
}

#[derive(Iden)]
enum Requests {
    Table,
    Id,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

impl Migration {
    pub fn request_ids_table() -> TableCreateStatement {
        Table::create()
            .table(RequestIds::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(RequestIds::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(RequestIds::RequestId).string().not_null())
            .col(ColumnDef::new(RequestIds::IdType).string().not_null())
            .col(ColumnDef::new(RequestIds::ExternalId).string().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(RequestIds::Table, RequestIds::RequestId)
                    .to(Requests::Table, Requests::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned()
    }

    pub fn request_ids_lookup_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-request-ids-lookup")
            .table(RequestIds::Table)
            .col(RequestIds::IdType)
            .col(RequestIds::ExternalId)
            .to_owned()
    }

    pub fn request_ids_unique_constraint() -> IndexCreateStatement {
        Index::create()
            .name("unique-request-id-type")
            .table(RequestIds::Table)
            .col(RequestIds::RequestId)
            .col(RequestIds::IdType)
            .unique()
            .to_owned()
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Self::request_ids_table()).await?;
        manager
            .create_index(Self::request_ids_lookup_index())
            .await?;
        manager
            .create_index(Self::request_ids_unique_constraint())
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("unique-request-id-type").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx-request-ids-lookup").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RequestIds::Table).to_owned())
            .await?;

        Ok(())
    }
}
