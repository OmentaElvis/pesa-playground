use super::history::{HistoryFilter, TransactionHistoryEntry};
use super::{FullTransactionLog, TransactionLog};
use crate::AppContext;
use anyhow::Context;
use anyhow::Result;

pub async fn get_transaction_log(
    ctx: &AppContext,
    transaction_id: u32,
) -> Result<Option<super::TransactionLog>> {
    TransactionLog::get(&ctx.db, transaction_id)
        .await
        .context("Failed to get transaction log")
}

pub async fn get_full_transaction_log(
    ctx: &AppContext,
    transaction_log_id: u32,
) -> Result<Option<FullTransactionLog>> {
    TransactionLog::get_full_log(&ctx.db, transaction_log_id)
        .await
        .context("Failed to get full transaction log")
}

pub async fn list_full_transaction_logs(
    ctx: &AppContext,
    account_id: i32,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<FullTransactionLog>> {
    TransactionLog::list_full_logs(
        &ctx.db,
        account_id,
        limit.unwrap_or(50),
        offset.unwrap_or(0),
    )
    .await
    .context("Failed to list full transaction logs")
}

pub async fn list_accounts_full_transaction_logs(
    ctx: &AppContext,
    accounts: Vec<u32>,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<FullTransactionLog>> {
    TransactionLog::list_account_logs(&ctx.db, accounts, limit.unwrap_or(50), offset.unwrap_or(0))
        .await
        .context("Failed to list full transaction logs")
}

pub async fn count_transaction_logs(ctx: &AppContext, accounts: Vec<u32>) -> Result<u64> {
    TransactionLog::count_transaction_logs(&ctx.db, accounts)
        .await
        .context("Failed to count transaction logs")
}

pub async fn get_transaction_history(
    ctx: &AppContext,
    filter: HistoryFilter,
) -> Result<Vec<TransactionHistoryEntry>> {
    TransactionHistoryEntry::get(&ctx.db, filter)
        .await
        .context("Failed to get transaction history")
}
