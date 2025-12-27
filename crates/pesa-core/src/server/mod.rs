use crate::{
    AppContext,
    server::{
        api::{b2c::task::B2C, c2b::register::registerurl, stkpush::task::Stkpush},
        async_handler::handle_async_request,
    },
};
use api::auth::oauth;
use axum::{
    Router,
    routing::{get, post},
};
use tokio::sync::oneshot;

pub mod access_token;
pub mod api;
pub mod async_handler;
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

    // ==== C2B ======
    UrlsAlreadyRegistered,
    C2BServerFailure,      // 500.003.1001
    C2BInvalidAccessToken, // 400.003.01
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
            UrlsAlreadyRegistered => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500.003.1001",
                "Urls are already registered.",
            ),
            C2BInvalidAccessToken => (
                StatusCode::BAD_REQUEST,
                "400.003.01",
                "Invalid Access Token",
            ),
            C2BServerFailure => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500.003.01",
                "Internal server error",
            ),

            Unknown(status) => (*status, "400.000.00", "Unknown error"),
        }
    }
}

#[derive(Clone)]
pub struct ApiState {
    pub context: AppContext,
    pub project_id: u32,
}

pub fn create_router(context: AppContext, project_id: u32, log: bool) -> Router {
    let state = ApiState {
        context,
        project_id,
    };

    let mut router = Router::new().route(
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
}))
    .route("/oauth/v1/generate", get(oauth))
    .route("/mpesa/stkpush/v1/processrequest", post(handle_async_request::<Stkpush>))
    .route("/mpesa/c2b/v2/registerurl", post(registerurl))
    .route("/mpesa/b2c/v3/paymentrequest", post(handle_async_request::<B2C>))
    .with_state(state.clone());

    if log {
        router = router.layer(axum::middleware::from_fn_with_state(
            state.clone(),
            log::logging_middleware,
        ));
    }

    router
}

pub async fn start_project_server(
    project_id: u32,
    port: u16,
    context: AppContext,
    shutdown_rx: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    context.event_manager.emit_all(
        "sandbox_status",
        json!({
            "project_id": project_id,
            "port": port,
            "status": "on",
        }),
    )?;

    let router = create_router(context.clone(), project_id, true);

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal(shutdown_rx))
        .await?;

    Ok(())
}

async fn shutdown_signal(shutdown_rx: oneshot::Receiver<()>) {
    shutdown_rx.await.ok();
}

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
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
        });

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
