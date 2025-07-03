use axum::{extract::State, http::HeaderMap, Json};
use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Utc};
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    api_keys::{AccessToken, ApiKey},
    callbacks::{callback_start, CallbackLog, CreateCallbackLog},
    project::Project,
    server::{ApiError, ApiState},
    transaction::{CreateTransactionRequest, TransactionRepository},
    user::User,
};

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TransactionType {
    CustomerPayBillOnline,
    CustomerBuyGoodsOnline,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StkPushRequest {
    pub business_short_code: String,
    pub password: String,
    pub timestamp: String,
    pub transaction_type: TransactionType,
    pub amount: String,
    pub party_a: String,
    pub party_b: String,
    pub phone_number: String,
    pub call_back_u_r_l: String,
    pub account_reference: String,
    pub transaction_desc: String,
}

fn generate_merchant_request_id() -> String {
    format!(
        "{}-{}-{}",
        rand::rng().random_range(10000..99999),
        rand::rng().random_range(10000000..99999999),
        rand::rng().random_range(0..9)
    )
}

fn generate_checkout_request_id() -> String {
    let timestamp = Utc::now().format("%d%m%Y%H%M%S").to_string(); // e.g. 02072025143500
    let rand_suffix: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    format!("ws_CO_{}{}", timestamp, rand_suffix)
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StkPushResponse {
    pub merchant_request_i_d: String,
    pub checkout_request_i_d: String,
    pub response_code: String,
    pub response_description: String,
    pub customer_message: String,
}

pub async fn stkpush(
    headers: HeaderMap,
    State(state): State<ApiState>,
    Json(req): Json<StkPushRequest>,
) -> Result<Json<StkPushResponse>, ApiError> {
    let invalid_access_token = "INVALID_ACCESS_TOKEN";
    let invalid_credentials = "INVALID_CREDENTIALS";

    let auth = if let Some(auth) = headers.get("Authorization") {
        match auth.to_str() {
            Err(_) => {
                return Err(ApiError::new(
                    crate::server::MpesaError::MissingAccessToken,
                    invalid_access_token.to_string(),
                ))
            }
            Ok(auth) => auth,
        }
    } else {
        return Err(ApiError::new(
            crate::server::MpesaError::InvalidAccessToken,
            invalid_access_token.to_string(),
        ));
    };

    if !auth.starts_with("Bearer ") {
        return Err(ApiError::new(
            crate::server::MpesaError::MissingAccessToken,
            invalid_access_token.to_string(),
        ));
    }

    let key = &auth[7..];
    let access_token = AccessToken::read_by_token(&state.pool, key)
        .await
        .map_err(|error| {
            ApiError::new(
                crate::server::MpesaError::InternalError,
                format!("An internal error occured: {}", error),
            )
        })?;

    if access_token.is_none() {
        return Err(ApiError::new(
            crate::server::MpesaError::InvalidAccessToken,
            invalid_access_token,
        ));
    }

    let access_token = access_token.unwrap();
    let now = DateTime::UNIX_EPOCH;

    if now.gt(&access_token.expires_at) {
        return Err(ApiError::new(
            crate::server::MpesaError::InvalidAccessToken,
            invalid_access_token,
        ));
    }

    let project = Project::find_by_id(&state.pool, access_token.project_id)
        .await
        .map_err(|error| {
            ApiError::new(
                crate::server::MpesaError::InternalError,
                format!(
                    "An internal error occured while trying to get project {}: {}",
                    access_token.project_id, error
                ),
            )
        })?;

    if project.is_none() {
        return Err(ApiError::new(
            crate::server::MpesaError::InternalError,
            format!("Project with id {} not found", access_token.project_id),
        ));
    }

    let project = project.unwrap();

    let api_key = ApiKey::read_by_project_id(&state.pool, access_token.project_id)
        .await
        .map_err(|error| {
            ApiError::new(
                crate::server::MpesaError::InternalError,
                format!("An internal error occured: {}", error),
            )
        })?;

    if api_key.is_none() {
        return Err(ApiError::new(
            crate::server::MpesaError::InvalidCredentials,
            invalid_credentials,
        ));
    }
    let api_key = api_key.unwrap();
    let passkey = api_key.passkey;

    let short_code = project.shortcode.clone().unwrap_or(req.business_short_code);
    let timestamp = req.timestamp;

    let password =
        general_purpose::STANDARD.encode(format!("{}{}{}", short_code, passkey, timestamp));

    if !password.eq(&req.password) {
        return Err(ApiError::new(
            crate::server::MpesaError::InvalidCredentials,
            invalid_credentials,
        ));
    }

    let amount = req.amount.parse::<f64>().map_err(|err| {
        ApiError::new(
            crate::server::MpesaError::InternalError,
            format!(
                "Failed to parse amount as number: {}, you provided: {} ",
                err, req.amount
            ),
        )
    })?;

    let user = User::get_user_by_phone(&state.pool, req.phone_number)
        .await
        .map_err(|err| {
            ApiError::new(
                crate::server::MpesaError::InternalError,
                format!("An error occured while trying to get user: {}", err),
            )
        })?;

    let (phone, id) = if let Some(user) = &user {
        (Some(user.phone.to_string()), Some(user.id))
    } else {
        (None, None)
    };

    let merchant_id = generate_merchant_request_id();
    let checkout_id = generate_checkout_request_id();

    let transaction = CreateTransactionRequest {
        project_id: access_token.project_id,
        amount,
        merchant_request_id: Some(merchant_id.clone()),
        checkout_request_id: Some(checkout_id.clone()),
        short_code: Some(short_code),
        status: "pending".to_string(),
        account_reference: None,
        transaction_desc: None,
        user_id: id.unwrap_or(0) as i64,
        phone: phone.unwrap_or("NOT_FOUND".to_string()),
    };

    let transaction = TransactionRepository::create(&state.pool, transaction)
        .await
        .map_err(|err| {
            ApiError::new(
                crate::server::MpesaError::InternalError,
                format!(
                    "An internal error occured while trying to create transaction: {}",
                    err
                ),
            )
        })?;

    let callback = CreateCallbackLog {
        transaction_id: Some(transaction.id),
        checkout_request_id: Some(checkout_id.clone()),
        callback_url: req.call_back_u_r_l,
        response_body: None,
        status: "pending".to_string(),
        error: None,
        callback_type: "stkpush".to_string(),
        payload: String::new(),
        response_status: None,
    };

    let callback = CallbackLog::create(&state.pool, callback)
        .await
        .map_err(|err| {
            ApiError::new(
                crate::server::MpesaError::InternalError,
                format!(
                    "An internal error occured while trying to log callback: {}",
                    err
                ),
            )
        })?;

    tokio::spawn(callback_start(
        state.clone(),
        transaction,
        user,
        callback,
        project,
    ));

    Ok(Json(StkPushResponse {
        merchant_request_i_d: merchant_id,
        checkout_request_i_d: checkout_id,
        response_code: String::from("0"),
        response_description: String::from("The service request has been accepted successfully."),
        customer_message: String::from("Success"),
    }))
}
