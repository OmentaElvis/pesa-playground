use std::time::Duration;

use anyhow::{Context, anyhow};
use axum::http::{HeaderMap, HeaderValue};
use serde_json::json;

use crate::transaction_costs::get_fee;
use crate::transactions::TransactionStatus;
use crate::transactions::ui::mask_msisdn_ke;
use crate::{
    accounts::{
        paybill_accounts::{self, PaybillAccount},
        till_accounts::{self, TillAccount},
        user_profiles::User,
    },
    business::BusinessSummary,
    projects::ProjectDetails,
    self_test::{
        callback::{CallbackCall, CallbackManager},
        context::TestContext,
        runner::TestStep,
        tests::get_access_token,
    },
    server::api::c2b::register::{RegisterUrlRequest, RegisterUrlResponse},
    server::api::c2b::{ResponseType, ValidationRequest, ValidationResponse},
    transactions::TransactionEngineError,
    transactions::ui::{LipaArgs, LipaPaymentType},
    transactions_log::FullTransactionLog,
};
use chrono::Utc;
pub struct C2BTest;

// --- Helper Enums/Structs for Test Harness ---

#[derive(Debug, PartialEq, Eq)]
enum TransactionOutcome {
    Success,
    Failed,
    /// For cases where API call itself fails, or validation rejects
    NoTransaction,
}

// Defines the expected outcomes for a single C2B test case.
struct C2BTestCase<'a> {
    name: &'a str,
    request: LipaArgs,
    /// For early bails in c2b_lipa_logic
    expected_bail_message: Option<&'a str>,
    expected_transaction_outcome: TransactionOutcome,
    // Expected details for the FullTransactionLog event
    expected_full_transaction_log: Option<FullTransactionLog>,
    /// Whether a validation URL was configured for the Paybill/Till being used in this test
    validation_url_configured: bool,
    /// Expected response from the test's validation callback (Some(true) for accept, Some(false) for reject, None if no validation callback expected)
    validation_callback_response: Option<bool>,
    /// Expected full ValidationRequest struct received by the validation callback
    expected_validation_request: Option<ValidationRequest>,
    /// Whether a confirmation URL was configured for the Paybill/Till being used in this test
    confirmation_url_configured: bool,
    /// Expected full ValidationRequest struct received by the confirmation callback
    expected_confirmation_request: Option<ValidationRequest>,
}

impl TestStep for C2BTest {
    async fn run(
        &self,
        context: &mut TestContext,
        callback_manager: &mut CallbackManager,
    ) -> anyhow::Result<()> {
        context.log("== Running C2B Suite ==").await;

        let project: ProjectDetails = context.get("project")?.unwrap();
        let business: BusinessSummary = context.get("business")?.unwrap();
        let broke_user: User = context.get("broke_user")?.unwrap();
        let rich_user: User = context.get("rich_user")?.unwrap();
        let base_url: String = context.get("base_url")?.unwrap();

        let token = get_access_token(context, &base_url, &project)
            .await
            .context("Failed to obtain access token.")?;

        let register_url_endpoint = format!("{}/mpesa/c2b/v2/registerurl", base_url);
        let mut auth_headers = HeaderMap::new();
        auth_headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", token.access_token))?,
        );

        // --- Create base Paybill and Till accounts for the business ---
        // These will be updated by the registerurl API calls
        context
            .log(">> Creating base Paybill and Till accounts...")
            .await;

        let base_paybill = paybill_accounts::PaybillAccount::create(
            &context.app_context.db,
            paybill_accounts::CreatePaybillAccount {
                business_id: business.id,
                paybill_number: 123000, // A fixed test paybill number
                response_type: None,
                validation_url: None,
                confirmation_url: None,
            },
        )
        .await
        .context("Failed to create base test paybill account")?;

        let base_till = till_accounts::TillAccount::create(
            &context.app_context.db,
            till_accounts::CreateTillAccount {
                business_id: business.id,
                till_number: 123001, // A fixed test till number
                location_description: Some("Test Till Location".to_string()),
                response_type: None,
                validation_url: None,
                confirmation_url: None,
            },
        )
        .await
        .context("Failed to create base test till account")?;

        // --- Generate callback URLs for our test server ---
        let test_validation_cb_url = callback_manager.get_callback_url("/test_c2b_validation");
        let test_confirmation_cb_url = callback_manager.get_callback_url("/test_c2b_confirmation");

        context
            .log(&format!(
                ">> Registering validation URL: {}",
                test_validation_cb_url
            ))
            .await;
        context
            .log(&format!(
                ">> Registering confirmation URL: {}",
                test_confirmation_cb_url
            ))
            .await;

        // --- Register validation and confirmation URLs for the test Paybill via API ---
        let register_paybill_req = RegisterUrlRequest {
            short_code: base_paybill.paybill_number,
            response_type: ResponseType::Completed,
            confirmation_url: test_confirmation_cb_url.clone(),
            validation_url: test_validation_cb_url.clone(),
        };

        let reg_paybill_res: RegisterUrlResponse = context
            .api_client
            .post_json(
                &register_url_endpoint,
                &register_paybill_req,
                Some(auth_headers.clone()),
            )
            .await
            .context("Failed to register URLs for test paybill")?;

        assert_eq!(
            reg_paybill_res.response_code, "000000",
            "Failed to register paybill URLs"
        );
        context
            .log(">> Successfully registered URLs for test Paybill")
            .await;

        // --- Register validation and confirmation URLs for the test Till via API ---
        let register_till_req = RegisterUrlRequest {
            short_code: base_till.till_number,
            response_type: ResponseType::Completed,
            confirmation_url: test_confirmation_cb_url.clone(),
            validation_url: test_validation_cb_url.clone(),
        };

        let reg_till_res: RegisterUrlResponse = context
            .api_client
            .post_json(
                &register_url_endpoint,
                &register_till_req,
                Some(auth_headers),
            )
            .await
            .context("Failed to register URLs for test till")?;

        assert_eq!(
            reg_till_res.response_code, "0",
            "Failed to register till URLs"
        );
        context
            .log(">> Successfully registered URLs for test Till")
            .await;

        // --- Retrieve the updated test Paybill and Till accounts ---
        let updated_test_paybill = paybill_accounts::PaybillAccount::get_by_paybill_number(
            &context.app_context.db,
            base_paybill.paybill_number,
        )
        .await?
        .context("Failed to retrieve updated test paybill")?;

        assert_eq!(
            updated_test_paybill.confirmation_url,
            Some(test_confirmation_cb_url.clone())
        );
        assert_eq!(
            updated_test_paybill.validation_url,
            Some(test_validation_cb_url.clone())
        );

        let updated_test_till = till_accounts::TillAccount::get_by_till_number(
            &context.app_context.db,
            base_till.till_number,
        )
        .await?
        .context("Failed to retrieve updated test till")?;

        assert_eq!(
            updated_test_till.validation_url,
            Some(test_validation_cb_url)
        );
        assert_eq!(
            updated_test_till.confirmation_url,
            Some(test_confirmation_cb_url)
        );

        // Store them in context for reuse by test cases
        context.set("test_paybill", &updated_test_paybill)?;
        context.set("test_till", &updated_test_till)?;

        // --- Run Test Cases ---

        // 1. Happy Path: Paybill Payment
        happy_path_paybill_payment(
            context,
            callback_manager,
            &business,
            &rich_user,
            &updated_test_paybill,
        )
        .await?;

        // 2. Happy Path: Till Payment
        happy_path_till_payment(
            context,
            callback_manager,
            &business,
            &rich_user,
            &updated_test_till,
        )
        .await?;

        // 3. Error: Invalid User Phone Number
        invalid_user_phone_number_test(context, callback_manager, &business, &rich_user).await?;

        // 4. Error: Paybill Missing Account Number
        paybill_missing_account_number_test(
            context,
            callback_manager,
            &business,
            &rich_user,
            &updated_test_paybill,
        )
        .await?;

        // 5. Error: Invalid Paybill Number (not registered with system)
        invalid_paybill_number_test(context, callback_manager, &business, &rich_user).await?;

        // 6. Error: Invalid Till Number (not registered with system)
        invalid_till_number_test(context, callback_manager, &business, &rich_user).await?;

        // 7. Error: Insufficient Funds
        insufficient_funds_test(
            context,
            callback_manager,
            &business,
            &broke_user,
            &updated_test_paybill,
        )
        .await?;

        // 8. Error: Validation URL Reject (Simulated)
        validation_url_reject_test(
            context,
            callback_manager,
            &business,
            &rich_user,
            &updated_test_paybill,
        )
        .await?;

        context.log("== C2B Suite Completed Successfully ==").await;
        Ok(())
    }
}

async fn execute_c2b_test_case(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    _business: &BusinessSummary,
    _user: &User,
    _paybill: Option<&PaybillAccount>,
    _till: Option<&TillAccount>,
    case: C2BTestCase<'_>,
) -> anyhow::Result<()> {
    context
        .log(&format!("-- Running Test Case: {} --", case.name))
        .await;

    // Pre-register listeners for validation/confirmation callbacks if configured
    let validation_callback_handle = if case.validation_url_configured {
        Some(callback_manager.register_callback::<ValidationRequest>("/test_c2b_validation")?)
    } else {
        None
    };
    let confirmation_callback_handle = if case.confirmation_url_configured {
        Some(callback_manager.register_callback::<ValidationRequest>("/test_c2b_confirmation")?)
    } else {
        None
    };

    // Listen for transaction events
    let new_transaction_event = context
        .event_manager
        .listen_for::<FullTransactionLog>("new_transaction", Duration::from_secs(5));

    // Directly call transactions::ui::lipa
    context
        .log(&format!(
            "Calling c2b_lipa_logic with args: {:#?}",
            case.request
        ))
        .await;

    let lipa_result =
        crate::transactions::ui::lipa(&context.app_context, case.request.clone()).await;

    // Handle expected immediate bail messages
    if let Some(expected_msg) = case.expected_bail_message {
        let error = lipa_result.expect_err(&format!(
            "[{}] Expected lipa() to bail with an error, but it succeeded.",
            case.name
        ));
        assert!(
            error.to_string().contains(expected_msg),
            "[{}] Expected bail message '{}' not found in error: {}",
            case.name,
            expected_msg,
            error
        );
        context
            .log(&format!(
                ">> Verified lipa() bailed early with message containing: '{}'",
                expected_msg
            ))
            .await;
        // Ensure no transaction or callbacks occur if it bailed early
        tokio::time::timeout(Duration::from_millis(100), new_transaction_event)
            .await
            .ok();
        if let Some(validation_callback_handle) = validation_callback_handle {
            let _ = tokio::time::timeout(Duration::from_millis(100), validation_callback_handle)
                .await
                .ok();
        }
        if let Some(confirmation_callback_handle) = confirmation_callback_handle {
            let _ = tokio::time::timeout(Duration::from_millis(100), confirmation_callback_handle)
                .await
                .ok();
        }

        return Ok(());
    } else {
        lipa_result?;
    }

    // Await validation callback if expected and respond
    if let Some(val_cb_handle) = validation_callback_handle {
        context.log(">> Waiting for validation callback...").await;
        let validation_call: CallbackCall<ValidationRequest> = val_cb_handle.await.context(
            format!("[{}] Did not receive validation callback", case.name),
        )?;
        context
            .log(&format!(
                ">> Received validation callback: {:#?}",
                validation_call.body
            ))
            .await;

        if let Some(expected_val_req) = case.expected_validation_request {
            let mut received_val_req_body = validation_call.body.clone();
            // Replace dynamic fields in received_val_req_body with expected_val_req's placeholders
            received_val_req_body.transaction_id = expected_val_req.transaction_id.clone();
            received_val_req_body.transaction_time = expected_val_req.transaction_time.clone();

            assert_eq!(
                received_val_req_body, expected_val_req,
                "[{}] Validation Request mismatch",
                case.name
            );
            context.log(">> Verified validation request body.").await;
        }

        if let Some(should_accept) = case.validation_callback_response {
            if should_accept {
                context
                    .log(">> Responding to validation callback with Accept (0)...")
                    .await;
                validation_call
                    .respond(
                        axum::http::StatusCode::OK,
                        &ValidationResponse {
                            result_code: crate::server::api::c2b::ResultCode::Ok,
                            result_desc: "Accepted".to_string(),
                            third_party_trans_id: Some("VALID123".to_string()),
                        },
                        None,
                    )
                    .await
                    .context("Failed to respond to validation callback")?;
            } else {
                context
                    .log(">> Responding to validation callback with Reject (1)...")
                    .await;
                validation_call
                    .respond(
                        axum::http::StatusCode::OK,
                        &ValidationResponse {
                            result_code: crate::server::api::c2b::ResultCode::C2B00011,
                            result_desc: "Rejected".to_string(),
                            third_party_trans_id: None,
                        },
                        None,
                    )
                    .await
                    .context("Failed to respond to validation callback")?;
            }
        }
    }

    // Await confirmation callback if expected
    if let Some(conf_cb_handle) = confirmation_callback_handle {
        context.log(">> Waiting for confirmation callback...").await;
        let confirmation_call: CallbackCall<ValidationRequest> = conf_cb_handle.await.context(
            format!("[{}] Did not receive confirmation callback", case.name),
        )?;
        context
            .log(&format!(
                ">> Received confirmation callback: {:#?}",
                confirmation_call.body
            ))
            .await;

        if let Some(expected_conf_req) = case.expected_confirmation_request {
            let mut received_conf_req_body = confirmation_call.body.clone();
            // Replace dynamic fields in received_conf_req_body with expected_conf_req's placeholders
            received_conf_req_body.transaction_id = expected_conf_req.transaction_id.clone();
            received_conf_req_body.transaction_time = expected_conf_req.transaction_time.clone();

            assert_eq!(
                received_conf_req_body, expected_conf_req,
                "[{}] Confirmation Request mismatch",
                case.name
            );
            context.log(">> Verified confirmation request body.").await;
        }

        // Respond to confirmation callback (usually 200 OK without specific body)
        confirmation_call
            .respond(axum::http::StatusCode::OK, &json!({"status": "OK"}), None)
            .await
            .context("Failed to respond to confirmation callback")?;
    }

    // Await new_transaction event and assert outcome
    match case.expected_transaction_outcome {
        TransactionOutcome::Success => {
            let mut received_log = new_transaction_event.await.context(format!(
                "[{}] Did not receive new_transaction event",
                case.name
            ))?;
            context
                .log(">> Verified new_transaction event was received for success.")
                .await;

            if let Some(expected_log) = case.expected_full_transaction_log {
                // Replace dynamic fields in received_log with expected_log's values for comparison
                received_log.transaction_id = expected_log.transaction_id.clone();
                received_log.transaction_date = expected_log.transaction_date;

                assert_eq!(
                    received_log, expected_log,
                    "[{}] Transaction Log mismatch",
                    case.name
                );
                context.log(">> Verified transaction log content.").await;
            }
        }
        TransactionOutcome::Failed => {
            tokio::time::timeout(Duration::from_millis(500), new_transaction_event)
                .await
                .map_err(|_| anyhow!("[{}] Expected new_transaction not to be received, and it wasn't (timed out).", case.name))
                .ok(); // Ignore timeout error, it means the event wasn't sent

            context
                .log(">> Verified no new_transaction event was received for failure.")
                .await;
        }
        TransactionOutcome::NoTransaction => {
            tokio::time::timeout(Duration::from_millis(500), new_transaction_event)
                .await
                .map_err(|_| {
                    anyhow!(
                        "[{}] Expected no new_transaction, and it wasn't (timed out).",
                        case.name
                    )
                })
                .ok();
            context
                .log(">> Verified no new_transaction event was received as expected.")
                .await;
        }
    }

    context
        .log(&format!("-- Test Case {} Passed --", case.name))
        .await;
    Ok(())
}

// --- Helper for creating LipaArgs ---
fn create_lipa_args(
    user_phone: String,
    amount: i64,
    payment_type: LipaPaymentType,
    business_number: u32,
    account_number: Option<String>,
) -> LipaArgs {
    LipaArgs {
        user_phone,
        amount,
        payment_type,
        business_number,
        account_number,
    }
}

// === TEST CASE IMPLEMENTATIONS ===

async fn happy_path_paybill_payment(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    business: &BusinessSummary,
    user: &User,
    paybill: &PaybillAccount,
) -> anyhow::Result<()> {
    // Fetch initial balances
    let initial_user_account =
        crate::accounts::Account::get_account(&context.app_context.db, user.account_id)
            .await?
            .context("User account not found")?;
    let initial_utility_account =
        crate::accounts::utility_accounts::UtilityAccount::find_by_business_id(
            &context.app_context.db,
            business.id,
        )
        .await?
        .context("Business utility account not found")?;

    let request_amount = 10 * 100; // KES 10.00
    let fee = get_fee(
        &context.app_context.db,
        &crate::transactions::TransactionType::Paybill,
        request_amount,
    )
    .await?;
    let total_deducted = request_amount + fee;

    let expected_user_balance_after = initial_user_account.balance - total_deducted;
    let expected_utility_account_balance_after = initial_utility_account.balance + request_amount;

    // Parse user name (as done in process_lipa)
    let parts: Vec<&str> = user.name.split_whitespace().collect();
    let (first_name, middle_name, last_name) = if parts.len() >= 3 {
        (parts[0], Some(parts[1]), Some(parts[2]))
    } else if parts.len() == 2 {
        (parts[0], None, Some(parts[1]))
    } else {
        (parts[0], None, None)
    };

    let request = create_lipa_args(
        user.phone.clone(),
        request_amount,
        LipaPaymentType::Paybill,
        paybill.paybill_number,
        Some("TESTACC".to_string()),
    );
    execute_c2b_test_case(
        context,
        callback_manager,
        business,
        user,
        Some(paybill),
        None, // Till account not involved
        C2BTestCase {
            name: "Happy Path: Paybill Payment",
            request: request.clone(),
            expected_bail_message: None,
            expected_transaction_outcome: TransactionOutcome::Success,
            expected_full_transaction_log: Some(FullTransactionLog {
                transaction_id: "".to_string(),        // Placeholder
                transaction_date: Utc::now().to_utc(), // Placeholder
                transaction_amount: request_amount,
                transaction_type: crate::transactions::TransactionType::Paybill.to_string(),
                from_name: user.name.clone(),
                to_name: business.name.clone(),
                from_id: Some(user.account_id),
                to_id: business.utility_account.account_id,
                new_balance: expected_user_balance_after,
                status: TransactionStatus::Completed.to_string(),
                fee,
                direction: crate::transactions_log::db::Direction::Outflow,
                notes: Some(crate::transactions::TransactionNote::PaybillPayment {
                    paybill_number: paybill.paybill_number,
                    bill_ref_number: "TESTACC".to_string(),
                }),
            }),
            validation_url_configured: paybill.validation_url.is_some(),
            validation_callback_response: Some(true), // Accept validation
            expected_validation_request: Some(ValidationRequest {
                transaction_type: crate::server::api::c2b::C2bTransactionType::PayBill,
                transaction_id: "".to_string(),   // Placeholder
                transaction_time: "".to_string(), // Placeholder
                transaction_amount: format!("{:.2}", request_amount as f64 / 100.0),
                business_shortcode: business.short_code.clone(),
                bill_ref_number: "TESTACC".to_string(),
                invoice_number: "".to_string(),
                org_account_balance: format!(
                    "{:.2}",
                    initial_utility_account.balance as f64 / 100.0
                ),
                third_party_transaction_id: "".to_string(), // Placeholder from process_lipa
                msisdn: mask_msisdn_ke(&user.phone),
                first_name: first_name.to_string(),
                middle_name: middle_name.map_or("".to_string(), |s| s.to_string()),
                last_name: last_name.map_or("".to_string(), |s| s.to_string()),
            }),
            confirmation_url_configured: paybill.confirmation_url.is_some(),
            expected_confirmation_request: Some(ValidationRequest {
                transaction_type: crate::server::api::c2b::C2bTransactionType::PayBill,
                transaction_id: "".to_string(),   // Placeholder
                transaction_time: "".to_string(), // Placeholder
                transaction_amount: format!("{:.2}", request_amount as f64 / 100.0),
                business_shortcode: business.short_code.clone(),
                bill_ref_number: "TESTACC".to_string(),
                invoice_number: "".to_string(),
                org_account_balance: format!(
                    "{:.2}",
                    expected_utility_account_balance_after as f64 / 100.0
                ),
                third_party_transaction_id: "VALID123".to_string(), // From validation callback response
                msisdn: mask_msisdn_ke(&user.phone),
                first_name: first_name.to_string(),
                middle_name: middle_name.map_or("".to_string(), |s| s.to_string()),
                last_name: last_name.map_or("".to_string(), |s| s.to_string()),
            }),
        },
    )
    .await
}

async fn happy_path_till_payment(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    business: &BusinessSummary,
    user: &User,
    till: &TillAccount,
) -> anyhow::Result<()> {
    // Fetch initial balances
    let initial_user_account =
        crate::accounts::Account::get_account(&context.app_context.db, user.account_id)
            .await?
            .context("User account not found")?;
    let initial_utility_account =
        crate::accounts::utility_accounts::UtilityAccount::find_by_business_id(
            &context.app_context.db,
            business.id,
        )
        .await?
        .context("Business utility account not found")?;

    let request_amount = 5 * 100; // KES 5.00
    let fee = get_fee(
        &context.app_context.db,
        &crate::transactions::TransactionType::BuyGoods,
        request_amount,
    )
    .await?;
    let total_deducted = request_amount + fee;

    let expected_user_balance_after = initial_user_account.balance - total_deducted;
    let expected_utility_account_balance_after = initial_utility_account.balance + request_amount;

    // Parse user name (as done in process_lipa)
    let parts: Vec<&str> = user.name.split_whitespace().collect();
    let (first_name, middle_name, last_name) = if parts.len() >= 3 {
        (parts[0], Some(parts[1]), Some(parts[2]))
    } else if parts.len() == 2 {
        (parts[0], None, Some(parts[1]))
    } else {
        (parts[0], None, None)
    };

    let request = create_lipa_args(
        user.phone.clone(),
        request_amount,
        LipaPaymentType::Till,
        till.till_number,
        None, // Till payments don't require account number
    );
    execute_c2b_test_case(
        context,
        callback_manager,
        business,
        user,
        None, // Paybill account not involved
        Some(till),
        C2BTestCase {
            name: "Happy Path: Till Payment",
            request: request.clone(),
            expected_bail_message: None,
            expected_transaction_outcome: TransactionOutcome::Success,
            expected_full_transaction_log: Some(FullTransactionLog {
                transaction_id: "".to_string(),        // Placeholder
                transaction_date: Utc::now().to_utc(), // Placeholder
                transaction_amount: request_amount,
                transaction_type: crate::transactions::TransactionType::BuyGoods.to_string(),
                from_name: user.name.clone(),
                to_name: business.name.clone(),
                from_id: Some(user.account_id),
                to_id: business.utility_account.account_id,
                new_balance: expected_user_balance_after,
                status: TransactionStatus::Completed.to_string(),
                fee,
                direction: crate::transactions_log::db::Direction::Outflow,
                notes: Some(crate::transactions::TransactionNote::TillPayment {
                    till_number: till.till_number,
                }),
            }),
            validation_url_configured: till.validation_url.is_some(),
            validation_callback_response: Some(true), // Accept validation
            expected_validation_request: Some(ValidationRequest {
                transaction_type: crate::server::api::c2b::C2bTransactionType::Till,
                transaction_id: "".to_string(),   // Placeholder
                transaction_time: "".to_string(), // Placeholder
                transaction_amount: format!("{:.2}", request_amount as f64 / 100.0),
                business_shortcode: business.short_code.clone(),
                bill_ref_number: "".to_string(), // No bill ref for till
                invoice_number: "".to_string(),
                org_account_balance: format!(
                    "{:.2}",
                    initial_utility_account.balance as f64 / 100.0
                ),
                third_party_transaction_id: "".to_string(), // Placeholder from process_lipa
                msisdn: mask_msisdn_ke(&user.phone),
                first_name: first_name.to_string(),
                middle_name: middle_name.map_or("".to_string(), |s| s.to_string()),
                last_name: last_name.map_or("".to_string(), |s| s.to_string()),
            }),
            confirmation_url_configured: till.confirmation_url.is_some(),
            expected_confirmation_request: Some(ValidationRequest {
                transaction_type: crate::server::api::c2b::C2bTransactionType::Till,
                transaction_id: "".to_string(),   // Placeholder
                transaction_time: "".to_string(), // Placeholder
                transaction_amount: format!("{:.2}", request_amount as f64 / 100.0),
                business_shortcode: business.short_code.clone(),
                bill_ref_number: "".to_string(), // No bill ref for till
                invoice_number: "".to_string(),
                org_account_balance: format!(
                    "{:.2}",
                    expected_utility_account_balance_after as f64 / 100.0
                ),
                third_party_transaction_id: "VALID123".to_string(), // From validation callback response
                msisdn: mask_msisdn_ke(&user.phone),
                first_name: first_name.to_string(),
                middle_name: middle_name.map_or("".to_string(), |s| s.to_string()),
                last_name: last_name.map_or("".to_string(), |s| s.to_string()),
            }),
        },
    )
    .await
}

async fn invalid_user_phone_number_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    business: &BusinessSummary,
    user: &User,
) -> anyhow::Result<()> {
    let request = create_lipa_args(
        "254700000000".to_string(), // Non-existent phone
        100,
        LipaPaymentType::Paybill,
        123456, // Dummy paybill
        Some("ACC123".to_string()),
    );
    execute_c2b_test_case(
        context,
        callback_manager,
        business, // Pass business
        user,     // Pass user
        None,     // Paybill account not involved
        None,     // Till account not involved
        C2BTestCase {
            name: "Error: Invalid User Phone Number",
            request,
            expected_bail_message: Some("User with phone number 254700000000 not found."),
            expected_transaction_outcome: TransactionOutcome::NoTransaction,
            expected_full_transaction_log: None,
            validation_url_configured: false,
            validation_callback_response: None,
            expected_validation_request: None,
            confirmation_url_configured: false,
            expected_confirmation_request: None,
        },
    )
    .await
}

async fn paybill_missing_account_number_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    business: &BusinessSummary,
    user: &User,
    paybill: &PaybillAccount,
) -> anyhow::Result<()> {
    let request = create_lipa_args(
        user.phone.clone(),
        1_00,
        LipaPaymentType::Paybill,
        paybill.paybill_number,
        None, // Missing account number
    );
    execute_c2b_test_case(
        context,
        callback_manager,
        business, // Pass business
        user,     // Pass user
        Some(paybill),
        None, // Till account not involved
        C2BTestCase {
            name: "Error: Paybill Missing Account Number",
            request: request.clone(),
            expected_bail_message: Some("Account number is required for paybill payments."),
            expected_transaction_outcome: TransactionOutcome::NoTransaction,
            expected_full_transaction_log: None,
            validation_url_configured: false,
            validation_callback_response: None,
            expected_validation_request: None,
            confirmation_url_configured: false,
            expected_confirmation_request: None,
        },
    )
    .await
}

async fn invalid_paybill_number_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    business: &BusinessSummary,
    user: &User,
) -> anyhow::Result<()> {
    let request = create_lipa_args(
        user.phone.clone(),
        1_00,
        LipaPaymentType::Paybill,
        999999, // Non-existent paybill
        Some("ACC123".to_string()),
    );
    execute_c2b_test_case(
        context,
        callback_manager,
        business,
        user,
        None, // Paybill not involved as it's invalid
        None, // Till not involved
        C2BTestCase {
            name: "Error: Invalid Paybill Number",
            request: request.clone(),
            expected_bail_message: Some("Paybill with business number 999999 not found."),
            expected_transaction_outcome: TransactionOutcome::NoTransaction,
            expected_full_transaction_log: None,
            validation_url_configured: false,
            validation_callback_response: None,
            expected_validation_request: None,
            confirmation_url_configured: false,
            expected_confirmation_request: None,
        },
    )
    .await
}

async fn invalid_till_number_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    business: &BusinessSummary,
    user: &User,
) -> anyhow::Result<()> {
    let request = create_lipa_args(
        user.phone.clone(),
        1_00,
        LipaPaymentType::Till,
        999999, // Non-existent till
        None,
    );
    execute_c2b_test_case(
        context,
        callback_manager,
        business, // Pass business
        user,     // Pass user
        None,     // Paybill not involved
        None,     // Till not involved as it's invalid
        C2BTestCase {
            name: "Error: Invalid Till Number",
            request: request.clone(),
            expected_bail_message: Some("Till account with business number 999999 not found."),
            expected_transaction_outcome: TransactionOutcome::NoTransaction,
            expected_full_transaction_log: None,
            validation_url_configured: false,
            validation_callback_response: None,
            expected_validation_request: None,
            confirmation_url_configured: false,
            expected_confirmation_request: None,
        },
    )
    .await
}

async fn insufficient_funds_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    business: &BusinessSummary,
    user: &User,
    paybill: &PaybillAccount,
) -> anyhow::Result<()> {
    let request = create_lipa_args(
        user.phone.clone(),
        100_000_000, // Very large amount 1M
        LipaPaymentType::Paybill,
        paybill.paybill_number,
        Some("ACC123".to_string()),
    );
    execute_c2b_test_case(
        context,
        callback_manager,
        business,
        user,
        Some(paybill),
        None, // Till account not involved
        C2BTestCase {
            name: "Error: Insufficient Funds",
            request: request.clone(),
            expected_bail_message: Some(
                TransactionEngineError::InsufficientFunds
                    .to_string()
                    .as_str(),
            ),
            expected_transaction_outcome: TransactionOutcome::NoTransaction,
            expected_full_transaction_log: None,
            validation_url_configured: paybill.validation_url.is_some(),
            validation_callback_response: None,
            expected_validation_request: None,
            confirmation_url_configured: paybill.confirmation_url.is_some(),
            expected_confirmation_request: None,
        },
    )
    .await
}

async fn validation_url_reject_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    business: &BusinessSummary,
    user: &User,
    paybill: &PaybillAccount,
) -> anyhow::Result<()> {
    // Fetch initial balances
    let initial_user_account =
        crate::accounts::Account::get_account(&context.app_context.db, user.account_id)
            .await?
            .context("User account not found")?;
    let initial_utility_account =
        crate::accounts::utility_accounts::UtilityAccount::find_by_business_id(
            &context.app_context.db,
            business.id,
        )
        .await?
        .context("Business utility account not found")?;

    let request_amount = 1_00;
    let fee = get_fee(
        &context.app_context.db,
        &crate::transactions::TransactionType::Paybill,
        request_amount,
    )
    .await?;

    // Balances should remain unchanged as transaction fails
    let expected_user_balance_after = initial_user_account.balance;
    // let expected_utility_account_balance_after = initial_utility_account.balance;

    // Parse user name (as done in process_lipa)
    let parts: Vec<&str> = user.name.split_whitespace().collect();
    let (first_name, middle_name, last_name) = if parts.len() >= 3 {
        (parts[0], Some(parts[1]), Some(parts[2]))
    } else if parts.len() == 2 {
        (parts[0], None, Some(parts[1]))
    } else {
        (parts[0], None, None)
    };

    let request = create_lipa_args(
        user.phone.clone(),
        request_amount,
        LipaPaymentType::Paybill,
        paybill.paybill_number,
        Some("TESTACC".to_string()),
    );
    execute_c2b_test_case(
        context,
        callback_manager,
        business,
        user,
        Some(paybill),
        None, // Till account not involved
        C2BTestCase {
            name: "Error: Validation URL Rejects",
            request: request.clone(),
            expected_bail_message: None,
            expected_transaction_outcome: TransactionOutcome::Failed, // Validation rejection leads to failed transaction
            expected_full_transaction_log: Some(FullTransactionLog {
                // Expect a FAILED log
                transaction_id: "".to_string(),        // Placeholder
                transaction_date: Utc::now().to_utc(), // Placeholder
                transaction_amount: request_amount,
                transaction_type: crate::transactions::TransactionType::Paybill.to_string(),
                from_name: user.name.clone(),
                to_name: business.name.clone(),
                from_id: Some(user.account_id),
                to_id: business.utility_account.account_id,
                new_balance: expected_user_balance_after, // Balance should be unchanged
                status: TransactionStatus::Failed.to_string(),
                fee,
                direction: crate::transactions_log::db::Direction::Outflow,
                notes: Some(crate::transactions::TransactionNote::PaybillPayment {
                    paybill_number: paybill.paybill_number,
                    bill_ref_number: "TESTACC".to_string(),
                }),
            }),
            validation_url_configured: paybill.validation_url.is_some(),
            validation_callback_response: Some(false), // Reject validation
            expected_validation_request: Some(ValidationRequest {
                transaction_type: crate::server::api::c2b::C2bTransactionType::PayBill,
                transaction_id: "".to_string(),   // Placeholder
                transaction_time: "".to_string(), // Placeholder
                transaction_amount: format!("{:.2}", request_amount as f64 / 100.0),
                business_shortcode: business.short_code.clone(),
                bill_ref_number: "TESTACC".to_string(),
                invoice_number: "".to_string(),
                org_account_balance: format!(
                    "{:.2}",
                    initial_utility_account.balance as f64 / 100.0
                ),
                third_party_transaction_id: "".to_string(), // Placeholder from process_lipa
                msisdn: mask_msisdn_ke(&user.phone),
                first_name: first_name.to_string(),
                middle_name: middle_name.map_or("".to_string(), |s| s.to_string()),
                last_name: last_name.map_or("".to_string(), |s| s.to_string()),
            }),
            confirmation_url_configured: paybill.confirmation_url.is_some(),
            expected_confirmation_request: None, // Confirmation won't be called
        },
    )
    .await
}
