use std::time::Duration;

use anyhow::{Context, anyhow};
use axum::http::{HeaderMap, HeaderValue};
use base64::{Engine, engine::general_purpose};
use chrono::Utc;
use serde_json::Value;

use crate::{
    accounts::user_profiles::User,
    business::BusinessSummary,
    projects::ProjectDetails,
    self_test::{
        callback::CallbackManager, context::TestContext, runner::TestStep, tests::get_access_token,
    },
    server::api::stkpush::{
        StkCallback, StkCallbackBodyWrapper, StkPushRequest, StkPushResponse, StkPushResultCode,
        task::StkpushEvent,
        ui::{UserResponse, resolve_stk_prompt},
    },
    transactions_log::FullTransactionLog,
};

pub struct StkpushTest;

/// Represents the action a user takes in response to an STK push prompt.
#[derive(Clone)]
enum StkPromptAction {
    Accept { pin: String },
    Cancel,
    Timeout,
}

/// Defines the expected outcomes for a single STK push test case.
struct StkTestCase<'a> {
    name: &'a str,
    request: StkPushRequest,
    /// Expected HTTP status code for the initial API request
    expected_api_status: u16,
    /// Action to take if the API call is successful and a prompt is expected
    prompt_action: Option<StkPromptAction>,
    /// Expected result code in the final callback, if one is expected
    expected_callback_result_code: Option<i32>,
}

impl TestStep for StkpushTest {
    async fn run(
        &self,
        context: &mut TestContext,
        callback_manager: &mut CallbackManager,
    ) -> anyhow::Result<()> {
        context.log("== Running STK Push Suite ==").await;
        let project: ProjectDetails = context.get("project")?.unwrap();
        let business: BusinessSummary = context.get("business")?.unwrap();
        let broke_user: User = context.get("broke_user")?.unwrap();
        let rich_user: User = context.get("rich_user")?.unwrap();
        let base_url: String = context.get("base_url")?.unwrap();

        let token = get_access_token(context, &base_url, &project)
            .await
            .context("Failed to obtain access token.")?;

        happy_path_customer_paybill(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &rich_user,
        )
        .await?;
        insufficient_funds_test(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &broke_user,
        )
        .await?;
        user_cancelled_test(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &rich_user,
        )
        .await?;
        wrong_pin_test(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &rich_user,
        )
        .await?;
        prompt_timeout_test(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &rich_user,
        )
        .await?;
        invalid_shortcode_test(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &rich_user,
        )
        .await?;
        invalid_phone_number_test(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
        )
        .await?;

        context
            .log("== STK Push Suite Completed Successfully ==")
            .await;
        Ok(())
    }
}

/// # Test Harness
/// A generic function to execute a single STK push test case from start to finish.
async fn execute_stk_test_case(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    case: StkTestCase<'_>,
) -> anyhow::Result<()> {
    context
        .log(&format!("-- Running Test Case: {} --", case.name))
        .await;

    // Pre-register listeners
    let callback = callback_manager
        .register_callback::<StkCallbackBodyWrapper>("/callback")
        .context("Failed to register stkpush callback")?;
    let stk_push_event = context
        .event_manager
        .listen_for::<StkpushEvent>("stk_push", Duration::from_secs(5));
    let new_transaction_event = context
        .event_manager
        .listen_for::<FullTransactionLog>("new_transaction", Duration::from_secs(5));

    // Send the STK Push API request
    context
        .log(&format!(
            "Sending paybill stkpush request to shortcode: {}",
            case.request.business_short_code
        ))
        .await;

    // Set the dynamic callback URL for the request
    let mut modified_request = case.request.clone(); // Clone to modify
    modified_request.call_back_u_r_l = callback.url().to_string();

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );
    let url = format!("{}/mpesa/stkpush/v1/processrequest", base_url);

    // Execute the request and handle expected API-level errors
    let response = context
        .api_client
        .post_json_raw(&url, &modified_request, Some(headers))
        .await?;

    let status = response.status();

    if status.as_u16() != case.expected_api_status {
        let error_body: Value = response.json().await?;
        return Err(anyhow!(
            "[{}] Expected API status {} but got {}. Body: {}",
            case.name,
            case.expected_api_status,
            status,
            error_body
        ));
    }
    context
        .log(&format!(">> Received expected API status: {}", status))
        .await;

    // If no further action is expected (e.g., API call was meant to fail), end the test here.
    if case.prompt_action.is_none() {
        context
            .log(&format!("-- Test Case {} Passed --", case.name))
            .await;
        return Ok(());
    }

    // Continue with standard flow if a prompt is expected
    let res: StkPushResponse = response.json().await?;
    context
        .log(&format!(">> Received API response body: {:#?}", res))
        .await;

    let stk_push_event_data = stk_push_event
        .await
        .context(format!("[{}] Did not receive stk_push ui event", case.name))?;
    assert_eq!(
        stk_push_event_data.checkout_id, res.checkout_request_id,
        "[{}] Checkout ID mismatch",
        case.name
    );

    let user_response = match case.prompt_action.unwrap() {
        StkPromptAction::Accept { pin } => {
            context.log(">> Simulating user accepting prompt...").await;
            UserResponse::Accepted { pin }
        }
        StkPromptAction::Cancel => {
            context.log(">> Simulating user cancelling prompt...").await;
            UserResponse::Cancelled
        }
        StkPromptAction::Timeout => {
            context.log(">> Simulating user prompt timeout...").await;
            UserResponse::Timeout
        }
    };
    resolve_stk_prompt(
        &context.app_context,
        res.checkout_request_id.clone(),
        user_response,
    )
    .await?;

    // Wait for the final callback and assert the outcome
    let callback_req = callback
        .await
        .context(format!("[{}] Did not receive callback", case.name))?;
    let StkCallback { result_code, .. } = callback_req.body.body.callback;

    let expected_code = case.expected_callback_result_code.unwrap();
    assert_eq!(
        result_code, expected_code,
        "Callback ResultCode did not match"
    );

    // If the transaction was supposed to be successful, check for the event
    if expected_code == 0 {
        new_transaction_event.await.context(format!(
            "[{}] Did not receive new transaction event",
            case.name
        ))?;
        context
            .log(">> Verified new_transaction event was received.")
            .await;
    }

    context
        .log(&format!("-- Test Case {} Passed --", case.name))
        .await;
    Ok(())
}

fn create_stk_request(
    business_short_code: String,
    passkey: &str,
    phone_number: String,
    party_a: String,
    amount: &str,
) -> StkPushRequest {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let password = general_purpose::STANDARD
        .encode(format!("{}{}{}", business_short_code, passkey, timestamp));
    StkPushRequest {
        business_short_code: business_short_code.clone(),
        password,
        timestamp,
        transaction_type: crate::server::api::stkpush::TransactionType::CustomerPayBillOnline,
        amount: amount.to_string(),
        party_a,
        party_b: business_short_code,
        phone_number,
        call_back_u_r_l: "/callback".to_string(),
        account_reference: "Test".to_string(),
        transaction_desc: "test".to_string(),
    }
}

// === TEST CASE IMPLEMENTATIONS ===

async fn happy_path_customer_paybill(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    business: &BusinessSummary,
    user: &User,
) -> Result<(), anyhow::Error> {
    execute_stk_test_case(
        context,
        callback_manager,
        token,
        base_url,
        StkTestCase {
            name: "Happy Path (Correct PIN)",
            request: create_stk_request(
                business.short_code.clone(),
                &context.get::<ProjectDetails>("project")?.unwrap().passkey,
                user.phone.clone(),
                user.phone.clone(),
                "10",
            ),
            expected_api_status: 200,
            prompt_action: Some(StkPromptAction::Accept {
                pin: user.pin.clone(),
            }),
            expected_callback_result_code: Some(StkPushResultCode::Success.code()),
        },
    )
    .await
}

async fn insufficient_funds_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    business: &BusinessSummary,
    user: &User,
) -> Result<(), anyhow::Error> {
    execute_stk_test_case(
        context,
        callback_manager,
        token,
        base_url,
        StkTestCase {
            name: "Insufficient Funds",
            request: create_stk_request(
                business.short_code.clone(),
                &context.get::<ProjectDetails>("project")?.unwrap().passkey,
                user.phone.clone(),
                user.phone.clone(),
                "1000000",
            ),
            expected_api_status: 200,
            prompt_action: Some(StkPromptAction::Accept {
                pin: user.pin.clone(),
            }),
            expected_callback_result_code: Some(StkPushResultCode::InsufficientBalance.code()),
        },
    )
    .await
}

async fn user_cancelled_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    business: &BusinessSummary,
    user: &User,
) -> Result<(), anyhow::Error> {
    execute_stk_test_case(
        context,
        callback_manager,
        token,
        base_url,
        StkTestCase {
            name: "User Cancelled",
            request: create_stk_request(
                business.short_code.clone(),
                &context.get::<ProjectDetails>("project")?.unwrap().passkey,
                user.phone.clone(),
                user.phone.clone(),
                "1",
            ),
            expected_api_status: 200,
            prompt_action: Some(StkPromptAction::Cancel),
            expected_callback_result_code: Some(StkPushResultCode::RequestCancelledByUser.code()),
        },
    )
    .await
}

async fn wrong_pin_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    business: &BusinessSummary,
    user: &User,
) -> Result<(), anyhow::Error> {
    execute_stk_test_case(
        context,
        callback_manager,
        token,
        base_url,
        StkTestCase {
            name: "Wrong PIN",
            request: create_stk_request(
                business.short_code.clone(),
                &context.get::<ProjectDetails>("project")?.unwrap().passkey,
                user.phone.clone(),
                user.phone.clone(),
                "1",
            ),
            expected_api_status: 200,
            prompt_action: Some(StkPromptAction::Accept {
                pin: "0000".to_string(),
            }),
            expected_callback_result_code: Some(
                StkPushResultCode::InitiatorInformationInvalid.code(),
            ),
        },
    )
    .await
}

async fn prompt_timeout_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    business: &BusinessSummary,
    user: &User,
) -> Result<(), anyhow::Error> {
    execute_stk_test_case(
        context,
        callback_manager,
        token,
        base_url,
        StkTestCase {
            name: "Prompt Timeout",
            request: create_stk_request(
                business.short_code.clone(),
                &context.get::<ProjectDetails>("project")?.unwrap().passkey,
                user.phone.clone(),
                user.phone.clone(),
                "1",
            ),
            expected_api_status: 200,
            prompt_action: Some(StkPromptAction::Timeout),
            expected_callback_result_code: Some(StkPushResultCode::DSTimeout.code()),
        },
    )
    .await
}

async fn invalid_shortcode_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    user: &User,
) -> Result<(), anyhow::Error> {
    execute_stk_test_case(
        context,
        callback_manager,
        token,
        base_url,
        StkTestCase {
            name: "Invalid Shortcode",
            request: create_stk_request(
                "999999".to_string(),
                &context.get::<ProjectDetails>("project")?.unwrap().passkey,
                user.phone.clone(),
                user.phone.clone(),
                "1",
            ),
            expected_api_status: 400,
            prompt_action: None,
            expected_callback_result_code: None,
        },
    )
    .await
}

async fn invalid_phone_number_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    business: &BusinessSummary,
) -> Result<(), anyhow::Error> {
    execute_stk_test_case(
        context,
        callback_manager,
        token,
        base_url,
        StkTestCase {
            name: "Invalid Phone Number",
            request: create_stk_request(
                business.short_code.clone(),
                &context.get::<ProjectDetails>("project")?.unwrap().passkey,
                "254700000000".to_string(),
                "254700000000".to_string(),
                "1",
            ),
            expected_api_status: 400,
            prompt_action: None,
            expected_callback_result_code: None,
        },
    )
    .await
}
