use axum::{Extension, Json, extract::State, http::HeaderMap};
use image::Luma;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::{
    server::{ApiError, ApiState, MpesaError, api::auth},
    utils::identifiers::Identifiers,
};

// ---- Transaction Codes ----

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum TrxCode {
    #[serde(rename = "BG")]
    BuyGoods,
    #[serde(rename = "WA")]
    WithdrawCash,
    #[serde(rename = "PB")]
    Paybill,
    #[serde(rename = "SM")]
    SendMoney,
    #[serde(rename = "SB")]
    SendToBusiness,
}

impl TrxCode {
    fn description(&self) -> &'static str {
        match self {
            TrxCode::BuyGoods => "Buy Goods",
            TrxCode::WithdrawCash => "Withdraw Cash",
            TrxCode::Paybill => "Paybill",
            TrxCode::SendMoney => "Send Money",
            TrxCode::SendToBusiness => "Send to Business",
        }
    }
}

// ---- Request / Response ----

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicQrRequest {
    #[serde(rename = "MerchantName")]
    pub merchant_name: String,
    #[serde(rename = "RefNo")]
    pub ref_no: String,
    #[serde(rename = "Amount")]
    pub amount: u32,
    #[serde(rename = "TrxCode")]
    pub trx_code: TrxCode,
    #[serde(rename = "CPI")]
    pub cpi: String,
    #[serde(rename = "Size")]
    pub size: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicQrResponse {
    #[serde(rename = "ResponseCode")]
    pub response_code: String,
    #[serde(rename = "RequestID")]
    pub request_id: String,
    #[serde(rename = "ResponseDescription")]
    pub response_description: String,
    #[serde(rename = "QRCode")]
    pub qr_code: String,
}

// ---- Error messages ----

#[derive(Debug, thiserror::Error)]
enum DynamicQrError {
    #[error("Invalid Credit Party Identifier.")]
    InvalidCpi,
    #[error("The Amount field is required and must be a positive number.")]
    InvalidAmount,
    #[error("QR code generation failed.")]
    QrGenerationFailed,
    #[error("Invalid Size.")]
    InvalidSize,
}

// ---- Public handler ----

fn build_payment_uri(req: &DynamicQrRequest) -> String {
    format!(
        "mpesa://pay?cpi={}&amount={}&ref={}&merchant={}&trx={}",
        req.cpi, req.amount, req.ref_no, req.merchant_name, req.trx_code.description()
    )
}

fn generate_qr_code_base64(data: &str, size: u32) -> Result<String, DynamicQrError> {
    let code = QrCode::new(data.as_bytes()).map_err(|_| DynamicQrError::QrGenerationFailed)?;

    let img = code
        .render::<Luma<u8>>()
        .dark_color(Luma([0u8]))
        .light_color(Luma([255u8]))
        .quiet_zone(true)
        .min_dimensions(size, size)
        .build();

    let mut png_bytes: Vec<u8> = Vec::new();
    let dyn_img = image::DynamicImage::from(img);
    dyn_img
        .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|_| DynamicQrError::QrGenerationFailed)?;

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &png_bytes,
    ))
}

pub async fn generate_qr_code(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Extension(ids): Extension<Identifiers>,
    Json(req): Json<DynamicQrRequest>,
) -> Result<Json<DynamicQrResponse>, ApiError> {
    let _api_key = auth::validate_bearer_token(&headers, &state).await?;

    if req.amount == 0 {
        return Err(ApiError::new(
            MpesaError::InvalidAmount,
            DynamicQrError::InvalidAmount.to_string(),
        ));
    }

    if req.cpi.trim().is_empty() {
        return Err(ApiError::new(
            MpesaError::InvalidPhoneNumber,
            DynamicQrError::InvalidCpi.to_string(),
        ));
    }

    let qr_size: u32 = req.size.parse().map_err(|_| {
        ApiError::new(MpesaError::InvalidAmount, DynamicQrError::InvalidSize.to_string())
    })?;

    let payment_uri = build_payment_uri(&req);

    let qr_base64 = generate_qr_code_base64(&payment_uri, qr_size)
        .map_err(|e| ApiError::new(MpesaError::InternalError, e.to_string()))?;

    Ok(Json(DynamicQrResponse {
        response_code: ids.conversation_id,
        request_id: ids.request_id,
        response_description: "QR Code Successfully Generated.".to_string(),
        qr_code: qr_base64,
    }))
}
