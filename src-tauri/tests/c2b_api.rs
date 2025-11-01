use axum::{body, body::Body, http::header::HeaderMap};
use base64::{engine::general_purpose, Engine};
use pesa_playground_lib::accounts::paybill_accounts::db::Entity as PaybillEntity;
use sea_orm::EntityTrait;
use serde_json::json;

mod common;

#[tokio::test]
async fn test_register_url_success() -> anyhow::Result<()> {
    let app = common::TestApp::new(1).await?;

    let business = common::create_test_business(&app.db, None).await?;
    let project = common::create_test_project(&app.db, business.id, None).await?;
    let api_key = common::create_test_api_key(&app.db, project.id).await?;
    let paybill = common::create_test_paybill(
        &app.db,
        common::CreateTestPaybillOptions {
            business_id: business.id,
            paybill_number: Some(600000),
            balance: Some(100000),
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

    let fetched_paybill = PaybillEntity::find_by_id(paybill.account_id)
        .one(&app.db)
        .await?;

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
