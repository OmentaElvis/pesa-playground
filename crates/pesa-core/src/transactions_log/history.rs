use crate::accounts::Account;
use crate::accounts::mmf_accounts::MmfAccount;
use crate::accounts::utility_accounts::UtilityAccount;
use crate::transactions::db as transactions_db;
use crate::transactions::{TransactionNote, TransactionStatus, TransactionType};
use crate::transactions_log::{self};
use sea_orm::prelude::DateTimeUtc;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, Order, QueryFilter, QueryOrder, QuerySelect,
};
use sea_query::Cond;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum HistoryScopeType {
    User,
    Business,
    All,
}

#[derive(Deserialize, Debug)]
pub struct HistoryScope {
    pub r#type: HistoryScopeType,
    pub id: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Deserialize, Debug)]
pub struct Sorting {
    pub by: String,
    pub direction: SortDirection,
}

#[derive(Deserialize, Debug)]
pub struct Filters {
    pub statuses: Option<Vec<TransactionStatus>>,
    pub search_query: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    pub limit: u64,
    pub offset: u64,
}

#[derive(Deserialize, Debug)]
pub struct HistoryFilter {
    pub scope: HistoryScope,
    pub pagination: Pagination,
    pub sorting: Option<Sorting>,
    pub filters: Option<Filters>,
}

#[derive(serde::Serialize, Clone)]
pub struct TransactionHistoryEntry {
    pub transaction_id: String,
    pub date: DateTimeUtc,
    pub status: TransactionStatus,
    pub transaction_type: TransactionType,
    pub fee: i64,
    pub amount: i64,
    pub sender_name: String,
    pub sender_id: Option<u32>,
    pub sender_balance: Option<i64>,
    pub receiver_name: String,
    pub receiver_id: u32,
    pub receiver_balance: Option<i64>,
    pub notes: Option<TransactionNote>,
}

impl TransactionHistoryEntry {
    pub async fn get<C>(
        conn: &C,
        filter: HistoryFilter,
    ) -> Result<Vec<TransactionHistoryEntry>, DbErr>
    where
        C: ConnectionTrait,
    {
        let account_ids: Option<Vec<u32>> = match filter.scope.r#type {
            HistoryScopeType::User => filter.scope.id.map(|id| vec![id]),
            HistoryScopeType::Business => {
                if let Some(business_id) = filter.scope.id {
                    let mut ids = Vec::new();

                    if let Some(mmf) = MmfAccount::find_by_business_id(conn, business_id).await? {
                        ids.push(mmf.account_id);
                    }

                    if let Some(utility) =
                        UtilityAccount::find_by_business_id(conn, business_id).await?
                    {
                        ids.push(utility.account_id);
                    }
                    Some(ids)
                } else {
                    None
                }
            }
            HistoryScopeType::All => None,
        };

        let mut query = transactions_db::Entity::find();

        if let Some(ids) = &account_ids
            && !ids.is_empty()
        {
            query = query.filter(
                Cond::any()
                    .add(transactions_db::Column::From.is_in(ids.clone()))
                    .add(transactions_db::Column::To.is_in(ids.clone())),
            );
        }

        if let Some(filters) = filter.filters {
            if let Some(statuses) = filters.statuses
                && !statuses.is_empty()
            {
                query = query.filter(transactions_db::Column::Status.is_in(statuses));
            }
            if let Some(search) = filters.search_query
                && !search.trim().is_empty()
            {
                let search_pattern = format!("%{}%", search);
                query = query.filter(
                    Cond::any()
                        .add(transactions_db::Column::Id.like(&search_pattern))
                        .add(transactions_db::Column::Notes.like(&search_pattern)),
                );
            }
        }

        if let Some(sorting) = filter.sorting {
            let order = match sorting.direction {
                SortDirection::Asc => Order::Asc,
                SortDirection::Desc => Order::Desc,
            };
            match sorting.by.as_str() {
                "date" => query = query.order_by(transactions_db::Column::CreatedAt, order),
                "amount" => query = query.order_by(transactions_db::Column::Amount, order),
                _ => {}
            }
        } else {
            query = query.order_by(transactions_db::Column::CreatedAt, Order::Desc);
        }

        query = query
            .offset(filter.pagination.offset)
            .limit(filter.pagination.limit);

        let txns: Vec<transactions_db::Model> = query.all(conn).await?;

        let mut history_entries = Vec::new();

        for txn in txns {
            let sender_name = Account::get_display_name(conn, txn.from.unwrap_or(0)).await?;
            let receiver_name = Account::get_display_name(conn, txn.to).await?;

            let logs = transactions_log::db::Entity::find()
                .filter(transactions_log::db::Column::TransactionId.eq(txn.id.clone()))
                .all(conn)
                .await?;

            let mut sender_balance = None;
            let mut receiver_balance = None;

            for log in logs {
                if Some(log.account_id) == txn.from {
                    sender_balance = Some(log.new_balance);
                }
                if log.account_id == txn.to {
                    receiver_balance = Some(log.new_balance);
                }
            }

            let notes = txn
                .notes
                .and_then(|notes_str| serde_json::from_str(notes_str.as_str()).ok());

            history_entries.push(TransactionHistoryEntry {
                transaction_id: txn.id,
                date: txn.created_at,
                status: txn.status,
                transaction_type: txn.transaction_type,
                fee: txn.fee,
                amount: txn.amount,
                sender_name,
                sender_id: txn.from,
                sender_balance,
                receiver_name,
                receiver_id: txn.to,
                receiver_balance,
                notes,
            });
        }

        Ok(history_entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{Account, AccountType};
    use crate::tests::TestDb;
    use crate::transactions::{Ledger, TransactionType};

    #[tokio::test]
    async fn test_get_transaction_history_user_scope() {
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
            200,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();

        let filter = HistoryFilter {
            scope: HistoryScope {
                r#type: HistoryScopeType::User,
                id: Some(acc1.id),
            },
            pagination: Pagination {
                limit: 10,
                offset: 0,
            },
            sorting: None,
            filters: None,
        };

        let result = TransactionHistoryEntry::get(&db.conn, filter).await;
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert!(!entries.is_empty());

        let entry = &entries[0];
        assert_eq!(entry.sender_id, Some(acc1.id));
        assert_eq!(entry.receiver_id, acc2.id);
    }

    #[tokio::test]
    async fn test_get_transaction_history_all_scope() {
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
            200,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();

        let filter = HistoryFilter {
            scope: HistoryScope {
                r#type: HistoryScopeType::All,
                id: None,
            },
            pagination: Pagination {
                limit: 10,
                offset: 0,
            },
            sorting: None,
            filters: None,
        };

        let result = TransactionHistoryEntry::get(&db.conn, filter).await;
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert!(!entries.is_empty());
    }

    #[tokio::test]
    async fn test_get_transaction_history_with_status_filter() {
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
            200,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .unwrap();

        let filter = HistoryFilter {
            scope: HistoryScope {
                r#type: HistoryScopeType::All,
                id: None,
            },
            pagination: Pagination {
                limit: 10,
                offset: 0,
            },
            sorting: None,
            filters: Some(Filters {
                statuses: Some(vec![TransactionStatus::Completed]),
                search_query: None,
            }),
        };

        let result = TransactionHistoryEntry::get(&db.conn, filter).await;
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert!(!entries.is_empty());
        assert_eq!(entries[0].status, TransactionStatus::Completed);
    }

    #[tokio::test]
    async fn test_get_transaction_history_with_pagination() {
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

        let filter = HistoryFilter {
            scope: HistoryScope {
                r#type: HistoryScopeType::All,
                id: None,
            },
            pagination: Pagination {
                limit: 2,
                offset: 1,
            },
            sorting: None,
            filters: None,
        };

        let result = TransactionHistoryEntry::get(&db.conn, filter).await;
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
    }
}
