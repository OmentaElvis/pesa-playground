use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};

pub mod db;
pub mod response;
pub mod stk;

#[derive(strum::EnumString, Deserialize, Serialize, Default)]
pub enum CallbackType {
    #[default]
    StkPush,
    C2b,
}

#[derive(strum::EnumString, Deserialize, Serialize, Default)]
pub enum CallbackStatus {
    #[default]
    Pending,
    Delivered,
    Failed,
}

#[derive(Deserialize, Serialize, Default)]
pub struct CallbackLog {
    pub id: u32,
    pub transaction_id: Option<String>,
    pub checkout_request_id: Option<String>,
    pub merchant_request_id: Option<String>,
    pub callback_url: String,
    pub callback_type: CallbackType, // e.g. "stkpush", "c2b"
    pub payload: String,             // raw JSON
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub status: CallbackStatus, // e.g. "delivered", "failed"
    pub error: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
}

impl From<db::Model> for CallbackLog {
    fn from(value: db::Model) -> Self {
        Self {
            id: value.id,
            transaction_id: value.transaction_id,
            checkout_request_id: value.checkout_request_id,
            merchant_request_id: value.merchant_request_id,
            callback_url: value.callback_url,
            callback_type: value.callback_type.parse().unwrap_or_default(),
            payload: value.payload,
            response_status: value.response_status,
            response_body: value.response_body,
            status: value.status.parse().unwrap_or_default(),
            error: value.error,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl CallbackLog {
    pub fn new(callback_type: CallbackType) -> Self {
        Self {
            id: 0,
            callback_type,
            ..Default::default()
        }
    }

    pub fn with_transaction_id(&mut self, transaction_id: &str) -> &mut Self {
        self.transaction_id = Some(transaction_id.to_string());
        self
    }

    pub fn with_checkout_request_id(&mut self, checkout_request_id: &str) -> &mut Self {
        self.checkout_request_id = Some(checkout_request_id.to_string());
        self
    }

    pub fn with_merchant_request_id(&mut self, merchant_request_id: &str) -> &mut Self {
        self.merchant_request_id = Some(merchant_request_id.to_string());
        self
    }

    pub fn with_callback_url(&mut self, callback_url: &str) -> &mut Self {
        self.callback_url = callback_url.to_string();
        self
    }

    pub fn with_payload(&mut self, payload: &str) -> &mut Self {
        self.payload = payload.to_string();
        self
    }

    pub fn with_response_status(&mut self, response_status: i32) -> &mut Self {
        self.response_status = Some(response_status);
        self
    }

    pub fn with_response_body(&mut self, response_body: &str) -> &mut Self {
        self.response_body = Some(response_body.to_string());
        self
    }

    pub fn with_status(&mut self, status: CallbackStatus) -> &mut Self {
        self.status = status;
        self
    }

    pub fn with_error(&mut self, error: &str) -> &mut Self {
        self.error = Some(error.to_string());
        self
    }
}
