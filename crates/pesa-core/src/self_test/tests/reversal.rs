use std::time::Duration;

use anyhow::{Context, anyhow};
use axum::http::{HeaderMap, HeaderValue};

use crate::{
    accounts::{user_profiles, utility_accounts::UtilityAccount},
    business::BusinessSummary,
    business_operators::BusinessOperator,
    projects::ProjectDetails,
    self_test::{
        callback::{CallbackEntry, CallbackManager},
        context::TestContext,
        runner::TestStep,
        tests::get_access_token,
    },
    server::api::reversal::{ReversalCallbackResponse, ReversalRequest, ReversalResponse},
    settings,
    transaction_costs,
    transactions::{Ledger, TransactionStatus, TransactionType, db as txn_db},
};

pub struct ReversalTest;

#[derive(Clone)]
struct ReversalTestCase<'a> {
    name: &'a str,
    /// Override the operator (None = use the default operator from context)
    initiator_override: Option<&'a str>,
    /// Security credential override (None = generate from the operator's password)
    security_credential_override: Option<&'a str>,
    /// TransactionID to reverse
    transaction_id: String,
    /// Expected HTTP status for the sync response
    expected_api_status: u16,
    /// Expected ResponseCode in the sync response (if status is 200)
    expected_sync_response_code: Option<&'a str>,
    /// Expected ResultCode in the callback (None if no callback expected)
    expected_callback_result_code: Option<&'a str>,
    /// Expected ResultDesc in the callback (None if no callback expected)
    expected_callback_result_desc: Option<&'a str>,
}

impl TestStep for ReversalTest {
    async fn run(
        &self,
        context: &mut TestContext,
        callback_manager: &mut CallbackManager,
    ) -> anyhow::Result<()> {
        context.log("== Running Reversal Suite ==").await;

        let project: ProjectDetails = context.get("project")?.unwrap();
        let business: BusinessSummary = context.get("business")?.unwrap();
        let operator: BusinessOperator = context.get("operator")?.unwrap();
        let base_url: String = context.get("base_url")?.unwrap();

        let token = get_access_token(context, &base_url, &project)
            .await
            .context("Failed to obtain access token.")?;

        // -- Setup: create customer users for C2B flows --
        let customer = user_profiles::ui::create_user(
            &context.app_context,
            "C2B Customer".to_string(),
            "254700111222".to_string(),
            100_000.00,
            "1111".to_string(),
        )
        .await
        .context("Failed to create customer")?;

        let p2p_sender = user_profiles::ui::create_user(
            &context.app_context,
            "P2P Sender".to_string(),
            "254700333444".to_string(),
            10_000.00,
            "2222".to_string(),
        )
        .await
        .context("Failed to create P2P sender")?;

        let p2p_recipient = user_profiles::ui::create_user(
            &context.app_context,
            "P2P Recipient".to_string(),
            "254700555666".to_string(),
            500.00,
            "3333".to_string(),
        )
        .await
        .context("Failed to create P2P recipient")?;

        // C2B payment #1: customer pays business utility account (Happy Path)
        let c2b_amount = 10_000_i64;
        let (c2b_txn, _) = Ledger::transfer(
            &context.app_context.db,
            Some(customer.account_id),
            business.utility_account.account_id,
            c2b_amount,
            &TransactionType::Paybill,
            None,
        )
        .await
        .context("Failed to execute C2B transaction")?;

        let c2b_receipt = c2b_txn.id.clone();
        context
            .log(&format!("Setup: C2B receipt = {}", c2b_receipt))
            .await;

        // P2P transfer for the invalid transaction type test
        let p2p_amount = 5_000_i64;
        let (p2p_txn, _) = Ledger::transfer(
            &context.app_context.db,
            Some(p2p_sender.account_id),
            p2p_recipient.account_id,
            p2p_amount,
            &TransactionType::SendMoney,
            None,
        )
        .await
        .context("Failed to execute P2P transfer")?;

        let p2p_receipt = p2p_txn.id.clone();
        context
            .log(&format!("Setup: P2P receipt = {}", p2p_receipt))
            .await;

        // Test 1: Happy path – reverse a C2B Paybill transaction
        execute_reversal_test_case(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &operator,
            ReversalTestCase {
                name: "Happy Path",
                initiator_override: None,
                security_credential_override: None,
                transaction_id: c2b_receipt.clone(),
                expected_api_status: 200,
                expected_sync_response_code: Some("0"),
                expected_callback_result_code: Some("0"),
                expected_callback_result_desc: Some(
                    "The service request is processed successfully.",
                ),
            },
        )
        .await?;

        // Test 2: Already reversed – same receipt again
        execute_reversal_test_case(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &operator,
            ReversalTestCase {
                name: "Already Reversed",
                initiator_override: None,
                security_credential_override: None,
                transaction_id: c2b_receipt.clone(),
                expected_api_status: 200,
                expected_sync_response_code: Some("0"),
                expected_callback_result_code: Some("R000001"),
                expected_callback_result_desc: Some("The transaction has already been reversed."),
            },
        )
        .await?;

        // Test 3: Invalid TransactionID – non-existent receipt
        execute_reversal_test_case(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &operator,
            ReversalTestCase {
                name: "Invalid TransactionID",
                initiator_override: None,
                security_credential_override: None,
                transaction_id: "NONEXISTENT123".to_string(),
                expected_api_status: 200,
                expected_sync_response_code: Some("0"),
                expected_callback_result_code: Some("R000002"),
                expected_callback_result_desc: Some("The OriginalTransactionID is invalid."),
            },
        )
        .await?;

        // Test 4: Invalid initiator credentials – wrong username
        execute_reversal_test_case(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &operator,
            ReversalTestCase {
                name: "Invalid Initiator",
                initiator_override: Some("nonexistent_operator"),
                security_credential_override: Some("invalid_base64=="),
                transaction_id: c2b_receipt.clone(),
                expected_api_status: 401,
                expected_sync_response_code: None,
                expected_callback_result_code: None,
                expected_callback_result_desc: None,
            },
        )
        .await?;

        // Test 5: Invalid transaction type – reversing a P2P (SendMoney) is not allowed
        execute_reversal_test_case(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &operator,
            ReversalTestCase {
                name: "Invalid Transaction Type",
                initiator_override: None,
                security_credential_override: None,
                transaction_id: p2p_receipt.clone(),
                expected_api_status: 200,
                expected_sync_response_code: Some("0"),
                expected_callback_result_code: Some("R000002"),
                expected_callback_result_desc: Some("The OriginalTransactionID is invalid."),
            },
        )
        .await?;

        // -- Setup for Insufficient Balance test --
        let payer = user_profiles::ui::create_user(
            &context.app_context,
            "C2B Payer".to_string(),
            "254700777888".to_string(),
            10_000.00,
            "4444".to_string(),
        )
        .await
        .context("Failed to create payer")?;

        let c2b_amount_2 = 10_000_i64;
        let (c2b_txn_2, _) = Ledger::transfer(
            &context.app_context.db,
            Some(payer.account_id),
            business.utility_account.account_id,
            c2b_amount_2,
            &TransactionType::Paybill,
            None,
        )
        .await
        .context("Failed to execute second C2B transaction")?;

        let c2b_receipt_2 = c2b_txn_2.id.clone();
        context
            .log(&format!("Setup: second C2B receipt = {}", c2b_receipt_2))
            .await;

        // Drain the utility account so it cannot cover the reversal.
        // After the C2B, utility balance = 500_000 + 10_000 = 510_000.
        // Draining 500_000 (5,000 KES) leaves 4_300 after the 57 KES fee,
        // which is < 10_000 (the C2B amount), so reversal will fail.
        let util = UtilityAccount::find_by_business_id(
            &context.app_context.db,
            business.id,
        )
        .await
        .context("Failed to find utility account")?
        .context("Utility account not found")?;

        let util_balance = util.balance;
        let drain_amount = (util_balance - c2b_amount_2).max(0);
        if drain_amount > 0 {
            let drain_fee = transaction_costs::get_fee(
                &context.app_context.db,
                &TransactionType::SendMoney,
                drain_amount,
            )
            .await
            .context("Failed to get drain fee")?;

            if drain_amount + drain_fee <= util_balance {
                Ledger::transfer(
                    &context.app_context.db,
                    Some(util.account_id),
                    payer.account_id,
                    drain_amount,
                    &TransactionType::SendMoney,
                    None,
                )
                .await
                .context("Failed to drain utility account")?;

                context
                    .log(&format!(
                        "Drained utility account: sent {} with fee {}, remaining < {}",
                        drain_amount, drain_fee, c2b_amount_2
                    ))
                    .await;
            }
        }

        // Test 6: Insufficient balance – utility account cannot cover the reversal
        execute_reversal_test_case(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &business,
            &operator,
            ReversalTestCase {
                name: "Insufficient Balance",
                initiator_override: None,
                security_credential_override: None,
                transaction_id: c2b_receipt_2,
                expected_api_status: 200,
                expected_sync_response_code: Some("0"),
                expected_callback_result_code: Some("1"),
                expected_callback_result_desc: Some("The balance is insufficient."),
            },
        )
        .await?;

        context
            .log("== Reversal Suite Completed Successfully ==")
            .await;
        Ok(())
    }
}

async fn make_reversal_request(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    business: &BusinessSummary,
    operator: &BusinessOperator,
    case: &ReversalTestCase<'_>,
) -> anyhow::Result<(
    u16,
    Option<ReversalResponse>,
    Option<CallbackEntry<ReversalCallbackResponse>>,
)> {
    let initiator = case
        .initiator_override
        .unwrap_or(&operator.username)
        .to_string();

    let security_credential: String = match &case.security_credential_override {
        Some(cred) => cred.to_string(),
        None => settings::ui::generate_security_credential(
            &context.app_context,
            operator.password.to_string(),
        )
        .await
        .context("Failed to generate security credential")?,
    };

    let callback_entry = if case.expected_callback_result_code.is_some() {
        let entry = callback_manager
            .register_callback::<ReversalCallbackResponse>(&format!(
                "/reversal_callback/{}",
                case.name.replace(' ', "_")
            ))
            .context("Failed to register callback")?;
        Some(entry)
    } else {
        None
    };

    let callback_url = callback_entry
        .as_ref()
        .map(|e| e.url())
        .unwrap_or("http://example.com/no-callback")
        .to_string();

    let request = ReversalRequest {
        initiator,
        security_credential,
        command_id: "TransactionReversal".to_string(),
        transaction_id: case.transaction_id.clone(),
        amount: "10000".to_string(),
        receiver_party: business.short_code.clone(),
        reciever_identifier_type: "11".to_string(),
        result_url: callback_url.clone(),
        queue_time_out_url: format!("{}/timeout", callback_url),
        remarks: format!("Self-test: {}", case.name),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token))
            .context("Failed to create Authorization header")?,
    );

    let url = format!("{}/mpesa/reversal/v1/request", base_url);
    let response = context
        .api_client
        .post_json_raw(&url, &request, Some(headers))
        .await
        .context("Failed to send reversal request")?;

    let status = response.status().as_u16();
    let sync_res: Option<ReversalResponse> = if status == 200 {
        Some(
            response
                .json()
                .await
                .context("Failed to parse reversal sync response")?,
        )
    } else {
        None
    };

    Ok((status, sync_res, callback_entry))
}

async fn execute_reversal_test_case(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    business: &BusinessSummary,
    operator: &BusinessOperator,
    case: ReversalTestCase<'_>,
) -> anyhow::Result<()> {
    context.log(&format!("-- Running: {} --", case.name)).await;

    let (status, sync_res, callback_entry) = make_reversal_request(
        context,
        callback_manager,
        token,
        base_url,
        business,
        operator,
        &case,
    )
    .await?;

    anyhow::ensure!(
        status == case.expected_api_status,
        "[{}] Expected HTTP {}, got {}. {:?}",
        case.name,
        case.expected_api_status,
        status,
        sync_res.as_ref().map(|r| &r.response_description)
    );

    // Check sync response fields
    if let Some(expected_code) = case.expected_sync_response_code {
        let sync = sync_res.context("Expected sync response but got none")?;
        anyhow::ensure!(
            sync.response_code == expected_code,
            "[{}] Expected ResponseCode '{}', got '{}'",
            case.name,
            expected_code,
            sync.response_code
        );
        anyhow::ensure!(
            !sync.conversation_id.is_empty(),
            "[{}] ConversationID should not be empty",
            case.name
        );
    }

    // Wait for callback (if expected)
    if let Some(callback_entry) = callback_entry {
        let callback = tokio::time::timeout(Duration::from_secs(10), callback_entry)
            .await
            .context(format!("[{}] Timed out waiting for callback", case.name))?
            .context(format!("[{}] Callback failed", case.name))?;

        if let Some(expected_code) = case.expected_callback_result_code {
            anyhow::ensure!(
                callback.body.result.result_code == expected_code,
                "[{}] Expected callback ResultCode '{}', got '{}': {}",
                case.name,
                expected_code,
                callback.body.result.result_code,
                callback.body.result.result_desc
            );
        }

        if let Some(expected_desc) = case.expected_callback_result_desc {
            anyhow::ensure!(
                callback.body.result.result_desc == expected_desc,
                "[{}] Expected callback ResultDesc '{}', got '{}'",
                case.name,
                expected_desc,
                callback.body.result.result_desc
            );
        }

        // For successful reversals, verify the OriginalTransactionID field
        if case.expected_callback_result_code == Some("0") {
            let has_original = callback
                .body
                .result
                .result_parameters
                .as_ref()
                .map(|params| {
                    params.result_parameter.iter().any(|p| {
                        p.key == "OriginalTransactionID"
                            && p.value == serde_json::Value::String(case.transaction_id.clone())
                    })
                })
                .unwrap_or(false);
            anyhow::ensure!(
                has_original,
                "[{}] ResultParameters should contain OriginalTransactionID matching the receipt",
                case.name
            );

            // Verify original transaction is marked Reversed in DB
            use sea_orm::EntityTrait;
            let original_txn = txn_db::Entity::find_by_id(&case.transaction_id)
                .one(&context.app_context.db)
                .await
                .context("Failed to look up original transaction")?
                .ok_or(anyhow!("Original transaction not found"))?;
            anyhow::ensure!(
                original_txn.status == TransactionStatus::Reversed,
                "[{}] Expected original txn status 'Reversed', got '{:?}'",
                case.name,
                original_txn.status
            );
        }
    }

    context.log(&format!(">> [{}] Passed", case.name)).await;
    Ok(())
}
