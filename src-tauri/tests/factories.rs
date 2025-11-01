use pesa_playground_lib::{accounts, projects, projects::SimulationMode, accounts::user_profiles::db::Entity as UserProfileEntity, accounts::paybill_accounts::db::Entity as PaybillEntity, accounts::till_accounts::db::Entity as TillEntity};
use sea_orm::EntityTrait;

use crate::common::{CreateTestBusinessOptions, CreateTestProjectOptions, CreateTestUserOptions, CreateTestPaybillOptions, CreateTestTillOptions};

mod common;

#[tokio::test]
async fn test_project_factory() -> anyhow::Result<()> {
    let db = common::setup_db().await?;
    let business = common::create_test_business(&db, None).await?;
    let project = common::create_test_project(&db, business.id, None).await?;

    let fetched_project = projects::db::Entity::find_by_id(project.id)
        .one(&db)
        .await?;

    assert!(fetched_project.is_some());
    let fetched_project = fetched_project.unwrap();

    assert_eq!(fetched_project.id, project.id);
    assert_eq!(fetched_project.business_id, business.id);
    assert_eq!(fetched_project.simulation_mode, "AlwaysSuccess");

    Ok(())
}

#[tokio::test]
async fn test_project_factory_with_options() -> anyhow::Result<()> {
    let db = common::setup_db().await?;
    let business = common::create_test_business(
        &db,
        Some(CreateTestBusinessOptions {
            short_code: Some("123456".to_string()),
            ..
            Default::default()
        }),
    )
    .await?;
    let project = common::create_test_project(
        &db,
        business.id,
        Some(CreateTestProjectOptions {
            simulation_mode: Some(SimulationMode::AlwaysFail),
            ..
            Default::default()
        }),
    )
    .await?;

    let fetched_project = projects::db::Entity::find_by_id(project.id)
        .one(&db)
        .await?;

    assert!(fetched_project.is_some());
    let fetched_project = fetched_project.unwrap();

    assert_eq!(fetched_project.id, project.id);
    assert_eq!(fetched_project.business_id, business.id);
    assert_eq!(fetched_project.simulation_mode, "AlwaysFail");

    Ok(())
}

#[tokio::test]
async fn test_user_factory() -> anyhow::Result<()> {
    let db = common::setup_db().await?;
    let user = common::create_test_user(
        &db,
        Some(CreateTestUserOptions {
            balance: Some(5000),
            ..
            Default::default()
        }),
    )
    .await?;

    let fetched_user = UserProfileEntity::find_by_id(user.account_id)
        .one(&db)
        .await?;

    assert!(fetched_user.is_some());
    let fetched_user = fetched_user.unwrap();

    assert_eq!(fetched_user.account_id, user.account_id);

    let fetched_account = accounts::db::Entity::find_by_id(user.account_id)
        .one(&db)
        .await?;
    
    assert!(fetched_account.is_some());
    let fetched_account = fetched_account.unwrap();

    assert_eq!(fetched_account.balance, 5000);


    Ok(())
}

#[tokio::test]
async fn test_paybill_factory() -> anyhow::Result<()> {
    let db = common::setup_db().await?;
    let business = common::create_test_business(&db, None).await?;
    let paybill = common::create_test_paybill(
        &db,
        CreateTestPaybillOptions {
            business_id: business.id,
            balance: Some(100000),
            paybill_number: Some(123456),
        },
    )
    .await?;

    let fetched_paybill = PaybillEntity::find_by_id(paybill.account_id)
        .one(&db)
        .await?;

    assert!(fetched_paybill.is_some());
    let fetched_paybill = fetched_paybill.unwrap();

    assert_eq!(fetched_paybill.account_id, paybill.account_id);
    assert_eq!(fetched_paybill.business_id, business.id);
    assert_eq!(fetched_paybill.paybill_number, 123456);

    let fetched_account = accounts::db::Entity::find_by_id(paybill.account_id)
        .one(&db)
        .await?;

    assert!(fetched_account.is_some());
    let fetched_account = fetched_account.unwrap();

    assert_eq!(fetched_account.balance, 100000);

    Ok(())
}

#[tokio::test]
async fn test_till_factory() -> anyhow::Result<()> {
    let db = common::setup_db().await?;
    let business = common::create_test_business(&db, None).await?;
    let till = common::create_test_till(
        &db,
        CreateTestTillOptions {
            business_id: business.id,
            balance: Some(50000),
            till_number: Some(654321),
        },
    )
    .await?;

    let fetched_till = TillEntity::find_by_id(till.account_id)
        .one(&db)
        .await?;

    assert!(fetched_till.is_some());
    let fetched_till = fetched_till.unwrap();

    assert_eq!(fetched_till.account_id, till.account_id);
    assert_eq!(fetched_till.business_id, business.id);
    assert_eq!(fetched_till.till_number, 654321);

    let fetched_account = accounts::db::Entity::find_by_id(till.account_id)
        .one(&db)
        .await?;

    assert!(fetched_account.is_some());
    let fetched_account = fetched_account.unwrap();

    assert_eq!(fetched_account.balance, 50000);

    Ok(())
}
