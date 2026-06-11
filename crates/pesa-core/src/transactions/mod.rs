use chrono::{DateTime, Utc};
pub use db::{TransactionStatus, TransactionType};
use once_cell::sync::Lazy;
use sea_orm::prelude::DateTimeUtc;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{Set, Unchanged},
    ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QuerySelect,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

use crate::{accounts::Account, server::api::b2c};
use crate::{
    transactions_log::{TransactionLog, db::Direction},
    utils::identifiers::Identifiers,
};
use serde_json;

pub mod db;
pub mod ui;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TransactionFilter {
    pub from: Option<u32>,
    pub to: Option<u32>,
    pub transaction_type: Option<TransactionType>,
    pub status: Option<TransactionStatus>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Error, PartialEq)]
pub enum TransactionEngineError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Account not found: {0}")]
    AccountNotFound(u32),

    #[error("Account is disabled: {0}")]
    AccountDisabled(u32),

    #[error("You cant send money to yourself")]
    SelfTransact,

    #[error("Transaction not found")]
    TransactionNotFound,

    #[error(
        "Attempt to use untracked set of Ids. The ids were not saved to database so will likely cause FK issue if used."
    )]
    IdentifiersError(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountTypeForFunding {
    Utility,
    Mmf,
    User,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TransactionNote {
    PaybillPayment {
        paybill_number: u32,
        bill_ref_number: String,
    },
    TillPayment {
        till_number: u32,
    },
    AccountSetupFunding {
        account_type: AccountTypeForFunding,
    },
    Disbursment {
        kind: b2c::CommandID,
    },
    Reversal {
        original_transaction_id: String,
    },
}

static GLOBAL_LEDGER_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: String,
    pub from: Option<u32>,
    pub to: u32,
    pub amount: i64,
    pub fee: i64,
    pub currency: String,
    pub status: TransactionStatus,
    pub reversal_of: Option<String>,
    pub transaction_type: TransactionType,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub notes: Option<TransactionNote>,
}

impl Transaction {
    pub async fn get<C>(conn: &C, id: &str) -> Result<Option<Transaction>, DbErr>
    where
        C: ConnectionTrait,
    {
        let model = db::Entity::find_by_id(id).one(conn).await?;
        Ok(model.map(Into::into))
    }

    pub async fn list_system<C>(
        conn: &C,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<Transaction>, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut query = db::Entity::find().filter(db::Column::From.is_null());

        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let models = query.all(conn).await?;
        Ok(models.into_iter().map(Into::into).collect())
    }

    pub async fn list<C>(conn: &C, filter: TransactionFilter) -> Result<Vec<Transaction>, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut query = db::Entity::find();

        match (filter.from, filter.to) {
            (Some(from), Some(to)) => {
                query = query.filter(
                    Condition::any()
                        .add(db::Column::From.eq(from))
                        .add(db::Column::To.eq(to)),
                );
            }
            (Some(from), None) => {
                query = query.filter(db::Column::From.eq(from));
            }
            (None, Some(to)) => {
                query = query.filter(db::Column::To.eq(to));
            }
            (None, None) => {}
        }

        if let Some(transaction_type) = filter.transaction_type {
            query = query.filter(db::Column::TransactionType.eq(transaction_type));
        }
        if let Some(status) = filter.status {
            query = query.filter(db::Column::Status.eq(status));
        }
        if let Some(limit) = filter.limit {
            query = query.limit(limit);
        }
        if let Some(offset) = filter.offset {
            query = query.offset(offset);
        }

        let models = query.all(conn).await?;
        Ok(models.into_iter().map(Into::into).collect())
    }

    pub async fn count<C>(conn: &C, filter: TransactionFilter) -> Result<u64, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut query = db::Entity::find();

        if let Some(from) = filter.from {
            query = query.filter(db::Column::From.eq(from));
        }
        if let Some(to) = filter.to {
            query = query.filter(db::Column::To.eq(to));
        }
        if let Some(transaction_type) = filter.transaction_type {
            query = query.filter(db::Column::TransactionType.eq(transaction_type));
        }
        if let Some(status) = filter.status {
            query = query.filter(db::Column::Status.eq(status));
        }

        query.count(conn).await
    }

    pub async fn total_volume<C>(conn: &C) -> Result<i64, DbErr>
    where
        C: ConnectionTrait,
    {
        let result: Option<i64> = db::Entity::find()
            .select_only()
            .column_as(db::Column::Amount.sum(), "sum")
            .into_tuple::<(Option<i64>,)>()
            .one(conn)
            .await?
            .map(|(val,)| val.unwrap_or_default());

        Ok(result.unwrap_or_default())
    }

    pub async fn total_fees<C>(conn: &C) -> Result<i64, DbErr>
    where
        C: ConnectionTrait,
    {
        let result: Option<i64> = db::Entity::find()
            .select_only()
            .column_as(db::Column::Fee.sum(), "sum")
            .into_tuple::<(Option<i64>,)>()
            .one(conn)
            .await?
            .map(|(val,)| val.unwrap_or_default());

        Ok(result.unwrap_or_default())
    }

    pub async fn get_by_checkout_request_id<C>(
        conn: &C,
        checkout_request_id: &str,
    ) -> Result<Option<Transaction>, DbErr>
    where
        C: ConnectionTrait,
    {
        let model = db::Entity::find()
            .filter(db::Column::Id.eq(checkout_request_id))
            .one(conn)
            .await?;
        Ok(model.map(Into::into))
    }
}

pub struct Ledger {}

impl Ledger {
    pub async fn transfer<C>(
        conn: &C,
        source: Option<u32>,
        destination: u32,
        amount: i64,
        txn_type: &TransactionType,
        notes: Option<&TransactionNote>,
    ) -> Result<(Transaction, Vec<crate::events::DomainEvent>), TransactionEngineError>
    where
        C: ConnectionTrait,
    {
        let transaction_id = Ledger::generate_receipt();
        Ledger::transfer_with_id(
            conn,
            &transaction_id,
            source,
            destination,
            amount,
            txn_type,
            notes,
        )
        .await
    }

    pub async fn transfer_with_id<C>(
        conn: &C,
        transaction_id: &str,
        source: Option<u32>,
        destination: u32,
        amount: i64,
        txn_type: &TransactionType,
        notes: Option<&TransactionNote>,
    ) -> Result<(Transaction, Vec<crate::events::DomainEvent>), TransactionEngineError>
    where
        C: ConnectionTrait,
    {
        let _guard = GLOBAL_LEDGER_LOCK.lock().await;
        let mut events = Vec::new();

        let mut source_account = if let Some(source) = source {
            let source_account = Account::get_account(conn, source).await?;
            if let Some(account) = source_account {
                if account.disabled {
                    return Err(TransactionEngineError::AccountDisabled(account.id));
                }
                if matches!(account.account_type, crate::accounts::AccountType::System) {
                    None
                } else {
                    Some(account)
                }
            } else {
                // This is an error, We were given an accout that does not exist
                return Err(TransactionEngineError::AccountNotFound(source));
            }
        } else {
            None
        };

        let destination_account = Account::get_account(conn, destination).await?;
        if destination_account.is_none() {
            return Err(TransactionEngineError::AccountNotFound(destination));
        }

        let mut destination_account = destination_account.unwrap();
        if destination_account.disabled {
            return Err(TransactionEngineError::AccountDisabled(
                destination_account.id,
            ));
        }

        let fee = crate::transaction_costs::get_fee(conn, txn_type, amount).await?;

        // check if source has enough funds
        if let Some(source) = &mut source_account {
            if source.id == destination_account.id {
                return Err(TransactionEngineError::SelfTransact);
            }

            if source.balance < amount + fee {
                return Err(TransactionEngineError::InsufficientFunds);
            }

            // fees should be added to business charges account
            if matches!(txn_type, TransactionType::Disbursment) {
                source.balance -= amount;
            } else {
                source.balance -= amount + fee;
            }

            let acc = crate::accounts::db::ActiveModel {
                id: Unchanged(source.id),
                balance: Set(source.balance),
                ..Default::default()
            };

            acc.update(conn).await?;
        }

        destination_account.balance += amount;
        let acc = crate::accounts::db::ActiveModel {
            id: Unchanged(destination_account.id),
            balance: Set(destination_account.balance),
            ..Default::default()
        };
        acc.update(conn).await?;

        let notes_string = notes.map(|n| serde_json::to_string(n).unwrap_or_default());

        let txn = db::ActiveModel {
            id: Set(transaction_id.to_string()),
            to: Set(destination_account.id),
            from: Set(source_account.as_ref().map(|f| f.id)),
            amount: Set(amount),
            fee: Set(fee),
            currency: Set("KES".to_string()),
            transaction_type: Set(txn_type.clone()),
            status: Set(TransactionStatus::Completed),
            created_at: Set(Utc::now().to_utc()),
            notes: Set(notes_string),
            ..Default::default()
        };

        let txn: Transaction = txn.insert(conn).await?.into();

        if let Some(source) = &source_account {
            let (_log, event) = TransactionLog::create(
                conn,
                txn.id.clone(),
                source.id,
                Direction::Outflow,
                source.balance,
            )
            .await?;
            events.push(event);
        }

        let (_log, event) = TransactionLog::create(
            conn,
            txn.id.clone(),
            destination_account.id,
            Direction::Inflow,
            destination_account.balance,
        )
        .await?;
        events.push(event);

        drop(_guard);

        Ok((txn, events))
    }

    pub async fn reverse<C>(
        conn: &C,
        original_id: &str,
        reversal_id: &str,
    ) -> Result<(Transaction, Vec<crate::events::DomainEvent>), TransactionEngineError>
    where
        C: ConnectionTrait,
    {
        let _guard = GLOBAL_LEDGER_LOCK.lock().await;
        let mut events = Vec::new();

        let transaction_model = db::Entity::find_by_id(original_id).one(conn).await?;
        if transaction_model.is_none() {
            return Err(TransactionEngineError::TransactionNotFound);
        }

        let transaction_model = transaction_model.unwrap();
        let transaction: Transaction = transaction_model.clone().into();
        let source_id = transaction.from;
        let dest_id = transaction.to;
        let amount = transaction.amount;

        // Check dest balance before making any changes
        let dest_account = crate::accounts::db::Entity::find_by_id(dest_id)
            .one(conn)
            .await?
            .ok_or(TransactionEngineError::AccountNotFound(dest_id))?;

        if dest_account.balance < amount {
            return Err(TransactionEngineError::InsufficientFunds);
        }

        // Insert the reversal transaction first so TransactionLog FK references it
        let txn = db::ActiveModel {
            id: Set(reversal_id.to_string()),
            to: Set(source_id.unwrap_or(dest_id)),
            from: Set(Some(dest_id)),
            amount: Set(amount),
            fee: Set(0),
            currency: Set("KES".to_string()),
            transaction_type: Set(TransactionType::Reversal),
            status: Set(TransactionStatus::Completed),
            created_at: Set(Utc::now().to_utc()),
            ..Default::default()
        };
        let txn: Transaction = txn.insert(conn).await?.into();

        // Debit dest account
        let dest_balance = dest_account.balance - amount;
        {
            let mut dest_model: crate::accounts::db::ActiveModel = dest_account.into();
            dest_model.balance = Set(dest_balance);
            dest_model.update(conn).await?;
        }

        let (_log, event) = TransactionLog::create(
            conn,
            reversal_id.to_string(),
            dest_id,
            Direction::Outflow,
            dest_balance,
        )
        .await?;
        events.push(event);

        // Credit back the funds to source
        if let Some(source_id) = source_id {
            if let Some(source) = crate::accounts::db::Entity::find_by_id(source_id)
                .one(conn)
                .await?
            {
                let source_balance = source.balance + amount;
                let mut source_model: crate::accounts::db::ActiveModel = source.into();
                source_model.balance = Set(source_balance);
                source_model.update(conn).await?;

                let (_log, event) = TransactionLog::create(
                    conn,
                    reversal_id.to_string(),
                    source_id,
                    Direction::Inflow,
                    source_balance,
                )
                .await?;
                events.push(event);
            } else {
                return Err(TransactionEngineError::AccountNotFound(source_id));
            }
        }

        // Mark original transaction as reversed
        let mut txn_model: db::ActiveModel = transaction_model.into();
        txn_model.status = Set(TransactionStatus::Reversed);
        txn_model.updated_at = Set(Some(DateTimeUtc::UNIX_EPOCH));
        txn_model.update(conn).await?;

        drop(_guard);
        Ok((txn, events))
    }

    pub fn generate_receipt() -> String {
        Identifiers::generate_transaction_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{Account, AccountType};
    use crate::tests::TestDb;

    #[tokio::test]
    async fn test_transfer_success() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 1000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

        let amount = 200;
        let (txn, _) = Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc2.id,
            amount,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();

        assert_eq!(txn.amount, amount);
        assert_eq!(txn.from, Some(acc1.id));
        assert_eq!(txn.to, acc2.id);

        // Verify balances updated in DB
        let acc1_after = Account::get_account(&db.conn, acc1.id)
            .await
            .unwrap()
            .unwrap();
        let acc2_after = Account::get_account(&db.conn, acc2.id)
            .await
            .unwrap()
            .unwrap();

        // Note: Ledger might calculate fees, check that too
        let fee = crate::transaction_costs::get_fee(&db.conn, &TransactionType::SendMoney, amount)
            .await
            .unwrap();
        assert_eq!(acc1_after.balance, 1000 - amount - fee);
        assert_eq!(acc2_after.balance, 500 + amount);
    }

    #[tokio::test]
    async fn test_transfer_insufficient_funds() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 100)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 0)
            .await
            .unwrap();

        let result = Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc2.id,
            1000,
            &TransactionType::SendMoney,
            None,
        )
        .await;

        assert!(result.is_err());
        // Should be TransactionEngineError::InsufficientFunds
    }

    #[tokio::test]
    async fn test_self_transfer_fails() {
        let db = TestDb::in_memory().await.unwrap();
        let acc = Account::create_account(&db.conn, AccountType::User, 1000)
            .await
            .unwrap();

        let result = Ledger::transfer(
            &db.conn,
            Some(acc.id),
            acc.id,
            100,
            &TransactionType::SendMoney,
            None,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reverse_transaction() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 1000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

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

        let acc1_before_rev = Account::get_account(&db.conn, acc1.id)
            .await
            .unwrap()
            .unwrap();
        let acc2_before_rev = Account::get_account(&db.conn, acc2.id)
            .await
            .unwrap()
            .unwrap();

        let rev_id = Ledger::generate_receipt();
        let (rev_txn, _) = Ledger::reverse(&db.conn, &txn.id, &rev_id).await.unwrap();
        assert_eq!(rev_txn.transaction_type, TransactionType::Reversal);

        let acc1_after = Account::get_account(&db.conn, acc1.id)
            .await
            .unwrap()
            .unwrap();
        let acc2_after = Account::get_account(&db.conn, acc2.id)
            .await
            .unwrap()
            .unwrap();

        // Balance should be restored (ignoring fees for now as reversal might not reverse fees)
        assert_eq!(acc1_after.balance, acc1_before_rev.balance + 200);
        assert_eq!(acc2_after.balance, acc2_before_rev.balance - 200);
    }

    #[tokio::test]
    async fn test_transfer_disabled_account() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 1000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

        // Disable acc1
        crate::accounts::user_profiles::User::disable_user(&db.conn, acc1.id)
            .await
            .unwrap();

        let result = Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc2.id,
            100,
            &TransactionType::SendMoney,
            None,
        )
        .await;

        assert_eq!(
            result.unwrap_err(),
            TransactionEngineError::AccountDisabled(acc1.id)
        );
    }

    async fn create_test_transaction(db: &TestDb) -> Transaction {
        let acc1 = Account::create_account(&db.conn, AccountType::User, 1000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

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

        txn
    }

    #[tokio::test]
    async fn test_get_transaction_by_id() {
        let db = TestDb::in_memory().await.unwrap();
        let created_txn = create_test_transaction(&db).await;

        let result = Transaction::get(&db.conn, &created_txn.id).await;
        assert!(result.is_ok());

        let found = result.unwrap();
        assert!(found.is_some());
        let txn = found.unwrap();
        assert_eq!(txn.id, created_txn.id);
        assert_eq!(txn.amount, 200);
    }

    #[tokio::test]
    async fn test_get_transaction_by_id_not_found() {
        let db = TestDb::in_memory().await.unwrap();

        let result = Transaction::get(&db.conn, "nonexistent_id").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_system_transactions() {
        let db = TestDb::in_memory().await.unwrap();

        // Creating User accounts creates Deposit transactions from NULL (system)
        let _acc1 = Account::create_account(&db.conn, AccountType::User, 1000)
            .await
            .unwrap();
        let _acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

        // After account creation: 2 transactions with from=NULL (both Deposit)

        Account::create_account(&db.conn, AccountType::User, 300)
            .await
            .unwrap();

        // After our transfer: 3 transactions with from=NULL

        let result = Transaction::list_system(&db.conn, None, None).await;
        assert!(result.is_ok());
        let txns = result.unwrap();
        assert_eq!(txns.len(), 3);
    }

    #[tokio::test]
    async fn test_list_system_transactions_with_limit() {
        let db = TestDb::in_memory().await.unwrap();

        let acc1 = Account::create_account(&db.conn, AccountType::System, 10000)
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
                &TransactionType::Deposit,
                None,
            )
            .await
            .unwrap();
        }

        let result = Transaction::list_system(&db.conn, Some(3), None).await;
        assert!(result.is_ok());
        let txns = result.unwrap();
        assert_eq!(txns.len(), 3);
    }

    #[tokio::test]
    async fn test_list_system_transactions_empty() {
        let db = TestDb::in_memory().await.unwrap();

        let result = Transaction::list_system(&db.conn, None, None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_transactions_with_from_filter() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 1000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();
        let acc3 = Account::create_account(&db.conn, AccountType::User, 300)
            .await
            .unwrap();

        // Account creation creates 3 Deposit transactions
        // Then transfers from acc1 to acc2 and acc3 = 2 more transactions from acc1

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
        Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc3.id,
            200,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();

        // Total: 5 transactions, 2 from acc1
        let filter = TransactionFilter {
            from: Some(acc1.id),
            ..Default::default()
        };

        let result = Transaction::list(&db.conn, filter).await;
        assert!(result.is_ok());
        let txns = result.unwrap();
        assert_eq!(txns.len(), 2);
    }

    #[tokio::test]
    async fn test_list_transactions_with_to_filter() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 1000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

        // Account creation: 2 Deposit transactions (to acc1, to acc2)
        // Then 2 transfers to acc2
        // Total: 4 transactions, 3 to acc2

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
        Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc2.id,
            200,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();

        let filter = TransactionFilter {
            to: Some(acc2.id),
            ..Default::default()
        };

        let result = Transaction::list(&db.conn, filter).await;
        assert!(result.is_ok());
        let txns = result.unwrap();
        assert_eq!(txns.len(), 3);
    }

    #[tokio::test]
    async fn test_list_transactions_with_type_filter() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 1000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

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

        let filter = TransactionFilter {
            transaction_type: Some(TransactionType::SendMoney),
            ..Default::default()
        };

        let result = Transaction::list(&db.conn, filter).await;
        assert!(result.is_ok());
        let txns = result.unwrap();
        assert_eq!(txns.len(), 1);
        assert_eq!(txns[0].transaction_type, TransactionType::SendMoney);
    }

    #[tokio::test]
    async fn test_list_transactions_with_status_filter() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 1000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

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

        let filter = TransactionFilter {
            status: Some(TransactionStatus::Completed),
            transaction_type: Some(TransactionType::SendMoney),
            ..Default::default()
        };

        let result = Transaction::list(&db.conn, filter).await;
        assert!(result.is_ok());
        let txns = result.unwrap();
        assert_eq!(txns.len(), 1);
        assert_eq!(txns[0].status, TransactionStatus::Completed);
    }

    #[tokio::test]
    async fn test_list_transactions_with_limit_and_offset() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 10000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

        // Account creation: 2 Deposit transactions
        // Then 10 SendMoney transfers = 12 total

        for i in 1..=10 {
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

        let filter = TransactionFilter {
            limit: Some(3),
            offset: Some(5),
            ..Default::default()
        };

        let result = Transaction::list(&db.conn, filter).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_count_all_transactions() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 10000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

        // Account creation: 2 Deposit transactions
        // Then 5 SendMoney transfers
        // Total: 7 transactions

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

        let filter = TransactionFilter::default();
        let result = Transaction::count(&db.conn, filter).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 7);
    }

    #[tokio::test]
    async fn test_count_transactions_with_filter() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 10000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

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
        Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc2.id,
            200,
            &TransactionType::Paybill,
            None,
        )
        .await
        .unwrap();

        let filter = TransactionFilter {
            transaction_type: Some(TransactionType::SendMoney),
            ..Default::default()
        };

        let result = Transaction::count(&db.conn, filter).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_count_transactions_by_status() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 10000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

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

        let filter = TransactionFilter {
            status: Some(TransactionStatus::Completed),
            transaction_type: Some(TransactionType::SendMoney),
            ..Default::default()
        };

        let result = Transaction::count(&db.conn, filter).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_total_transaction_volume() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 10000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

        // Account creation creates 2 Deposit transactions with amounts 10000 and 500
        // Then we add 3 SendMoney transfers: 100 + 200 + 300 = 600

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
        Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc2.id,
            200,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();
        Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc2.id,
            300,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();

        // Total volume = 10000 + 500 + 100 + 200 + 300 = 11100
        let result = Transaction::total_volume(&db.conn).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 11100);
    }

    #[tokio::test]
    async fn test_total_transaction_volume_empty() {
        let db = TestDb::in_memory().await.unwrap();

        let result = Transaction::total_volume(&db.conn).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_total_transaction_fees() {
        let db = TestDb::in_memory().await.unwrap();
        let acc1 = Account::create_account(&db.conn, AccountType::User, 10000)
            .await
            .unwrap();
        let acc2 = Account::create_account(&db.conn, AccountType::User, 500)
            .await
            .unwrap();

        // Account creation creates Deposit transactions which have no fees
        // Only SendMoney transactions have fees

        let fee1 = crate::transaction_costs::get_fee(&db.conn, &TransactionType::SendMoney, 100)
            .await
            .unwrap();
        let fee2 = crate::transaction_costs::get_fee(&db.conn, &TransactionType::SendMoney, 200)
            .await
            .unwrap();

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
        Ledger::transfer(
            &db.conn,
            Some(acc1.id),
            acc2.id,
            200,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();

        // Total fees = fee1 + fee2 (no fees from Deposit transactions)
        let result = Transaction::total_fees(&db.conn).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), fee1 + fee2);
    }

    #[tokio::test]
    async fn test_total_transaction_fees_empty() {
        let db = TestDb::in_memory().await.unwrap();

        let result = Transaction::total_fees(&db.conn).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_get_by_checkout_request_id() {
        let db = TestDb::in_memory().await.unwrap();
        let created_txn = create_test_transaction(&db).await;

        let result = Transaction::get_by_checkout_request_id(&db.conn, &created_txn.id).await;
        assert!(result.is_ok());

        let found = result.as_ref().unwrap();
        assert!(found.is_some());
        assert_eq!(found.as_ref().unwrap().id, created_txn.id);
    }

    #[tokio::test]
    async fn test_get_by_checkout_request_id_not_found() {
        let db = TestDb::in_memory().await.unwrap();

        let result =
            Transaction::get_by_checkout_request_id(&db.conn, "nonexistent_checkout").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
