use tauri::State;

use crate::db::Database;
use crate::accounts::{Account, AccountType};

#[tauri::command]
pub async fn create_account(
    state: State<'_, Database>,
    account_type: AccountType,
    initial_balance: i64,
) -> Result<u32, String> {
    let db = &state.conn;
    Account::create_account(db, account_type, initial_balance)
        .await
        .map_err(|err| format!("Failed to create account: {}", err))
}

#[tauri::command]
pub async fn get_account(state: State<'_, Database>, id: u32) -> Result<Account, String> {
    let db = &state.conn;
    Account::get_account(db, id)
        .await
        .map_err(|err| format!("Failed to get account: {}", err))?
        .ok_or_else(|| format!("Account with ID {} not found", id))
}