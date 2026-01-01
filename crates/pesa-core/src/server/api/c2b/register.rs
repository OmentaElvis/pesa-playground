use axum::{Json, extract::State, http::HeaderMap};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::{
    accounts::{paybill_accounts::PaybillAccount, till_accounts::TillAccount},
    server::{
        ApiError, ApiState, MpesaError,
        api::{auth, c2b::ResponseType},
    },
};

/// Request to register validation and confirmation URLs
#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterUrlRequest {
    #[serde(rename = "ShortCode")]
    pub short_code: u32,
    #[serde(rename = "ResponseType")]
    pub response_type: ResponseType,
    #[serde(rename = "ConfirmationURL")]
    pub confirmation_url: String,
    #[serde(rename = "ValidationURL")]
    pub validation_url: String,
}

/// Response after registering URLs
#[derive(Debug, Serialize, Deserialize)]
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
            response_code: "0".to_string(),
            originator_conversation_id: uuid::Uuid::new_v4().to_string(),
            response_description: "Success".to_string(),
        }));
    }

    Err(ApiError::new(
        MpesaError::C2BInvalidAccessToken,
        "The shortcode is not registered to the project",
    ))
}
