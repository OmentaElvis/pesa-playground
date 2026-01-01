use anyhow::Context;
use axum::http::{HeaderMap, HeaderValue};

use crate::{
    business::BusinessSummary,
    business_operators::BusinessOperator,
    projects::ProjectDetails,
    self_test::{
        callback::{CallbackCall, CallbackManager},
        context::TestContext,
        runner::TestStep,
        tests::get_access_token,
    },
    server::api::balance_query::{
        BalanceQueryCallbackResponse, BalanceQueryRequest, BalanceQueryRequestResponse, CommandID,
        IdentifierType,
    },
    settings,
};

pub struct BalanceQueryTest;

struct BalanceQueryTestCase<'a> {
    name: &'a str,
    command_id: CommandID,
    party_a: String,
    identifier_type: IdentifierType,
    should_succeed: bool,
    expected_api_status: u16,
    expected_callback_result_code: i32,
}

impl TestStep for BalanceQueryTest {
    async fn run(
        &self,
        context: &mut TestContext,
        callback_manager: &mut CallbackManager,
    ) -> anyhow::Result<()> {
        context.log("== Running Balance Query Suite ==").await;

        let project: ProjectDetails = context
            .get("project")
            .context("Failed to get project from TestContext")?
            .unwrap();
        let business: BusinessSummary = context
            .get("business")
            .context("Failed to get business from TestContext")?
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

        happy_path_balance_query(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &operator,
            &business,
        )
        .await
        .context("Balance Query happy path test failed")?;

        invalid_shortcode_test(
            context,
            callback_manager,
            &token.access_token,
            &base_url,
            &operator,
            &business,
        )
        .await
        .context("Balance Query invalid shortcode test failed")?;

        context
            .log("== Balance Query Suite Completed Successfully ==")
            .await;
        Ok(())
    }
}

async fn execute_balance_query_test_case(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    operator: &BusinessOperator,
    business: &BusinessSummary,
    case: BalanceQueryTestCase<'_>,
) -> anyhow::Result<()> {
    context
        .log(&format!("-- Running Test Case: {} --", case.name))
        .await;

    let callback = callback_manager
        .register_callback::<BalanceQueryCallbackResponse>("/balance_query_callback")
        .context("Failed to register Balance Query callback")?;

    let security_credential =
        settings::ui::generate_security_credential(&context.app_context, operator.password.clone())
            .await
            .context("Failed to generate security credential")?;

    let request = BalanceQueryRequest {
        initiator: operator.username.clone(),
        security_credential,
        command_id: case.command_id.clone(),
        party_a: case.party_a.clone(),
        identifier_type: case.identifier_type.clone(),
        remarks: "Balance Query Test".to_string(),
        queue_time_out_url: callback_manager.get_callback_url("/balance_query_timeout"),
        result_url: callback.url().to_string(),
    };

    context.log(">> Balance Query Request Details:").await;
    context
        .log(&format!("  Initiator: {}", request.initiator))
        .await;
    context
        .log(&format!("  CommandID: {:?}", request.command_id))
        .await;
    context
        .log(&format!("  PartyA (Shortcode): {}", request.party_a))
        .await;
    context
        .log(&format!("  IdentifierType: {:?}", request.identifier_type))
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
    let url = format!("{}/mpesa/accountbalance/v1/query", base_url);

    let response = context
        .api_client
        .post_json_raw(&url, &request, Some(headers))
        .await
        .context("Failed to send Balance Query HTTP request")?;

    let status = response.status();
    if status.as_u16() != case.expected_api_status {
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "[{}] Expected API status {} but got {}. Body: {}",
            case.name,
            case.expected_api_status,
            status,
            error_body
        );
    }

    if !case.should_succeed {
        context
            .log(">> Test case expected to fail at API level, and it did. Test passed.")
            .await;
        return Ok(());
    }

    // --- Get initial state for successful cases ---
    let initial_business_state =
        crate::business::ui::get_business(&context.app_context, business.id)
            .await
            .context("Failed to get initial business state")?;

    context.log(">> Initial Business State:").await;
    context
        .log(&format!(
            "  MMF Account Balance: {}",
            initial_business_state.mmf_account.balance
        ))
        .await;
    context
        .log(&format!(
            "  Utility Account Balance: {}",
            initial_business_state.utility_account.balance
        ))
        .await;
    context
        .log(&format!(
            "  Charges Amount: {}",
            initial_business_state.charges_amount
        ))
        .await;

    let res: BalanceQueryRequestResponse = response
        .json()
        .await
        .context("Failed to parse success response as JSON")?;
    context
        .log(&format!(">> Received API response body: {:#?}", res))
        .await;

    let callback_req: CallbackCall<BalanceQueryCallbackResponse> = callback
        .await
        .context(format!("[{}] Did not receive callback", case.name))?;

    let b2c_callback = callback_req.body;
    assert_eq!(
        b2c_callback.result.result_code,
        case.expected_callback_result_code.to_string(),
        "Callback ResultCode did not match"
    );

    // --- Assertions for successful cases ---
    if case.should_succeed {
        let result_params = b2c_callback
            .result
            .result_parameters
            .as_ref()
            .context("ResultParameters not found in callback")?;
        let account_balance_param = result_params
            .result_parameter
            .iter()
            .find(|p| p.key == "AccountBalance")
            .context("AccountBalance key not found in ResultParameters")?;
        let balance_string = account_balance_param
            .value
            .as_str()
            .context("AccountBalance value is not a string")?;

        let mut utility_balance_found = false;
        let mut working_balance_found = false;

        // Parse the pipe-separated string
        for account_details in balance_string.split('&') {
            let parts: Vec<&str> = account_details.split('|').collect();
            if parts.len() < 3 {
                continue;
            }
            let account_name = parts[0];
            let balance_value_str = parts[2];
            let balance_value = (balance_value_str.parse::<f64>().unwrap_or(0.0) * 100.0) as i64;

            if account_name == "Utility Account" {
                assert_eq!(
                    balance_value, initial_business_state.utility_account.balance,
                    "Utility Account balance mismatch in callback"
                );
                utility_balance_found = true;
            } else if account_name == "Working Account" {
                assert_eq!(
                    balance_value, initial_business_state.mmf_account.balance,
                    "Working Account (MMF) balance mismatch in callback"
                );
                working_balance_found = true;
            }
        }

        assert!(
            utility_balance_found,
            "Utility Account balance was not found in the callback string"
        );
        assert!(
            working_balance_found,
            "Working Account balance was not found in the callback string"
        );

        context
            .log(&format!(
                ">> Verified balances from callback: Utility={}, Working={}",
                initial_business_state.utility_account.balance,
                initial_business_state.mmf_account.balance
            ))
            .await;
    }

    context
        .log(&format!("-- Test Case {} Passed --", case.name))
        .await;
    Ok(())
}

async fn happy_path_balance_query(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    operator: &BusinessOperator,
    business: &BusinessSummary,
) -> anyhow::Result<()> {
    execute_balance_query_test_case(
        context,
        callback_manager,
        token,
        base_url,
        operator,
        business,
        BalanceQueryTestCase {
            name: "Happy Path (Query Own Shortcode)",
            command_id: CommandID::AccountBalance,
            party_a: business.short_code.clone(),
            identifier_type: IdentifierType::OrganisationShortCode,
            should_succeed: true,
            expected_api_status: 200,
            expected_callback_result_code: 0,
        },
    )
    .await
}

async fn invalid_shortcode_test(
    context: &mut TestContext,
    callback_manager: &mut CallbackManager,
    token: &str,
    base_url: &str,
    operator: &BusinessOperator,
    business: &BusinessSummary,
) -> anyhow::Result<()> {
    execute_balance_query_test_case(
        context,
        callback_manager,
        token,
        base_url,
        operator,
        business,
        BalanceQueryTestCase {
            name: "Error: Invalid Shortcode",
            command_id: CommandID::AccountBalance,
            party_a: "000000".to_string(), // Non-existent shortcode
            identifier_type: IdentifierType::OrganisationShortCode,
            should_succeed: false,
            expected_api_status: 400,
            expected_callback_result_code: 0, // Not applicable
        },
    )
    .await
}
