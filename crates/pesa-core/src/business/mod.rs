use anyhow::{Context, Result, anyhow};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};

use crate::accounts::paybill_accounts::{CreatePaybillAccount, PaybillAccount};
use crate::accounts::{mmf_accounts::MmfAccount, utility_accounts::UtilityAccount};
use crate::business_operators::BusinessOperator;
use crate::transactions::Ledger;
pub mod db;
pub mod ui;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Business {
    pub id: u32,
    pub name: String,
    pub short_code: String,
    pub charges_amount: i64,
}
impl From<&db::Model> for Business {
    fn from(value: &db::Model) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            short_code: value.short_code.clone(),
            charges_amount: value.charges_amount,
        }
    }
}

impl Business {
    pub async fn create<C>(conn: &C, input: CreateBusiness) -> Result<Business>
    where
        C: ConnectionTrait,
    {
        // verify shortcode is valid
        let shortcodes = db::Entity::find()
            .filter(db::Column::ShortCode.eq(input.short_code.clone()))
            .count(conn)
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
            short_code: Set(input.short_code.clone()),
            charges_amount: Set(0),
            ..Default::default()
        }
        .insert(conn)
        .await
        .context("Failed to create business")?;

        // 2. Create MMF Account
        MmfAccount::create(
            conn,
            created_business.id,
            (input.initial_working_balance * 100.0) as i64,
        )
        .await?;

        // 3. Create Utility Account
        UtilityAccount::create(
            conn,
            created_business.id,
            (input.initial_utility_balance * 100.0) as i64,
        )
        .await?;

        // 4. Create the default 'admin' operator for this new business
        BusinessOperator::create(
            conn,
            "admin".to_string(),
            "admin".to_string(),
            created_business.id,
        )
        .await?;

        PaybillAccount::create(
            conn,
            CreatePaybillAccount {
                business_id: created_business.id,
                paybill_number: created_business
                    .short_code
                    .parse()
                    .context("Expected a numeric shortcode")?,
                response_type: Some(crate::server::api::c2b::ResponseType::Completed),
                validation_url: None,
                confirmation_url: None,
            },
        )
        .await
        .context("Failed to create default paybill")?;

        Ok(Business {
            id: created_business.id,
            name: created_business.name,
            short_code: created_business.short_code,
            charges_amount: created_business.charges_amount,
        })
    }
    pub async fn get_all<C>(conn: &C) -> Result<Vec<Business>>
    where
        C: ConnectionTrait,
    {
        let businesses = db::Entity::find()
            .all(conn)
            .await
            .context("Failed to fetch businesses")?;

        // Map the rows to `BusinessSummary` structs.
        let businesses: Vec<Business> = businesses.iter().map(|b| b.into()).collect();

        Ok(businesses)
    }
    pub async fn update<C>(conn: &C, id: u32, input: UpdateBusiness) -> Result<Option<Business>>
    where
        C: ConnectionTrait,
    {
        let business = db::Entity::find_by_id(id)
            .one(conn)
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
            .update(conn)
            .await
            .context(format!("Failed to update business {}", id))?;

        Ok(Some((&updated_business).into()))
    }
    pub async fn delete<C>(conn: &C, id: u32) -> Result<bool>
    where
        C: ConnectionTrait,
    {
        let result = db::Entity::delete_by_id(id)
            .exec(conn)
            .await
            .context(format!("Failed to delete business with ID {}", id))?;

        Ok(result.rows_affected > 0)
    }
    pub async fn get_summary<C>(conn: &C, id: u32) -> Result<BusinessSummary>
    where
        C: ConnectionTrait,
    {
        // Fetch the business details.
        let business = db::Entity::find_by_id(id)
            .one(conn)
            .await
            .context(format!("Failed to fetch business with ID {}", id))?
            .ok_or_else(|| anyhow!("Business with ID {} not found", id))?;

        let mmf = MmfAccount::find_by_business_id(conn, id)
            .await
            .context("Failed to fetch mmf account")?
            .context(format!("Working account for business({id}) not found."))?;

        let utility = UtilityAccount::find_by_business_id(conn, id)
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
    pub async fn settle_revenue<C>(conn: &C, business_id: u32) -> Result<()>
    where
        C: ConnectionTrait,
    {
        // 1. Fetch the business
        let business_model = db::Entity::find_by_id(business_id)
            .one(conn)
            .await?
            .ok_or_else(|| anyhow!("Business with ID {} not found", business_id))?;

        // 2. Fetch associated accounts
        let utility_account = UtilityAccount::find_by_business_id(conn, business_id)
            .await?
            .context(format!(
                "Failed to find utility account for business with id: {}",
                business_id
            ))?;

        // 3. Settle Charges Paid Account (if negative)
        if business_model.charges_amount < 0 {
            let amount_to_settle = -business_model.charges_amount as i64;
            Ledger::transfer(
                conn,
                Some(utility_account.account_id),
                0,
                amount_to_settle,
                &crate::transactions::TransactionType::ChargeSettlement,
                None,
            )
            .await?;
        }

        // 4. Sweep remaining balance from Utility Account to MMF Account
        let updated_utility_account = UtilityAccount::find_by_id(conn, utility_account.account_id)
            .await?
            .context(format!(
                "Utility account with ID {} not found after charge settlement",
                utility_account.account_id
            ))?;

        let mmf_account = MmfAccount::find_by_business_id(conn, business_id)
            .await?
            .context(format!(
                "Failed to get mmf account for business id: {}",
                business_id
            ))?;

        if updated_utility_account.balance > 0 {
            let amount_to_sweep = updated_utility_account.balance as i64;
            Ledger::transfer(
                conn,
                Some(utility_account.account_id),
                mmf_account.account_id,
                amount_to_sweep,
                &crate::transactions::TransactionType::RevenueSweep,
                None,
            )
            .await?;
        }
        Ok(())
    }

    pub async fn get_by_id<C>(conn: &C, id: u32) -> Result<Option<Business>>
    where
        C: ConnectionTrait,
    {
        let business = db::Entity::find_by_id(id).one(conn).await?;
        Ok(business.as_ref().map(|b| b.into()))
    }

    pub async fn get_by_short_code<C>(conn: &C, short_code: &str) -> Result<Option<Business>>
    where
        C: ConnectionTrait,
    {
        let business = db::Entity::find()
            .filter(db::Column::ShortCode.eq(short_code))
            .one(conn)
            .await?;
        Ok(business.as_ref().map(|b| b.into()))
    }
    pub async fn increment_charges_amount<C>(
        conn: &C,
        id: u32,
        amount: i64,
    ) -> Result<Business, DbErr>
    where
        C: ConnectionTrait,
    {
        let business = if let Some(business) = db::Entity::find_by_id(id).one(conn).await? {
            let current_amount = business.charges_amount;
            let mut business = business.into_active_model();
            business.charges_amount = Set(current_amount + amount);
            business.update(conn).await?
        } else {
            return Err(DbErr::Custom("Business not found".to_string()));
        };

        Ok(Business {
            id: business.id,
            name: business.name,
            short_code: business.short_code,
            charges_amount: business.charges_amount,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateBusiness {
    pub name: String,
    pub short_code: String,
    pub initial_working_balance: f64,
    pub initial_utility_balance: f64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UpdateBusiness {
    pub name: Option<String>,
    pub short_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BusinessDetails {
    pub id: u32,
    pub name: String,
    pub short_code: String,
    pub mmf_account_id: u32,
    pub utility_account_id: u32,
    pub charges_amount: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BusinessSummary {
    pub id: u32,
    pub name: String,
    pub short_code: String,

    pub mmf_account: MmfAccount,
    pub utility_account: UtilityAccount,
    pub charges_amount: i64,
}
