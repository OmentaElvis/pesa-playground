use crate::accounts::{Account, AccountType};
use crate::AppContext;

pub async fn create_account(
    ctx: &AppContext,
    account_type: AccountType,
    initial_balance: i64,
) -> Result<u32, String> {
    let db = &ctx.db;
    let acc = Account::create_account(db, account_type, initial_balance)
        .await
        .map_err(|err| format!("Failed to create account: {}", err))?;

    Ok(acc.id)
}

pub async fn get_account(ctx: &AppContext, id: u32) -> Result<Account, String> {
    let db = &ctx.db;
    Account::get_account(db, id)
        .await
        .map_err(|err| format!("Failed to get account: {}", err))?
        .ok_or_else(|| format!("Account with ID {} not found", id))
}
