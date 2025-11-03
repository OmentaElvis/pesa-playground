use super::db;
use super::FullTransactionLog;
use crate::transactions_log::TransactionLog;
use crate::AppContext;
use anyhow::Context;
use anyhow::Result;
use sea_orm::EntityTrait;

pub async fn get_transaction_log(
    ctx: &AppContext,
    transaction_id: i32,
) -> Result<Option<db::Model>> {
    let db = &ctx.db;
    let transaction_log = db::Entity::find_by_id(transaction_id)
        .one(db)
        .await
        .context("Failed to get transaction log")?;

    Ok(transaction_log)
}

pub async fn get_full_transaction_log(
    ctx: &AppContext,
    transaction_log_id: i32,
) -> Result<Option<FullTransactionLog>> {
    let db = &ctx.db;
    TransactionLog::get_full_log(db, transaction_log_id)
        .await
        .context("Failed to get full transaction log")
}

pub async fn list_full_transaction_logs(
    ctx: &AppContext,
    account_id: i32,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<FullTransactionLog>> {
    let db = &ctx.db;
    TransactionLog::list_full_logs(db, account_id, limit.unwrap_or(50), offset.unwrap_or(0))
        .await
        .context("Failed to list full transaction logs")
}

pub async fn list_accounts_full_transaction_logs(
    ctx: &AppContext,
    accounts: Vec<u32>,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<FullTransactionLog>> {
    let db = &ctx.db;
    TransactionLog::list_account_logs(db, accounts, limit.unwrap_or(50), offset.unwrap_or(0))
        .await
        .context("Failed to list full transaction logs")
}

pub async fn count_transaction_logs(ctx: &AppContext, accounts: Vec<u32>) -> Result<u64> {
    let db = &ctx.db;
    TransactionLog::count_transaction_logs(db, accounts)
        .await
        .context("Failed to list full transaction logs")
}
