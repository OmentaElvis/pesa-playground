pub mod db;
pub mod history;
pub mod ui;

pub use history::{
    HistoryFilter, HistoryScope, HistoryScopeType, Pagination, SortDirection, Sorting,
    TransactionHistoryEntry,
};

use crate::transactions::{TransactionNote, TransactionStatus, TransactionType};
use serde_json;

use sea_orm::prelude::DateTimeUtc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, Set,
};
use sea_orm::{PaginatorTrait, QuerySelect};
use serde::{Deserialize, Serialize};

use self::db::{ActiveModel, Direction};
use crate::accounts::Account;
use crate::transactions;

#[derive(Serialize)]
pub struct TransactionLog {
    pub id: u32,
    pub transaction_id: String,
    pub account_id: u32,
    pub direction: Direction,
    pub new_balance: i64,
}

#[derive(Serialize, Clone, Deserialize, Debug, PartialEq)]
pub struct FullTransactionLog {
    pub transaction_id: String,
    pub transaction_date: DateTimeUtc,
    pub transaction_amount: i64,
    pub transaction_type: TransactionType,
    pub from_name: String,
    pub to_name: String,
    pub from_id: Option<u32>,
    pub to_id: u32,
    pub new_balance: i64,
    pub status: TransactionStatus,
    pub fee: i64,
    pub direction: Direction,
    pub notes: Option<TransactionNote>,
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
    pub async fn get<C>(conn: &C, id: u32) -> Result<Option<TransactionLog>, DbErr>
    where
        C: ConnectionTrait,
    {
        let model = db::Entity::find_by_id(id).one(conn).await?;
        Ok(model.map(|m| m.into()))
    }

    pub async fn create<C>(
        conn: &C,
        transaction_id: String,
        account_id: u32,
        direction: Direction,
        new_balance: i64,
    ) -> Result<(Self, crate::events::DomainEvent), DbErr>
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

        let log = log.insert(conn).await?;

        let full_log = Self::get_full_log(conn, log.id)
            .await?
            .expect("Log just created should be found");

        let event = crate::events::DomainEvent::TransactionCreated(full_log);

        Ok((log.into(), event))
    }

    pub async fn get_full_log<C>(
        db: &C,
        transaction_log_id: u32,
    ) -> Result<Option<FullTransactionLog>, DbErr>
    where
        C: ConnectionTrait,
    {
        if let Some(log) = db::Entity::find_by_id(transaction_log_id).one(db).await?
            && let Some(transaction) = transactions::db::Entity::find_by_id(&log.transaction_id)
                .one(db)
                .await?
        {
            let from_name = if let Some(from_id) = transaction.from {
                get_account_name(db, from_id).await?
            } else {
                "System".to_string()
            };

            let to_name = get_account_name(db, transaction.to).await?;

            let notes = if let Some(notes_str) = &transaction.notes {
                serde_json::from_str(notes_str).unwrap_or(None)
            } else {
                None
            };

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
                notes,
            }));
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
    Account::get_display_name(db, account_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{Account, AccountType};
    use crate::tests::TestDb;
    use crate::transactions::{Ledger, TransactionType};

    async fn setup_test_data(db: &TestDb) -> (Account, Account) {
        let acc1 = Account::create_account(&db.conn, AccountType::User, 1000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();
        (acc1, acc2)
    }

    #[tokio::test]
    async fn test_transaction_log_get() {
        let db = TestDb::in_memory().await.unwrap();
        let (acc1, acc2) = setup_test_data(&db).await;

        let (_, _) = Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc2.id,
            200,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();

        let logs = db::Entity::find()
            .filter(db::Column::AccountId.eq(acc2.id))
            .all(&db.conn)
            .await
            .unwrap();

        assert!(!logs.is_empty());
        let log_id = logs[0].id;

        let result = TransactionLog::get(&db.conn, log_id).await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().account_id, acc2.id);
    }

    #[tokio::test]
    async fn test_transaction_log_get_not_found() {
        let db = TestDb::in_memory().await.unwrap();

        let result = TransactionLog::get(&db.conn, 99999).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_full_log() {
        let db = TestDb::in_memory().await.unwrap();
        let (acc1, acc2) = setup_test_data(&db).await;

        let (txn, _) = Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc2.id,
            200,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();

        let logs = db::Entity::find()
            .filter(db::Column::TransactionId.eq(txn.id.clone()))
            .all(&db.conn)
            .await
            .unwrap();

        let log_id = logs[0].id;

        let result = TransactionLog::get_full_log(&db.conn, log_id).await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().transaction_id, txn.id);
    }

    #[tokio::test]
    async fn test_list_full_logs() {
        let db = TestDb::in_memory().await.unwrap();
        let (acc1, acc2) = setup_test_data(&db).await;

        Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc2.id,
            100,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();

        let result = TransactionLog::list_full_logs(&db.conn, acc2.id as i32, 10, 0).await;
        assert!(result.is_ok());
        let logs = result.unwrap();
        assert!(!logs.is_empty());
    }

    #[tokio::test]
    async fn test_list_full_logs_with_pagination() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 10000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

        for i in 1..=5 {
            Ledger::transfer(
                &db.conn,
                Some(acc1.id),
                acc2.id,
                100 * i,
                &TransactionType::SendMoney,
                None,
            )
            .await
            .unwrap();
        }

        let result = TransactionLog::list_full_logs(&db.conn, acc2.id as i32, 2, 2).await;
        assert!(result.is_ok());
        let logs = result.unwrap();
        assert_eq!(logs.len(), 2);
    }

    #[tokio::test]
    async fn test_list_account_logs() {
        let db = TestDb::in_memory().await.unwrap();
        let (acc1, acc2) = setup_test_data(&db).await;

        Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc2.id,
            100,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();

        let result =
            TransactionLog::list_account_logs(&db.conn, vec![acc1.id, acc2.id], 10, 0).await;
        assert!(result.is_ok());
        let logs = result.unwrap();
        assert!(!logs.is_empty());
    }

    #[tokio::test]
    async fn test_count_transaction_logs() {
        let db = TestDb::in_memory().await.unwrap();
        let (acc1, acc2) = setup_test_data(&db).await;

        Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc2.id,
            100,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();

        let result = TransactionLog::count_transaction_logs(&db.conn, vec![acc1.id, acc2.id]).await;
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[tokio::test]
    async fn test_get_account_name_system() {
        let db = TestDb::in_memory().await.unwrap();

        let result = get_account_name(&db.conn, 0).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "System");
    }

    #[tokio::test]
    async fn test_get_account_name_user() {
        let db = TestDb::in_memory().await.unwrap();
        let acc = Account::create_account(&db.conn, AccountType::User, 100)
            .await
            .unwrap();

        let result = get_account_name(&db.conn, acc.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Unknown");
    }
}
