use sea_orm_migration::prelude::*;

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
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-api-logs-request-id")
                    .from(ApiLogs::Table, ApiLogs::RequestId)
                    .to(Requests::Table, Requests::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        // Create foreign key constraint for transactions.request_id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-transactions-request-id")
                    .from(Transactions::Table, Transactions::RequestId)
                    .to(Requests::Table, Requests::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        // Create foreign key constraint for callback_logs.request_id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-callback-logs-request-id")
                    .from(CallbackLogs::Table, CallbackLogs::RequestId)
                    .to(Requests::Table, Requests::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        // Create foreign key constraint for transaction_jobs.request_id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-transaction-jobs-request-id")
                    .from(TransactionJobs::Table, TransactionJobs::RequestId)
                    .to(Requests::Table, Requests::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
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

        manager
            .create_index(
                Index::create()
                    .name("idx-transaction-jobs-request-id")
                    .table(TransactionJobs::Table)
                    .col(TransactionJobs::RequestId)
                    .to_owned(),
            )
            .await?;

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

        manager
            .drop_index(
                Index::drop()
                    .name("idx-transaction-jobs-request-id")
                    .to_owned(),
            )
            .await?;

        // Drop foreign keys
        manager
            .drop_foreign_key(ForeignKey::drop().name("fk-api-logs-request-id").to_owned())
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk-transactions-request-id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk-callback-logs-request-id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk-transaction-jobs-request-id")
                    .to_owned(),
            )
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
