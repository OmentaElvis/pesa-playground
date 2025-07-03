use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use base64::{engine::general_purpose, Engine};
use chrono::{Duration, Utc};
use rand::{distr::Alphanumeric, Rng};
use sea_query::{Expr, SqliteQueryBuilder};
use serde::{Deserialize, Serialize};
use sqlx::Executor;
use sqlx::Row;

use crate::{
    api_keys::{AccessTokens, ApiKeys},
    server::{ApiError, ApiState},
};

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
    rand::rng()
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
            ))
        }
        Some(GrantType::ClientCredentials) => {}
        None => {
            return Err(ApiError::new(
                crate::server::MpesaError::InvalidGrantType,
                missing_grant_type.to_string(),
            ))
        }
    }

    let auth = if let Some(auth) = headers.get("Authorization") {
        match auth.to_str() {
            Err(_) => {
                return Err(ApiError::new(
                    crate::server::MpesaError::InvalidAuthenticationPassed,
                    auth_error.to_string(),
                ))
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

    let sql = {
        sea_query::Query::select()
            .column(ApiKeys::ProjectId)
            .from(ApiKeys::Table)
            .and_where(Expr::col(ApiKeys::ConsumerKey).eq(key))
            .and_where(Expr::col(ApiKeys::ConsumerSecret).eq(secret))
            .to_string(SqliteQueryBuilder)
    };

    let row = state.pool.fetch_one(sql.as_str()).await.map_err(|e| {
        println!("{}", e);
        ApiError::new(
            crate::server::MpesaError::InvalidAuthenticationPassed,
            auth_error.to_string(),
        )
    })?;

    let project_id: i64 = row.get(0);
    if project_id != state.project_id {
        return Err(ApiError::new(
            crate::server::MpesaError::InvalidAuthenticationPassed,
            auth_error.to_string(),
        ));
    }

    let access_token = generate_access_token();
    let sql = {
        sea_query::Query::insert()
            .into_table(AccessTokens::Table)
            .columns([
                AccessTokens::ProjectId,
                AccessTokens::Token,
                AccessTokens::ExpiresAt,
            ])
            .values([
                project_id.into(),
                access_token.to_string().into(),
                (Utc::now() + Duration::hours(1)).timestamp().into(),
            ])
            .unwrap()
            .to_string(SqliteQueryBuilder)
    };

    if let Err(err) = state.pool.execute(sql.as_str()).await {
        println!("{}", err);
    }

    Ok(Json(AuthResponse {
        access_token: access_token.to_string(),
        expires_in: Duration::hours(1).num_seconds().to_string(),
    }))
}
