use axum::{body, body::Body, http::header::HeaderMap};
use base64::{Engine, engine::general_purpose};
use chrono::{Duration, Utc};
use pesa_playground_lib::accounts::paybill_accounts::db::Entity as PaybillEntity;
use pesa_playground_lib::accounts::till_accounts::db::Entity as TillEntity;
use pesa_playground_lib::server::ApiError;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;

mod common;

#[tokio::test]
async fn test_register_url_success() -> anyhow::Result<()> {
    let app = common::TestApp::new(1, false).await?;

    let business = common::create_test_business(&app.db, None).await?;
    let project = common::create_test_project(&app.db, business.id, None).await?;
    let api_key = common::create_test_api_key(&app.db, project.id).await?;
    let paybill = common::create_test_paybill(
        &app.db,
        common::CreateTestPaybillOptions {
            business_id: business.id,
            paybill_number: Some(600000),
            balance: Some(100000),
            ..Default::default()
        },
    )
    .await?;

    // Get access token
    let credentials = format!("{}:{}", api_key.consumer_key, api_key.consumer_secret);
    let token = general_purpose::STANDARD.encode(credentials);
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Basic {}", token).parse()?,
    );
    let response = app
        .get(
            "/oauth/v1/generate?grant_type=client_credentials",
            Some(headers),
        )
        .await?;

    assert_eq!(response.status(), 200);

    let body = body::to_bytes(response.into_body(), usize::MAX).await?;
    let auth_response: serde_json::Value = serde_json::from_slice(&body)?;
    let access_token = auth_response["access_token"].as_str().unwrap();

    // Register URL
    let request_body = json!({
        "ShortCode": 600000,
        "ResponseType": "Completed",
        "ConfirmationURL": "https://example.com/confirmation",
        "ValidationURL": "https://example.com/validation"
    });

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Bearer {}", access_token).parse()?,
    );
    let response = app
        .post(
            "/mpesa/c2b/v1/registerurl",
            Body::from(request_body.to_string()),
            Some(headers),
        )
        .await?;

    assert_eq!(response.status(), 200);

    let body = body::to_bytes(response.into_body(), usize::MAX).await?;
    let register_response: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(register_response["ResponseCode"], "000000");

    let fetched_paybill = PaybillEntity::find_by_id(paybill.id).one(&app.db).await?;

    assert!(fetched_paybill.is_some());
    let fetched_paybill = fetched_paybill.unwrap();

    assert_eq!(
        fetched_paybill.validation_url,
        Some("https://example.com/validation".to_string())
    );
    assert_eq!(
        fetched_paybill.confirmation_url,
        Some("https://example.com/confirmation".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn test_register_url_no_auth_header() -> anyhow::Result<()> {
    let app = common::TestApp::new(1, false).await?;

    let request_body = json!({
        "ShortCode": 600000,
        "ResponseType": "Completed",
        "ConfirmationURL": "https://example.com/confirmation",
        "ValidationURL": "https://example.com/validation"
    });

    let response = app
        .post(
            "/mpesa/c2b/v1/registerurl",
            Body::from(request_body.to_string()),
            None,
        )
        .await?;

    assert_eq!(response.status(), 401);
    let body = body::to_bytes(response.into_body(), usize::MAX).await?;
    let error_response: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(error_response["errorMessage"], "Invalid Access Token");

    Ok(())
}

#[tokio::test]
async fn test_register_url_invalid_auth_header_format() -> anyhow::Result<()> {
    let app = common::TestApp::new(1, false).await?;

    let request_body = json!({
        "ShortCode": 600000,
        "ResponseType": "Completed",
        "ConfirmationURL": "https://example.com/confirmation",
        "ValidationURL": "https://example.com/validation"
    });

    let mut headers = HeaderMap::new();
    headers.insert(axum::http::header::AUTHORIZATION, "Invalid token".parse()?);

    let response = app
        .post(
            "/mpesa/c2b/v1/registerurl",
            Body::from(request_body.to_string()),
            Some(headers),
        )
        .await?;

    assert_eq!(response.status(), 401);
    let body = body::to_bytes(response.into_body(), usize::MAX).await?;
    let error_response: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(error_response["errorMessage"], "Invalid Access Token");

    Ok(())
}

#[tokio::test]
async fn test_register_url_invalid_access_token() -> anyhow::Result<()> {
    let app = common::TestApp::new(1, false).await?;

    let request_body = json!({
        "ShortCode": 600000,
        "ResponseType": "Completed",
        "ConfirmationURL": "https://example.com/confirmation",
        "ValidationURL": "https://example.com/validation"
    });

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        "Bearer invalid_token".parse()?,
    );

    let response = app
        .post(
            "/mpesa/c2b/v1/registerurl",
            Body::from(request_body.to_string()),
            Some(headers),
        )
        .await?;

    assert_eq!(response.status(), 401);
    let body = body::to_bytes(response.into_body(), usize::MAX).await?;
    let error_response: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(error_response["errorMessage"], "Invalid Access Token");

    Ok(())
}

#[tokio::test]
async fn test_register_url_expired_access_token() -> anyhow::Result<()> {
    let app = common::TestApp::new(1, false).await?;
    let business = common::create_test_business(&app.db, None).await?;
    let project = common::create_test_project(&app.db, business.id, None).await?;
    let _api_key = common::create_test_api_key(&app.db, project.id).await?;

    // Create an expired access token
    let expired_token = "expired_token";
    let access_token_model = pesa_playground_lib::server::access_token::db::ActiveModel {
        token: Set(expired_token.to_string()),
        project_id: Set(project.id),
        expires_at: Set(Utc::now() - Duration::hours(1)),
        created_at: Set(Utc::now()),
    };
    access_token_model.insert(&app.db).await?;

    let request_body = json!({
        "ShortCode": 600000,
        "ResponseType": "Completed",
        "ConfirmationURL": "https://example.com/confirmation",
        "ValidationURL": "https://example.com/validation"
    });

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Bearer {}", expired_token).parse()?,
    );

    let response = app
        .post(
            "/mpesa/c2b/v1/registerurl",
            Body::from(request_body.to_string()),
            Some(headers),
        )
        .await?;

    assert_eq!(response.status(), 400);
    let body = body::to_bytes(response.into_body(), usize::MAX).await?;
    let error_response: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(error_response["errorMessage"], "Invalid Access Token");

    Ok(())
}

#[tokio::test]
async fn test_register_url_short_code_not_found() -> anyhow::Result<()> {
    let app = common::TestApp::new(1, false).await?;
    let business = common::create_test_business(&app.db, None).await?;
    let project = common::create_test_project(&app.db, business.id, None).await?;
    let api_key = common::create_test_api_key(&app.db, project.id).await?;

    // Get access token
    let credentials = format!("{}:{}", api_key.consumer_key, api_key.consumer_secret);
    let token = general_purpose::STANDARD.encode(credentials);
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Basic {}", token).parse()?,
    );
    let response = app
        .get(
            "/oauth/v1/generate?grant_type=client_credentials",
            Some(headers),
        )
        .await?;
    let body = body::to_bytes(response.into_body(), usize::MAX).await?;
    let auth_response: serde_json::Value = serde_json::from_slice(&body)?;
    let access_token = auth_response["access_token"].as_str().unwrap();

    let request_body = json!({
        "ShortCode": 999999, // A shortcode that doesn't exist
        "ResponseType": "Completed",
        "ConfirmationURL": "https://example.com/confirmation",
        "ValidationURL": "https://example.com/validation"
    });

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Bearer {}", access_token).parse()?,
    );

    let response = app
        .post(
            "/mpesa/c2b/v1/registerurl",
            Body::from(request_body.to_string()),
            Some(headers),
        )
        .await?;

    assert_eq!(response.status(), 400);

    let error = response.extensions().get::<ApiError>().unwrap();
    assert_eq!(
        error.internal_description,
        "The shortcode is not registered to the project"
    );

    let body = body::to_bytes(response.into_body(), usize::MAX).await?;
    let error_response: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(error_response["errorMessage"], "Invalid Access Token");

    Ok(())
}

#[tokio::test]
async fn test_register_url_already_registered() -> anyhow::Result<()> {
    let app = common::TestApp::new(1, false).await?;
    let business = common::create_test_business(&app.db, None).await?;
    let project = common::create_test_project(&app.db, business.id, None).await?;
    let api_key = common::create_test_api_key(&app.db, project.id).await?;
    let _paybill = common::create_test_paybill(
        &app.db,
        common::CreateTestPaybillOptions {
            business_id: business.id,
            paybill_number: Some(600000),
            balance: Some(100000),
            ..Default::default()
        },
    )
    .await?;

    // Get access token
    let credentials = format!("{}:{}", api_key.consumer_key, api_key.consumer_secret);
    let token = general_purpose::STANDARD.encode(credentials);
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Basic {}", token).parse()?,
    );
    let response = app
        .get(
            "/oauth/v1/generate?grant_type=client_credentials",
            Some(headers),
        )
        .await?;
    let body = body::to_bytes(response.into_body(), usize::MAX).await?;
    let auth_response: serde_json::Value = serde_json::from_slice(&body)?;
    let access_token = auth_response["access_token"].as_str().unwrap();

    // First registration
    let request_body = json!({
        "ShortCode": 600000,
        "ResponseType": "Completed",
        "ConfirmationURL": "https://example.com/confirmation",
        "ValidationURL": "https://example.com/validation"
    });
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Bearer {}", access_token).parse()?,
    );
    app.post(
        "/mpesa/c2b/v1/registerurl",
        Body::from(request_body.to_string()),
        Some(headers.clone()),
    )
    .await?;

    // Second registration
    let response = app
        .post(
            "/mpesa/c2b/v1/registerurl",
            Body::from(request_body.to_string()),
            Some(headers),
        )
        .await?;

    assert_eq!(response.status(), 500);
    let body = body::to_bytes(response.into_body(), usize::MAX).await?;
    let error_response: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(
        error_response["errorMessage"],
        "Urls are already registered."
    );

    Ok(())
}
#[tokio::test]
async fn test_register_url_success_for_till() -> anyhow::Result<()> {
    let app = common::TestApp::new(1, false).await?;
    let business = common::create_test_business(&app.db, None).await?;
    let project = common::create_test_project(&app.db, business.id, None).await?;
    let api_key = common::create_test_api_key(&app.db, project.id).await?;
    let till = common::create_test_till(
        &app.db,
        common::CreateTestTillOptions {
            business_id: business.id,
            till_number: Some(123456),
            balance: Some(100000),
            ..Default::default()
        },
    )
    .await?;

    // Get access token
    let credentials = format!("{}:{}", api_key.consumer_key, api_key.consumer_secret);
    let token = general_purpose::STANDARD.encode(credentials);
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Basic {}", token).parse()?,
    );
    let response = app
        .get(
            "/oauth/v1/generate?grant_type=client_credentials",
            Some(headers),
        )
        .await?;
    let body = body::to_bytes(response.into_body(), usize::MAX).await?;
    let auth_response: serde_json::Value = serde_json::from_slice(&body)?;
    let access_token = auth_response["access_token"].as_str().unwrap();

    // Register URL
    let request_body = json!({
        "ShortCode": 123456,
        "ResponseType": "Completed",
        "ConfirmationURL": "https://example.com/confirmation_till",
        "ValidationURL": "https://example.com/validation_till"
    });

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Bearer {}", access_token).parse()?,
    );
    let response = app
        .post(
            "/mpesa/c2b/v1/registerurl",
            Body::from(request_body.to_string()),
            Some(headers),
        )
        .await?;

    assert_eq!(response.status(), 200);

    let body = body::to_bytes(response.into_body(), usize::MAX).await?;
    let register_response: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(register_response["ResponseCode"], "000000");

    let fetched_till = TillEntity::find_by_id(till.id).one(&app.db).await?;

    assert!(fetched_till.is_some());
    let fetched_till = fetched_till.unwrap();

    assert_eq!(
        fetched_till.validation_url,
        Some("https://example.com/validation_till".to_string())
    );
    assert_eq!(
        fetched_till.confirmation_url,
        Some("https://example.com/confirmation_till".to_string())
    );

    Ok(())
}
