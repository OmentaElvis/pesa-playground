use crate::api_keys::db as api_keys_db;
use crate::server::access_token::db as access_token_db;
use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use base64::{Engine, engine::general_purpose};
use chrono::{DateTime, Duration, Utc};
use rand::{Rng, distributions::Alphanumeric};
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{EntityTrait, Set};
use serde::{Deserialize, Serialize};

use crate::server::ApiState;
use crate::{
    api_keys::ApiKey,
    server::{MpesaError, access_token::AccessToken},
};

use crate::server::ApiError;
pub const INVALID_ACCESS_TOKEN: &str = "INVALID_ACCESS_TOKEN";
pub const INVALID_CREDENTIALS: &str = "INVALID_CREDENTIALS";

#[derive(Deserialize, Debug)]
pub struct OAuthQuery {
    grant_type: Option<GrantType>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum GrantType {
    ClientCredentials,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    access_token: String,
    expires_in: String,
}

fn generate_access_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

pub async fn oauth(
    State(state): State<ApiState>,
    Query(query): Query<OAuthQuery>,
    headers: HeaderMap,
) -> Result<Json<AuthResponse>, ApiError> {
    let auth_error = "AUTH_ERROR";
    let invalid_grant_type = "INVALID_GRANT_TYPE";
    let missing_grant_type = "MISSING_GRANT_TYPE";

    match query.grant_type {
        Some(GrantType::Unknown) => {
            return Err(ApiError::new(
                crate::server::MpesaError::InvalidGrantType,
                invalid_grant_type.to_string(),
            ));
        }
        Some(GrantType::ClientCredentials) => {}
        None => {
            return Err(ApiError::new(
                crate::server::MpesaError::InvalidGrantType,
                missing_grant_type.to_string(),
            ));
        }
    }

    let auth = if let Some(auth) = headers.get("Authorization") {
        match auth.to_str() {
            Err(_) => {
                return Err(ApiError::new(
                    crate::server::MpesaError::InvalidAuthenticationPassed,
                    auth_error.to_string(),
                ));
            }
            Ok(auth) => auth,
        }
    } else {
        return Err(ApiError::new(
            crate::server::MpesaError::InvalidAuthenticationPassed,
            auth_error.to_string(),
        ));
    };

    if !auth.starts_with("Basic ") {
        return Err(ApiError::new(
            crate::server::MpesaError::InvalidAuthenticationPassed,
            auth_error.to_string(),
        ));
    }

    let b64 = &auth[6..];
    let decoded = general_purpose::STANDARD.decode(b64).map_err(|_| {
        ApiError::new(
            crate::server::MpesaError::InvalidAuthenticationPassed,
            auth_error.to_string(),
        )
    })?;

    let decoded_str = std::str::from_utf8(&decoded).map_err(|_| {
        ApiError::new(
            crate::server::MpesaError::InvalidAuthenticationPassed,
            auth_error.to_string(),
        )
    })?;

    let mut parts = decoded_str.splitn(2, ':');
    let key = parts.next().ok_or_else(|| {
        ApiError::new(
            crate::server::MpesaError::InvalidAuthenticationPassed,
            auth_error.to_string(),
        )
    })?;

    let secret = parts.next().ok_or_else(|| {
        ApiError::new(
            crate::server::MpesaError::InvalidAuthenticationPassed,
            auth_error.to_string(),
        )
    })?;

    let api_key = api_keys_db::Entity::find()
        .filter(api_keys_db::Column::ConsumerKey.eq(key))
        .filter(api_keys_db::Column::ConsumerSecret.eq(secret))
        .one(&state.context.db)
        .await
        .map_err(|e| {
            println!("{}", e);
            ApiError::new(
                crate::server::MpesaError::InvalidAuthenticationPassed,
                auth_error.to_string(),
            )
        })?;

    let api_key = api_key.ok_or_else(|| {
        ApiError::new(
            crate::server::MpesaError::InvalidAuthenticationPassed,
            auth_error.to_string(),
        )
    })?;

    let project_id: u32 = api_key.project_id;
    if project_id != state.project_id {
        return Err(ApiError::new(
            crate::server::MpesaError::InvalidAuthenticationPassed,
            auth_error.to_string(),
        ));
    }

    let access_token = generate_access_token();
    let new_access_token = access_token_db::ActiveModel {
        project_id: Set(project_id),
        token: Set(access_token.to_string()),
        expires_at: Set(Utc::now() + Duration::hours(1)),
        created_at: Set(Utc::now().to_utc()),
    };

    if let Err(err) = new_access_token.insert(&state.context.db).await {
        println!("{}", err);
    }

    Ok(Json(AuthResponse {
        access_token: access_token.to_string(),
        expires_in: Duration::hours(1).num_seconds().to_string(),
    }))
}

pub async fn validate_bearer_token(
    headers: &HeaderMap,
    state: &ApiState,
) -> Result<ApiKey, ApiError> {
    let auth = if let Some(auth) = headers.get("Authorization") {
        match auth.to_str() {
            Err(_) => {
                return Err(ApiError::new(
                    MpesaError::InvalidAccessToken,
                    INVALID_ACCESS_TOKEN,
                ));
            }
            Ok(auth) => auth,
        }
    } else {
        return Err(ApiError::new(
            MpesaError::InvalidAccessToken,
            INVALID_ACCESS_TOKEN,
        ));
    };

    if !auth.starts_with("Bearer ") {
        return Err(ApiError::new(
            MpesaError::InvalidAccessToken,
            INVALID_ACCESS_TOKEN,
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
            INVALID_ACCESS_TOKEN,
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
    Ok(api_key.unwrap())
}
