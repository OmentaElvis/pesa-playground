use api::{auth::oauth, stkpush::stkpush};
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tauri::AppHandle;
use tokio::sync::oneshot;

pub mod api;
pub mod log;

#[derive(Debug, Clone)]
pub enum MpesaError {
    // ==== Auth / OAuth ====
    InvalidCredentials,          // 401.001.01
    InvalidAccessToken,          // 401.001.02
    MissingAccessToken,          // 401.001.03
    InvalidAuthenticationPassed, // 400.008.01
    InvalidGrantType,            // 400.008.02

    // ==== Common Input Validation ====
    InvalidPhoneNumber,       // 400.002.02
    InvalidAmount,            // 400.002.05
    InvalidShortcode,         // 400.002.07
    InvalidCommandId,         // 400.002.10
    InvalidCallbackUrl,       // 400.002.12
    InvalidIdentifierType,    // 400.002.13
    MissingRequiredParameter, // 400.002.14

    // ==== STK Push Specific ====
    AlreadyProcessingRequest, // 409.002.01
    RejectedByUser,           // 500.002.03
    UserInputTimeout,         // 500.002.04
    STKPushFailed,            // 500.002.05

    // ==== Transaction Status ====
    TransactionNotFound,     // 404.004.01
    TransactionNotCompleted, // 409.004.02

    // ==== Reversal ====
    ReversalNotAllowed,    // 403.005.01
    ReversalWindowExpired, // 400.005.03

    // ==== Internal / Generic ====
    InternalError,        // 500.001.01
    RateLimitExceeded,    // 429.001.01
    InvalidRequestFormat, // 400.001.01
    Unknown(StatusCode),  // fallback
}

impl MpesaError {
    pub fn to_response(&self) -> (StatusCode, &'static str, &'static str) {
        use MpesaError::*;
        match self {
            // --- OAuth Errors ---
            InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "401.001.01",
                "Invalid credentials",
            ),
            InvalidAccessToken => (
                StatusCode::UNAUTHORIZED,
                "401.001.02",
                "Invalid Access Token",
            ),
            MissingAccessToken => (
                StatusCode::UNAUTHORIZED,
                "401.001.03",
                "Access token missing",
            ),

            // --- Input Errors ---
            InvalidPhoneNumber => (
                StatusCode::BAD_REQUEST,
                "400.002.02",
                "Invalid Phone Number",
            ),
            InvalidAmount => (StatusCode::BAD_REQUEST, "400.002.05", "Invalid Amount"),
            InvalidShortcode => (StatusCode::BAD_REQUEST, "400.002.07", "Invalid Shortcode"),
            InvalidCommandId => (StatusCode::BAD_REQUEST, "400.002.10", "Invalid Command ID"),
            InvalidCallbackUrl => (
                StatusCode::BAD_REQUEST,
                "400.002.12",
                "Invalid Callback URL",
            ),
            InvalidIdentifierType => (
                StatusCode::BAD_REQUEST,
                "400.002.13",
                "Invalid IdentifierType",
            ),
            MissingRequiredParameter => (
                StatusCode::BAD_REQUEST,
                "400.002.14",
                "Missing required parameter",
            ),

            // --- STK Push ---
            AlreadyProcessingRequest => (
                StatusCode::CONFLICT,
                "409.002.01",
                "Duplicate STK request in progress",
            ),
            RejectedByUser => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500.002.03",
                "Request cancelled by user",
            ),
            UserInputTimeout => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500.002.04",
                "User input timed out",
            ),
            STKPushFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500.002.05",
                "STK Push failed",
            ),

            // --- Transaction ---
            TransactionNotFound => (StatusCode::NOT_FOUND, "404.004.01", "Transaction not found"),
            TransactionNotCompleted => (
                StatusCode::CONFLICT,
                "409.004.02",
                "Transaction not completed yet",
            ),

            // --- Reversal ---
            ReversalNotAllowed => (
                StatusCode::FORBIDDEN,
                "403.005.01",
                "Reversal not permitted",
            ),
            ReversalWindowExpired => (
                StatusCode::BAD_REQUEST,
                "400.005.03",
                "Reversal window expired",
            ),

            // --- Internal ---
            InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500.001.01",
                "Internal server error",
            ),
            RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "429.001.01",
                "Too many requests",
            ),
            InvalidRequestFormat => (StatusCode::BAD_REQUEST, "400.001.01", "Malformed request"),
            InvalidGrantType => (
                StatusCode::BAD_REQUEST,
                "400.008.02",
                "Invalid grant type passed",
            ),
            InvalidAuthenticationPassed => (
                StatusCode::BAD_REQUEST,
                "400.008.02",
                "Invalid Authentication passed",
            ),

            Unknown(status) => (*status, "400.000.00", "Unknown error"),
        }
    }
}

#[derive(Clone)]
pub struct ApiState {
    pub pool: SqlitePool,
    pub project_id: i64,
    pub handle: AppHandle,
}

pub async fn start_project_server(
    project_id: i64,
    port: u16,
    pool: SqlitePool,
    shutdown_rx: oneshot::Receiver<()>,
    handle: AppHandle,
) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    let state = ApiState {
        pool,
        project_id,
        handle,
    };
    let router = Router::new().route(
        "/",
        get(|| async {
    let banner = r#"    
______              ______ _                                             _ 
| ___ \             | ___ \ |                                           | |
| |_/ /__  ___  __ _| |_/ / | __ _ _   _  __ _ _ __ ___  _   _ _ __   __| |
|  __/ _ \/ __|/ _` |  __/| |/ _` | | | |/ _` | '__/ _ \| | | | '_ \ / _` |
| | |  __/\__ \ (_| | |   | | (_| | |_| | (_| | | | (_) | |_| | | | | (_| |
\_|  \___||___/\__,_\_|   |_|\__,_|\__, |\__, |_|  \___/ \__,_|_| |_|\__,_|
                                    __/ | __/ |                            
                                   |___/ |___/                             
    "#;
    format!("{banner}\n\n🧪 Welcome to Pesa Playground Sandbox.\nTry /mpesa/stkpush/v1/processrequest")
})).route("/oauth/v1/generate", get(oauth))
.route("/mpesa/stkpush/v1/processrequest", post(stkpush))
.with_state(state.clone())
.layer(axum::middleware::from_fn_with_state(state.clone(), log::logging_middleware));

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal(shutdown_rx))
        .await?;
    Ok(())
}

async fn shutdown_signal(shutdown_rx: oneshot::Receiver<()>) {
    shutdown_rx.await.ok();
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use serde_json::json;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ApiError {
    pub mpesa_error: MpesaError,
    pub internal_description: String,
}

impl ApiError {
    pub fn new(mpesa_error: MpesaError, internal_description: impl Into<String>) -> Self {
        Self {
            mpesa_error,
            internal_description: internal_description.into(),
        }
    }
}
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (status, code, msg) = self.mpesa_error.to_response();
        let body = json!({
            "status": status.to_string(),
            "errorCode": code,
            "errorMessage": msg,
        })
        .to_string();
        write!(f, "{}", body)
    }
}
impl std::error::Error for ApiError {}

// Implement IntoResponse to return only the public message
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = self.mpesa_error.to_response();
        let body = json!({
            "errorCode": code,
            "errorMessage": msg,
        })
        .to_string();

        let mut response = (status, Json(body)).into_response();
        response.extensions_mut().insert(self);

        response
    }
}

// Extension trait to add internal descriptions to Results
pub trait ResultExt<T> {
    fn with_internal_desc(
        self,
        mpesa_msg: MpesaError,
        internal_desc: impl Into<String>,
    ) -> Result<T, ApiError>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: std::error::Error,
{
    fn with_internal_desc(
        self,
        mpesa_msg: MpesaError,
        internal_desc: impl Into<String>,
    ) -> Result<T, ApiError> {
        self.map_err(|_| ApiError::new(mpesa_msg, internal_desc))
    }
}
