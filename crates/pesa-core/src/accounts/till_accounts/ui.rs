use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, JoinType, QuerySelect, RelationTrait, Set,
    TransactionTrait,
};

use crate::accounts::{self, AccountType};
use crate::server::api::c2b::ResponseType;
use crate::transactions::Ledger;
use crate::AppContext;

use super::{db, CreateTillAccount, TillAccount, TillAccountDetails, UpdateTillAccount};

pub async fn create_till_account(db: &AppContext, input: CreateTillAccount) -> Result<TillAccount> {
    let txn = db.db.begin().await.context("Failed to start transaction")?;

    let new_account = accounts::db::ActiveModel {
        account_type: Set(AccountType::Till.to_string()),
        balance: Set(0),
        disabled: Set(false),
        created_at: Set(Utc::now().to_utc()),
        ..Default::default()
    };

    let account_model = new_account
        .insert(&txn)
        .await
        .context("Failed to create new account")?;

    let new_till = db::ActiveModel {
        account_id: Set(account_model.id),
        business_id: Set(input.business_id),
        till_number: Set(input.till_number),
        location_description: Set(input.location_description.clone()),
        response_type: Set(input.response_type.map(|res| res.to_string())),
        validation_url: Set(input.validation_url),
        confirmation_url: Set(input.confirmation_url),
    };

    let till_model = &new_till
        .insert(&txn)
        .await
        .context("Failed to create new till number")?;

    Ledger::transfer(
        &txn,
        None,
        till_model.account_id,
        input.initial_balance * 100,
        &crate::transactions::TransactionType::Deposit,
    )
    .await
    .context(format!(
        "Failed to deposit funds to new account({})",
        till_model.account_id
    ))?;

    txn.commit()
        .await
        .context("Failed to complete transaction")?;

    let till: TillAccount = till_model.into();

    Ok(till)
}

pub async fn get_till_account(state: &AppContext, id: u32) -> Result<TillAccountDetails> {
    let db = &state.db;

    let till_account = db::Entity::find_by_id(id)
        .join(JoinType::InnerJoin, db::Relation::Account.def())
        .select_only()
        .column(db::Column::AccountId)
        .column(db::Column::BusinessId)
        .column(db::Column::TillNumber)
        .column(db::Column::LocationDescription)
        .column(crate::accounts::db::Column::Balance)
        .column(crate::accounts::db::Column::CreatedAt)
        .column(crate::accounts::db::Column::AccountType)
        .into_model::<TillAccountDetails>()
        .one(db)
        .await
        .context(format!("Failed to fetch till account with ID {}", id))?
        .ok_or_else(|| anyhow::anyhow!("Till account with ID {} not found", id))?;

    Ok(till_account)
}

pub async fn get_till_accounts(ctx: &AppContext) -> Result<Vec<TillAccountDetails>> {
    let db = &ctx.db;

    let till_accounts = db::Entity::find()
        .join(JoinType::InnerJoin, db::Relation::Account.def())
        .select_only()
        .column(db::Column::AccountId)
        .column(db::Column::BusinessId)
        .column(db::Column::TillNumber)
        .column(db::Column::LocationDescription)
        .column(crate::accounts::db::Column::Balance)
        .column(crate::accounts::db::Column::CreatedAt)
        .column(crate::accounts::db::Column::AccountType)
        .into_model::<TillAccountDetails>()
        .all(db)
        .await
        .context("Failed to fetch till accounts")?;

    Ok(till_accounts)
}

pub async fn get_till_accounts_by_business_id(
    ctx: &AppContext,
    business_id: u32,
) -> Result<Vec<TillAccountDetails>> {
    use crate::accounts::till_accounts::db::Column;
    use sea_orm::QueryFilter;

    let db = &ctx.db;

    let till_accounts = db::Entity::find()
        .filter(Column::BusinessId.eq(business_id))
        .join(JoinType::InnerJoin, db::Relation::Account.def())
        .select_only()
        .column(db::Column::AccountId)
        .column(db::Column::BusinessId)
        .column(db::Column::TillNumber)
        .column(db::Column::LocationDescription)
        .column(crate::accounts::db::Column::Balance)
        .column(crate::accounts::db::Column::CreatedAt)
        .column(crate::accounts::db::Column::AccountType)
        .into_model::<TillAccountDetails>()
        .all(db)
        .await
        .context(format!(
            "Failed to fetch till accounts for business {}",
            business_id
        ))?;

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
        account_id: updated_till_account.account_id,
        business_id: updated_till_account.business_id,
        till_number: updated_till_account.till_number,
        location_description: updated_till_account.location_description,
        response_type: updated_till_account
            .response_type
            .map(|r| r.parse().unwrap_or(ResponseType::Cancelled)),
        validation_url: updated_till_account.validation_url,
        confirmation_url: updated_till_account.confirmation_url,
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
