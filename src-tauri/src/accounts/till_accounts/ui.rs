use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, JoinType, QuerySelect, RelationTrait, Set,
    TransactionTrait,
};
use tauri::State;

use crate::accounts::{self, AccountType};
use crate::db::Database;
use crate::transactions::Ledger;

use super::{db, CreateTillAccount, TillAccount, TillAccountDetails, UpdateTillAccount};

#[tauri::command]
pub async fn create_till_account(
    state: State<'_, Database>,
    input: CreateTillAccount,
) -> Result<TillAccount, String> {
    let txn = state
        .conn
        .begin()
        .await
        .map_err(|err| format!("Failed to start transaction: {}", err))?;

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
        .map_err(|err| format!("Failed to create new account: {}", err))?;

    let new_till = db::ActiveModel {
        account_id: Set(account_model.id),
        business_id: Set(input.business_id),
        till_number: Set(input.till_number),
        store_number: Set(input.store_number),
        location_description: Set(input.location_description.clone()),
    };

    let till_model = &new_till
        .insert(&txn)
        .await
        .map_err(|err| format!("Failed to create new till number: {}", err))?;

    Ledger::transfer(
        &txn,
        None,
        till_model.account_id,
        input.initial_balance,
        &crate::transactions::TransactionType::Deposit,
    )
    .await
    .map_err(|err| {
        format!(
            "Failed to deposit funds to new account({}): {}",
            till_model.account_id, err
        )
    })?;

    txn.commit()
        .await
        .map_err(|err| format!("Failed to complete transaction: {}", err))?;

    let till: TillAccount = till_model.into();

    Ok(till)
}

#[tauri::command]
pub async fn get_till_account(
    state: State<'_, Database>,
    id: u32,
) -> Result<TillAccountDetails, String> {
    let db = &state.conn;

    let till_account = db::Entity::find_by_id(id)
        .join(JoinType::InnerJoin, db::Relation::Account.def())
        .select_only()
        .column(db::Column::AccountId)
        .column(db::Column::BusinessId)
        .column(db::Column::TillNumber)
        .column(db::Column::StoreNumber)
        .column(db::Column::LocationDescription)
        .column(crate::accounts::db::Column::Balance)
        .column(crate::accounts::db::Column::CreatedAt)
        .column(crate::accounts::db::Column::AccountType)
        .into_model::<TillAccountDetails>()
        .one(db)
        .await
        .map_err(|err| format!("Failed to fetch till account with ID {}: {}", id, err))?
        .ok_or_else(|| format!("Till account with ID {} not found", id))?;

    Ok(till_account)
}

#[tauri::command]
pub async fn get_till_accounts(
    state: State<'_, Database>,
) -> Result<Vec<TillAccountDetails>, String> {
    let db = &state.conn;

    let till_accounts = db::Entity::find()
        .join(JoinType::InnerJoin, db::Relation::Account.def())
        .select_only()
        .column(db::Column::AccountId)
        .column(db::Column::BusinessId)
        .column(db::Column::TillNumber)
        .column(db::Column::StoreNumber)
        .column(db::Column::LocationDescription)
        .column(crate::accounts::db::Column::Balance)
        .column(crate::accounts::db::Column::CreatedAt)
        .column(crate::accounts::db::Column::AccountType)
        .into_model::<TillAccountDetails>()
        .all(db)
        .await
        .map_err(|err| format!("Failed to fetch till accounts: {}", err))?;

    Ok(till_accounts)
}

#[tauri::command]
pub async fn get_till_accounts_by_business_id(
    state: State<'_, Database>,
    business_id: u32,
) -> Result<Vec<TillAccountDetails>, String> {
    use crate::accounts::till_accounts::db::Column;
    use sea_orm::QueryFilter;

    let db = &state.conn;

    let till_accounts = db::Entity::find()
        .filter(Column::BusinessId.eq(business_id))
        .join(JoinType::InnerJoin, db::Relation::Account.def())
        .select_only()
        .column(db::Column::AccountId)
        .column(db::Column::BusinessId)
        .column(db::Column::TillNumber)
        .column(db::Column::StoreNumber)
        .column(db::Column::LocationDescription)
        .column(crate::accounts::db::Column::Balance)
        .column(crate::accounts::db::Column::CreatedAt)
        .column(crate::accounts::db::Column::AccountType)
        .into_model::<TillAccountDetails>()
        .all(db)
        .await
        .map_err(|err| {
            format!(
                "Failed to fetch till accounts for business {}: {}",
                business_id, err
            )
        })?;

    Ok(till_accounts)
}

#[tauri::command]
pub async fn update_till_account(
    state: State<'_, Database>,
    id: u32,
    input: UpdateTillAccount,
) -> Result<Option<TillAccount>, String> {
    let db = &state.conn;
    let till_account = db::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|err| format!("Failed to fetch till account with ID {}: {}", id, err))?
        .ok_or_else(|| format!("Till account with ID {} not found", id))?;

    let mut active_model: db::ActiveModel = till_account.into();

    if let Some(business_id) = input.business_id {
        active_model.business_id = Set(business_id);
    }
    if let Some(till_number) = input.till_number {
        active_model.till_number = Set(till_number);
    }
    if let Some(store_number) = input.store_number {
        active_model.store_number = Set(store_number);
    }
    if let Some(location_description) = input.location_description {
        active_model.location_description = Set(Some(location_description.clone()));
    }

    let updated_till_account = active_model
        .update(db)
        .await
        .map_err(|err| format!("Failed to update till account {}: {}", id, err))?;

    Ok(Some(TillAccount {
        account_id: updated_till_account.account_id,
        business_id: updated_till_account.business_id,
        till_number: updated_till_account.till_number,
        store_number: updated_till_account.store_number,
        location_description: updated_till_account.location_description,
    }))
}

#[tauri::command]
pub async fn delete_till_account(state: State<'_, Database>, id: u32) -> Result<bool, String> {
    let db = &state.conn;
    let result = db::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| format!("Failed to delete till account with ID {}: {}", id, e))?;

    Ok(result.rows_affected > 0)
}
