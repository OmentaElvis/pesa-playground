use super::db;
use super::Ledger;
use super::Transaction;
use super::TransactionType;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use serde::Deserialize;
use tauri::State;

use crate::db::Database;

#[derive(Deserialize, Debug, Clone)]
pub struct TransactionFilter {
    pub from: Option<u32>,
    pub to: Option<u32>,
    pub transaction_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for TransactionFilter {
    fn default() -> Self {
        Self {
            from: None,
            to: None,
            transaction_type: None,
            status: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}

#[tauri::command]
pub async fn get_transaction(
    state: State<'_, Database>,
    transaction_id: String,
) -> Result<Option<Transaction>, String> {
    let db = &state.conn;
    let transaction = db::Entity::find_by_id(transaction_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to get transaction: {}", e))?;

    Ok(transaction.map(|t| t.into()))
}

#[tauri::command]
pub async fn list_transactions(
    state: State<'_, Database>,
    filter: TransactionFilter,
) -> Result<Vec<Transaction>, String> {
    let db = &state.conn;
    let mut query = crate::transactions::db::Entity::find();

    match (filter.from, filter.to) {
        (Some(from), Some(to)) => {
            query = query.filter(
                Condition::any()
                    .add(crate::transactions::db::Column::From.eq(from))
                    .add(crate::transactions::db::Column::To.eq(to)),
            )
        }
        (Some(from), None) => {
            query = query.filter(crate::transactions::db::Column::From.eq(from));
        }
        (None, Some(to)) => {
            query = query.filter(crate::transactions::db::Column::To.eq(to));
        }
        (None, None) => {}
    }

    if let Some(transaction_type) = filter.transaction_type {
        query = query.filter(crate::transactions::db::Column::TransactionType.eq(transaction_type));
    }
    if let Some(status) = filter.status {
        query = query.filter(crate::transactions::db::Column::Status.eq(status));
    }

    if let Some(limit) = filter.limit {
        query = query.limit(limit as u64);
    }

    if let Some(offset) = filter.offset {
        query = query.offset(offset as u64);
    }

    let transactions = query
        .all(db)
        .await
        .map_err(|e| format!("Failed to list transactions: {}", e))?;

    Ok(transactions.into_iter().map(|t| t.into()).collect())
}

#[tauri::command]
pub async fn count_transactions(
    state: State<'_, Database>,
    filter: TransactionFilter,
) -> Result<u64, String> {
    let db = &state.conn;
    let mut query = crate::transactions::db::Entity::find();

    if let Some(from) = filter.from {
        query = query.filter(crate::transactions::db::Column::From.eq(from));
    }
    if let Some(to) = filter.to {
        query = query.filter(crate::transactions::db::Column::To.eq(to));
    }
    if let Some(transaction_type) = filter.transaction_type {
        query = query.filter(crate::transactions::db::Column::TransactionType.eq(transaction_type));
    }
    if let Some(status) = filter.status {
        query = query.filter(crate::transactions::db::Column::Status.eq(status));
    }

    query
        .count(db)
        .await
        .map_err(|e| format!("Failed to count transactions: {}", e))
}

#[tauri::command]
pub async fn get_transaction_by_checkout_request(
    state: State<'_, Database>,
    checkout_request_id: String,
) -> Result<Option<Transaction>, String> {
    let db = &state.conn;
    let transaction = crate::transactions::db::Entity::find()
        .filter(crate::transactions::db::Column::Id.eq(checkout_request_id))
        .one(db)
        .await
        .map_err(|e| format!("Failed to get transaction by checkout request ID: {}", e))?;

    Ok(transaction.map(|t| t.into()))
}

#[tauri::command]
pub async fn get_user_transactions(
    state: State<'_, Database>,
    user_id: u32,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<Transaction>, String> {
    let filter = TransactionFilter {
        from: Some(user_id),
        to: None,
        transaction_type: None,
        status: None,
        limit: limit.or(Some(20)),
        offset: offset.or(Some(0)),
    };

    list_transactions(state, filter).await
}

#[tauri::command]
pub async fn get_recent_transactions(
    state: State<'_, Database>,
    limit: Option<u32>,
) -> Result<Vec<Transaction>, String> {
    let filter = TransactionFilter {
        from: None,
        to: None,
        transaction_type: None,
        status: None,
        limit: limit.or(Some(10)),
        offset: Some(0),
    };

    list_transactions(state, filter).await
}

#[tauri::command]
pub async fn get_transaction_stats(state: State<'_, Database>) -> Result<TransactionStats, String> {
    let filter = TransactionFilter {
        from: None,
        to: None,
        transaction_type: None,
        status: None,
        limit: None,
        offset: None,
    };

    let total_count = count_transactions(state.clone(), filter.clone())
        .await
        .map_err(|err| format!("Failed to get total count: {}", err))?;

    let successful_filter = TransactionFilter {
        status: Some("SUCCESS".to_string()),
        ..filter.clone()
    };

    let successful_count = count_transactions(state.clone(), successful_filter)
        .await
        .map_err(|err| format!("Failed to get successful count: {}", err))?;

    let pending_filter = TransactionFilter {
        status: Some("PENDING".to_string()),
        ..filter.clone()
    };

    let pending_count = count_transactions(state.clone(), pending_filter)
        .await
        .map_err(|err| format!("Failed to get pending count: {}", err))?;

    let failed_filter = TransactionFilter {
        status: Some("FAILED".to_string()),
        ..filter
    };

    let failed_count = count_transactions(state.clone(), failed_filter)
        .await
        .map_err(|err| format!("Failed to get failed count: {}", err))?;

    Ok(TransactionStats {
        total_count,
        successful_count,
        pending_count,
        failed_count,
    })
}

#[tauri::command]
pub async fn transfer(
    state: State<'_, Database>,
    source: Option<u32>,
    destination: u32,
    amount: i64,
    txn_type: TransactionType,
) -> Result<Transaction, String> {
    Ledger::transfer(&state.conn, source, destination, amount, &txn_type)
        .await
        .map_err(|err| format!("Transfer Error: {}", err))
}

#[tauri::command]
pub async fn reverse(state: State<'_, Database>, id: String) -> Result<Transaction, String> {
    Ledger::reverse(&state.conn, &id)
        .await
        .map_err(|err| format!("Transfer Error: {}", err))
}

#[derive(serde::Serialize)]
pub struct TransactionStats {
    pub total_count: u64,
    pub successful_count: u64,
    pub pending_count: u64,
    pub failed_count: u64,
}
