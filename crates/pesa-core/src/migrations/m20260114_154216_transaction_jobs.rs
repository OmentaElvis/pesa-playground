use sea_orm::{DbErr, DeriveMigrationName, Iden};
use sea_orm_migration::{MigrationTrait, SchemaManager, async_trait};
use sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};

#[derive(Iden)]
enum TransactionJobs {
    Table,
    Id,
    RequestId,
    OriginatorConversationId,
    ConversationId,
    TransactionId,
    Status,
    ProcessAfter,
    Payload,
    ResultPayload,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

impl Migration {
    pub fn transaction_jobs_table() -> TableCreateStatement {
        Table::create()
            .table(TransactionJobs::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(TransactionJobs::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(TransactionJobs::RequestId)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionJobs::OriginatorConversationId)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionJobs::ConversationId)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionJobs::TransactionId)
                    .string()
                    .not_null(),
            )
            .col(ColumnDef::new(TransactionJobs::Status).string().not_null())
            .col(
                ColumnDef::new(TransactionJobs::ProcessAfter)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(ColumnDef::new(TransactionJobs::Payload).json().not_null())
            .col(ColumnDef::new(TransactionJobs::ResultPayload).json().null())
            .col(
                ColumnDef::new(TransactionJobs::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionJobs::UpdatedAt)
                    .timestamp_with_time_zone()
                    .null(),
            )
            .to_owned()
    }
    pub fn request_id_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-transaction-jobs-request-id")
            .table(TransactionJobs::Table)
            .col(TransactionJobs::RequestId)
            .to_owned()
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Self::transaction_jobs_table()).await?;
        manager.create_index(Self::request_id_index()).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TransactionJobs::Table).to_owned())
            .await
    }
}
