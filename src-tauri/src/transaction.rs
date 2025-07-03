use chrono::{DateTime, Utc};
use sea_query::{ColumnDef, Expr, ForeignKey, Iden, Query, SqliteQueryBuilder, Table};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Row, SqlitePool};
use tauri::State;

use crate::{db::Database, project::Projects, user::Users};

#[derive(sea_query::Iden)]
pub enum Transactions {
    Table,
    Id,
    ProjectId,
    UserId,
    Phone,
    Amount,
    ShortCode,
    AccountReference,
    TransactionDesc,
    Status,
    ResultCode,
    ResultDesc,
    CheckoutRequestId,
    MerchantRequestId,
    CreatedAt,
    CompletedAt,
    MpesaReceiptNumber,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub project_id: i64,
    pub user_id: i64,
    pub phone: String,
    pub amount: f64,
    pub short_code: Option<String>,
    pub account_reference: Option<String>,
    pub transaction_desc: Option<String>,
    pub status: String,
    pub result_code: Option<String>,
    pub result_desc: Option<String>,
    pub checkout_request_id: Option<String>,
    pub merchant_request_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub mpesa_receipt_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    pub project_id: i64,
    pub user_id: i64,
    pub phone: String,
    pub amount: f64,
    pub short_code: Option<String>,
    pub account_reference: Option<String>,
    pub transaction_desc: Option<String>,
    pub status: String,
    pub checkout_request_id: Option<String>,
    pub merchant_request_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTransactionRequest {
    pub status: Option<String>,
    pub result_code: Option<String>,
    pub result_desc: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub mpesa_receipt_number: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TransactionFilter {
    pub project_id: Option<i64>,
    pub user_id: Option<String>,
    pub phone: Option<String>,
    pub status: Option<String>,
    pub result_code: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for TransactionFilter {
    fn default() -> Self {
        Self {
            project_id: None,
            user_id: None,
            phone: None,
            status: None,
            result_code: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}

pub struct TransactionRepository;

impl TransactionRepository {
    /// Create a new transaction
    pub async fn create(
        db: &SqlitePool,
        request: CreateTransactionRequest,
    ) -> Result<Transaction, sqlx::Error> {
        let now = Utc::now();

        let sql = {
            Query::insert()
                .into_table(Transactions::Table)
                .columns([
                    Transactions::ProjectId,
                    Transactions::UserId,
                    Transactions::Phone,
                    Transactions::Amount,
                    Transactions::ShortCode,
                    Transactions::AccountReference,
                    Transactions::TransactionDesc,
                    Transactions::Status,
                    Transactions::CheckoutRequestId,
                    Transactions::MerchantRequestId,
                    Transactions::CreatedAt,
                ])
                .values_panic([
                    request.project_id.into(),
                    request.user_id.into(),
                    request.phone.clone().into(),
                    request.amount.into(),
                    request.short_code.clone().into(),
                    request.account_reference.clone().into(),
                    request.transaction_desc.clone().into(),
                    request.status.clone().into(),
                    request.checkout_request_id.clone().into(),
                    request.merchant_request_id.clone().into(),
                    now.to_rfc3339().into(),
                ])
                .to_string(SqliteQueryBuilder)
        };
        let result = db.execute(sql.as_str()).await?;
        let id = result.last_insert_rowid();

        Ok(Transaction {
            id,
            project_id: request.project_id,
            user_id: request.user_id,
            phone: request.phone,
            amount: request.amount,
            short_code: request.short_code,
            account_reference: request.account_reference,
            transaction_desc: request.transaction_desc,
            status: request.status,
            result_code: None,
            result_desc: None,
            checkout_request_id: request.checkout_request_id,
            merchant_request_id: request.merchant_request_id,
            created_at: now,
            completed_at: None,
            mpesa_receipt_number: None,
        })
    }

    /// Get transaction by ID
    pub async fn get_by_id(db: &SqlitePool, id: i64) -> Result<Option<Transaction>, sqlx::Error> {
        let sql = {
            Query::select()
                .columns([
                    Transactions::Id,
                    Transactions::ProjectId,
                    Transactions::UserId,
                    Transactions::Phone,
                    Transactions::Amount,
                    Transactions::ShortCode,
                    Transactions::AccountReference,
                    Transactions::TransactionDesc,
                    Transactions::Status,
                    Transactions::ResultCode,
                    Transactions::ResultDesc,
                    Transactions::CheckoutRequestId,
                    Transactions::MerchantRequestId,
                    Transactions::CreatedAt,
                    Transactions::CompletedAt,
                    Transactions::MpesaReceiptNumber,
                ])
                .from(Transactions::Table)
                .and_where(Expr::col(Transactions::Id).eq(id))
                .to_string(SqliteQueryBuilder)
        };

        let row = db.fetch_optional(sql.as_str()).await?;

        match row {
            Some(row) => Ok(Some(Self::row_to_transaction(row)?)),
            None => Ok(None),
        }
    }

    /// Update transaction
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        request: UpdateTransactionRequest,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        let sql = {
            let mut query = Query::update();
            query.table(Transactions::Table);

            if let Some(status) = &request.status {
                query.value(Transactions::Status, status.clone());
            }
            if let Some(result_code) = &request.result_code {
                query.value(Transactions::ResultCode, result_code.clone());
            }
            if let Some(result_desc) = &request.result_desc {
                query.value(Transactions::ResultDesc, result_desc.clone());
            }
            if let Some(completed_at) = &request.completed_at {
                query.value(Transactions::CompletedAt, completed_at.to_rfc3339());
            }
            if let Some(mpesa_receipt_number) = &request.mpesa_receipt_number {
                query.value(
                    Transactions::MpesaReceiptNumber,
                    mpesa_receipt_number.clone(),
                );
            }

            query.and_where(Expr::col(Transactions::Id).eq(id));

            query.to_string(SqliteQueryBuilder)
        };

        let result = pool.execute(sql.as_str()).await?;

        if result.rows_affected() > 0 {
            Self::get_by_id(pool, id).await
        } else {
            Ok(None)
        }
    }

    /// Delete transaction
    pub async fn delete(db: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
        let sql = Query::delete()
            .from_table(Transactions::Table)
            .and_where(Expr::col(Transactions::Id).eq(id))
            .to_string(SqliteQueryBuilder);

        let result = db.execute(sql.as_str()).await?;
        Ok(result.rows_affected() > 0)
    }

    /// List transactions with filtering
    pub async fn list(
        db: &SqlitePool,
        filter: TransactionFilter,
    ) -> Result<Vec<Transaction>, sqlx::Error> {
        let sql = {
            let mut query = Query::select();
            query
                .columns([
                    Transactions::Id,
                    Transactions::ProjectId,
                    Transactions::UserId,
                    Transactions::Phone,
                    Transactions::Amount,
                    Transactions::ShortCode,
                    Transactions::AccountReference,
                    Transactions::TransactionDesc,
                    Transactions::Status,
                    Transactions::ResultCode,
                    Transactions::ResultDesc,
                    Transactions::CheckoutRequestId,
                    Transactions::MerchantRequestId,
                    Transactions::CreatedAt,
                    Transactions::CompletedAt,
                    Transactions::MpesaReceiptNumber,
                ])
                .from(Transactions::Table);

            // Apply filters
            if let Some(project_id) = &filter.project_id {
                query.and_where(Expr::col(Transactions::ProjectId).eq(*project_id));
            }
            if let Some(user_id) = &filter.user_id {
                query.and_where(Expr::col(Transactions::UserId).eq(user_id.clone()));
            }
            if let Some(phone) = &filter.phone {
                query.and_where(Expr::col(Transactions::Phone).eq(phone.clone()));
            }
            if let Some(status) = &filter.status {
                query.and_where(Expr::col(Transactions::Status).eq(status.clone()));
            }
            if let Some(result_code) = &filter.result_code {
                query.and_where(Expr::col(Transactions::ResultCode).eq(result_code.clone()));
            }

            // Order by created_at descending
            query.order_by(Transactions::CreatedAt, sea_query::Order::Desc);

            // Apply pagination
            if let Some(limit) = filter.limit {
                query.limit(limit as u64);
            }
            if let Some(offset) = filter.offset {
                query.offset(offset as u64);
            }

            query.to_string(SqliteQueryBuilder)
        };

        let rows = db.fetch_all(sql.as_str()).await?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(Self::row_to_transaction(row)?);
        }

        Ok(transactions)
    }

    /// Count transactions with filtering
    pub async fn count(pool: &SqlitePool, filter: TransactionFilter) -> Result<i64, sqlx::Error> {
        let sql = {
            let mut query = Query::select();
            query
                .expr(Expr::col(Transactions::Id).count())
                .from(Transactions::Table);

            // Apply same filters as list
            if let Some(project_id) = &filter.project_id {
                query.and_where(Expr::col(Transactions::ProjectId).eq(*project_id));
            }
            if let Some(user_id) = &filter.user_id {
                query.and_where(Expr::col(Transactions::UserId).eq(user_id.clone()));
            }
            if let Some(phone) = &filter.phone {
                query.and_where(Expr::col(Transactions::Phone).eq(phone.clone()));
            }
            if let Some(status) = &filter.status {
                query.and_where(Expr::col(Transactions::Status).eq(status.clone()));
            }
            if let Some(result_code) = &filter.result_code {
                query.and_where(Expr::col(Transactions::ResultCode).eq(result_code.clone()));
            }

            query.to_string(SqliteQueryBuilder)
        };

        let row = pool.fetch_one(sql.as_str()).await?;
        Ok(row.get::<i64, _>(0))
    }

    /// Get transactions by checkout request ID
    pub async fn get_by_checkout_request_id(
        db: &SqlitePool,
        checkout_request_id: &str,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        let sql = Query::select()
            .columns([
                Transactions::Id,
                Transactions::ProjectId,
                Transactions::UserId,
                Transactions::Phone,
                Transactions::Amount,
                Transactions::ShortCode,
                Transactions::AccountReference,
                Transactions::TransactionDesc,
                Transactions::Status,
                Transactions::ResultCode,
                Transactions::ResultDesc,
                Transactions::CheckoutRequestId,
                Transactions::MerchantRequestId,
                Transactions::CreatedAt,
                Transactions::CompletedAt,
                Transactions::MpesaReceiptNumber,
            ])
            .from(Transactions::Table)
            .and_where(Expr::col(Transactions::CheckoutRequestId).eq(checkout_request_id))
            .to_string(SqliteQueryBuilder);

        let row = db.fetch_optional(sql.as_str()).await?;

        match row {
            Some(row) => Ok(Some(Self::row_to_transaction(row)?)),
            None => Ok(None),
        }
    }

    /// Helper function to convert SQLite row to Transaction struct
    fn row_to_transaction(row: sqlx::sqlite::SqliteRow) -> Result<Transaction, sqlx::Error> {
        let created_at_str: String = row.try_get(Transactions::CreatedAt.to_string().as_str())?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?
            .with_timezone(&Utc);

        let completed_at = if let Ok(Some(completed_at_str)) =
            row.try_get::<Option<String>, _>(Transactions::CompletedAt.to_string().as_str())
        {
            Some(
                DateTime::parse_from_rfc3339(&completed_at_str)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        Ok(Transaction {
            id: row.try_get(Transactions::Id.to_string().as_str())?,
            project_id: row.try_get(Transactions::ProjectId.to_string().as_str())?,
            user_id: row.try_get(Transactions::UserId.to_string().as_str())?,
            phone: row.try_get(Transactions::Phone.to_string().as_str())?,
            amount: row.try_get(Transactions::Amount.to_string().as_str())?,
            short_code: row.try_get(Transactions::ShortCode.to_string().as_str())?,
            account_reference: row.try_get(Transactions::AccountReference.to_string().as_str())?,
            transaction_desc: row.try_get(Transactions::TransactionDesc.to_string().as_str())?,
            status: row.try_get(Transactions::Status.to_string().as_str())?,
            result_code: row.try_get(Transactions::ResultCode.to_string().as_str())?,
            result_desc: row.try_get(Transactions::ResultDesc.to_string().as_str())?,
            checkout_request_id: row
                .try_get(Transactions::CheckoutRequestId.to_string().as_str())?,
            merchant_request_id: row
                .try_get(Transactions::MerchantRequestId.to_string().as_str())?,
            created_at,
            completed_at,
            mpesa_receipt_number: row
                .try_get(Transactions::MpesaReceiptNumber.to_string().as_str())?,
        })
    }

    pub async fn init_table(db: &SqlitePool) -> anyhow::Result<()> {
        let transactions_sql = {
            Table::create()
                .table(Transactions::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Transactions::Id)
                        .integer()
                        .not_null()
                        .primary_key()
                        .auto_increment(),
                )
                .col(ColumnDef::new(Transactions::ProjectId).integer().not_null())
                .col(ColumnDef::new(Transactions::UserId).integer().not_null())
                .col(ColumnDef::new(Transactions::Phone).text().not_null())
                .col(ColumnDef::new(Transactions::Amount).float().not_null()) // Store in cents
                .col(ColumnDef::new(Transactions::ShortCode).text().not_null())
                .col(ColumnDef::new(Transactions::AccountReference).text())
                .col(ColumnDef::new(Transactions::TransactionDesc).text())
                .col(ColumnDef::new(Transactions::Status).text().not_null())
                .col(ColumnDef::new(Transactions::ResultCode).text())
                .col(ColumnDef::new(Transactions::ResultDesc).text())
                .col(
                    ColumnDef::new(Transactions::CheckoutRequestId)
                        .text()
                        .not_null()
                        .unique_key(),
                )
                .col(ColumnDef::new(Transactions::MerchantRequestId).text())
                .col(
                    ColumnDef::new(Transactions::MpesaReceiptNumber)
                        .text()
                        .unique_key(),
                )
                .col(
                    ColumnDef::new(Transactions::CreatedAt)
                        .integer()
                        .default(Expr::cust("(strftime('%s', 'now'))")),
                )
                .col(ColumnDef::new(Transactions::CompletedAt).integer())
                .foreign_key(
                    ForeignKey::create()
                        .from(Transactions::Table, Transactions::UserId)
                        .to(Users::Table, Users::Id),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(Transactions::Table, Transactions::ProjectId)
                        .to(Projects::Table, Projects::Id),
                )
                .to_string(SqliteQueryBuilder)
        };
        db.execute(transactions_sql.as_str()).await?;

        Ok(())
    }
}

#[tauri::command]
pub async fn create_transaction(
    state: State<'_, Database>,
    request: CreateTransactionRequest,
) -> Result<Transaction, String> {
    TransactionRepository::create(&state.pool, request)
        .await
        .map_err(|err| format!("Failed to create transaction: {}", err))
}

#[tauri::command]
pub async fn get_transaction(
    state: State<'_, Database>,
    transaction_id: i64,
) -> Result<Option<Transaction>, String> {
    TransactionRepository::get_by_id(&state.pool, transaction_id)
        .await
        .map_err(|err| format!("Failed to get transaction: {}", err))
}

#[tauri::command]
pub async fn update_transaction(
    state: State<'_, Database>,
    transaction_id: i64,
    request: UpdateTransactionRequest,
) -> Result<Option<Transaction>, String> {
    TransactionRepository::update(&state.pool, transaction_id, request)
        .await
        .map_err(|err| format!("Failed to update transaction: {}", err))
}

#[tauri::command]
pub async fn delete_transaction(
    state: State<'_, Database>,
    transaction_id: String,
) -> Result<bool, String> {
    TransactionRepository::delete(&state.pool, &transaction_id)
        .await
        .map_err(|err| format!("Failed to delete transaction: {}", err))
}

#[tauri::command]
pub async fn list_transactions(
    state: State<'_, Database>,
    filter: TransactionFilter,
) -> Result<Vec<Transaction>, String> {
    TransactionRepository::list(&state.pool, filter)
        .await
        .map_err(|err| format!("Failed to list transactions: {}", err))
}

#[tauri::command]
pub async fn count_transactions(
    state: State<'_, Database>,
    project_id: Option<i64>,
    user_id: Option<String>,
    phone: Option<String>,
    status: Option<String>,
    result_code: Option<String>,
) -> Result<i64, String> {
    let filter = TransactionFilter {
        project_id,
        user_id,
        phone,
        status,
        result_code,
        limit: None,
        offset: None,
    };

    TransactionRepository::count(&state.pool, filter)
        .await
        .map_err(|err| format!("Failed to count transactions: {}", err))
}

#[tauri::command]
pub async fn get_transaction_by_checkout_request(
    state: State<'_, Database>,
    checkout_request_id: String,
) -> Result<Option<Transaction>, String> {
    TransactionRepository::get_by_checkout_request_id(&state.pool, &checkout_request_id)
        .await
        .map_err(|err| format!("Failed to get transaction by checkout request ID: {}", err))
}

// Utility command to get transactions for specific user with pagination
#[tauri::command]
pub async fn get_user_transactions(
    state: State<'_, Database>,
    user_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<Transaction>, String> {
    let filter = TransactionFilter {
        project_id: None,
        user_id: Some(user_id),
        phone: None,
        status: None,
        result_code: None,
        limit: limit.or(Some(20)),
        offset: offset.or(Some(0)),
    };

    TransactionRepository::list(&state.pool, filter)
        .await
        .map_err(|err| format!("Failed to get user transactions: {}", err))
}

// Utility command to get transactions for specific project
#[tauri::command]
pub async fn get_project_transactions(
    state: State<'_, Database>,
    project_id: i64,
    status: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<Transaction>, String> {
    let filter = TransactionFilter {
        project_id: Some(project_id),
        user_id: None,
        phone: None,
        status,
        result_code: None,
        limit: limit.or(Some(50)),
        offset: offset.or(Some(0)),
    };

    TransactionRepository::list(&state.pool, filter)
        .await
        .map_err(|err| format!("Failed to get project transactions: {}", err))
}

// Command to get recent transactions (last 24 hours, last week, etc.)
#[tauri::command]
pub async fn get_recent_transactions(
    state: State<'_, Database>,
    limit: Option<u32>,
) -> Result<Vec<Transaction>, String> {
    let filter = TransactionFilter {
        project_id: None,
        user_id: None,
        phone: None,
        status: None,
        result_code: None,
        limit: limit.or(Some(10)),
        offset: Some(0),
    };

    TransactionRepository::list(&state.pool, filter)
        .await
        .map_err(|err| format!("Failed to get recent transactions: {}", err))
}

// Command to get transaction statistics
#[tauri::command]
pub async fn get_transaction_stats(
    state: State<'_, Database>,
    project_id: Option<i64>,
) -> Result<TransactionStats, String> {
    let filter = TransactionFilter {
        project_id,
        user_id: None,
        phone: None,
        status: None,
        result_code: None,
        limit: None,
        offset: None,
    };

    let total_count = TransactionRepository::count(&state.pool, filter.clone())
        .await
        .map_err(|err| format!("Failed to get total count: {}", err))?;

    let successful_filter = TransactionFilter {
        project_id,
        status: Some("SUCCESS".to_string()),
        ..filter.clone()
    };

    let successful_count = TransactionRepository::count(&state.pool, successful_filter)
        .await
        .map_err(|err| format!("Failed to get successful count: {}", err))?;

    let pending_filter = TransactionFilter {
        project_id,
        status: Some("PENDING".to_string()),
        ..filter.clone()
    };

    let pending_count = TransactionRepository::count(&state.pool, pending_filter)
        .await
        .map_err(|err| format!("Failed to get pending count: {}", err))?;

    let failed_filter = TransactionFilter {
        project_id,
        status: Some("FAILED".to_string()),
        ..filter
    };

    let failed_count = TransactionRepository::count(&state.pool, failed_filter)
        .await
        .map_err(|err| format!("Failed to get failed count: {}", err))?;

    Ok(TransactionStats {
        total_count,
        successful_count,
        pending_count,
        failed_count,
    })
}

// Helper struct for transaction statistics
#[derive(serde::Serialize)]
pub struct TransactionStats {
    pub total_count: i64,
    pub successful_count: i64,
    pub pending_count: i64,
    pub failed_count: i64,
}
