use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, Set, TransactionTrait};

use crate::AppContext;
use crate::server::api::c2b::ResponseType;

use super::{CreateTillAccount, TillAccount, UpdateTillAccount, db};

pub async fn create_till_account(
    ctx: &AppContext,
    input: CreateTillAccount,
) -> Result<TillAccount> {
    let txn = ctx
        .db
        .begin()
        .await
        .context("Failed to start transaction")?;

    let new_till = db::ActiveModel {
        business_id: Set(input.business_id),
        till_number: Set(input.till_number),
        location_description: Set(input.location_description.clone()),
        response_type: Set(input.response_type.map(|res| res.to_string())),
        validation_url: Set(input.validation_url),
        confirmation_url: Set(input.confirmation_url),
        created_at: Set(Utc::now().to_utc()),
        ..Default::default()
    };

    let till_model = &new_till
        .insert(&txn)
        .await
        .context("Failed to create new till number")?;

    txn.commit()
        .await
        .context("Failed to complete transaction")?;

    let till: TillAccount = till_model.into();

    Ok(till)
}
pub async fn get_till_account(state: &AppContext, id: u32) -> Result<TillAccount> {
    let db = &state.db;

    let till_account = &db::Entity::find_by_id(id)
        .one(db)
        .await
        .context(format!("Failed to fetch till account with ID {}", id))?
        .ok_or_else(|| anyhow::anyhow!("Till account with ID {} not found", id))?;

    Ok(till_account.into())
}
pub async fn get_till_accounts(ctx: &AppContext) -> Result<Vec<TillAccount>> {
    let db = &ctx.db;

    let till_accounts = db::Entity::find()
        .all(db)
        .await
        .context("Failed to fetch till accounts")?
        .into_iter()
        .map(|model| (&model).into())
        .collect();

    Ok(till_accounts)
}
pub async fn get_till_accounts_by_business_id(
    ctx: &AppContext,
    business_id: u32,
) -> Result<Vec<TillAccount>> {
    use crate::accounts::till_accounts::db::Column;
    use sea_orm::QueryFilter;

    let db = &ctx.db;

    let till_accounts = db::Entity::find()
        .filter(Column::BusinessId.eq(business_id))
        .all(db)
        .await
        .context(format!(
            "Failed to fetch till accounts for business {}",
            business_id
        ))?
        .into_iter()
        .map(|model| (&model).into())
        .collect();

    Ok(till_accounts)
}
pub async fn update_till_account(
    ctx: &AppContext,
    id: u32,
    input: UpdateTillAccount,
) -> Result<Option<TillAccount>> {
    let db = &ctx.db;
    let till_account = db::Entity::find_by_id(id)
        .one(db)
        .await
        .context(format!("Failed to fetch till account with ID {}", id))?
        .ok_or_else(|| anyhow!("Till account with ID {} not found", id))?;

    let mut active_model: db::ActiveModel = till_account.into();

    if let Some(business_id) = input.business_id {
        active_model.business_id = Set(business_id);
    }
    if let Some(till_number) = input.till_number {
        active_model.till_number = Set(till_number);
    }
    if let Some(location_description) = input.location_description {
        active_model.location_description = Set(Some(location_description.clone()));
    }
    if let Some(response_type) = input.response_type {
        active_model.response_type = Set(Some(response_type.to_string()));
    }
    if let Some(validation_url) = input.validation_url {
        active_model.validation_url = Set(Some(validation_url));
    }
    if let Some(confirmation_url) = input.confirmation_url {
        active_model.confirmation_url = Set(Some(confirmation_url));
    }

    let updated_till_account = active_model
        .update(db)
        .await
        .context(format!("Failed to update till account {}", id))?;

    Ok(Some(TillAccount {
        id: updated_till_account.id,
        business_id: updated_till_account.business_id,
        till_number: updated_till_account.till_number,
        location_description: updated_till_account.location_description,
        response_type: updated_till_account
            .response_type
            .map(|r| r.parse().unwrap_or(ResponseType::Cancelled)),
        validation_url: updated_till_account.validation_url,
        confirmation_url: updated_till_account.confirmation_url,
        created_at: updated_till_account.created_at,
    }))
}
pub async fn delete_till_account(ctx: &AppContext, id: u32) -> Result<bool> {
    let db = &ctx.db;
    let result = db::Entity::delete_by_id(id)
        .exec(db)
        .await
        .context(format!("Failed to delete till account with ID {}", id))?;

    Ok(result.rows_affected > 0)
}
