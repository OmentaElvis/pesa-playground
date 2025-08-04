use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, JoinType, QuerySelect, RelationTrait, Set,
    TransactionTrait,
};
use tauri::State;

use crate::accounts::AccountType;
use crate::db::Database;
use crate::transactions::Ledger;

use super::db;
use super::{CreatePaybillAccount, PaybillAccount, PaybillAccountDetails, UpdatePaybillAccount};

#[tauri::command]
pub async fn create_paybill_account(
    state: State<'_, Database>,
    input: CreatePaybillAccount,
) -> Result<PaybillAccount, String> {
    let txn = state
        .conn
        .begin()
        .await
        .map_err(|err| format!("Failed to start transaction: {}", err))?;

    let new_account = crate::accounts::db::ActiveModel {
        account_type: Set(AccountType::Paybill.to_string()),
        balance: Set(0),
        disabled: Set(false),
        created_at: Set(Utc::now().to_utc()),
        ..Default::default()
    };

    let account_model = new_account
        .insert(&txn)
        .await
        .map_err(|err| format!("Failed to create new account: {}", err))?;

    let new_paybill = db::ActiveModel {
        account_id: Set(account_model.id),
        business_id: Set(input.business_id),
        paybill_number: Set(input.paybill_number),
        account_validation_regex: Set(input.account_validation_regex),
        validation_url: Set(input.validation_url),
        confirmation_url: Set(input.confirmation_url),
    };

    let paybill_model = &new_paybill
        .insert(&txn)
        .await
        .map_err(|err| format!("Failed to create new paybill account: {}", err))?;

    Ledger::transfer(
        &txn,
        None,
        paybill_model.account_id,
        input.initial_balance,
        &crate::transactions::TransactionType::Deposit,
    )
    .await
    .map_err(|err| {
        format!(
            "Failed to deposit funds to new account({}): {}",
            paybill_model.account_id, err
        )
    })?;

    txn.commit()
        .await
        .map_err(|err| format!("Failed to complete transaction: {}", err))?;

    let paybill: PaybillAccount = paybill_model.into();

    Ok(paybill)
}

#[tauri::command]
pub async fn get_paybill_account(
    state: State<'_, Database>,
    id: u32,
) -> Result<PaybillAccountDetails, String> {
    let db = &state.conn;

    let paybill_account = db::Entity::find_by_id(id)
        .join(JoinType::InnerJoin, db::Relation::Account.def())
        .select_only()
        .column(db::Column::AccountId)
        .column(db::Column::BusinessId)
        .column(db::Column::PaybillNumber)
        .column(db::Column::AccountValidationRegex)
        .column(db::Column::ValidationUrl)
        .column(db::Column::ConfirmationUrl)
        .column(crate::accounts::db::Column::Balance)
        .column(crate::accounts::db::Column::CreatedAt)
        .column(crate::accounts::db::Column::AccountType)
        .into_model::<PaybillAccountDetails>()
        .one(db)
        .await
        .map_err(|err| format!("Failed to fetch paybill account with ID {}: {}", id, err))?
        .ok_or_else(|| format!("Paybill account with ID {} not found", id))?;

    Ok(paybill_account)
}

#[tauri::command]
pub async fn get_paybill_accounts(
    state: State<'_, Database>,
) -> Result<Vec<PaybillAccountDetails>, String> {
    let db = &state.conn;

    let paybill_accounts = db::Entity::find()
        .join(JoinType::InnerJoin, db::Relation::Account.def())
        .select_only()
        .column(db::Column::AccountId)
        .column(db::Column::BusinessId)
        .column(db::Column::PaybillNumber)
        .column(db::Column::AccountValidationRegex)
        .column(db::Column::ValidationUrl)
        .column(db::Column::ConfirmationUrl)
        .column(crate::accounts::db::Column::Balance)
        .column(crate::accounts::db::Column::CreatedAt)
        .column(crate::accounts::db::Column::AccountType)
        .into_model::<PaybillAccountDetails>()
        .all(db)
        .await
        .map_err(|err| format!("Failed to fetch paybill accounts: {}", err))?;

    Ok(paybill_accounts)
}

#[tauri::command]
pub async fn get_paybill_accounts_by_business_id(
    state: State<'_, Database>,
    business_id: u32,
) -> Result<Vec<PaybillAccountDetails>, String> {
    use crate::accounts::paybill_accounts::db::Column;
    use sea_orm::QueryFilter;

    let db = &state.conn;

    let paybill_accounts = db::Entity::find()
        .filter(Column::BusinessId.eq(business_id))
        .join(JoinType::InnerJoin, db::Relation::Account.def())
        .select_only()
        .column(db::Column::AccountId)
        .column(db::Column::BusinessId)
        .column(db::Column::PaybillNumber)
        .column(db::Column::AccountValidationRegex)
        .column(db::Column::ValidationUrl)
        .column(db::Column::ConfirmationUrl)
        .column(crate::accounts::db::Column::Balance)
        .column(crate::accounts::db::Column::CreatedAt)
        .column(crate::accounts::db::Column::AccountType)
        .into_model::<PaybillAccountDetails>()
        .all(db)
        .await
        .map_err(|err| {
            format!(
                "Failed to fetch paybill accounts for business {}: {}",
                business_id, err
            )
        })?;

    Ok(paybill_accounts)
}

#[tauri::command]
pub async fn update_paybill_account(
    state: State<'_, Database>,
    id: u32,
    input: UpdatePaybillAccount,
) -> Result<Option<PaybillAccount>, String> {
    let db = &state.conn;
    let paybill_account = db::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|err| format!("Failed to fetch paybill account with ID {}: {}", id, err))?
        .ok_or_else(|| format!("Paybill account with ID {} not found", id))?;

    let mut active_model: db::ActiveModel = paybill_account.into();

    if let Some(business_id) = input.business_id {
        active_model.business_id = Set(business_id);
    }
    if let Some(paybill_number) = input.paybill_number {
        active_model.paybill_number = Set(paybill_number);
    }
    if let Some(account_validation_regex) = input.account_validation_regex {
        active_model.account_validation_regex = Set(Some(account_validation_regex));
    }
    if let Some(validation_url) = input.validation_url {
        active_model.validation_url = Set(Some(validation_url));
    }
    if let Some(confirmation_url) = input.confirmation_url {
        active_model.confirmation_url = Set(Some(confirmation_url));
    }

    let updated_paybill_account = active_model
        .update(db)
        .await
        .map_err(|err| format!("Failed to update paybill account {}: {}", id, err))?;

    Ok(Some(PaybillAccount {
        account_id: updated_paybill_account.account_id,
        business_id: updated_paybill_account.business_id,
        paybill_number: updated_paybill_account.paybill_number,
        account_validation_regex: updated_paybill_account.account_validation_regex,
        validation_url: updated_paybill_account.validation_url,
        confirmation_url: updated_paybill_account.confirmation_url,
    }))
}

#[tauri::command]
pub async fn delete_paybill_account(state: State<'_, Database>, id: u32) -> Result<bool, String> {
    let db = &state.conn;
    let result = db::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| format!("Failed to delete paybill account with ID {}: {}", id, e))?;

    Ok(result.rows_affected > 0)
}
