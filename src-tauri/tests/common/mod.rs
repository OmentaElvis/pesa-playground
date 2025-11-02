use std::sync::Arc;

use axum::{
    body::Body,
    http::{HeaderMap, Request},
    response::Response,
    Router,
};
use chrono::Utc;
use fake::{
    faker::{company::en::CompanyName, name::en::Name, phone_number::en::PhoneNumber},
    Fake,
};
use pesa_playground_lib::{
    accounts, api_keys, business,
    db::run_migrations,
    projects::{self, SimulationMode},
    server, AppContext, TauriEventManager,
};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, Set};
use tauri::test::mock_builder;
use tower::ServiceExt;

pub struct TestApp {
    pub router: Router,
    pub db: DatabaseConnection,
}

impl TestApp {
    pub async fn new(project_id: u32, log: bool) -> anyhow::Result<Self> {
        let db = setup_db().await?;
        let t = mock_builder().build(tauri::generate_context!()).unwrap();

        let app_handle = TauriEventManager(t.handle().clone());
        let ctx = AppContext {
            db: db.clone(),
            event_manager: Arc::new(app_handle),
        };

        let router = server::create_router(ctx, project_id, log);
        Ok(Self { router, db })
    }

    pub async fn get(&self, url: &str, headers: Option<HeaderMap>) -> anyhow::Result<Response> {
        let mut request_builder = Request::builder().uri(url);
        if let Some(headers) = headers {
            for (key, value) in headers {
                request_builder = request_builder.header(key.unwrap(), value);
            }
        }
        let request = request_builder.body(Body::empty())?;
        let response = self.router.clone().oneshot(request).await?;
        Ok(response)
    }

    pub async fn post(
        &self,
        url: &str,
        body: Body,
        headers: Option<HeaderMap>,
    ) -> anyhow::Result<Response> {
        let mut request_builder = Request::builder()
            .uri(url)
            .method("POST")
            .header("Content-Type", "application/json");
        if let Some(headers) = headers {
            for (key, value) in headers {
                request_builder = request_builder.header(key.unwrap(), value);
            }
        }
        let request = request_builder.body(body)?;
        let response = self.router.clone().oneshot(request).await?;
        Ok(response)
    }
}

pub async fn setup_db() -> anyhow::Result<DatabaseConnection> {
    let db = Database::connect("sqlite::memory:").await?;
    run_migrations(&db).await?;
    Ok(db)
}

#[derive(Default)]
pub struct CreateTestBusinessOptions {
    pub name: Option<String>,
    pub short_code: Option<String>,
}

pub async fn create_test_business(
    db: &DatabaseConnection,
    options: Option<CreateTestBusinessOptions>,
) -> anyhow::Result<business::db::Model> {
    let options = options.unwrap_or_default();
    let business = business::db::ActiveModel {
        name: Set(options.name.unwrap_or_else(|| CompanyName().fake())),
        short_code: Set(options.short_code.unwrap_or_else(|| "600000".to_string())),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(business)
}

#[derive(Default)]
pub struct CreateTestProjectOptions {
    pub name: Option<String>,
    pub simulation_mode: Option<SimulationMode>,
}

pub async fn create_test_project(
    db: &DatabaseConnection,
    business_id: u32,
    options: Option<CreateTestProjectOptions>,
) -> anyhow::Result<projects::db::Model> {
    let options = options.unwrap_or_default();
    let project = projects::db::ActiveModel {
        name: Set(options.name.unwrap_or_else(|| CompanyName().fake())),
        business_id: Set(business_id),
        simulation_mode: Set(options
            .simulation_mode
            .unwrap_or(SimulationMode::AlwaysSuccess)
            .to_string()),
        stk_delay: Set(0),
        created_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(project)
}

#[derive(Default)]
pub struct CreateTestUserOptions {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub pin: Option<String>,
    pub balance: Option<i64>,
}

pub async fn create_test_user(
    db: &DatabaseConnection,
    options: Option<CreateTestUserOptions>,
) -> anyhow::Result<accounts::user_profiles::db::Model> {
    let options = options.unwrap_or_default();

    let account = accounts::db::ActiveModel {
        balance: Set(options.balance.unwrap_or(10000)),
        account_type: Set("user".to_string()),
        created_at: Set(Utc::now()),
        disabled: Set(false),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let user = accounts::user_profiles::db::ActiveModel {
        account_id: Set(account.id),
        name: Set(options.name.unwrap_or_else(|| Name().fake())),
        phone: Set(options.phone.unwrap_or_else(|| PhoneNumber().fake())),
        pin: Set(options.pin.unwrap_or_else(|| "1234".to_string())),
    }
    .insert(db)
    .await?;

    Ok(user)
}

use pesa_playground_lib::server::api::c2b::ResponseType;

#[derive(Default)]
pub struct CreateTestPaybillOptions {
    pub business_id: u32,
    pub paybill_number: Option<u32>,
    pub balance: Option<i64>,
    pub validation_url: Option<String>,
    pub confirmation_url: Option<String>,
    pub response_type: Option<ResponseType>,
}

pub async fn create_test_paybill(
    db: &DatabaseConnection,
    options: CreateTestPaybillOptions,
) -> anyhow::Result<accounts::paybill_accounts::db::Model> {
    let account = accounts::db::ActiveModel {
        balance: Set(options.balance.unwrap_or(1000000)),
        account_type: Set("paybill".to_string()),
        created_at: Set(Utc::now()),
        disabled: Set(false),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let paybill = accounts::paybill_accounts::db::ActiveModel {
        account_id: Set(account.id),
        business_id: Set(options.business_id),
        paybill_number: Set(options.paybill_number.unwrap_or(600000)),
        validation_url: Set(options.validation_url),
        confirmation_url: Set(options.confirmation_url),
        response_type: Set(options.response_type.map(|rt| rt.to_string())),
    }
    .insert(db)
    .await?;

    Ok(paybill)
}

#[derive(Default, Debug)]
pub struct CreateTestTillOptions {
    pub business_id: u32,
    pub till_number: Option<u32>,
    pub balance: Option<i64>,
    pub validation_url: Option<String>,
    pub confirmation_url: Option<String>,
    pub response_type: Option<ResponseType>,
}

pub async fn create_test_till(
    db: &DatabaseConnection,
    options: CreateTestTillOptions,
) -> anyhow::Result<accounts::till_accounts::db::Model> {
    let account = accounts::db::ActiveModel {
        balance: Set(options.balance.unwrap_or(1000000)),
        account_type: Set("till".to_string()),
        created_at: Set(Utc::now()),
        disabled: Set(false),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let till = accounts::till_accounts::db::ActiveModel {
        account_id: Set(account.id),
        business_id: Set(options.business_id),
        till_number: Set(options.till_number.unwrap_or(123456)),
        validation_url: Set(options.validation_url),
        confirmation_url: Set(options.confirmation_url),
        response_type: Set(options.response_type.map(|rt| rt.to_string())),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(till)
}

pub async fn create_test_api_key(
    db: &DatabaseConnection,
    project_id: u32,
) -> anyhow::Result<api_keys::db::Model> {
    let api_key = api_keys::db::ActiveModel {
        project_id: Set(project_id),
        consumer_key: Set(nanoid::nanoid!(20)),
        consumer_secret: Set(nanoid::nanoid!(20)),
        passkey: Set(nanoid::nanoid!(30)),
        created_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(api_key)
}
