use anyhow::{Context, Result};
use sea_orm::TransactionTrait;

use crate::AppContext;

use super::{CreateTillAccount, TillAccount, UpdateTillAccount};

pub async fn create_till_account(
    ctx: &AppContext,
    input: CreateTillAccount,
) -> Result<TillAccount> {
    let txn = ctx
        .db
        .begin()
        .await
        .context("Failed to start transaction")?;

    let till = TillAccount::create(&txn, input).await?;

    txn.commit()
        .await
        .context("Failed to complete transaction")?;

    Ok(till)
}
pub async fn get_till_account(state: &AppContext, id: u32) -> Result<TillAccount> {
    TillAccount::get_by_id(&state.db, id).await
}
pub async fn get_till_accounts(ctx: &AppContext) -> Result<Vec<TillAccount>> {
    TillAccount::get_all(&ctx.db).await
}
pub async fn get_till_accounts_by_business_id(
    ctx: &AppContext,
    business_id: u32,
) -> Result<Vec<TillAccount>> {
    TillAccount::get_by_business_id(&ctx.db, business_id).await
}
pub async fn update_till_account(
    ctx: &AppContext,
    id: u32,
    input: UpdateTillAccount,
) -> Result<Option<TillAccount>> {
    TillAccount::update(&ctx.db, id, input).await
}
pub async fn delete_till_account(ctx: &AppContext, id: u32) -> Result<bool> {
    TillAccount::delete(&ctx.db, id).await
}
