use chrono::{DateTime, Utc};
use db::Column;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    QueryFilter,
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::transactions::{Ledger, TransactionNote, TransactionType};

pub mod db;
pub mod mmf_accounts;
pub mod paybill_accounts;
pub mod till_accounts;
pub mod ui;
pub mod user_profiles;
pub mod utility_accounts;

#[derive(EnumString, Display, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    User,
    System,
    Mmf,
    Utility,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
        if id == 0 {
            // system account
            let account = db::Entity::find_by_id(id).one(conn).await?;
            if account.is_none() {
                let model = db::ActiveModel {
                    id: Set(0),
                    balance: Set(0),
                    account_type: Set(AccountType::System.to_string()),
                    created_at: Set(Utc::now()),
                    disabled: Set(false),
                };

                let account = model.insert(conn).await?;

                return Ok(Some(account.into()));
            } else {
                return Ok(account.map(|acc| acc.into()));
            }
        }
        let account = db::Entity::find()
            .filter(Column::Id.eq(id))
            .one(conn)
            .await?;

        Ok(account.map(|acc| acc.into()))
    }

    pub async fn get_display_name<C>(conn: &C, id: u32) -> Result<String, DbErr>
    where
        C: ConnectionTrait,
    {
        if id == 0 {
            return Ok("System".to_string());
        }

        if let Some(account) = db::Entity::find_by_id(id).one(conn).await? {
            let account: Account = account.into();
            match account.account_type {
                AccountType::User => {
                    if let Some(user) = user_profiles::db::Entity::find_by_id(id).one(conn).await? {
                        return Ok(user.name);
                    }
                }
                AccountType::Utility => {
                    if let Some(utility) = utility_accounts::db::Entity::find_by_id(id)
                        .one(conn)
                        .await?
                        && let Some(business) =
                            crate::business::db::Entity::find_by_id(utility.business_id)
                                .one(conn)
                                .await?
                    {
                        return Ok(business.name);
                    }
                }
                AccountType::Mmf => {
                    if let Some(mmf) = mmf_accounts::db::Entity::find_by_id(id).one(conn).await?
                        && let Some(business) =
                            crate::business::db::Entity::find_by_id(mmf.business_id)
                                .one(conn)
                                .await?
                    {
                        return Ok(business.name);
                    }
                }
                _ => {}
            }
        }
        Ok("Unknown".to_string())
    }

    pub async fn create_account<C>(
        conn: &C,
        account_type: AccountType,
        initial_balance: i64,
    ) -> anyhow::Result<Self>
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

        let notes = match account_type {
            AccountType::Utility => Some(TransactionNote::AccountSetupFunding {
                account_type: crate::transactions::AccountTypeForFunding::Utility,
            }),
            AccountType::Mmf => Some(TransactionNote::AccountSetupFunding {
                account_type: crate::transactions::AccountTypeForFunding::Mmf,
            }),
            AccountType::User => Some(TransactionNote::AccountSetupFunding {
                account_type: crate::transactions::AccountTypeForFunding::User,
            }),
            _ => None,
        };

        let (txn, _) = Ledger::transfer(
            conn,
            None,
            account.id,
            initial_balance,
            &TransactionType::Deposit,
            notes.as_ref(),
        )
        .await?;

        let mut account_dto: Account = account.into();
        account_dto.balance = txn.amount; // txn.amount is what was deposited

        Ok(account_dto)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestDb;
    use crate::transactions::db as transactions_db;

    #[tokio::test]
    async fn test_get_system_account_creates_it_if_missing() {
        let db = TestDb::in_memory().await.unwrap();

        // System account is ID 0
        let account = Account::get_account(&db.conn, 0).await.unwrap();

        assert!(account.is_some());
        let acc = account.unwrap();
        assert_eq!(acc.id, 0);
        assert_eq!(acc.account_type, AccountType::System);
    }

    #[tokio::test]
    async fn test_create_account_with_initial_balance() {
        let db = TestDb::in_memory().await.unwrap();

        let initial_balance = 1000;
        let account = Account::create_account(&db.conn, AccountType::User, initial_balance)
            .await
            .unwrap();

        assert_eq!(account.balance, initial_balance);
        assert_eq!(account.account_type, AccountType::User);

        // Verify side effect: Transaction in ledger
        let transactions = transactions_db::Entity::find()
            .filter(transactions_db::Column::To.eq(account.id))
            .all(&db.conn)
            .await
            .unwrap();

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].amount, initial_balance);
        assert_eq!(transactions[0].transaction_type, TransactionType::Deposit);
    }

    #[tokio::test]
    async fn test_get_nonexistent_account() {
        let db = TestDb::in_memory().await.unwrap();

        let account = Account::get_account(&db.conn, 999).await.unwrap();
        assert!(account.is_none());
    }

    #[tokio::test]
    async fn test_get_system_account_repeatedly() {
        let db = TestDb::in_memory().await.unwrap();

        let acc1 = Account::get_account(&db.conn, 0).await.unwrap().unwrap();
        let acc2 = Account::get_account(&db.conn, 0).await.unwrap().unwrap();

        assert_eq!(acc1.id, acc2.id);
        assert_eq!(acc1.id, 0);

        // Check DB directly to ensure only one row with ID 0 exists
        let count = db::Entity::find()
            .filter(db::Column::Id.eq(0))
            .all(&db.conn)
            .await
            .unwrap()
            .len();
        assert_eq!(count, 1);
    }
}
