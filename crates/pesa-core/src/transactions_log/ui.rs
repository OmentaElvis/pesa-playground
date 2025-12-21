use super::db;
use super::{FullTransactionLog, TransactionLog};
use crate::accounts::{mmf_accounts, utility_accounts};
use crate::transactions::TransactionNote;
use crate::transactions_log;
use crate::{
    transactions::{self, TransactionStatus},
    AppContext,
};
use anyhow::Context;
use anyhow::Result;
use sea_orm::{prelude::*, sea_query::Cond, ColumnTrait, Order, QueryOrder, QuerySelect};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum HistoryScopeType {
    User,
    Business,
    All,
}

#[derive(Deserialize, Debug)]
pub struct HistoryScope {
    pub r#type: HistoryScopeType,
    pub id: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Deserialize, Debug)]
pub struct Sorting {
    pub by: String,
    pub direction: SortDirection,
}

#[derive(Deserialize, Debug)]
pub struct Filters {
    pub statuses: Option<Vec<TransactionStatus>>,
    pub search_query: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    pub limit: u64,
    pub offset: u64,
}

#[derive(Deserialize, Debug)]
pub struct HistoryFilter {
    pub scope: HistoryScope,
    pub pagination: Pagination,
    pub sorting: Option<Sorting>,
    pub filters: Option<Filters>,
}

#[derive(serde::Serialize, Clone)]
pub struct TransactionHistoryEntry {
    pub transaction_id: String,
    pub date: DateTimeUtc,
    pub status: String,
    pub transaction_type: String,
    pub fee: i64,
    pub amount: i64,

    pub sender_name: String,
    pub sender_id: Option<u32>,
    pub sender_balance: Option<i64>,

    pub receiver_name: String,
    pub receiver_id: u32,
    pub receiver_balance: Option<i64>,

    pub notes: Option<TransactionNote>,
}

pub async fn get_transaction_log(
    ctx: &AppContext,
    transaction_id: u32,
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
    transaction_log_id: u32,
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

pub async fn get_transaction_history(
    ctx: &AppContext,
    filter: HistoryFilter,
) -> Result<Vec<TransactionHistoryEntry>> {
    let db = &ctx.db;

    // 1. Determine account IDs from scope
    let account_ids: Option<Vec<u32>> = match filter.scope.r#type {
        HistoryScopeType::User => {
            if let Some(user_id) = filter.scope.id {
                Some(vec![user_id])
            } else {
                None
            }
        }
        HistoryScopeType::Business => {
            if let Some(business_id) = filter.scope.id {
                let mut ids = Vec::new();

                if let Some(mmf) = mmf_accounts::db::Entity::find()
                    .filter(mmf_accounts::db::Column::BusinessId.eq(business_id))
                    .one(db)
                    .await?
                {
                    ids.push(mmf.account_id);
                }
                if let Some(utility) = utility_accounts::db::Entity::find()
                    .filter(utility_accounts::db::Column::BusinessId.eq(business_id))
                    .one(db)
                    .await?
                {
                    ids.push(utility.account_id);
                }
                Some(ids)
            } else {
                None
            }
        }
        HistoryScopeType::All => None,
    };

    // 2. Build the main query
    let mut query = transactions::db::Entity::find();

    // Apply scope filter
    if let Some(ids) = &account_ids {
        if !ids.is_empty() {
            query = query.filter(
                Cond::any()
                    .add(transactions::db::Column::From.is_in(ids.clone()))
                    .add(transactions::db::Column::To.is_in(ids.clone())),
            );
        }
    }

    // Apply advanced filters
    if let Some(filters) = filter.filters {
        if let Some(statuses) = filters.statuses {
            if !statuses.is_empty() {
                query = query.filter(
                    transactions::db::Column::Status.is_in(
                        statuses
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>(),
                    ),
                );
            }
        }
        if let Some(search) = filters.search_query {
            if !search.trim().is_empty() {
                let search_pattern = format!("%{}%", search);
                query = query.filter(
                    Cond::any()
                        .add(transactions::db::Column::Id.like(&search_pattern))
                        .add(transactions::db::Column::Notes.like(&search_pattern)),
                );
            }
        }
    }

    // Apply sorting
    if let Some(sorting) = filter.sorting {
        let order = match sorting.direction {
            SortDirection::Asc => Order::Asc,
            SortDirection::Desc => Order::Desc,
        };
        match sorting.by.as_str() {
            "date" => query = query.order_by(transactions::db::Column::CreatedAt, order),
            "amount" => query = query.order_by(transactions::db::Column::Amount, order),
            _ => (), // Default sort by date desc is implicitly handled
        }
    } else {
        query = query.order_by(transactions::db::Column::CreatedAt, Order::Desc);
    }

    // Apply pagination
    query = query
        .offset(filter.pagination.offset)
        .limit(filter.pagination.limit);

    // Execute query and fetch joined data
    let txns: Vec<transactions::db::Model> = query.all(db).await?;

    let mut history_entries = Vec::new();

    for txn in txns {
        let (sender_name, receiver_name) = (
            super::get_account_name(db, txn.from.unwrap_or(0)).await?,
            super::get_account_name(db, txn.to).await?,
        );

        let logs = transactions_log::db::Entity::find()
            .filter(transactions_log::db::Column::TransactionId.eq(txn.id.clone()))
            .all(db)
            .await?;

        let mut sender_balance = None;
        let mut receiver_balance = None;

        for log in logs {
            if Some(log.account_id) == txn.from {
                sender_balance = Some(log.new_balance);
            }

            if log.account_id == txn.to {
                receiver_balance = Some(log.new_balance);
            }
        }

        let notes = txn
            .notes
            .and_then(|notes_str| serde_json::from_str(&notes_str).ok());

        history_entries.push(TransactionHistoryEntry {
            transaction_id: txn.id,
            date: txn.created_at,
            status: txn.status,
            transaction_type: txn.transaction_type,
            fee: txn.fee,
            amount: txn.amount,
            sender_name,
            sender_id: txn.from,
            sender_balance,
            receiver_name,
            receiver_id: txn.to,
            receiver_balance,
            notes,
        });
    }

    Ok(history_entries)
}
