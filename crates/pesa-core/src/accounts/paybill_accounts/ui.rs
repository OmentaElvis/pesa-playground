use anyhow::{Context, Result};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, Set, TransactionTrait};

use crate::accounts::paybill_accounts::PaybillAccountDetails;
use crate::AppContext;

use super::db;
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

    let new_paybill = db::ActiveModel {
        business_id: Set(input.business_id),
        paybill_number: Set(input.paybill_number),
        response_type: Set(input.response_type.map(|res| res.to_string())),
        validation_url: Set(input.validation_url),
        confirmation_url: Set(input.confirmation_url),
        created_at: Set(Utc::now().to_utc()),
        ..Default::default()
    };

    let paybill_model = &new_paybill
        .insert(&txn)
        .await
        .context("Failed to create new paybill account")?;

    txn.commit()
        .await
        .context("Failed to complete transaction")?;

    let paybill: PaybillAccount = paybill_model.into();

    Ok(paybill)
}
pub async fn get_paybill_account(ctx: &AppContext, id: u32) -> Result<PaybillAccountDetails> {
    let db = &ctx.db;

    let paybill_account = db::Entity::find_by_id(id)
        .into_model::<PaybillAccountDetails>()
        .one(db)
        .await
        .context(format!("Failed to fetch paybill account with ID {}", id))?
        .ok_or_else(|| anyhow::anyhow!("Paybill account with ID {} not found", id))?;

    Ok(paybill_account)
}
pub async fn get_paybill_accounts(ctx: &AppContext) -> Result<Vec<PaybillAccountDetails>> {
    let db = &ctx.db;

    let paybill_accounts = db::Entity::find()
        .into_model::<PaybillAccountDetails>()
        .all(db)
        .await
        .context("Failed to fetch paybill accounts")?;

    Ok(paybill_accounts)
}
pub async fn get_paybill_accounts_by_business_id(
    ctx: &AppContext,
    business_id: u32,
) -> Result<Vec<PaybillAccountDetails>> {
    use crate::accounts::paybill_accounts::db::Column;
    use sea_orm::QueryFilter;

    let db = &ctx.db;

    let paybill_accounts = db::Entity::find()
        .filter(Column::BusinessId.eq(business_id))
        .into_model::<PaybillAccountDetails>()
        .all(db)
        .await
        .context(format!(
            "Failed to fetch paybill accounts for business {}",
            business_id
        ))?;

    Ok(paybill_accounts)
}
pub async fn update_paybill_account(
    ctx: &AppContext,
    id: u32,
    input: UpdatePaybillAccount,
) -> Result<Option<PaybillAccount>> {
    let db = &ctx.db;
    let paybill_account = db::Entity::find_by_id(id)
        .one(db)
        .await
        .context(format!("Failed to fetch paybill account with ID {}", id))?
        .ok_or_else(|| anyhow::anyhow!("Paybill account with ID {} not found", id))?;

    let mut active_model: db::ActiveModel = paybill_account.into();

    if let Some(business_id) = input.business_id {
        active_model.business_id = Set(business_id);
    }
    if let Some(paybill_number) = input.paybill_number {
        active_model.paybill_number = Set(paybill_number);
    }
    if let Some(validation_url) = input.validation_url {
        active_model.validation_url = Set(Some(validation_url));
    }
    if let Some(confirmation_url) = input.confirmation_url {
        active_model.confirmation_url = Set(Some(confirmation_url));
    }
    if let Some(response_type) = &input.response_type {
        active_model.response_type = Set(Some(response_type.to_string()));
    }

    let updated_paybill_account = active_model
        .update(db)
        .await
        .context(format!("Failed to update paybill account {}", id))?;

    Ok(Some(PaybillAccount {
        id: updated_paybill_account.id,
        business_id: updated_paybill_account.business_id,
        paybill_number: updated_paybill_account.paybill_number,
        validation_url: updated_paybill_account.validation_url,
        confirmation_url: updated_paybill_account.confirmation_url,
        response_type: input.response_type,
    }))
}
pub async fn delete_paybill_account(state: &AppContext, id: u32) -> Result<bool> {
    let db = &state.db;
    let result = db::Entity::delete_by_id(id)
        .exec(db)
        .await
        .context(format!("Failed to delete paybill account with ID {}", id))?;

    Ok(result.rows_affected > 0)
}
