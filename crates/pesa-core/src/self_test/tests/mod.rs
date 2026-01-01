use anyhow::Context;
use axum::http::{HeaderMap, HeaderValue};
use base64::{Engine, engine::general_purpose};

use crate::{
    define_tests,
    projects::ProjectDetails,
    self_test::{context::TestContext, runner::TestStep},
    server::api::auth::AuthResponse,
};

pub mod b2c;
pub mod c2b;
pub mod create_project;
pub mod send_money;
pub mod stkpush;

define_tests!(
    InitProject {
        name: "init_project",
        description: "Prepares a test project for the next sequence of tests",
        ctor: create_project::InitProjectTest
    },
    SendMoney {
        name: "send_money",
        description: "Validates user-to-user (P2P) transfers, covering successful transactions, insufficient funds, and invalid account scenarios.",
        ctor: send_money::SendMoneyTest
    },
    Stkpush {
        name: "stkpush",
        description: "Performs stkpush",
        ctor: stkpush::StkpushTest
    },
    C2B {
        name: "c2b",
        description: "Performs C2B tests",
        ctor: c2b::C2BTest
    },
    B2C {
        name: "b2c",
        description: "Performs B2C tests",
        ctor: b2c::B2CTest
    },
);

pub(super) async fn get_access_token(
    context: &TestContext,
    base_url: &str,
    project: &ProjectDetails,
) -> anyhow::Result<AuthResponse> {
    let token = general_purpose::STANDARD.encode(format!(
        "{}:{}",
        project.consumer_key, project.consumer_secret
    ));

    let url = format!(
        "{}/oauth/v1/generate?grant_type=client_credentials",
        base_url
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Basic {token}"))?,
    );

    let res: AuthResponse = context
        .api_client
        .get_json(&url, Some(headers))
        .await
        .context("Failed to obtain access token from server")?;

    context
        .log(&format!(
            "------- Auth Access Token -----\n token: {} \n expiry: {} \n------------------------",
            res.access_token, res.expires_in
        ))
        .await;

    Ok(res)
}
