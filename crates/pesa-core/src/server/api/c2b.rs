use axum::{Json, extract::State, http::HeaderMap};
use chrono::DateTime;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use strum::{Display, EnumString};

use crate::{
    accounts::{paybill_accounts::PaybillAccount, till_accounts::TillAccount},
    server::{ApiError, ApiState, MpesaError},
};
use serde::{Deserialize, Serialize};

use super::auth;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Display, EnumString)]
#[serde(rename_all = "PascalCase")]
#[strum(serialize_all = "PascalCase")]
pub enum ResponseType {
    Completed,
    Cancelled,
}

/// Request to register validation and confirmation URLs
#[derive(Debug, Deserialize)]
pub struct RegisterUrlRequest {
    #[serde(rename = "ShortCode")]
    pub short_code: u32,
    #[serde(rename = "ResponseType")]
    pub response_type: ResponseType, // "Completed" or "Cancelled"
    #[serde(rename = "ConfirmationURL")]
    pub confirmation_url: String,
    #[serde(rename = "ValidationURL")]
    pub validation_url: String,
}

/// Response after registering URLs
#[derive(Debug, Serialize)]
pub struct RegisterUrlResponse {
    #[serde(rename = "ResponseCode")]
    pub response_code: String,
    #[serde(rename = "OriginatorCoversationID")]
    pub originator_conversation_id: String,
    #[serde(rename = "ResponseDescription")]
    pub response_description: String,
}

pub async fn registerurl(
    headers: HeaderMap,
    State(state): State<ApiState>,
    Json(req): Json<RegisterUrlRequest>,
) -> Result<Json<RegisterUrlResponse>, ApiError> {
    let urls_already_registered = "URLS_ALREADY_REGISTERED";

    let _api_key = auth::validate_bearer_token(&headers, &state).await?;

    let short_code = req.short_code;
    if let Some(paybill) = PaybillAccount::get_by_paybill_number(&state.context.db, short_code)
        .await
        .map_err(|err| ApiError::new(MpesaError::InternalError, err.to_string()))?
    {
        if paybill.validation_url.is_some() || paybill.confirmation_url.is_some() {
            return Err(ApiError::new(
                MpesaError::UrlsAlreadyRegistered,
                urls_already_registered,
            ));
        }
        use crate::accounts::paybill_accounts::db;

        let model = db::Entity::find_by_id(paybill.id)
            .one(&state.context.db)
            .await
            .map_err(|err| ApiError::new(MpesaError::C2BServerFailure, err.to_string()))?
            .unwrap();

        let mut update_paybill: db::ActiveModel = model.into();
        update_paybill.response_type = Set(Some(req.response_type.to_string()));
        update_paybill.validation_url = Set(Some(req.validation_url.to_string()));
        update_paybill.confirmation_url = Set(Some(req.confirmation_url.to_string()));
        update_paybill
            .update(&state.context.db)
            .await
            .map_err(|err| ApiError::new(MpesaError::C2BServerFailure, err.to_string()))?;

        // TODO Generate valid conversation id
        return Ok(Json(RegisterUrlResponse {
            response_code: "000000".to_string(),
            originator_conversation_id: uuid::Uuid::new_v4().to_string(),
            response_description: "Success".to_string(),
        }));
    }

    if let Some(till) = TillAccount::get_by_till_number(&state.context.db, short_code)
        .await
        .map_err(|err| ApiError::new(MpesaError::InternalError, err.to_string()))?
    {
        if till.validation_url.is_some() || till.confirmation_url.is_some() {
            return Err(ApiError::new(
                MpesaError::UrlsAlreadyRegistered,
                urls_already_registered,
            ));
        }

        use crate::accounts::till_accounts::db;
        let model = db::Entity::find_by_id(till.id)
            .one(&state.context.db)
            .await
            .map_err(|err| ApiError::new(MpesaError::C2BServerFailure, err.to_string()))?
            .unwrap();

        let mut update_till: db::ActiveModel = model.into();
        update_till.response_type = Set(Some(req.response_type.to_string()));
        update_till.validation_url = Set(Some(req.validation_url.to_string()));
        update_till.confirmation_url = Set(Some(req.confirmation_url.to_string()));
        update_till
            .update(&state.context.db)
            .await
            .map_err(|err| ApiError::new(MpesaError::C2BServerFailure, err.to_string()))?;

        // TODO Generate valid conversation id
        return Ok(Json(RegisterUrlResponse {
            response_code: "000000".to_string(),
            originator_conversation_id: uuid::Uuid::new_v4().to_string(),
            response_description: "Success".to_string(),
        }));
    }

    Err(ApiError::new(
        MpesaError::C2BInvalidAccessToken,
        "The shortcode is not registered to the project",
    ))
}

#[derive(Debug, Deserialize, Serialize)]
pub enum C2bTransactionType {
    #[serde(rename = "Pay Bill")]
    PayBill,
    #[serde(rename = "Buy Goods")]
    Till,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidationRequest {
    #[serde(rename = "TransactionType")]
    pub transaction_type: C2bTransactionType,
    #[serde(rename = "TransID")]
    pub transaction_id: String,
    #[serde(rename = "TransTime")]
    pub transaction_time: String,
    #[serde(rename = "TransAmount")]
    pub transaction_amount: String,
    #[serde(rename = "BusinessShortCode")]
    pub business_shortcode: String,
    #[serde(rename = "BillRefNumber")]
    pub bill_ref_number: String,
    #[serde(rename = "InvoiceNumber")]
    pub invoice_number: String,
    #[serde(rename = "OrgAccountBalance")]
    pub org_account_balance: String,
    #[serde(rename = "ThirdPartyTransID")]
    pub third_party_transaction_id: String,
    #[serde(rename = "MSISDN")]
    pub msisdn: String,
    #[serde(rename = "FirstName")]
    pub first_name: String,
    #[serde(rename = "MiddleName")]
    pub middle_name: String,
    #[serde(rename = "LastName")]
    pub last_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ResultCode {
    C2B00011,
    C2B00012,
    C2B00013,
    C2B00014,
    C2B00015,
    C2B00016,
    #[serde(rename = "0")]
    Ok,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResultResponse {
    #[serde(rename = "ResultCode")]
    pub result_code: ResultCode,
    #[serde(rename = "ResultDesc")]
    pub result_desc: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidationResponse {
    #[serde(rename = "ResultCode")]
    pub result_code: ResultCode,
    #[serde(rename = "ResultDesc")]
    pub result_desc: String,
    #[serde(rename = "ThirdPartyTransID")]
    pub third_party_trans_id: Option<String>,
}
