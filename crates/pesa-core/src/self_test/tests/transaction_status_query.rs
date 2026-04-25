use std::{collections::HashMap, time::Duration};

use anyhow::Context;
use axum::http::{HeaderMap, HeaderValue};
use serde_json::{Number, Value};

use crate::{
    accounts::user_profiles::User,
    business::BusinessSummary,
    business_operators::BusinessOperator,
    projects::{Project, ProjectDetails, UpdateProject},
    self_test::{
        callback::{CallbackCall, CallbackManager},
        context::TestContext,
        runner::TestStep,
        tests::get_access_token,
    },
    server::api::{
        b2c::{B2CCallbackResponse, B2CRequest, B2CRequestResponse, CommandID},
        transaction_status::{
            CommandID as TxnStatusCmd, IdentifierType, TxnStatusCallbackPayload, TxnStatusRequest,
            TxnStatusRequestResponse,
        },
    },
    settings,
    transactions_log::FullTransactionLog,
};

pub struct TransactionStatusQueryTest;

const FAST_ORIGINATOR_ID: &str = "test-txn-status-originator";
const SLOW_ORIGINATOR_ID: &str = "slow-txn-status-originator";

impl TestStep for TransactionStatusQueryTest {
    async fn run(
        &self,
        context: &mut TestContext,
        callback_manager: &mut CallbackManager,
    ) -> anyhow::Result<()> {
        context
            .log("== Running Transaction Status Query Suite ==")
            .await;

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

        let token = get_access_token(context, &base_url, &project)
            .await
            .context("Failed to obtain access token.")?;

        // Test 1: Happy path - completed transaction
        let transaction_id = create_b2c_and_get_transaction_id(
            context,
            callback_manager,
            CreateB2cTransaction {
                token: &token.access_token,
                base_url: &base_url,
                business: &business,
                user: &rich_user,
                operator: &operator,
                originator_id: FAST_ORIGINATOR_ID,
                wait: true,
            },
        )
        .await
        .context("Failed to create B2C transaction for testing")?;

        happy_path_transaction_status_query(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &operator,
            &business,
            &transaction_id.unwrap(),
        )
        .await
        .context("Transaction Status Query happy path test failed")?;

        // Test 2: OriginatorConversationID fallback
        originator_id_test(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &operator,
            &business,
        )
        .await
        .context("Transaction Status Query originator ID test failed")?;

        // Test 3: Not found test
        transaction_not_found_test(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &operator,
            &business,
        )
        .await
        .context("Transaction Status Query not found test failed")?;

        // Test 4: Pending transaction
        Project::update(
            &context.app_context.db,
            project.id,
            UpdateProject {
                txn_delay: Some(5 * 60),
                ..Default::default()
            },
        )
        .await
        .context("Failed to update project delay")?;

        let _ = create_b2c_and_get_transaction_id(
            context,
            callback_manager,
            CreateB2cTransaction {
                token: &token.access_token,
                base_url: &base_url,
                business: &business,
                user: &rich_user,
                operator: &operator,
                originator_id: SLOW_ORIGINATOR_ID,
                wait: false,
            },
        )
        .await
        .context("Failed to create B2C transaction for testing")?;

        pending_transaction_test(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &operator,
            &business,
        )
        .await
        .context("Pending transaction status failed")?;

        Project::update(
            &context.app_context.db,
            project.id,
            UpdateProject {
                txn_delay: Some(0),
                ..Default::default()
            },
        )
        .await
        .context("Failed to reset project txn delay")?;

        context
            .log("== Transaction Status Query Suite Completed Successfully ==")
            .await;
        Ok(())
    }
}

struct CreateB2cTransaction<'a> {
    token: &'a str,
    base_url: &'a str,
    business: &'a BusinessSummary,
    user: &'a User,
    operator: &'a BusinessOperator,
    originator_id: &'a str,
    wait: bool,
}

async fn create_b2c_and_get_transaction_id<'a>(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    args: CreateB2cTransaction<'a>,
) -> anyhow::Result<Option<String>> {
    context
        .log("-- Creating B2C transaction for testing --")
        .await;

    let callback = callback_manager
        .register_callback::<B2CCallbackResponse>("/b2c_callback")
        .context("Failed to register B2C callback")?;

    let new_transaction_event = context
        .event_manager
        .listen_for::<FullTransactionLog>("new_transaction", Duration::from_secs(5));

    let security_credential = settings::ui::generate_security_credential(
        &context.app_context,
        args.operator.password.to_string(),
    )
    .await
    .context("Failed to generate security credential")?;

    let request = B2CRequest {
        originator_conversation_id: args.originator_id.to_string(),
        initiator_name: args.operator.username.clone(),
        security_credential,
        command_id: CommandID::BusinessPayment,
        amount: "50.00".to_string(),
        party_a: args.business.short_code.clone(),
        party_b: args.user.phone.clone(),
        remarks: "Test for Transaction Status Query".to_string(),
        queue_time_out_url: callback_manager.get_callback_url("/b2c_timeout"),
        result_url: callback.url().to_string(),
        occassion: "Test".to_string(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", args.token))
            .context("Failed to create Authorization header")?,
    );
    let url = format!("{}/mpesa/b2c/v3/paymentrequest", args.base_url);

    let response = context
        .api_client
        .post_json_raw(&url, &request, Some(headers))
        .await
        .context("Failed to send B2C HTTP request")?;

    let status = response.status();
    if status != 200 {
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!("B2C request failed with status {}: {}", status, error_body);
    }

    let _res: B2CRequestResponse = response
        .json()
        .await
        .context("Failed to parse B2C response")?;

    if !args.wait {
        // dont wait for callback as it might timeout
        return Ok(None);
    }

    let callback_req: CallbackCall<B2CCallbackResponse> =
        callback.await.context("Did not receive B2C callback")?;

    let b2c_callback = callback_req.body;
    assert_eq!(b2c_callback.result.result_code, "0", "B2C should succeed");

    let transaction_event = new_transaction_event
        .await
        .context("Did not receive new transaction event")?;

    context
        .log(&format!(
            ">> Created transaction for testing: ID={}",
            transaction_event.transaction_id
        ))
        .await;

    Ok(Some(transaction_event.transaction_id))
}

async fn happy_path_transaction_status_query(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    operator: &BusinessOperator,
    business: &BusinessSummary,
    transaction_id: &str,
) -> anyhow::Result<()> {
    context.log("-- Running Test Case: Happy Path --").await;

    let callback = callback_manager
        .register_callback::<TxnStatusCallbackPayload>("/transaction_status_callback")
        .context("Failed to register Transaction Status callback")?;

    let security_credential =
        settings::ui::generate_security_credential(&context.app_context, operator.password.clone())
            .await
            .context("Failed to generate security credential")?;

    let request = TxnStatusRequest {
        initiator: operator.username.clone(),
        security_credential,
        command_id: TxnStatusCmd::TransactionStatusQuery,
        transaction_id: transaction_id.to_string(),
        original_conversation_id: None,
        party_a: business.short_code.clone(),
        identifier_type: IdentifierType::OrganisationShortCode,
        remarks: "Transaction Status Query Test".to_string(),
        queue_time_out_url: callback_manager.get_callback_url("/transaction_status_timeout"),
        result_url: callback.url().to_string(),
        occassion: "Test occasion".to_string(),
    };

    context
        .log(">> Transaction Status Query Request Details:")
        .await;
    context
        .log(&format!("  Initiator: {}", request.initiator))
        .await;
    context
        .log(&format!("  CommandID: {:?}", request.command_id))
        .await;
    context
        .log(&format!("  TransactionID: {}", request.transaction_id))
        .await;
    context
        .log(&format!("  PartyA (Shortcode): {}", request.party_a))
        .await;
    context
        .log(&format!("  ResultURL: {}", request.result_url))
        .await;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token))
            .context("Failed to create Authorization header")?,
    );
    let url = format!("{}/mpesa/transactionstatus/v1/query", base_url);

    let response = context
        .api_client
        .post_json_raw(&url, &request, Some(headers))
        .await
        .context("Failed to send Transaction Status Query HTTP request")?;

    let status = response.status();
    if status != 200 {
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "Expected API status 200 but got {}. Body: {}",
            status,
            error_body
        );
    }

    let res: TxnStatusRequestResponse = response
        .json()
        .await
        .context("Failed to parse success response as JSON")?;
    context
        .log(&format!(">> Received API response body: {:#?}", res))
        .await;

    let callback_req: CallbackCall<TxnStatusCallbackPayload> =
        callback.await.context("Did not receive callback")?;

    let txn_status_callback = callback_req.body;
    assert_eq!(
        txn_status_callback.result.result_code, "0",
        "Callback ResultCode did not match"
    );

    let result_params = txn_status_callback
        .result
        .result_parameters
        .as_ref()
        .context("ResultParameters not found in callback")?;

    let receipt_no = result_params
        .result_parameter
        .iter()
        .find(|p| p.key == "ReceiptNo")
        .context("ReceiptNo key not found in ResultParameters")?;
    let receipt_value = receipt_no
        .value
        .as_str()
        .context("ReceiptNo value is not a string")?;
    assert_eq!(
        receipt_value, transaction_id,
        "ReceiptNo in callback does not match queried transaction ID"
    );

    let transaction_status = result_params
        .result_parameter
        .iter()
        .find(|p| p.key == "TransactionStatus")
        .context("TransactionStatus key not found in ResultParameters")?;
    let status_value = transaction_status
        .value
        .as_str()
        .context("TransactionStatus value is not a string")?;
    assert_eq!(
        status_value, "Completed",
        "TransactionStatus should be Completed"
    );

    context
        .log(&format!(
            ">> Verified transaction status: ID={}, Status={}",
            receipt_value, status_value
        ))
        .await;

    context.log("-- Test Case Happy Path Passed --").await;
    Ok(())
}

async fn transaction_not_found_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    operator: &BusinessOperator,
    business: &BusinessSummary,
) -> anyhow::Result<()> {
    context
        .log("-- Running Test Case: Transaction Not Found --")
        .await;

    let callback = callback_manager
        .register_callback::<TxnStatusCallbackPayload>("/transaction_status_callback")
        .context("Failed to register Transaction Status callback")?;

    let security_credential =
        settings::ui::generate_security_credential(&context.app_context, operator.password.clone())
            .await
            .context("Failed to generate security credential")?;

    let request = TxnStatusRequest {
        initiator: operator.username.clone(),
        security_credential,
        command_id: TxnStatusCmd::TransactionStatusQuery,
        transaction_id: "NONEXISTENT123".to_string(),
        original_conversation_id: None,
        party_a: business.short_code.clone(),
        identifier_type: IdentifierType::OrganisationShortCode,
        remarks: "Transaction Status Query Test".to_string(),
        queue_time_out_url: callback_manager.get_callback_url("/transaction_status_timeout"),
        result_url: callback.url().to_string(),
        occassion: "Test occasion".to_string(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token))
            .context("Failed to create Authorization header")?,
    );
    let url = format!("{}/mpesa/transactionstatus/v1/query", base_url);

    let response = context
        .api_client
        .post_json_raw(&url, &request, Some(headers))
        .await
        .context("Failed to send Transaction Status Query HTTP request")?;

    let status = response.status();
    if status != 200 {
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "Expected API status 200 but got {}. Body: {}",
            status,
            error_body
        );
    }

    let _res: TxnStatusRequestResponse = response
        .json()
        .await
        .context("Failed to parse response as JSON")?;

    let callback_req: CallbackCall<TxnStatusCallbackPayload> =
        callback.await.context("Did not receive callback")?;

    let txn_status_callback = callback_req.body;
    assert_eq!(
        txn_status_callback.result.result_code, "404.004.01",
        "Callback ResultCode should be TransactionNotFound"
    );

    context
        .log("-- Test Case Transaction Not Found Passed --")
        .await;
    Ok(())
}

async fn pending_transaction_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    operator: &BusinessOperator,
    business: &BusinessSummary,
) -> anyhow::Result<()> {
    context
        .log("-- Running Test Case: Pending Transaction (Delayed) --")
        .await;

    let callback = callback_manager
        .register_callback::<TxnStatusCallbackPayload>("/transaction_status_callback")
        .context("Failed to register Transaction Status callback")?;

    let security_credential = settings::ui::generate_security_credential(
        &context.app_context,
        operator.password.to_string(),
    )
    .await
    .context("Failed to generate security credential")?;

    let request = TxnStatusRequest {
        initiator: operator.username.clone(),
        security_credential,
        command_id: TxnStatusCmd::TransactionStatusQuery,
        transaction_id: "".to_string(),
        original_conversation_id: Some(SLOW_ORIGINATOR_ID.to_string()),
        party_a: business.short_code.clone(),
        identifier_type: IdentifierType::OrganisationShortCode,
        remarks: "Transaction Status Query Test".to_string(),
        queue_time_out_url: callback_manager.get_callback_url("/transaction_status_timeout"),
        result_url: callback.url().to_string(),
        occassion: "Test occasion".to_string(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token))
            .context("Failed to create Authorization header")?,
    );
    let url = format!("{}/mpesa/transactionstatus/v1/query", base_url);

    let response = context
        .api_client
        .post_json_raw(&url, &request, Some(headers))
        .await
        .context("Failed to send Transaction Status Query HTTP request")?;

    let status = response.status();
    if status != 200 {
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "Expected API status 200 but got {}. Body: {}",
            status,
            error_body
        );
    }

    let _res: TxnStatusRequestResponse = response
        .json()
        .await
        .context("Failed to parse response as JSON")?;

    let callback_req: CallbackCall<TxnStatusCallbackPayload> =
        callback.await.context("Did not receive callback")?;

    let result_parameters: HashMap<_, _> = callback_req
        .body
        .result
        .result_parameters
        .as_ref()
        .expect("Expected result parameters to have value")
        .result_parameter
        .iter()
        .map(|e| (e.key.as_str(), &e.value))
        .collect();

    assert_eq!(
        result_parameters.get("TransactionStatus"),
        Some(&&Value::String("Pending".to_string()))
    );

    assert_eq!(
        result_parameters.get("Amount"),
        Some(&&Value::Number(Number::from_f64(50.0).unwrap()))
    );

    assert_eq!(
        result_parameters.get("OriginatorConversationID"),
        Some(&&Value::String(SLOW_ORIGINATOR_ID.to_string()))
    );

    let txn_status_callback = callback_req.body;
    assert_eq!(
        txn_status_callback.result.result_code, "0",
        "Callback ResultCode should be Success when querying by OriginatorConversationID"
    );

    context
        .log("-- Test Case Pending Transaction Passed --")
        .await;
    Ok(())
}

async fn originator_id_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    operator: &BusinessOperator,
    business: &BusinessSummary,
) -> anyhow::Result<()> {
    context
        .log("-- Running Test Case: Query by OriginatorConversationID --")
        .await;

    let callback = callback_manager
        .register_callback::<TxnStatusCallbackPayload>("/transaction_status_callback")
        .context("Failed to register Transaction Status callback")?;

    let security_credential = settings::ui::generate_security_credential(
        &context.app_context,
        operator.password.to_string(),
    )
    .await
    .context("Failed to generate security credential")?;

    let request = TxnStatusRequest {
        initiator: operator.username.clone(),
        security_credential,
        command_id: TxnStatusCmd::TransactionStatusQuery,
        transaction_id: "NONEXISTENT123".to_string(),
        original_conversation_id: Some(FAST_ORIGINATOR_ID.to_string()),
        party_a: business.short_code.clone(),
        identifier_type: IdentifierType::OrganisationShortCode,
        remarks: "Transaction Status Query Test".to_string(),
        queue_time_out_url: callback_manager.get_callback_url("/transaction_status_timeout"),
        result_url: callback.url().to_string(),
        occassion: "Test occasion".to_string(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token))
            .context("Failed to create Authorization header")?,
    );
    let url = format!("{}/mpesa/transactionstatus/v1/query", base_url);

    let response = context
        .api_client
        .post_json_raw(&url, &request, Some(headers))
        .await
        .context("Failed to send Transaction Status Query HTTP request")?;

    let status = response.status();
    if status != 200 {
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "Expected API status 200 but got {}. Body: {}",
            status,
            error_body
        );
    }

    let _res: TxnStatusRequestResponse = response
        .json()
        .await
        .context("Failed to parse response as JSON")?;

    let callback_req: CallbackCall<TxnStatusCallbackPayload> =
        callback.await.context("Did not receive callback")?;

    let txn_status_callback = callback_req.body;
    assert_eq!(
        txn_status_callback.result.result_code, "0",
        "Callback ResultCode should be Success when querying by OriginatorConversationID"
    );

    let result_params = txn_status_callback
        .result
        .result_parameters
        .as_ref()
        .context("ResultParameters not found in callback")?;

    let receipt_no = result_params
        .result_parameter
        .iter()
        .find(|p| p.key == "ReceiptNo")
        .context("ReceiptNo key not found in ResultParameters")?;
    let receipt_value = receipt_no
        .value
        .as_str()
        .context("ReceiptNo value is not a string")?;

    let transaction_status = result_params
        .result_parameter
        .iter()
        .find(|p| p.key == "TransactionStatus")
        .context("TransactionStatus key not found in ResultParameters")?;
    let status_value = transaction_status
        .value
        .as_str()
        .context("TransactionStatus value is not a string")?;

    context
        .log(&format!(
            ">> Verified transaction status via originator_id: ID={}, Status={}",
            receipt_value, status_value
        ))
        .await;

    context.log("-- Test Case Originator ID Passed --").await;
    Ok(())
}
