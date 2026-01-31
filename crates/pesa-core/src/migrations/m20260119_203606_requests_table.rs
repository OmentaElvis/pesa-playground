use sea_orm_migration::prelude::*;

// Local enum definitions for foreign key references
#[derive(Iden)]
enum ApiKeys {
    Table,
    Id,
}

#[derive(Iden)]
enum BusinessOperators {
    Table,
    Id,
}

#[derive(Iden)]
enum Projects {
    Table,
    Id,
}

#[derive(Iden)]
enum Businesses {
    Table,
    Id,
}

#[derive(Iden)]
enum UserProfiles {
    Table,
    AccountId,
}

#[derive(Iden)]
enum Requests {
    Table,
    Id,
    SourceType,
    SourceApiKeyId,
    SourceOperatorId,
    SourceComponent,
    RequestType,
    RequestStatus,
    CreatedAt,
    StartedAt,
    CompletedAt,
    ProjectId,
    BusinessId,
    UserId,
    RequestBody,
    ResponseBody,
    ErrorMessage,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

impl Migration {
    pub fn requests_table() -> TableCreateStatement {
        Table::create()
            .table(Requests::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Requests::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(Requests::SourceType).string().not_null())
            .col(ColumnDef::new(Requests::SourceApiKeyId).integer().null())
            .col(ColumnDef::new(Requests::SourceOperatorId).integer().null())
            .col(ColumnDef::new(Requests::SourceComponent).string().null())
            .col(ColumnDef::new(Requests::RequestType).string().not_null())
            .col(
                ColumnDef::new(Requests::RequestStatus)
                    .string()
                    .not_null()
                    .default("pending"),
            )
            .col(
                ColumnDef::new(Requests::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(Requests::StartedAt)
                    .timestamp_with_time_zone()
                    .null(),
            )
            .col(
                ColumnDef::new(Requests::CompletedAt)
                    .timestamp_with_time_zone()
                    .null(),
            )
            .col(ColumnDef::new(Requests::ProjectId).integer().null())
            .col(ColumnDef::new(Requests::BusinessId).integer().null())
            .col(ColumnDef::new(Requests::UserId).integer().null())
            .col(ColumnDef::new(Requests::RequestBody).text().null())
            .col(ColumnDef::new(Requests::ResponseBody).text().null())
            .col(ColumnDef::new(Requests::ErrorMessage).text().null())
            .foreign_key(
                ForeignKey::create()
                    .from(Requests::Table, Requests::SourceApiKeyId)
                    .to(ApiKeys::Table, ApiKeys::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Requests::Table, Requests::SourceOperatorId)
                    .to(BusinessOperators::Table, BusinessOperators::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Requests::Table, Requests::ProjectId)
                    .to(Projects::Table, Projects::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Requests::Table, Requests::BusinessId)
                    .to(Businesses::Table, Businesses::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Requests::Table, Requests::UserId)
                    .to(UserProfiles::Table, UserProfiles::AccountId)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .to_owned()
    }

    pub fn requests_project_id_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-requests-project-id")
            .table(Requests::Table)
            .col(Requests::ProjectId)
            .to_owned()
    }

    pub fn requests_business_id_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-requests-business-id")
            .table(Requests::Table)
            .col(Requests::BusinessId)
            .to_owned()
    }

    pub fn requests_source_type_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-requests-source-type")
            .table(Requests::Table)
            .col(Requests::SourceType)
            .to_owned()
    }

    pub fn requests_created_at_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-requests-created-at")
            .table(Requests::Table)
            .col(Requests::CreatedAt)
            .to_owned()
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Self::requests_table()).await?;
        manager
            .create_index(Self::requests_project_id_index())
            .await?;
        manager
            .create_index(Self::requests_business_id_index())
            .await?;
        manager
            .create_index(Self::requests_source_type_index())
            .await?;
        manager
            .create_index(Self::requests_created_at_index())
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Requests::Table).to_owned())
            .await?;

        Ok(())
    }
}
