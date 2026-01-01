use anyhow::{Context, Result};
use sea_orm::TransactionTrait;

use crate::AppContext;
use crate::accounts::paybill_accounts::PaybillAccountDetails;

use super::{CreatePaybillAccount, PaybillAccount, UpdatePaybillAccount};

pub async fn create_paybill_account(
    ctx: &AppContext,
    input: CreatePaybillAccount,
) -> Result<PaybillAccount> {
    let txn = ctx
        .db
        .begin()
        .await
        .context("Failed to start transaction")?;

    let paybill = PaybillAccount::create(&txn, input).await?;

    txn.commit()
        .await
        .context("Failed to complete transaction")?;

    Ok(paybill)
}
pub async fn get_paybill_account(ctx: &AppContext, id: u32) -> Result<PaybillAccountDetails> {
    PaybillAccount::get_by_id(&ctx.db, id).await
}
pub async fn get_paybill_accounts(ctx: &AppContext) -> Result<Vec<PaybillAccountDetails>> {
    PaybillAccount::get_all(&ctx.db).await
}
pub async fn get_paybill_accounts_by_business_id(
    ctx: &AppContext,
    business_id: u32,
) -> Result<Vec<PaybillAccountDetails>> {
    PaybillAccount::get_by_business_id(&ctx.db, business_id).await
}
pub async fn update_paybill_account(
    ctx: &AppContext,
    id: u32,
    input: UpdatePaybillAccount,
) -> Result<Option<PaybillAccount>> {
    PaybillAccount::update(&ctx.db, id, input).await
}
pub async fn delete_paybill_account(state: &AppContext, id: u32) -> Result<bool> {
    PaybillAccount::delete(&state.db, id).await
}
