use chrono::{DateTime, Utc};
use db::Column;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    QueryFilter,
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::transactions::{Ledger, TransactionType};

pub mod db;
pub mod paybill_accounts;
pub mod till_accounts;
pub mod ui;
pub mod user_profiles;

#[derive(EnumString, Display, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    User,
    System,
    Paybill,
    Till,
}

#[derive(Serialize)]
pub struct Account {
    pub id: u32,
    pub account_type: AccountType,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
    pub disabled: bool,
}

impl From<db::Model> for Account {
    fn from(value: db::Model) -> Self {
        Account {
            id: value.id,
            account_type: value.account_type.parse().unwrap_or(AccountType::User),
            balance: value.balance,
            created_at: value.created_at,
            disabled: value.disabled,
        }
    }
}

impl Account {
    pub async fn get_account<C>(conn: &C, id: u32) -> Result<Option<Account>, DbErr>
    where
        C: ConnectionTrait,
    {
        let account = db::Entity::find()
            .filter(Column::Id.eq(id))
            .one(conn)
            .await?;

        Ok(account.map(|acc| acc.into()))
    }

    pub async fn create_account<C>(
        conn: &C,
        account_type: AccountType,
        initial_balance: i64,
    ) -> anyhow::Result<u32>
    where
        C: ConnectionTrait,
    {
        let create = db::ActiveModel {
            account_type: Set(account_type.to_string()),
            balance: Set(0),
            created_at: Set(Utc::now()),
            disabled: Set(false),
            ..Default::default()
        };

        let account = create.insert(conn).await?;

        Ledger::transfer(
            conn,
            None,
            account.id,
            initial_balance,
            &TransactionType::Deposit,
        )
        .await?;
        Ok(account.id)
    }
}
