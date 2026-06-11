use anyhow::Context;
use axum::http::{HeaderMap, HeaderValue};
use base64::Engine;

use crate::{
    self_test::{
        callback::CallbackManager,
        context::TestContext,
        runner::TestStep,
        tests::get_access_token,
    },
    server::api::dynamic_qr::{DynamicQrRequest, DynamicQrResponse, TrxCode},
};

pub struct DynamicQrTest;

impl TestStep for DynamicQrTest {
    async fn run(
        &self,
        context: &mut TestContext,
        _callback_manager: &mut CallbackManager,
    ) -> anyhow::Result<()> {
        context.log("== Running Dynamic QR Suite ==").await;

        let project = context
            .get("project")
            .context("Failed to get project from TestContext")?
            .unwrap();
        let base_url: String = context
            .get("base_url")
            .context("Failed to get base_url from TestContext")?
            .unwrap();

        let token = get_access_token(context, &base_url, &project)
            .await
            .context("Failed to obtain access token.")?;

        happy_path(context, &token.access_token, &base_url)
            .await
            .context("Happy path test failed")?;

        zero_amount_test(context, &token.access_token, &base_url)
            .await
            .context("Zero amount test failed")?;

        empty_cpi_test(context, &token.access_token, &base_url)
            .await
            .context("Empty CPI test failed")?;

        context
            .log("== Dynamic QR Suite Completed Successfully ==")
            .await;
        Ok(())
    }
}

async fn happy_path(
    context: &mut TestContext,
    token: &str,
    base_url: &str,
) -> anyhow::Result<()> {
    context.log("-- Running: Happy Path --").await;

    let request = DynamicQrRequest {
        merchant_name: "Test Merchant".to_string(),
        ref_no: "INV-001".to_string(),
        amount: 100,
        trx_code: TrxCode::BuyGoods,
        cpi: "174379".to_string(),
        size: "300".to_string(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token))
            .context("Failed to create Authorization header")?,
    );

    let url = format!("{}/mpesa/qrcode/v1/generate", base_url);
    let response = context
        .api_client
        .post_json_raw(&url, &request, Some(headers))
        .await
        .context("Failed to send Dynamic QR request")?;

    let status = response.status();
    anyhow::ensure!(
        status == 200,
        "Expected 200 OK, got {}. Body: {}",
        status,
        response.text().await.unwrap_or_default()
    );

    let res: DynamicQrResponse = response
        .json()
        .await
        .context("Failed to parse Dynamic QR response")?;

    anyhow::ensure!(
        res.response_code.starts_with("AG_"),
        "Expected ResponseCode to start with 'AG_', got '{}'",
        res.response_code
    );
    anyhow::ensure!(
        !res.request_id.is_empty(),
        "RequestID should not be empty"
    );
    anyhow::ensure!(
        res.response_description == "QR Code Successfully Generated.",
        "Expected 'QR Code Successfully Generated.', got '{}'",
        res.response_description
    );

    // Verify QRCode is valid base64 with PNG magic bytes
    let qr_bytes = base64::engine::general_purpose::STANDARD
        .decode(&res.qr_code)
        .context("QRCode is not valid base64")?;
    anyhow::ensure!(
        qr_bytes.starts_with(&[0x89, b'P', b'N', b'G']),
        "QRCode does not start with PNG magic bytes"
    );

    context.log(">> QR Code generated successfully (valid PNG base64)").await;
    Ok(())
}

async fn zero_amount_test(
    context: &mut TestContext,
    token: &str,
    base_url: &str,
) -> anyhow::Result<()> {
    context.log("-- Running: Zero Amount Error --").await;

    let request = DynamicQrRequest {
        merchant_name: "Test".to_string(),
        ref_no: "REF-ERR".to_string(),
        amount: 0,
        trx_code: TrxCode::Paybill,
        cpi: "174379".to_string(),
        size: "200".to_string(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token))
            .context("Failed to create Authorization header")?,
    );

    let url = format!("{}/mpesa/qrcode/v1/generate", base_url);
    let response = context
        .api_client
        .post_json_raw(&url, &request, Some(headers))
        .await
        .context("Failed to send zero amount request")?;

    let status = response.status();
    anyhow::ensure!(
        status == 400,
        "Expected 400 for zero amount, got {}",
        status
    );

    context.log(">> Correctly rejected zero amount").await;
    Ok(())
}

async fn empty_cpi_test(
    context: &mut TestContext,
    token: &str,
    base_url: &str,
) -> anyhow::Result<()> {
    context.log("-- Running: Empty CPI Error --").await;

    let request = DynamicQrRequest {
        merchant_name: "Test".to_string(),
        ref_no: "REF-ERR2".to_string(),
        amount: 50,
        trx_code: TrxCode::SendMoney,
        cpi: "".to_string(),
        size: "200".to_string(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token))
            .context("Failed to create Authorization header")?,
    );

    let url = format!("{}/mpesa/qrcode/v1/generate", base_url);
    let response = context
        .api_client
        .post_json_raw(&url, &request, Some(headers))
        .await
        .context("Failed to send empty CPI request")?;

    let status = response.status();
    anyhow::ensure!(
        status == 400,
        "Expected 400 for empty CPI, got {}",
        status
    );

    context.log(">> Correctly rejected empty CPI").await;
    Ok(())
}
