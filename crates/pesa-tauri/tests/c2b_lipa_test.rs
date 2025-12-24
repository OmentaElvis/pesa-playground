use pesa_playground_lib::transactions::ui::{LipaArgs, LipaPaymentType, c2b_lipa_logic};

use crate::common::TestApp;

mod common;

#[tokio::test]
async fn test_lipa_paybill_success() -> anyhow::Result<()> {
    let ctx = TestApp::new_context().await?;
    let db = &ctx.db;
    let business = common::create_test_business(db, None).await?;
    let _project = common::create_test_project(db, business.id, None).await?;
    let user = common::create_test_user(
        db,
        Some(common::CreateTestUserOptions {
            balance: Some(20000),
            ..Default::default()
        }),
    )
    .await?;
    let _paybill = common::create_test_paybill(
        db,
        common::CreateTestPaybillOptions {
            business_id: business.id,
            paybill_number: Some(600000),
            balance: Some(100000),
            ..Default::default()
        },
    )
    .await?;

    let lipa_args = LipaArgs {
        user_phone: user.phone,
        amount: 1000,
        payment_type: LipaPaymentType::Paybill,
        business_number: 600000,
        account_number: Some("12345".to_string()),
    };

    let result = c2b_lipa_logic(&ctx, lipa_args).await;

    assert!(result.is_ok());

    // We need to wait for the background task to finish
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // TODO: Add assertions to check if the balances have been updated correctly

    Ok(())
}

#[tokio::test]
async fn test_lipa_till_success() -> anyhow::Result<()> {
    let ctx = TestApp::new_context().await?;
    let db = &ctx.db;
    let business = common::create_test_business(db, None).await?;
    let _project = common::create_test_project(db, business.id, None).await?;
    let user = common::create_test_user(
        db,
        Some(common::CreateTestUserOptions {
            balance: Some(20000),
            ..Default::default()
        }),
    )
    .await?;

    let _till = common::create_test_till(
        db,
        common::CreateTestTillOptions {
            business_id: business.id,
            till_number: Some(123456),
            balance: Some(100000),
            ..Default::default()
        },
    )
    .await?;

    let lipa_args = LipaArgs {
        user_phone: user.phone,
        amount: 1000,
        payment_type: LipaPaymentType::Till,
        business_number: 123456,
        account_number: None,
    };

    let result = c2b_lipa_logic(&ctx, lipa_args).await;

    assert!(result.is_ok());

    // We need to wait for the background task to finish
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // TODO: Add assertions to check if the balances have been updated correctly

    Ok(())
}

#[tokio::test]
async fn test_lipa_insufficient_funds() -> anyhow::Result<()> {
    let ctx = TestApp::new_context().await?;
    let db = &ctx.db;
    let business = common::create_test_business(db, None).await?;
    let _project = common::create_test_project(db, business.id, None).await?;
    let user = common::create_test_user(
        db,
        Some(common::CreateTestUserOptions {
            balance: Some(500),
            ..Default::default()
        }),
    )
    .await?;
    let _paybill = common::create_test_paybill(
        db,
        common::CreateTestPaybillOptions {
            business_id: business.id,
            paybill_number: Some(600000),
            balance: Some(100000),
            ..Default::default()
        },
    )
    .await?;

    let lipa_args = LipaArgs {
        user_phone: user.phone,
        amount: 1000,
        payment_type: LipaPaymentType::Paybill,
        business_number: 600000,
        account_number: Some("12345".to_string()),
    };

    let result = c2b_lipa_logic(&ctx, lipa_args).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Insufficient funds");

    Ok(())
}

#[tokio::test]
async fn test_lipa_invalid_business_number() -> anyhow::Result<()> {
    let ctx = TestApp::new_context().await?;
    let db = &ctx.db;
    let business = common::create_test_business(db, None).await?;
    let _project = common::create_test_project(db, business.id, None).await?;
    let user = common::create_test_user(
        db,
        Some(common::CreateTestUserOptions {
            balance: Some(20000),
            ..Default::default()
        }),
    )
    .await?;

    let lipa_args = LipaArgs {
        user_phone: user.phone,
        amount: 1000,
        payment_type: LipaPaymentType::Paybill,
        business_number: 999999, // Invalid business number
        account_number: Some("12345".to_string()),
    };

    let result = c2b_lipa_logic(&ctx, lipa_args).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Paybill with business number 999999 not found."
    );

    Ok(())
}

#[tokio::test]
async fn test_lipa_missing_account_number_for_paybill() -> anyhow::Result<()> {
    let ctx = TestApp::new_context().await?;
    let db = &ctx.db;
    let business = common::create_test_business(db, None).await?;
    let _project = common::create_test_project(db, business.id, None).await?;
    let user = common::create_test_user(
        db,
        Some(common::CreateTestUserOptions {
            balance: Some(20000),
            ..Default::default()
        }),
    )
    .await?;
    let _paybill = common::create_test_paybill(
        db,
        common::CreateTestPaybillOptions {
            business_id: business.id,
            paybill_number: Some(600000),
            balance: Some(100000),
            ..Default::default()
        },
    )
    .await?;

    let lipa_args = LipaArgs {
        user_phone: user.phone,
        amount: 1000,
        payment_type: LipaPaymentType::Paybill,
        business_number: 600000,
        account_number: None, // Missing account number
    };

    let result = c2b_lipa_logic(&ctx, lipa_args).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Account number is required for paybill payments."
    );

    Ok(())
}

#[tokio::test]
async fn test_lipa_validation_fails() -> anyhow::Result<()> {
    let ctx = TestApp::new_context().await?;
    let db = &ctx.db;
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/validate")
        .with_status(422)
        .with_body("invalid json")
        .create_async()
        .await;

    let business = common::create_test_business(db, None).await?;
    let _project = common::create_test_project(db, business.id, None).await?;
    let user = common::create_test_user(
        db,
        Some(common::CreateTestUserOptions {
            balance: Some(20000),
            ..Default::default()
        }),
    )
    .await?;
    let _paybill = common::create_test_paybill(
        db,
        common::CreateTestPaybillOptions {
            business_id: business.id,
            paybill_number: Some(600000),
            balance: Some(100000),
            validation_url: Some(format!("{}/validate", server.url())),
            ..Default::default()
        },
    )
    .await?;

    let lipa_args = LipaArgs {
        user_phone: user.phone,
        amount: 1000,
        payment_type: LipaPaymentType::Paybill,
        business_number: 600000,
        account_number: Some("12345".to_string()),
    };

    let result = c2b_lipa_logic(&ctx, lipa_args).await;

    assert!(result.is_ok());

    // We need to wait for the background task to finish
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    mock.assert_async().await;

    // TODO: Add assertions to check if the api log was created

    Ok(())
}
