pub mod db;
pub mod ui;

use sea_orm::prelude::DateTimeUtc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, Set,
};
use sea_orm::{PaginatorTrait, QuerySelect};
use serde::Serialize;

use self::db::{ActiveModel, Direction};
use crate::accounts::{self, Account};
use crate::transactions;

#[derive(Serialize)]
pub struct TransactionLog {
    pub id: i32,
    pub transaction_id: String,
    pub account_id: i32,
    pub direction: Direction,
    pub new_balance: i64,
}

#[derive(Serialize, Clone)]
pub struct FullTransactionLog {
    pub transaction_id: String,
    pub transaction_date: DateTimeUtc,
    pub transaction_amount: i64,
    pub transaction_type: String,
    pub from_name: String,
    pub to_name: String,
    pub from_id: Option<u32>,
    pub to_id: u32,
    pub new_balance: i64,
    pub status: String,
    pub fee: i64,
    pub direction: Direction,
}

impl From<db::Model> for TransactionLog {
    fn from(model: db::Model) -> Self {
        Self {
            id: model.id,
            transaction_id: model.transaction_id,
            account_id: model.account_id,
            direction: model.direction,
            new_balance: model.new_balance,
        }
    }
}

impl TransactionLog {
    pub async fn create<C>(
        db: &C,
        transaction_id: String,
        account_id: i32,
        direction: Direction,
        new_balance: i64,
    ) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let log = ActiveModel {
            transaction_id: Set(transaction_id),
            account_id: Set(account_id),
            direction: Set(direction),
            new_balance: Set(new_balance),
            ..Default::default()
        };

        let log = log.insert(db).await?;

        Ok(log.into())
    }

    pub async fn get_full_log<C>(
        db: &C,
        transaction_log_id: i32,
    ) -> Result<Option<FullTransactionLog>, DbErr>
    where
        C: ConnectionTrait,
    {
        if let Some(log) = db::Entity::find_by_id(transaction_log_id).one(db).await? {
            if let Some(transaction) = transactions::db::Entity::find_by_id(&log.transaction_id)
                .one(db)
                .await?
            {
                let from_name = if let Some(from_id) = transaction.from {
                    get_account_name(db, from_id).await?
                } else {
                    "System".to_string()
                };

                let to_name = get_account_name(db, transaction.to).await?;

                return Ok(Some(FullTransactionLog {
                    transaction_id: transaction.id,
                    transaction_date: transaction.created_at,
                    transaction_amount: transaction.amount,
                    transaction_type: transaction.transaction_type,
                    from_name,
                    to_name,
                    from_id: transaction.from,
                    to_id: transaction.to,
                    new_balance: log.new_balance,
                    status: transaction.status,
                    fee: transaction.fee,
                    direction: log.direction,
                }));
            }
        }
        Ok(None)
    }

    pub async fn list_full_logs<C>(
        db: &C,
        account_id: i32,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<FullTransactionLog>, DbErr>
    where
        C: ConnectionTrait,
    {
        let logs = db::Entity::find()
            .filter(db::Column::AccountId.eq(account_id))
            .limit(limit)
            .offset(offset)
            .all(db)
            .await?;

        let mut full_logs = Vec::new();

        for log in logs {
            if let Some(full_log) = Self::get_full_log(db, log.id).await? {
                full_logs.push(full_log);
            }
        }

        Ok(full_logs)
    }

    pub async fn list_account_logs<C: ConnectionTrait>(
        db: &C,
        accounts: Vec<u32>,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<FullTransactionLog>, DbErr> {
        let logs = db::Entity::find()
            .filter(db::Column::AccountId.is_in(accounts))
            .limit(limit)
            .offset(offset)
            .all(db)
            .await?;

        let mut full_logs = Vec::new();

        for log in logs {
            if let Some(full_log) = Self::get_full_log(db, log.id).await? {
                full_logs.push(full_log);
            }
        }

        Ok(full_logs)
    }

    pub async fn count_transaction_logs<C: ConnectionTrait>(
        db: &C,
        accounts: Vec<u32>,
    ) -> Result<u64, DbErr> {
        let count = db::Entity::find()
            .filter(db::Column::AccountId.is_in(accounts))
            .count(db)
            .await?;

        Ok(count)
    }
}

pub async fn get_account_name<C>(db: &C, account_id: u32) -> Result<String, DbErr>
where
    C: ConnectionTrait,
{
    if let Some(account) = accounts::db::Entity::find_by_id(account_id).one(db).await? {
        let account: Account = account.into();
        match account.account_type {
            accounts::AccountType::User => {
                if let Some(user) = accounts::user_profiles::db::Entity::find_by_id(account_id)
                    .one(db)
                    .await?
                {
                    return Ok(user.name);
                }
            }
            accounts::AccountType::Till => {
                if let Some(till) = accounts::till_accounts::db::Entity::find_by_id(account_id)
                    .one(db)
                    .await?
                {
                    if let Some(business) =
                        crate::business::db::Entity::find_by_id(till.business_id)
                            .one(db)
                            .await?
                    {
                        return Ok(business.name);
                    }
                }
            }
            accounts::AccountType::Paybill => {
                if let Some(paybill) =
                    accounts::paybill_accounts::db::Entity::find_by_id(account_id)
                        .one(db)
                        .await?
                {
                    if let Some(business) =
                        crate::business::db::Entity::find_by_id(paybill.business_id)
                            .one(db)
                            .await?
                    {
                        return Ok(business.name);
                    }
                }
            }
            _ => {}
        }
    }
    Ok("Unknown".to_string())
}
