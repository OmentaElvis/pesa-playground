use axum::{extract::State, http::HeaderMap, Json};
use base64::{engine::general_purpose, Engine};
use chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::server::{ApiState, MpesaError};
use crate::{
    accounts::{
        paybill_accounts::PaybillAccount, till_accounts::TillAccount, user_profiles::User, Account,
    },
    api_keys::ApiKey,
    business::Business,
    callbacks::stk::init::StkpushInit,
    projects::Project,
    server::{access_token::AccessToken, ApiError},
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

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StkPushResponse {
    #[serde(rename = "MerchantRequestID")]
    pub merchant_request_id: String,
    #[serde(rename = "CheckoutRequestID")]
    pub checkout_request_id: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: i32,
    #[serde(rename = "ResponseDescription")]
    pub response_description: String,
    #[serde(rename = "CustomerMessage")]
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
                    MpesaError::InvalidAccessToken,
                    invalid_access_token,
                ))
            }
            Ok(auth) => auth,
        }
    } else {
        return Err(ApiError::new(
            MpesaError::InvalidAccessToken,
            invalid_access_token,
        ));
    };

    if !auth.starts_with("Bearer ") {
        return Err(ApiError::new(
            MpesaError::InvalidAccessToken,
            invalid_access_token,
        ));
    }

    let key = &auth[7..];
    let access_token = AccessToken::read_by_token(&state.context.db, key)
        .await
        .map_err(|error| {
            ApiError::new(
                crate::server::MpesaError::InternalError,
                format!("An internal error occured: {}", error),
            )
        })?;

    if access_token.is_none() {
        return Err(ApiError::new(
            MpesaError::InvalidAccessToken,
            invalid_access_token,
        ));
    }

    let access_token = access_token.unwrap();
    let now = DateTime::UNIX_EPOCH;

    if now.gt(&access_token.expires_at) {
        return Err(ApiError::new(
            MpesaError::InvalidAccessToken,
            "The access token has expired.",
        ));
    }

    let api_key = ApiKey::read_by_project_id(&state.context.db, access_token.project_id)
        .await
        .map_err(|error| {
            ApiError::new(
                crate::server::MpesaError::InternalError,
                format!("An internal error occured: {}", error),
            )
        })?;

    if api_key.is_none() {
        return Err(ApiError::new(
            MpesaError::InvalidCredentials,
            "Invalid credentials",
        ));
    }
    let api_key = api_key.unwrap();
    let passkey = api_key.passkey;
    let timestamp = req.timestamp;

    let (account_id, business_id) = match req.transaction_type {
        TransactionType::CustomerBuyGoodsOnline => {
            let till = match TillAccount::get_by_till_number(
                &state.context.db,
                req.party_b.parse().unwrap_or_default(),
            )
            .await
            {
                Ok(Some(till)) => till,
                Ok(None) => {
                    return Err(ApiError::new(
                        MpesaError::InvalidShortcode,
                        "Invalid Till Number",
                    ))
                }
                Err(err) => {
                    return Err(ApiError::new(MpesaError::InternalError, err.to_string()));
                }
            };
            (till.account_id, till.business_id)
        }
        TransactionType::CustomerPayBillOnline => {
            let paybill = match PaybillAccount::get_by_paybill_number(
                &state.context.db,
                req.party_b.parse().unwrap_or_default(),
            )
            .await
            {
                Ok(Some(paybill)) => paybill,
                Ok(None) => {
                    return Err(ApiError::new(
                        MpesaError::InvalidShortcode,
                        "Invalid Paybill Number",
                    ))
                }
                Err(err) => {
                    return Err(ApiError::new(MpesaError::InternalError, err.to_string()));
                }
            };

            (paybill.account_id, paybill.business_id)
        }
    };

    let business = match Business::get_by_id(&state.context.db, business_id).await {
        Ok(Some(business)) => business,
        Ok(None) => {
            return Err(ApiError::new(
                MpesaError::InvalidShortcode,
                "Invalid business shortcode.",
            ));
        }
        Err(err) => {
            return Err(ApiError::new(MpesaError::InternalError, err.to_string()));
        }
    };

    let entity_account = match Account::get_account(&state.context.db, account_id).await {
        Ok(Some(account)) => account,
        Ok(None) => {
            return Err(ApiError::new(
                crate::server::MpesaError::InternalError,
                format!("Failed to read acount {}", account_id),
            ))
        }
        Err(err) => {
            return Err(ApiError::new(
                crate::server::MpesaError::InternalError,
                err.to_string(),
            ));
        }
    };

    let short_code = business.short_code.clone();

    let password =
        general_purpose::STANDARD.encode(format!("{}{}{}", short_code, passkey, timestamp));

    if !password.eq(&req.password) {
        return Err(ApiError::new(
            MpesaError::InvalidCredentials,
            "Invalid password",
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

    let user = match User::get_user_by_phone(&state.context.db, &req.phone_number)
        .await
        .map_err(|err| {
            ApiError::new(
                crate::server::MpesaError::InternalError,
                format!("An error occured while trying to get user: {}", err),
            )
        })? {
        Some(user) => user,
        None => {
            return Err(ApiError::new(
                MpesaError::InvalidPhoneNumber,
                "Invalid phone number",
            ));
        }
    };

    let project = match Project::get_by_id(&state.context.db, state.project_id).await {
        Ok(Some(project)) => project,
        Ok(None) => {
            return Err(ApiError::new(
                crate::server::MpesaError::InvalidCredentials,
                invalid_credentials,
            ));
        }
        Err(err) => {
            return Err(ApiError::new(
                crate::server::MpesaError::InternalError,
                err.to_string(),
            ));
        }
    };

    let amount = (amount * 100.0).round() as i64;
    let init = StkpushInit::new(
        req.call_back_u_r_l.to_string(),
        user,
        entity_account,
        amount,
    );

    let merchant_id = init.merchant_request_id.to_string();
    let checkout_id = init.checkout_request_id.to_string();

    tokio::spawn(init.start(
        state.clone(),
        project,
        match req.transaction_type {
            TransactionType::CustomerPayBillOnline => crate::transactions::TransactionType::Paybill,
            TransactionType::CustomerBuyGoodsOnline => {
                crate::transactions::TransactionType::BuyGoods
            }
        },
    ));

    Ok(Json(StkPushResponse {
        merchant_request_id: merchant_id,
        checkout_request_id: checkout_id,
        response_code: 0,
        response_description: String::from("The service request has been accepted successfully."),
        customer_message: String::from("Success"),
    }))
}
