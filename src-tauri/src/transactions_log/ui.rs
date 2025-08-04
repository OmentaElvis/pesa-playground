use super::db;
use super::FullTransactionLog;
use crate::db::Database;
use crate::transactions_log::TransactionLog;
use sea_orm::EntityTrait;
use tauri::State;

#[tauri::command]
pub async fn get_transaction_log(
    state: State<'_, Database>,
    transaction_id: i32,
) -> Result<Option<db::Model>, String> {
    let db = &state.conn;
    let transaction_log = db::Entity::find_by_id(transaction_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to get transaction log: {}", e))?;

    Ok(transaction_log)
}

#[tauri::command]
pub async fn get_full_transaction_log(
    state: State<'_, Database>,
    transaction_log_id: i32,
) -> Result<Option<FullTransactionLog>, String> {
    let db = &state.conn;
    TransactionLog::get_full_log(db, transaction_log_id)
        .await
        .map_err(|e| format!("Failed to get full transaction log: {}", e))
}

#[tauri::command]
pub async fn list_full_transaction_logs(
    state: State<'_, Database>,
    account_id: i32,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<FullTransactionLog>, String> {
    let db = &state.conn;
    TransactionLog::list_full_logs(db, account_id, limit.unwrap_or(50), offset.unwrap_or(0))
        .await
        .map_err(|e| format!("Failed to list full transaction logs: {}", e))
}
