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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::business::{Business, CreateBusiness};
    use crate::tests::TestDb;

    async fn setup_test_business(db: &TestDb) -> u32 {
        let input = CreateBusiness {
            name: "Test Business".to_string(),
            short_code: "123456".to_string(),
            initial_working_balance: 0.0,
            initial_utility_balance: 0.0,
        };
        let business = Business::create(&db.conn, input).await.unwrap();
        business.id
    }

    #[tokio::test]
    async fn test_create_utility_account() {
        let db = TestDb::in_memory().await.unwrap();
        let business_id = setup_test_business(&db).await;

        let result = UtilityAccount::create(&db.conn, business_id, 2000).await;

        assert!(result.is_ok());
        let utility = result.unwrap();
        assert_eq!(utility.business_id, business_id);
        assert_eq!(utility.balance, 2000);
        assert!(utility.account_id > 0);

        // Verify base account exists
        let account = Account::get_account(&db.conn, utility.account_id)
            .await
            .unwrap();
        assert!(account.is_some());
        assert_eq!(account.unwrap().balance, 2000);
    }

    #[tokio::test]
    async fn test_find_utility_by_business_id() {
        let db = TestDb::in_memory().await.unwrap();
        let business_id = setup_test_business(&db).await;
        let utility = UtilityAccount::find_by_business_id(&db.conn, business_id)
            .await
            .unwrap();
        assert!(utility.is_some());
    }

    #[tokio::test]
    async fn test_find_utility_not_found() {
        let db = TestDb::in_memory().await.unwrap();
        let result = UtilityAccount::find_by_id(&db.conn, 999).await.unwrap();
        assert!(result.is_none());
    }
}
