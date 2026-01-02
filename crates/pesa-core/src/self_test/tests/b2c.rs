use std::time::Duration;

use anyhow::{Context, anyhow};
use axum::http::{HeaderMap, HeaderValue};
use serde_json::Value;

use crate::{
    accounts::{Account, user_profiles::User},
    business::{self, BusinessSummary},
    business_operators::BusinessOperator,
    projects::ProjectDetails,
    self_test::{
        callback::{CallbackCall, CallbackManager},
        context::TestContext,
        runner::TestStep,
        tests::get_access_token,
    },
    server::api::b2c::{B2CCallbackResponse, B2CRequest, B2CRequestResponse, CommandID},
    settings,
    transactions_log::FullTransactionLog,
};

pub struct B2CTest;

/// Defines the expected outcomes for a single B2C test case.
struct B2CTestCase<'a> {
    name: &'a str,
    command_id: CommandID,
    amount: &'a str,
    recipient_phone: String,
    expected_api_status: u16,
    expected_callback_result_code: i32,
    should_succeed: bool,
}

impl TestStep for B2CTest {
    async fn run(
        &self,
        context: &mut TestContext,
        callback_manager: &mut CallbackManager,
    ) -> anyhow::Result<()> {
        context.log("== Running B2C Suite ==").await;

        let project: ProjectDetails = context
            .get("project")
            .context("Failed to get project from TestContext")?
            .unwrap();
        let business: BusinessSummary = context
            .get("business")
            .context("Failed to get business from TestContext")?
            .unwrap();
        let rich_user: User = context
            .get("rich_user")
            .context("Failed to get rich_user from TestContext")?
            .unwrap();
        let base_url: String = context
            .get("base_url")
            .context("Failed to get base_url from TestContext")?
            .unwrap();

        let operator: BusinessOperator = context
            .get("operator")
            .context("Failed to get operator from TestContext")?
            .unwrap();

        context
            .log(&format!("Using operator: {:#?}", operator))
            .await;

        let current_settings = context.app_context.settings.get().await;
        if let Some(keys) = current_settings.encryption_keys {
            context
                .log(&format!("Public Key: {}", keys.public_key))
                .await;
            context
                .log(&format!("Private Key: {}", keys.private_key))
                .await;
        } else {
            context.log("Encryption keys not found in settings. This might indicate an issue if security credentials are used.").await;
        }

        let token = get_access_token(context, &base_url, &project)
            .await
            .context("Failed to obtain access token.")?;

        happy_path_b2c(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &rich_user,
            &operator,
        )
        .await
        .context("Happy path test failed")?;
        insufficient_funds_test(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &rich_user,
            &operator,
        )
        .await
        .context("Insufficient funds test failed")?;
        invalid_recipient_test(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &operator,
        )
        .await
        .context("Invalid recipient test failed")?;
        invalid_credentials_test(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &rich_user,
            &operator,
        )
        .await
        .context("Invalid credentials test failed")?;

        context.log("== B2C Suite Completed Successfully ==").await;
        Ok(())
    }
}

/// # Test Harness
/// A generic function to execute a single B2C test case from start to finish.
async fn execute_b2c_test_case(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    business_summary: &BusinessSummary,
    operator: &BusinessOperator,
    case: B2CTestCase<'_>,
) -> anyhow::Result<()> {
    context
        .log(&format!("-- Running Test Case: {} --", case.name))
        .await;

    // --- Preparation ---
    let callback = callback_manager
        .register_callback::<B2CCallbackResponse>("/b2c_callback")
        .context("Failed to register B2C callback")?;
    let new_transaction_event = context
        .event_manager
        .listen_for::<FullTransactionLog>("new_transaction", Duration::from_secs(5));

    let security_credential = settings::ui::generate_security_credential(
        &context.app_context,
        operator.password.to_string(),
    )
    .await
    .context("Failed to generate security credential from password.")?;

    context
        .log(&format!(
            "Generated security credential: {}",
            security_credential
        ))
        .await;

    let initial_business_state =
        business::ui::get_business(&context.app_context, business_summary.id)
            .await
            .context("Failed to get initial business state")?;

    let initial_charges_amount = initial_business_state.charges_amount;
    let initial_utility_balance = initial_business_state.utility_account.balance;

    let recipient_user_initial_balance = if case.should_succeed {
        let user = User::get_user_by_phone(&context.app_context.db, &case.recipient_phone)
            .await
            .context("Failed to get user by phone while fetching initial recipient balance")?
            .unwrap();

        Some(user.balance)
    } else {
        None
    };

    context
        .log(&format!(
            "Initial Business Charges Amount: {}",
            initial_charges_amount
        ))
        .await;
    context
        .log(&format!(
            "Initial Utility Account Balance: {}",
            initial_utility_balance
        ))
        .await;
    if let Some(balance) = recipient_user_initial_balance {
        context
            .log(&format!(
                "Initial Recipient User Balance ({}): {}",
                case.recipient_phone, balance
            ))
            .await;
    }

    let request = B2CRequest {
        originator_conversation_id: "test-originator-conv-id".to_string(),
        initiator_name: operator.username.clone(),
        security_credential,
        command_id: case.command_id.clone(),
        amount: case.amount.to_string(),
        party_a: business_summary.short_code.clone(),
        party_b: case.recipient_phone.clone(),
        remarks: "Test B2C Payment".to_string(),
        queue_time_out_url: callback_manager.get_callback_url("/b2c_timeout"),
        result_url: callback.url().to_string(),
        occassion: "Test".to_string(),
    };

    context.log("B2C Request Details:").await;
    context
        .log(&format!(
            "  OriginatorConversationID: {}",
            request.originator_conversation_id
        ))
        .await;
    context
        .log(&format!("  InitiatorName: {}", request.initiator_name))
        .await;
    context
        .log(&format!("  CommandID: {:?}", request.command_id))
        .await;
    context.log(&format!("  Amount: {}", request.amount)).await;
    context
        .log(&format!(
            "  PartyA (Business Shortcode): {}",
            request.party_a
        ))
        .await;
    context
        .log(&format!("  PartyB (Recipient Phone): {}", request.party_b))
        .await;
    context
        .log(&format!("  ResultURL: {}", request.result_url))
        .await;

    // --- Execution ---
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token))
            .context("Failed to create Authorization header")?,
    );
    let url = format!("{}/mpesa/b2c/v3/paymentrequest", base_url);

    let response = context
        .api_client
        .post_json_raw(&url, &request, Some(headers))
        .await
        .context("Failed to send http post request")?;

    let status = response.status();

    if status.as_u16() != case.expected_api_status {
        let internal = response
            .headers()
            .get("X-Internal-Desc")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_owned())
            .unwrap_or("None".to_string());

        let error_body: Value = response
            .json()
            .await
            .context("Failed to parse error body as JSON")?;

        return Err(anyhow!(
            "[{}] Expected API status {} but got {}. Body: {} internal err: {}",
            case.name,
            case.expected_api_status,
            status,
            error_body,
            internal
        ));
    }
    context
        .log(&format!(">> Received expected API status: {}", status))
        .await;

    if !case.should_succeed {
        context
            .log(">> Test case expected to fail at API level, and it did. Test passed.")
            .await;
        return Ok(());
    }

    let res: B2CRequestResponse = response
        .json()
        .await
        .context("Failed to parse success response as JSON")?;
    context
        .log(&format!(">> Received API response body: {:#?}", res))
        .await;

    // --- Validation ---
    let callback_req: CallbackCall<B2CCallbackResponse> = callback
        .await
        .context(format!("[{}] Did not receive callback", case.name))?;
    let b2c_callback = callback_req.body;
    assert_eq!(
        b2c_callback.result.result_code,
        case.expected_callback_result_code.to_string(),
        "Callback ResultCode did not match"
    );

    if case.should_succeed {
        let transaction_event = new_transaction_event.await.context(format!(
            "[{}] Did not receive new transaction event",
            case.name
        ))?;
        context
            .log(">> Verified new_transaction event was received.")
            .await;

        context
            .log(&format!(
                "Transaction Event - ID: {}, Amount: {}, Fee: {}",
                transaction_event.transaction_id,
                transaction_event.transaction_amount,
                transaction_event.fee
            ))
            .await;

        let final_business_state =
            business::ui::get_business(&context.app_context, business_summary.id)
                .await
                .context("Failed to get final business state")?;

        context
            .log(&format!(
                "Final Business Charges Amount: {}",
                final_business_state.charges_amount
            ))
            .await;
        context
            .log(&format!(
                "Final Utility Account Balance: {}",
                final_business_state.utility_account.balance
            ))
            .await;

        // 1. Check utility account debit
        let expected_utility_balance =
            initial_utility_balance - transaction_event.transaction_amount;
        assert_eq!(
            final_business_state.utility_account.balance, expected_utility_balance,
            "Utility account balance mismatch"
        );

        // 2. Check charges amount
        let expected_charges_amount = initial_charges_amount - transaction_event.fee;
        assert_eq!(
            final_business_state.charges_amount, expected_charges_amount,
            "Business charges amount mismatch"
        );

        // 3. Check recipient balance
        let recipient_user =
            User::get_user_by_phone(&context.app_context.db, &case.recipient_phone)
                .await
                .context("Failed to get recipient user by phone")?
                .unwrap();
        let recipient_account =
            Account::get_account(&context.app_context.db, recipient_user.account_id)
                .await
                .context("Failed to get recipient account")?
                .unwrap();
        let expected_recipient_balance =
            recipient_user_initial_balance.unwrap() + transaction_event.transaction_amount;
        assert_eq!(
            recipient_account.balance, expected_recipient_balance,
            "Recipient account balance mismatch"
        );

        // 4. Check ResultParameters
        let result_params = b2c_callback.result.result_parameters.as_ref().unwrap();
        assert!(
            result_params
                .result_parameter
                .iter()
                .any(|p| p.key == "TransactionAmount"
                    && p.value.as_f64().unwrap() * 100.0
                        == transaction_event.transaction_amount as f64)
        );
        assert!(
            result_params
                .result_parameter
                .iter()
                .any(|p| p.key == "ReceiverPartyPublicName")
        );
    } else {
        // For failures, ensure balances haven't changed
        let final_business_state =
            business::ui::get_business(&context.app_context, business_summary.id)
                .await
                .context("Failed to get final business state for failure case")?;
        assert_eq!(
            final_business_state.utility_account.balance, initial_utility_balance,
            "Utility account balance should not change on failure"
        );
        assert_eq!(
            final_business_state.charges_amount, initial_charges_amount,
            "Charges amount should not change on failure"
        );
    }

    context
        .log(&format!("-- Test Case {} Passed --", case.name))
        .await;
    Ok(())
}

// === TEST CASE IMPLEMENTATIONS ===

async fn happy_path_b2c(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    business: &BusinessSummary,
    user: &User,
    operator: &BusinessOperator,
) -> anyhow::Result<()> {
    execute_b2c_test_case(
        context,
        callback_manager,
        token,
        base_url,
        business,
        operator,
        B2CTestCase {
            name: "Happy Path (BusinessPayment)",
            command_id: CommandID::BusinessPayment,
            amount: "100.00",
            recipient_phone: user.phone.clone(),
            expected_api_status: 200,
            expected_callback_result_code: 0,
            should_succeed: true,
        },
    )
    .await
    .context("B2C happy path test case failed")
}

async fn insufficient_funds_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    business: &BusinessSummary,
    user: &User,
    operator: &BusinessOperator,
) -> anyhow::Result<()> {
    execute_b2c_test_case(
        context,
        callback_manager,
        token,
        base_url,
        business,
        operator,
        B2CTestCase {
            name: "Error: Insufficient Funds",
            command_id: CommandID::BusinessPayment,
            amount: "100000000.00", // 100M, certainly more than available
            recipient_phone: user.phone.clone(),
            expected_api_status: 200,
            expected_callback_result_code: 1, // B2CResultCodes::InsufficientBalance
            should_succeed: false,
        },
    )
    .await
    .context("B2C insufficient funds test case failed")
}

async fn invalid_recipient_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    business: &BusinessSummary,
    operator: &BusinessOperator,
) -> anyhow::Result<()> {
    execute_b2c_test_case(
        context,
        callback_manager,
        token,
        base_url,
        business,
        operator,
        B2CTestCase {
            name: "Error: Invalid Recipient",
            command_id: CommandID::BusinessPayment,
            amount: "1.00",
            recipient_phone: "254700000000".to_string(), // Non-existent user
            expected_api_status: 400, // The API rejects this before the async flow
            expected_callback_result_code: 8006, // Not applicable, but required by struct
            should_succeed: false,
        },
    )
    .await
    .context("B2C invalid recipient test case failed")
}

async fn invalid_credentials_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    business: &BusinessSummary,
    user: &User,
    operator: &BusinessOperator,
) -> anyhow::Result<()> {
    let mut operator = operator.clone();
    operator.password = "wrong_password".to_string();

    execute_b2c_test_case(
        context,
        callback_manager,
        token,
        base_url,
        business,
        &operator,
        B2CTestCase {
            name: "Error: Invalid Operator Credentials",
            command_id: CommandID::BusinessPayment,
            amount: "1.00",
            recipient_phone: user.phone.clone(),
            expected_api_status: 401,            // Unauthorized
            expected_callback_result_code: 2001, // Not applicable
            should_succeed: false,
        },
    )
    .await
    .context("B2C invalid credentials test case failed")
}
