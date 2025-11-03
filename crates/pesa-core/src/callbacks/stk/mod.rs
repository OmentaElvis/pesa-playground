use std::collections::HashMap;

use once_cell::sync::Lazy;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, Mutex};

pub mod init;
pub mod process;
pub mod ui;

pub static STK_RESPONSE_REGISTRY: Lazy<Mutex<HashMap<String, oneshot::Sender<UserResponse>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum UserResponse {
    Accepted { pin: String },
    Cancelled,
    Offline,
    Timeout,
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum StkCodes {
    /// 0
    Success,
    /// 1037
    DSTimeout,
    /// 1025
    SystemError,
    /// 9999
    ErrorSendingPushRequest,
    /// 1025
    ErrorSendingPushRequest1037,
    /// 1032
    RequestCancelledByUser,
    /// 1037
    NoResponseFromUser,
    /// 1
    InsufficientBalance,
    /// 2001
    InitiatorInformationInvalid,
    /// 1019
    TransactionHasExpired,
    /// 1001
    UnableToObtainSubscriberLock,
}

impl StkCodes {
    pub fn code(&self) -> i32 {
        match self {
            StkCodes::Success => 0,
            StkCodes::DSTimeout => 1037,
            StkCodes::SystemError => 1025,
            StkCodes::ErrorSendingPushRequest => 9999,
            StkCodes::ErrorSendingPushRequest1037 => 1037,
            StkCodes::RequestCancelledByUser => 1032,
            StkCodes::NoResponseFromUser => 1037,
            StkCodes::InsufficientBalance => 1,
            StkCodes::InitiatorInformationInvalid => 2001,
            StkCodes::TransactionHasExpired => 1019,
            StkCodes::UnableToObtainSubscriberLock => 1001,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            StkCodes::Success => "The service request is processed successfully.",
            StkCodes::DSTimeout => "DS timeout error.",
            StkCodes::SystemError => "Internal system error.",
            StkCodes::ErrorSendingPushRequest => "Error sending push request.",
            StkCodes::ErrorSendingPushRequest1037 => "DS timeout error.",
            StkCodes::RequestCancelledByUser => "Request cancelled by user.",
            StkCodes::NoResponseFromUser => "No response from the user.",
            StkCodes::InsufficientBalance => "The balance is insufficient for the transaction.",
            StkCodes::InitiatorInformationInvalid => "Initiator information is invalid.",
            StkCodes::TransactionHasExpired => "Transaction has expired.",
            StkCodes::UnableToObtainSubscriberLock => "Unable to obtain subscriber lock.",
        }
    }
}

impl StkCodes {
    pub fn random_failure() -> Self {
        use StkCodes::*;
        const FAILURES: &[StkCodes] = &[
            DSTimeout,
            SystemError,
            ErrorSendingPushRequest,
            ErrorSendingPushRequest1037,
            RequestCancelledByUser,
            InsufficientBalance,
            InitiatorInformationInvalid,
            TransactionHasExpired,
            UnableToObtainSubscriberLock,
        ];

        let mut rng = rand::rng();
        FAILURES.choose(&mut rng).unwrap().clone()
    }
    pub fn random() -> Self {
        use StkCodes::*;
        const FAILURES: &[StkCodes] = &[
            Success,
            DSTimeout,
            SystemError,
            ErrorSendingPushRequest,
            ErrorSendingPushRequest1037,
            RequestCancelledByUser,
            InsufficientBalance,
            InitiatorInformationInvalid,
            TransactionHasExpired,
            UnableToObtainSubscriberLock,
        ];

        let mut rng = rand::rng();
        FAILURES.choose(&mut rng).unwrap().clone()
    }
}
