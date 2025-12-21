use crate::accounts::{self, Account, AccountType};
use anyhow::Context;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    FromQueryResult, QueryFilter, QuerySelect, RelationTrait, SelectColumns,
};
use serde::{Deserialize, Serialize};

pub use db::Entity as UtilityAccountEntity;

pub mod db;
pub mod ui;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
pub struct UtilityAccount {
    pub account_id: u32,
    pub business_id: u32,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
    pub disabled: bool,
}

impl UtilityAccount {
    pub async fn find_by_id<C>(db: &C, id: u32) -> Result<Option<Self>, DbErr>
    where
        C: ConnectionTrait,
    {
        if let Some(account) = db::Entity::find_by_id(id)
            .join(sea_orm::JoinType::InnerJoin, db::Relation::Account.def())
            .select_column(db::Column::AccountId)
            .select_column(db::Column::BusinessId)
            .select_column(accounts::db::Column::Balance)
            .select_column(accounts::db::Column::AccountType)
            .select_column(accounts::db::Column::CreatedAt)
            .select_column(accounts::db::Column::Disabled)
            .into_model::<UtilityAccount>()
            .one(db)
            .await?
        {
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    pub async fn create<C>(db: &C, business_id: u32, initial_balance: i64) -> anyhow::Result<Self>
    where
        C: ConnectionTrait,
    {
        let account = Account::create_account(db, AccountType::Utility, initial_balance)
            .await
            .context("Failed to create new account for Utility")?;

        let new_utility = db::ActiveModel {
            account_id: Set(account.id),
            business_id: Set(business_id),
        };

        let utility_model = &new_utility
            .insert(db)
            .await
            .context("Failed to create new Utility account")?;

        Ok(UtilityAccount {
            account_id: utility_model.account_id,
            business_id: utility_model.business_id,
            balance: initial_balance,
            created_at: account.created_at,
            disabled: account.disabled,
        })
    }

    pub async fn find_by_business_id<C>(db: &C, biz_id: u32) -> Result<Option<Self>, DbErr>
    where
        C: ConnectionTrait,
    {
        let utility = db::Entity::find()
            .filter(db::Column::BusinessId.eq(biz_id))
            .join(sea_orm::JoinType::InnerJoin, db::Relation::Account.def())
            .select_column(db::Column::AccountId)
            .select_column(db::Column::BusinessId)
            .select_column(accounts::db::Column::Balance)
            .select_column(accounts::db::Column::AccountType)
            .select_column(accounts::db::Column::CreatedAt)
            .select_column(accounts::db::Column::Disabled)
            .into_model::<UtilityAccount>()
            .one(db)
            .await?;

        Ok(utility)
    }
}
