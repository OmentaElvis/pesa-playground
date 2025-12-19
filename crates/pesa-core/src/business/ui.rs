use crate::accounts::mmf_accounts::MmfAccount;
use crate::accounts::utility_accounts::UtilityAccount;
use crate::accounts::{mmf_accounts, utility_accounts};
use crate::transactions::Ledger;
use anyhow::{anyhow, Context, Result};
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, TransactionTrait,
};

use crate::AppContext;

use super::db;
use crate::business::{Business, BusinessSummary, CreateBusiness, UpdateBusiness};

pub async fn create_business(ctx: &AppContext, input: CreateBusiness) -> Result<Business> {
    let txn = ctx.db.begin().await?;

    // verify shortcode is valid
    let shortcodes = db::Entity::find()
        .filter(db::Column::ShortCode.eq(input.short_code.clone()))
        .count(&txn)
        .await?;

    if shortcodes > 0 {
        return Err(anyhow!(
            "Shortcode ({}) is already used by another business.",
            input.short_code
        ));
    }

    // 1. Create a generic business entity
    let created_business = db::ActiveModel {
        name: Set(input.name.clone()),
        short_code: Set(input.short_code),
        charges_amount: Set(0),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .context("Failed to create business")?;

    // 2. Create MMF Account
    mmf_accounts::MmfAccount::create(
        &txn,
        created_business.id,
        (input.initial_working_balance * 100.0) as i64,
    )
    .await?;

    // 3. Create Utility Account
    utility_accounts::UtilityAccount::create(
        &txn,
        created_business.id,
        (input.initial_utility_balance * 100.0) as i64,
    )
    .await?;

    txn.commit().await?;

    Ok(Business {
        id: created_business.id,
        name: created_business.name,
        short_code: created_business.short_code,
        charges_amount: created_business.charges_amount,
    })
}
pub async fn get_business(ctx: &AppContext, id: u32) -> Result<BusinessSummary> {
    let db = &ctx.db;

    // Fetch the business details.
    let business = db::Entity::find_by_id(id)
        .one(db)
        .await
        .context(format!("Failed to fetch business with ID {}", id))?
        .ok_or_else(|| anyhow!("Business with ID {} not found", id))?;

    let mmf = mmf_accounts::MmfAccount::find_by_business_id(db, id)
        .await
        .context("Failed to fetch mmf account")?
        .context(format!("Working account for business({id}) not found."))?;

    let utility = utility_accounts::UtilityAccount::find_by_business_id(db, id)
        .await
        .context("Failed to fetch utility account")?
        .context(format!("Utility account for business({id}) not found."))?;

    // Return the combined business details.
    Ok(BusinessSummary {
        id: business.id,
        name: business.name.clone(),
        short_code: business.short_code.clone(),
        charges_amount: business.charges_amount,
        mmf_account: mmf,
        utility_account: utility,
    })
}
pub async fn get_businesses(ctx: &AppContext) -> Result<Vec<Business>> {
    let db = &ctx.db;

    let businesses = db::Entity::find()
        .all(db)
        .await
        .context("Failed to fetch businesses")?;

    // Map the rows to `BusinessSummary` structs.
    let businesses: Vec<Business> = businesses
        .iter()
        .map(|business| Business {
            id: business.id,
            name: business.name.clone(),
            short_code: business.short_code.clone(),
            charges_amount: business.charges_amount,
        })
        .collect();

    Ok(businesses)
}
pub async fn update_business(
    ctx: &AppContext,
    id: u32,
    input: UpdateBusiness,
) -> Result<Option<Business>> {
    let db = &ctx.db;
    let business = db::Entity::find_by_id(id)
        .one(db)
        .await
        .context(format!("Failed to fetch business with ID {}", id))?
        .ok_or_else(|| anyhow!("Business with ID {} not found", id))?;

    let mut active_model: db::ActiveModel = business.into();

    if let Some(name) = input.name {
        active_model.name = Set(name.clone());
    }
    if let Some(short_code) = input.short_code {
        active_model.short_code = Set(short_code.clone());
    }

    let updated_business = active_model
        .update(db)
        .await
        .context(format!("Failed to update business {}", id))?;

    Ok(Some(Business {
        id: updated_business.id,
        name: updated_business.name,
        short_code: updated_business.short_code,
        charges_amount: updated_business.charges_amount,
    }))
}
pub async fn delete_business(ctx: &AppContext, id: u32) -> Result<bool> {
    let db = &ctx.db;
    let result = db::Entity::delete_by_id(id)
        .exec(db)
        .await
        .context(format!("Failed to delete business with ID {}", id))?;

    Ok(result.rows_affected > 0)
}

pub async fn revenue_settlement(ctx: &AppContext, business_id: u32) -> Result<()> {
    let txn = ctx.db.begin().await?;
    // 1. Fetch the business
    let business_model = db::Entity::find_by_id(business_id)
        .one(&txn)
        .await?
        .ok_or_else(|| anyhow!("Business with ID {} not found", business_id))?;

    // 2. Fetch associated accounts
    let utility_account = UtilityAccount::find_by_business_id(&txn, business_id)
        .await?
        .context(format!(
            "Failed to find utility account for business with id: {}",
            business_id
        ))?;

    // 3. Settle Charges Paid Account (if negative)
    if business_model.charges_amount < 0 {
        let amount_to_settle = -business_model.charges_amount as i64;
        Ledger::transfer(
            &txn,
            Some(utility_account.account_id),
            0,
            amount_to_settle,
            &crate::transactions::TransactionType::ChargeSettlement,
            None,
        )
        .await?;
    }

    // 4. Sweep remaining balance from Utility Account to MMF Account
    let updated_utility_account = UtilityAccount::find_by_id(&txn, utility_account.account_id)
        .await?
        .context(format!(
            "Utility account with ID {} not found after charge settlement",
            utility_account.account_id
        ))?;

    let mmf_account = MmfAccount::find_by_business_id(&txn, business_id)
        .await?
        .context(format!(
            "Failed to get mmf account for business id: {}",
            business_id
        ))?;

    if updated_utility_account.balance > 0 {
        let amount_to_sweep = updated_utility_account.balance as i64;
        Ledger::transfer(
            &txn,
            Some(utility_account.account_id),
            mmf_account.account_id,
            amount_to_sweep,
            &crate::transactions::TransactionType::RevenueSweep,
            None,
        )
        .await?;
    }

    txn.commit().await?;

    Ok(())
}
