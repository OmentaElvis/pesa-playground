use chrono::Utc;
use rand::{Rng, distributions::Alphanumeric, seq::SliceRandom};
use serde::{Deserialize, Serialize};

pub mod task;
pub mod ui;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TransactionType {
    CustomerPayBillOnline,
    CustomerBuyGoodsOnline,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StkPushRequest {
    pub business_short_code: String,
    pub password: String,
    pub timestamp: String,
    pub transaction_type: TransactionType,
    pub amount: String,
    pub party_a: String,
    pub party_b: String,
    pub phone_number: String,
    pub call_back_u_r_l: String,
    pub account_reference: String,
    pub transaction_desc: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StkPushResponse {
    #[serde(rename = "MerchantRequestID")]
    pub merchant_request_id: String,
    #[serde(rename = "CheckoutRequestID")]
    pub checkout_request_id: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: i32,
    #[serde(rename = "ResponseDescription")]
    pub response_description: String,
    #[serde(rename = "CustomerMessage")]
    pub customer_message: String,
}

#[derive(Serialize, Debug)]
pub struct CallbackItem {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: serde_json::Value,
}

#[derive(Serialize, Debug)]
pub struct CallbackMetadata {
    #[serde(rename = "Item")]
    item: Vec<CallbackItem>,
}

#[derive(Serialize, Debug)]
pub struct StkCallback {
    #[serde(rename = "MerchantRequestID")]
    pub merchant_request_id: String,
    #[serde(rename = "CheckoutRequestID")]
    pub checkout_request_id: String,
    #[serde(rename = "ResultCode")]
    pub result_code: i32,
    #[serde(rename = "ResultDesc")]
    pub result_desc: String,
    #[serde(rename = "CallbackMetadata")]
    metadata: Option<CallbackMetadata>,
}

#[derive(Serialize, Debug)]
pub struct StkCallbackBody {
    #[serde(rename = "stkCallback")]
    callback: StkCallback,
}

#[derive(Serialize, Debug)]
pub struct StkCallbackBodyWrapper {
    #[serde(rename = "Body")]
    pub body: StkCallbackBody,
}

#[derive(Debug, thiserror::Error)]
pub enum StkPushResultCode {
    #[error("The service request is processed successfully.")]
    Success,

    #[error("DS timeout error.")]
    DSTimeout,

    #[error("Internal system error.")]
    SystemError,

    #[error("Error sending push request.")]
    ErrorSendingPushRequest,

    #[error("Error sending push request.")]
    ErrorSendingPushRequest1037,

    #[error("Request cancelled by user.")]
    RequestCancelledByUser,

    #[error("No response from the user.")]
    NoResponseFromUser,

    #[error("The balance is insufficient for the transaction.")]
    InsufficientBalance,

    #[error("Initiator information is invalid.")]
    InitiatorInformationInvalid,

    #[error("Transaction has expired.")]
    TransactionHasExpired,

    #[error("Unable to obtain subscriber lock.")]
    UnableToObtainSubscriberLock,

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl StkPushResultCode {
    pub fn code(&self) -> i32 {
        match self {
            StkPushResultCode::Success => 0,
            StkPushResultCode::DSTimeout => 1037,
            StkPushResultCode::SystemError => 1025,
            StkPushResultCode::ErrorSendingPushRequest => 9999,
            StkPushResultCode::ErrorSendingPushRequest1037 => 1037,
            StkPushResultCode::RequestCancelledByUser => 1032,
            StkPushResultCode::NoResponseFromUser => 1037,
            StkPushResultCode::InsufficientBalance => 1,
            StkPushResultCode::InitiatorInformationInvalid => 2001,
            StkPushResultCode::TransactionHasExpired => 1019,
            StkPushResultCode::UnableToObtainSubscriberLock => 1001,
            Self::Internal(_) => 500,
        }
    }
}
impl StkPushResultCode {
    pub fn random_failure() -> Self {
        use StkPushResultCode::*;
        const FAILURES: &[fn() -> StkPushResultCode] = &[
            || DSTimeout,
            || SystemError,
            || ErrorSendingPushRequest,
            || ErrorSendingPushRequest1037,
            || RequestCancelledByUser,
            || InsufficientBalance,
            || InitiatorInformationInvalid,
            || TransactionHasExpired,
            || UnableToObtainSubscriberLock,
        ];

        let mut rng = rand::thread_rng();
        FAILURES.choose(&mut rng).unwrap()()
    }
    pub fn random() -> Self {
        use StkPushResultCode::*;
        const FAILURES: &[fn() -> StkPushResultCode] = &[
            || Success,
            || DSTimeout,
            || SystemError,
            || ErrorSendingPushRequest,
            || ErrorSendingPushRequest1037,
            || RequestCancelledByUser,
            || InsufficientBalance,
            || InitiatorInformationInvalid,
            || TransactionHasExpired,
            || UnableToObtainSubscriberLock,
        ];

        let mut rng = rand::thread_rng();
        FAILURES.choose(&mut rng).unwrap()()
    }
}

pub fn generate_merchant_request_id() -> String {
    format!(
        "{}-{}-{}",
        rand::thread_rng().gen_range(10000..99999),
        rand::thread_rng().gen_range(10000000..99999999),
        rand::thread_rng().gen_range(0..9)
    )
}

pub fn generate_checkout_request_id() -> String {
    let timestamp = Utc::now().format("%d%m%Y%H%M%S").to_string(); // e.g. 02072025143500
    let rand_suffix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    format!("ws_CO_{}{}", timestamp, rand_suffix)
}
