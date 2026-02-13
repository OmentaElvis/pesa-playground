use sea_orm_migration::prelude::*;
use crate::migrations::sqlite_helpers::{AddForeignKeyBuilder, DropForeignKeyBuilder};

// Define the necessary table and column enums for this migration
#[derive(Iden)]
enum ApiLogs {
    Table,
    RequestId,
}

#[derive(Iden)]
enum Transactions {
    Table,
    RequestId,
}

#[derive(Iden)]
enum CallbackLogs {
    Table,
    RequestId,
}

#[derive(Iden)]
enum TransactionJobs {
    Table,
    RequestId,
}

#[derive(Iden)]
enum Requests {
    Table,
    Id,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        // Add request_id column to api_logs
        manager
            .alter_table(
                Table::alter()
                    .table(ApiLogs::Table)
                    .add_column(ColumnDef::new(ApiLogs::RequestId).string().null())
                    .to_owned(),
            )
            .await?;

        // Add request_id column to transactions
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .add_column(ColumnDef::new(Transactions::RequestId).string().null())
                    .to_owned(),
            )
            .await?;

        // Add request_id column to callback_logs
        manager
            .alter_table(
                Table::alter()
                    .table(CallbackLogs::Table)
                    .add_column(ColumnDef::new(CallbackLogs::RequestId).string().null())
                    .to_owned(),
            )
            .await?;

        // Create foreign key constraint for api_logs.request_id
        AddForeignKeyBuilder::create()
            .from(ApiLogs::Table)
            .to(Requests::Table)
            .with_column(ApiLogs::RequestId, Requests::Id)
            .on_delete(ForeignKeyAction::SetNull)
            .on_update(ForeignKeyAction::NoAction)
            .execute(manager, ApiLogs::Table)
            .await?;

        // Create foreign key constraint for transactions.request_id
        AddForeignKeyBuilder::create()
            .from(Transactions::Table)
            .to(Requests::Table)
            .with_column(Transactions::RequestId, Requests::Id)
            .on_delete(ForeignKeyAction::SetNull)
            .on_update(ForeignKeyAction::NoAction)
            .execute(manager, Transactions::Table)
            .await?;

        // Create foreign key constraint for callback_logs.request_id
        AddForeignKeyBuilder::create()
            .from(CallbackLogs::Table)
            .to(Requests::Table)
            .with_column(CallbackLogs::RequestId, Requests::Id)
            .on_delete(ForeignKeyAction::SetNull)
            .on_update(ForeignKeyAction::NoAction)
            .execute(manager, CallbackLogs::Table)
            .await?;

        // Create foreign key constraint for transaction_jobs.request_id
        AddForeignKeyBuilder::create()
            .from(TransactionJobs::Table)
            .to(Requests::Table)
            .with_column(TransactionJobs::RequestId, Requests::Id)
            .on_delete(ForeignKeyAction::SetNull)
            .on_update(ForeignKeyAction::NoAction)
            .execute(manager, TransactionJobs::Table)
            .await?;

        // Create indexes for better query performance
        manager
            .create_index(
                Index::create()
                    .name("idx-api-logs-request-id")
                    .table(ApiLogs::Table)
                    .col(ApiLogs::RequestId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-transactions-request-id")
                    .table(Transactions::Table)
                    .col(Transactions::RequestId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-callback-logs-request-id")
                    .table(CallbackLogs::Table)
                    .col(CallbackLogs::RequestId)
                    .to_owned(),
            )
            .await?;

        // Index for transaction_jobs.request_id already exists in m20260114_154216_transaction_jobs

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes
        manager
            .drop_index(Index::drop().name("idx-api-logs-request-id").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx-transactions-request-id").to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-callback-logs-request-id")
                    .to_owned(),
            )
            .await?;

        // Index for transaction_jobs.request-id is dropped in m20260114_154216_transaction_jobs

        // Drop foreign keys
        DropForeignKeyBuilder::create()
            .from_table(ApiLogs::Table)
            .to(Requests::Table)
            .with_column(ApiLogs::RequestId, Requests::Id)
            .execute(manager, ApiLogs::Table)
            .await?;

        DropForeignKeyBuilder::create()
            .from_table(Transactions::Table)
            .to(Requests::Table)
            .with_column(Transactions::RequestId, Requests::Id)
            .execute(manager, Transactions::Table)
            .await?;

        DropForeignKeyBuilder::create()
            .from_table(CallbackLogs::Table)
            .to(Requests::Table)
            .with_column(CallbackLogs::RequestId, Requests::Id)
            .execute(manager, CallbackLogs::Table)
            .await?;

        DropForeignKeyBuilder::create()
            .from_table(TransactionJobs::Table)
            .to(Requests::Table)
            .with_column(TransactionJobs::RequestId, Requests::Id)
            .execute(manager, TransactionJobs::Table)
            .await?;

        // Drop columns
        manager
            .alter_table(
                Table::alter()
                    .table(ApiLogs::Table)
                    .drop_column(ApiLogs::RequestId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .drop_column(Transactions::RequestId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(CallbackLogs::Table)
                    .drop_column(CallbackLogs::RequestId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
