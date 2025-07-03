use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use rand::{
    distr::{Alphanumeric, Uniform},
    seq::IndexedRandom,
    Rng,
};
use reqwest::Client;
use sea_query::{ColumnDef, Expr, Iden, Order, Query, SqliteQueryBuilder, Table};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Executor, Row, SqlitePool};
use tauri::Emitter;
use tokio::{
    sync::{oneshot, Mutex},
    time::{sleep, Instant},
};

use crate::{
    api_logs::{ApiLogRepository, CreateApiLogRequest},
    project::Project,
    server::ApiState,
    transaction::{Transaction, TransactionRepository, UpdateTransactionRequest},
    user::User,
};

#[derive(Iden)]
pub enum CallbackLogs {
    Table,
    Id,
    TransactionId,
    CheckoutRequestId,
    CallbackUrl,
    CallbackType,
    Payload,
    ResponseStatus,
    ResponseBody,
    Status,
    Error,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackLog {
    pub id: i64,
    pub transaction_id: Option<i64>,
    pub checkout_request_id: Option<String>,
    pub callback_url: String,
    pub callback_type: String, // e.g. "stkpush", "c2b"
    pub payload: String,       // raw JSON
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub status: String, // e.g. "delivered", "failed"
    pub error: Option<String>,
    pub created_at: i64, // Unix timestamp
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCallbackLog {
    pub transaction_id: Option<i64>,
    pub checkout_request_id: Option<String>,
    pub callback_url: String,
    pub callback_type: String,
    pub payload: String,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCallbackLog {
    pub transaction_id: Option<i64>,
    pub checkout_request_id: Option<String>,
    pub callback_url: Option<String>,
    pub callback_type: Option<String>,
    pub payload: Option<String>,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CallbackLogFilter {
    pub transaction_id: Option<i64>,
    pub checkout_request_id: Option<String>,
    pub callback_type: Option<String>,
    pub status: Option<String>,
    pub has_error: Option<bool>,
    pub response_status: Option<i32>,
    pub created_after: Option<i64>,
    pub created_before: Option<i64>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl CallbackLog {
    pub async fn init_table(db: &SqlitePool) -> Result<()> {
        let sql = {
            Table::create()
                .table(CallbackLogs::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(CallbackLogs::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(CallbackLogs::TransactionId).integer().null())
                .col(
                    ColumnDef::new(CallbackLogs::CheckoutRequestId)
                        .string()
                        .null(),
                )
                .col(
                    ColumnDef::new(CallbackLogs::CallbackUrl)
                        .string()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(CallbackLogs::CallbackType)
                        .string()
                        .not_null(),
                )
                .col(ColumnDef::new(CallbackLogs::Payload).text().not_null())
                .col(
                    ColumnDef::new(CallbackLogs::ResponseStatus)
                        .integer()
                        .null(),
                )
                .col(ColumnDef::new(CallbackLogs::ResponseBody).text().null())
                .col(ColumnDef::new(CallbackLogs::Status).string().not_null())
                .col(ColumnDef::new(CallbackLogs::Error).text().null())
                .col(ColumnDef::new(CallbackLogs::CreatedAt).integer().not_null())
                .col(ColumnDef::new(CallbackLogs::UpdatedAt).integer().null())
                .to_string(SqliteQueryBuilder)
        };
        db.execute(sql.as_str()).await?;
        Ok(())
    }

    // CREATE
    pub async fn create(db: &SqlitePool, data: CreateCallbackLog) -> Result<CallbackLog> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        let sql = {
            Query::insert()
                .into_table(CallbackLogs::Table)
                .columns([
                    CallbackLogs::TransactionId,
                    CallbackLogs::CheckoutRequestId,
                    CallbackLogs::CallbackUrl,
                    CallbackLogs::CallbackType,
                    CallbackLogs::Payload,
                    CallbackLogs::ResponseStatus,
                    CallbackLogs::ResponseBody,
                    CallbackLogs::Status,
                    CallbackLogs::Error,
                    CallbackLogs::CreatedAt,
                ])
                .values_panic([
                    data.transaction_id.into(),
                    data.checkout_request_id.into(),
                    data.callback_url.into(),
                    data.callback_type.into(),
                    data.payload.into(),
                    data.response_status.into(),
                    data.response_body.into(),
                    data.status.into(),
                    data.error.into(),
                    now.into(),
                ])
                .to_string(SqliteQueryBuilder)
        };

        let result = sqlx::query(&sql).execute(db).await?;
        let id = result.last_insert_rowid();

        Self::find_by_id(db, id).await
    }

    // READ - Single by ID
    pub async fn find_by_id(db: &SqlitePool, id: i64) -> Result<CallbackLog> {
        let sql = {
            Query::select()
                .columns([
                    CallbackLogs::Id,
                    CallbackLogs::TransactionId,
                    CallbackLogs::CheckoutRequestId,
                    CallbackLogs::CallbackUrl,
                    CallbackLogs::CallbackType,
                    CallbackLogs::Payload,
                    CallbackLogs::ResponseStatus,
                    CallbackLogs::ResponseBody,
                    CallbackLogs::Status,
                    CallbackLogs::Error,
                    CallbackLogs::CreatedAt,
                    CallbackLogs::UpdatedAt,
                ])
                .from(CallbackLogs::Table)
                .and_where(Expr::col(CallbackLogs::Id).eq(id))
                .to_string(SqliteQueryBuilder)
        };

        let row = sqlx::query(&sql).fetch_one(db).await?;
        Self::from_row(&row)
    }

    // READ - All with optional filtering
    pub async fn find_all(db: &SqlitePool, filter: CallbackLogFilter) -> Result<Vec<CallbackLog>> {
        let sql = {
            let mut query = Query::select();
            query
                .columns([
                    CallbackLogs::Id,
                    CallbackLogs::TransactionId,
                    CallbackLogs::CheckoutRequestId,
                    CallbackLogs::CallbackUrl,
                    CallbackLogs::CallbackType,
                    CallbackLogs::Payload,
                    CallbackLogs::ResponseStatus,
                    CallbackLogs::ResponseBody,
                    CallbackLogs::Status,
                    CallbackLogs::Error,
                    CallbackLogs::CreatedAt,
                    CallbackLogs::UpdatedAt,
                ])
                .from(CallbackLogs::Table);

            // Apply filters
            if let Some(transaction_id) = filter.transaction_id {
                query.and_where(Expr::col(CallbackLogs::TransactionId).eq(transaction_id));
            }
            if let Some(checkout_request_id) = filter.checkout_request_id {
                query.and_where(Expr::col(CallbackLogs::CheckoutRequestId).eq(checkout_request_id));
            }
            if let Some(callback_type) = filter.callback_type {
                query.and_where(Expr::col(CallbackLogs::CallbackType).eq(callback_type));
            }
            if let Some(status) = filter.status {
                query.and_where(Expr::col(CallbackLogs::Status).eq(status));
            }
            if let Some(has_error) = filter.has_error {
                if has_error {
                    query.and_where(Expr::col(CallbackLogs::Error).is_not_null());
                } else {
                    query.and_where(Expr::col(CallbackLogs::Error).is_null());
                }
            }
            if let Some(response_status) = filter.response_status {
                query.and_where(Expr::col(CallbackLogs::ResponseStatus).eq(response_status));
            }
            if let Some(created_after) = filter.created_after {
                query.and_where(Expr::col(CallbackLogs::CreatedAt).gt(created_after));
            }
            if let Some(created_before) = filter.created_before {
                query.and_where(Expr::col(CallbackLogs::CreatedAt).lt(created_before));
            }

            // Order by created_at descending (newest first)
            query.order_by(CallbackLogs::CreatedAt, Order::Desc);

            // Apply pagination
            if let Some(limit) = filter.limit {
                query.limit(limit);
            }
            if let Some(offset) = filter.offset {
                query.offset(offset);
            }

            query.to_string(SqliteQueryBuilder)
        };
        let rows = sqlx::query(&sql).fetch_all(db).await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(Self::from_row(&row)?);
        }

        Ok(results)
    }

    // UPDATE
    pub async fn update(db: &SqlitePool, id: i64, data: UpdateCallbackLog) -> Result<CallbackLog> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        let sql = {
            let mut query = Query::update();
            query.table(CallbackLogs::Table);

            // Only update fields that are provided
            if let Some(transaction_id) = data.transaction_id {
                query.value(CallbackLogs::TransactionId, transaction_id);
            }
            if let Some(checkout_request_id) = data.checkout_request_id {
                query.value(CallbackLogs::CheckoutRequestId, checkout_request_id);
            }
            if let Some(callback_url) = data.callback_url {
                query.value(CallbackLogs::CallbackUrl, callback_url);
            }
            if let Some(callback_type) = data.callback_type {
                query.value(CallbackLogs::CallbackType, callback_type);
            }
            if let Some(payload) = data.payload {
                query.value(CallbackLogs::Payload, payload);
            }
            if let Some(response_status) = data.response_status {
                query.value(CallbackLogs::ResponseStatus, response_status);
            }
            if let Some(response_body) = data.response_body {
                query.value(CallbackLogs::ResponseBody, response_body);
            }
            if let Some(status) = data.status {
                query.value(CallbackLogs::Status, status);
            }
            if let Some(error) = data.error {
                query.value(CallbackLogs::Error, error);
            }

            // Always update the updated_at timestamp
            query.value(CallbackLogs::UpdatedAt, now);
            query.and_where(Expr::col(CallbackLogs::Id).eq(id));

            query.to_string(SqliteQueryBuilder)
        };

        sqlx::query(&sql).execute(db).await?;

        Self::find_by_id(db, id).await
    }

    // DELETE
    pub async fn delete(db: &SqlitePool, id: i64) -> Result<bool> {
        let sql = {
            Query::delete()
                .from_table(CallbackLogs::Table)
                .and_where(Expr::col(CallbackLogs::Id).eq(id))
                .to_string(SqliteQueryBuilder)
        };

        let result = sqlx::query(&sql).execute(db).await?;
        Ok(result.rows_affected() > 0)
    }

    // UTILITY FUNCTIONS

    // Find by transaction ID
    pub async fn find_by_transaction_id(
        db: &SqlitePool,
        transaction_id: i64,
    ) -> Result<Vec<CallbackLog>> {
        let filter = CallbackLogFilter {
            transaction_id: Some(transaction_id),
            ..Default::default()
        };
        Self::find_all(db, filter).await
    }

    // Find by checkout request ID
    pub async fn find_by_checkout_request_id(
        db: &SqlitePool,
        checkout_request_id: &str,
    ) -> Result<Vec<CallbackLog>> {
        let filter = CallbackLogFilter {
            checkout_request_id: Some(checkout_request_id.to_string()),
            ..Default::default()
        };
        Self::find_all(db, filter).await
    }

    // Find by callback type
    pub async fn find_by_callback_type(
        db: &SqlitePool,
        callback_type: &str,
    ) -> Result<Vec<CallbackLog>> {
        let filter = CallbackLogFilter {
            callback_type: Some(callback_type.to_string()),
            ..Default::default()
        };
        Self::find_all(db, filter).await
    }

    // Find by status
    pub async fn find_by_status(db: &SqlitePool, status: &str) -> Result<Vec<CallbackLog>> {
        let filter = CallbackLogFilter {
            status: Some(status.to_string()),
            ..Default::default()
        };
        Self::find_all(db, filter).await
    }

    // Find failed callbacks (with errors)
    pub async fn find_failed_callbacks(db: &SqlitePool) -> Result<Vec<CallbackLog>> {
        let filter = CallbackLogFilter {
            has_error: Some(true),
            ..Default::default()
        };
        Self::find_all(db, filter).await
    }

    // Find successful callbacks (without errors)
    pub async fn find_successful_callbacks(db: &SqlitePool) -> Result<Vec<CallbackLog>> {
        let filter = CallbackLogFilter {
            has_error: Some(false),
            ..Default::default()
        };
        Self::find_all(db, filter).await
    }

    // Find callbacks by response status
    pub async fn find_by_response_status(
        db: &SqlitePool,
        status_code: i32,
    ) -> Result<Vec<CallbackLog>> {
        let filter = CallbackLogFilter {
            response_status: Some(status_code),
            ..Default::default()
        };
        Self::find_all(db, filter).await
    }

    // Find recent callbacks (within last N seconds)
    pub async fn find_recent_callbacks(db: &SqlitePool, seconds: i64) -> Result<Vec<CallbackLog>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        let since = now - seconds;

        let filter = CallbackLogFilter {
            created_after: Some(since),
            ..Default::default()
        };
        Self::find_all(db, filter).await
    }

    // Count callbacks by status
    pub async fn count_by_status(db: &SqlitePool, status: &str) -> Result<i64> {
        let sql = {
            Query::select()
                .expr(Expr::col(CallbackLogs::Id).count())
                .from(CallbackLogs::Table)
                .and_where(Expr::col(CallbackLogs::Status).eq(status))
                .to_string(SqliteQueryBuilder)
        };

        let row = sqlx::query(&sql).fetch_one(db).await?;
        Ok(row.get(0))
    }

    // Count total callbacks
    pub async fn count_total(db: &SqlitePool) -> Result<i64> {
        let sql = {
            Query::select()
                .expr(Expr::col(CallbackLogs::Id).count())
                .from(CallbackLogs::Table)
                .to_string(SqliteQueryBuilder)
        };

        let row = sqlx::query(&sql).fetch_one(db).await?;
        Ok(row.get(0))
    }

    // Update callback status
    pub async fn update_status(db: &SqlitePool, id: i64, status: &str) -> Result<CallbackLog> {
        let update_data = UpdateCallbackLog {
            status: Some(status.to_string()),
            ..Default::default()
        };
        Self::update(db, id, update_data).await
    }

    // Update callback response
    pub async fn update_response(
        db: &SqlitePool,
        id: i64,
        response_status: i32,
        response_body: Option<String>,
    ) -> Result<CallbackLog> {
        let update_data = UpdateCallbackLog {
            response_status: Some(response_status),
            response_body,
            ..Default::default()
        };
        Self::update(db, id, update_data).await
    }

    // Mark callback as failed with error
    pub async fn mark_as_failed(db: &SqlitePool, id: i64, error: &str) -> Result<CallbackLog> {
        let update_data = UpdateCallbackLog {
            status: Some("failed".to_string()),
            error: Some(error.to_string()),
            ..Default::default()
        };
        Self::update(db, id, update_data).await
    }

    // Helper function to convert sqlx Row to CallbackLog
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<CallbackLog> {
        Ok(CallbackLog {
            id: row.get(CallbackLogs::Id.to_string().as_str()),
            transaction_id: row.get(CallbackLogs::TransactionId.to_string().as_str()),
            checkout_request_id: row.get(CallbackLogs::CheckoutRequestId.to_string().as_str()),
            callback_url: row.get(CallbackLogs::CallbackUrl.to_string().as_str()),
            callback_type: row.get(CallbackLogs::CallbackType.to_string().as_str()),
            payload: row.get(CallbackLogs::Payload.to_string().as_str()),
            response_status: row.get(CallbackLogs::ResponseStatus.to_string().as_str()),
            response_body: row.get(CallbackLogs::ResponseBody.to_string().as_str()),
            status: row.get(CallbackLogs::Status.to_string().as_str()),
            error: row.get(CallbackLogs::Error.to_string().as_str()),
            created_at: row.get(CallbackLogs::CreatedAt.to_string().as_str()),
            updated_at: row.get(CallbackLogs::UpdatedAt.to_string().as_str()),
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum UserResponse {
    Accepted { pin: String },
    Cancelled,
    Offline,
    Timeout,
    Failed(String),
}

pub static STK_RESPONSE_REGISTRY: Lazy<Mutex<HashMap<String, oneshot::Sender<UserResponse>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone)]
pub enum StkCodes {
    /// 0
    Success,
    /// 1037
    DSTimeout,
    /// 1025
    SystemError,
    /// 9999
    ErrorSendingPushRequest,
    /// 1025
    ErrorSendingPushRequest1037,
    /// 1032
    RequestCancelledByUser,
    /// 1037
    NoResponseFromUser,
    /// 1
    InsufficientBalance,
    /// 2001
    InitiatorInformationInvalid,
    /// 1019
    TransactionHasExpired,
    /// 1001
    UnableToObtainSubscriberLock,
}

impl StkCodes {
    pub fn code(&self) -> i32 {
        match self {
            StkCodes::Success => 0,
            StkCodes::DSTimeout => 1037,
            StkCodes::SystemError => 1025,
            StkCodes::ErrorSendingPushRequest => 9999,
            StkCodes::ErrorSendingPushRequest1037 => 1037,
            StkCodes::RequestCancelledByUser => 1032,
            StkCodes::NoResponseFromUser => 1037,
            StkCodes::InsufficientBalance => 1,
            StkCodes::InitiatorInformationInvalid => 2001,
            StkCodes::TransactionHasExpired => 1019,
            StkCodes::UnableToObtainSubscriberLock => 1001,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            StkCodes::Success => "The service request is processed successfully.",
            StkCodes::DSTimeout => "DS timeout error.",
            StkCodes::SystemError => "Internal system error.",
            StkCodes::ErrorSendingPushRequest => "Error sending push request.",
            StkCodes::ErrorSendingPushRequest1037 => "DS timeout error.",
            StkCodes::RequestCancelledByUser => "Request cancelled by user.",
            StkCodes::NoResponseFromUser => "No response from the user.",
            StkCodes::InsufficientBalance => "The balance is insufficient for the transaction.",
            StkCodes::InitiatorInformationInvalid => "Initiator information is invalid.",
            StkCodes::TransactionHasExpired => "Transaction has expired.",
            StkCodes::UnableToObtainSubscriberLock => "Unable to obtain subscriber lock.",
        }
    }
}

pub async fn return_body(
    state: &ApiState,
    status: StkCodes,
    url: String,
    merchant_id: String,
    checkout_id: String,
    metadata: Option<Value>,
) {
    let mut callback = json!({
        "MerchantRequestID": merchant_id,
        "CheckoutRequestID": checkout_id,
        "ResultCode": status.code(),
        "ResultDesc": status.message(),
    });

    if status.code() == 0 {
        // Only attach CallbackMetadata on success
        let callback_metadata = match metadata {
            Some(Value::Array(items)) => {
                json!({ "Item": items })
            }
            Some(Value::Object(map)) => {
                // Convert object to array format
                let item_vec: Vec<_> = map
                    .into_iter()
                    .map(|(k, v)| json!({ "Name": k, "Value": v }))
                    .collect();
                json!({ "Item": item_vec })
            }
            _ => Value::Null,
        };

        if !callback_metadata.is_null() {
            callback
                .as_object_mut()
                .unwrap()
                .insert("CallbackMetadata".to_string(), callback_metadata);
        }
    }

    let body_json = json!({
        "Body": {
            "stkCallback": callback
        }
    });

    // POST the callback
    let client = Client::new();

    const MAX_ATTEMPTS: usize = 4;
    for attempt in 1..=MAX_ATTEMPTS {
        let start = Instant::now();
        let result = client.post(&url).json(&body_json).send().await;
        let duration = start.elapsed();
        let db = &state.pool;

        match result {
            Ok(resp) => {
                let status_code = resp.status();
                let response_headers_map = headers_to_map(resp.headers());
                let response_body = resp.text().await.unwrap_or_default();

                if status_code.is_success() {
                    println!("[CALLBACK] Delivered successfully on attempt {attempt}");
                    let _ = ApiLogRepository::create(
                        db,
                        CreateApiLogRequest {
                            project_id: state.project_id,
                            method: "POST".into(),
                            path: url.clone(),
                            status_code: status_code.as_u16(),
                            request_body: Some(
                                json!({
                                    "headers": {
                                        "content-type": "application/json"
                                    },
                                    "body": body_json
                                })
                                .to_string(),
                            ),
                            response_body: Some(
                                json!({
                                    "headers": response_headers_map,
                                    "body": response_body
                                })
                                .to_string(),
                            ),
                            duration: duration.as_millis(),
                            error_desc: None,
                        },
                    )
                    .await;
                    let _ = state.handle.emit("new-api-log", state.project_id);
                    return;
                } else {
                    let _ = ApiLogRepository::create(
                        db,
                        CreateApiLogRequest {
                            project_id: state.project_id,
                            method: "POST".into(),
                            path: url.clone(),
                            status_code: status_code.as_u16(),
                            request_body: Some(
                                json!({
                                    "headers": {
                                        "content-type": "application/json"
                                    },
                                    "body": body_json
                                })
                                .to_string(),
                            ),
                            response_body: Some(
                                json!({
                                    "headers": response_headers_map,
                                    "body": response_body
                                })
                                .to_string(),
                            ),
                            duration: duration.as_millis(),
                            error_desc: Some(format!(
                                "Non-2xx callback response (attempt {attempt})"
                            )),
                        },
                    )
                    .await;
                    let _ = state.handle.emit("new-api-log", state.project_id);
                }
            }

            Err(err) => {
                eprintln!("[CALLBACK] Attempt {attempt} failed: {err}");

                let _ = ApiLogRepository::create(
                    db,
                    CreateApiLogRequest {
                        project_id: state.project_id,
                        method: "POST".into(),
                        path: url.clone(),
                        status_code: 0, // 0 = no HTTP status received
                        request_body: Some(
                            json!({
                                "headers": {
                                    "content-type": "application/json"
                                },
                                "body": body_json
                            })
                            .to_string(),
                        ),
                        response_body: None,
                        duration: duration.as_millis(),
                        error_desc: Some(format!(
                            "Callback request failed (attempt {attempt}): {err}"
                        )),
                    },
                )
                .await;
                let _ = state.handle.emit("new-api-log", state.project_id);
            }
        }

        // Exponential backoff with jitter
        let backoff_ms = 2_u64.pow(attempt as u32) * 1000 + rand::rng().random_range(0..500);
        sleep(Duration::from_millis(backoff_ms)).await;
    }
    eprintln!("[CALLBACK] Final failure after {MAX_ATTEMPTS} attempts to {url}");
}

fn headers_to_map(headers: &reqwest::header::HeaderMap) -> serde_json::Map<String, Value> {
    headers
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                Value::String(v.to_str().unwrap_or_default().to_string()),
            )
        })
        .collect()
}

pub fn generate_mpesa_receipt() -> String {
    // Ensure it starts with an uppercase letter
    let first_char = rand::rng()
        .sample_iter(&Uniform::new_inclusive(b'A', b'Z').unwrap())
        .take(1)
        .map(char::from)
        .collect::<String>();

    // Followed by 9 random alphanumeric characters
    let rest: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .take(9)
        .map(char::from)
        .collect();

    format!("{first_char}{rest}")
}
impl StkCodes {
    pub fn random_failure() -> Self {
        use StkCodes::*;
        const FAILURES: &[StkCodes] = &[
            DSTimeout,
            SystemError,
            ErrorSendingPushRequest,
            ErrorSendingPushRequest1037,
            RequestCancelledByUser,
            InsufficientBalance,
            InitiatorInformationInvalid,
            TransactionHasExpired,
            UnableToObtainSubscriberLock,
        ];

        let mut rng = rand::rng();
        FAILURES.choose(&mut rng).unwrap().clone()
    }
    pub fn random() -> Self {
        use StkCodes::*;
        const FAILURES: &[StkCodes] = &[
            Success,
            DSTimeout,
            SystemError,
            ErrorSendingPushRequest,
            ErrorSendingPushRequest1037,
            RequestCancelledByUser,
            InsufficientBalance,
            InitiatorInformationInvalid,
            TransactionHasExpired,
            UnableToObtainSubscriberLock,
        ];

        let mut rng = rand::rng();
        FAILURES.choose(&mut rng).unwrap().clone()
    }
}

pub async fn callback_execute(
    state: &ApiState,
    transaction: &Transaction,
    user: Option<&User>,
    callback: CallbackLog,
    project: Project,
) -> anyhow::Result<(StkCodes, Option<String>)> {
    let checkout_id = callback.checkout_request_id.clone().unwrap();
    let merchant_id = transaction.merchant_request_id.clone().unwrap();

    let receipt = generate_mpesa_receipt();

    match project.simulation_mode.as_str() {
        "always-success" => {
            return_body(
                state,
                StkCodes::Success,
                callback.callback_url,
                merchant_id,
                checkout_id,
                Some(json!({
                    "Amount": transaction.amount,
                    "MpesaReceiptNumber": receipt,
                    "TransactionDate": Utc::now().format("%Y%m%d%H%M%S").to_string(),
                    "PhoneNumber": transaction.phone,
                })),
            )
            .await;
            return Ok((StkCodes::Success, Some(receipt)));
        }
        "always-fail" => {
            let status = StkCodes::random_failure();
            return_body(
                state,
                status.clone(),
                callback.callback_url,
                merchant_id,
                checkout_id,
                None,
            )
            .await;
            return Ok((status, None));
        }
        "random" => {
            let status = StkCodes::random();
            return_body(
                state,
                status.clone(),
                callback.callback_url,
                merchant_id,
                checkout_id,
                match status {
                    StkCodes::Success => Some(json!({
                        "Amount": transaction.amount,
                        "MpesaReceiptNumber": receipt,
                        "TransactionDate": Utc::now().format("%Y%m%d%H%M%S").to_string(),
                        "PhoneNumber": transaction.phone,
                    })),
                    _ => None,
                },
            )
            .await;
            return Ok((
                status.clone(),
                match status {
                    StkCodes::Success => Some(receipt),
                    _ => None,
                },
            ));
        }
        // next section is realistic
        "realistic" => {}
        _ => {}
    }
    // realistic
    if user.is_none() {
        // user was not found
        return_body(
            state,
            StkCodes::DSTimeout,
            callback.callback_url,
            merchant_id,
            checkout_id,
            None,
        )
        .await;
        return Ok((StkCodes::DSTimeout, None));
    }

    let user = user.unwrap();
    if user.status == Some("disabled".to_string()) {
        return_body(
            state,
            StkCodes::DSTimeout,
            callback.callback_url,
            merchant_id,
            checkout_id,
            None,
        )
        .await;
        return Ok((StkCodes::DSTimeout, None));
    }

    let mut reg = STK_RESPONSE_REGISTRY.lock().await;
    if reg.contains_key(&checkout_id) {
        // another task is handling the user, stop moving too fast
        return_body(
            state,
            StkCodes::UnableToObtainSubscriberLock,
            callback.callback_url,
            merchant_id,
            checkout_id,
            None,
        )
        .await;
        return Ok((StkCodes::UnableToObtainSubscriberLock, None));
    }

    let (tx, rx) = oneshot::channel();
    reg.insert(checkout_id.clone(), tx);

    if state
        .handle
        .emit(
            "stk_push",
            json!({
                "checkout_id": checkout_id,
                "project": project,
                "user": user,
                "callback": callback,
            }),
        )
        .is_err()
    {
        return_body(
            state,
            StkCodes::ErrorSendingPushRequest,
            callback.callback_url,
            merchant_id,
            checkout_id,
            None,
        )
        .await;
        return Ok((StkCodes::ErrorSendingPushRequest, None));
    }

    drop(reg);

    let status = match tokio::time::timeout(Duration::from_secs(30), rx).await {
        Ok(Ok(value)) => match value {
            UserResponse::Accepted { pin } => {
                if pin.eq(&user.pin) {
                    let balance = user.balance - transaction.amount;
                    if balance < 0.0 {
                        StkCodes::InsufficientBalance
                    } else {
                        StkCodes::Success
                    }
                } else {
                    StkCodes::InitiatorInformationInvalid
                }
            }
            UserResponse::Offline => StkCodes::DSTimeout,
            UserResponse::Timeout => StkCodes::NoResponseFromUser,
            UserResponse::Cancelled => StkCodes::RequestCancelledByUser,
            UserResponse::Failed(_) => StkCodes::ErrorSendingPushRequest1037,
        },
        Ok(Err(_)) => StkCodes::SystemError,
        Err(_) => StkCodes::NoResponseFromUser,
    };
    let mut reg = STK_RESPONSE_REGISTRY.lock().await;
    reg.remove(&checkout_id);
    drop(reg);

    return_body(
        state,
        status.clone(),
        callback.callback_url,
        merchant_id,
        checkout_id,
        match status {
            StkCodes::Success => Some(json!({
                "Amount": transaction.amount,
                "MpesaReceiptNumber": receipt,
                "TransactionDate": Utc::now().format("%Y%m%d%H%M%S").to_string(),
                "PhoneNumber": transaction.phone,
            })),
            _ => None,
        },
    )
    .await;
    Ok((
        status.clone(),
        match status {
            StkCodes::Success => Some(receipt),
            _ => None,
        },
    ))
}

pub async fn callback_start(
    state: ApiState,
    transaction: Transaction,
    user: Option<User>,
    callback: CallbackLog,
    project: Project,
) {
    let (status, code) =
        match callback_execute(&state, &transaction, user.as_ref(), callback, project).await {
            Ok(s) => s,
            Err(err) => {
                println!("[error: {}] {}", state.project_id, err);
                return;
            }
        };
    let update = UpdateTransactionRequest {
        status: match &status {
            StkCodes::Success => Some("completed".to_string()),
            _ => None,
        },
        result_code: Some(status.code().to_string()),
        result_desc: Some(status.message().to_string()),
        mpesa_receipt_number: code,
        completed_at: Some(DateTime::UNIX_EPOCH),
    };

    if let Err(err) = TransactionRepository::update(&state.pool, transaction.id, update).await {
        println!("[error: {}] {}", state.project_id, err);
    }
    // only reduce balance if it was success
    if let (Some(user), StkCodes::Success) = (user, status) {
        let balance = user.balance - transaction.amount;
        if let Err(err) =
            User::update_by_id(&state.pool, user.id as i64, None, Some(balance), None, None).await
        {
            println!("[error: {}] {}", state.project_id, err);
        }
    }
}

#[tauri::command]
pub async fn resolve_stk_prompt(checkout_id: String, result: UserResponse) {
    let mut reg = STK_RESPONSE_REGISTRY.lock().await;
    if let Some(sender) = reg.remove(&checkout_id) {
        let _ = sender.send(result);
    }
}
