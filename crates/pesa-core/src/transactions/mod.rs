use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{Set, Unchanged},
    ConnectionTrait, DbErr, EntityTrait,
    prelude::DateTimeUtc,
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
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

#[derive(Debug, Error, PartialEq)]
pub enum TransactionEngineError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Account not found: {0}")]
    AccountNotFound(u32),

    #[error("You cant send money to yourself")]
    SelfTransact,

    #[error("Transaction not found")]
    TransactionNotFound,
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
}

static GLOBAL_LEDGER_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Display, EnumString, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Paybill,
    BuyGoods,
    SendMoney,
    Airtime,
    Reversal,
    Withdraw,
    Deposit,
    ChargeSettlement,
    RevenueSweep,
    TopupUtility,
    Disbursment,
    Unknown(String),
}

#[derive(Display, EnumString, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Pending,
    Failed,
    Completed,
    Reversed,
    Unknown(String),
}

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
            transaction_type: Set(txn_type.to_string()),
            status: Set(TransactionStatus::Completed.to_string()),
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
        id: &str,
    ) -> Result<(Transaction, Vec<crate::events::DomainEvent>), TransactionEngineError>
    where
        C: ConnectionTrait,
    {
        let _guard = GLOBAL_LEDGER_LOCK.lock().await;
        let mut events = Vec::new();

        let transaction = db::Entity::find_by_id(id).one(conn).await?;
        if transaction.is_none() {
            return Err(TransactionEngineError::TransactionNotFound);
        }

        let transaction = transaction.unwrap();
        let source_id = transaction.from;
        let dest_id = transaction.to;
        let amount = transaction.amount;

        // look through the accounts and restore balances
        if let Some(dest) = crate::accounts::db::Entity::find_by_id(dest_id)
            .one(conn)
            .await?
        {
            if dest.balance < amount {
                // we are trying to reverse but target has insufficient funds to do so
                // TODO implement correct real world logic for this scenario
                return Err(TransactionEngineError::InsufficientFunds);
            }
            let balance = dest.balance - amount;
            let mut dest_model: crate::accounts::db::ActiveModel = dest.into();
            dest_model.balance = Set(balance);
            dest_model.update(conn).await?;

            let (_log, event) = TransactionLog::create(
                conn,
                transaction.id.clone(),
                dest_id,
                Direction::Outflow,
                balance,
            )
            .await?;
            events.push(event);
        } else {
            return Err(TransactionEngineError::AccountNotFound(dest_id));
        }

        // credit back the funds to source
        if let Some(source_id) = source_id {
            if let Some(source) = crate::accounts::db::Entity::find_by_id(source_id)
                .one(conn)
                .await?
            {
                let balance = source.balance + amount;
                let mut source_model: crate::accounts::db::ActiveModel = source.into();
                source_model.balance = Set(balance);
                source_model.update(conn).await?;

                let (_log, event) = TransactionLog::create(
                    conn,
                    transaction.id.clone(),
                    source_id,
                    Direction::Inflow,
                    balance,
                )
                .await?;
                events.push(event);
            } else {
                return Err(TransactionEngineError::AccountNotFound(source_id));
            }
        }

        let mut txn: db::ActiveModel = transaction.into();
        txn.status = Set(TransactionStatus::Reversed.to_string());
        txn.updated_at = Set(Some(DateTimeUtc::UNIX_EPOCH));
        txn.update(conn).await?;

        let txn = db::ActiveModel {
            id: Set(Ledger::generate_receipt()),
            to: Set(source_id.unwrap_or(dest_id)),
            from: Set(Some(dest_id)),
            amount: Set(amount),
            currency: Set("KES".to_string()),
            transaction_type: Set(TransactionType::Reversal.to_string()),
            status: Set(TransactionStatus::Completed.to_string()),
            ..Default::default()
        };
        let txn: Transaction = txn.insert(conn).await?.into();

        drop(_guard);
        Ok((txn, events))
    }

    pub fn generate_receipt() -> String {
        Identifiers::generate_transaction_id()
    }
}
