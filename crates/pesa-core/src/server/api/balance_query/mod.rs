use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    server::{api::balance_query::task::BalanceQuery, async_handler::IntoCallbackPayload},
    transactions::Ledger,
};

pub mod task;

// --- Request Payload ---
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum CommandID {
    AccountBalance,
    WorkingAccountBalance,
    UtilityAccountBalance,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum IdentifierType {
    #[serde(rename = "4")]
    OrganisationShortCode,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BalanceQueryRequest {
    #[serde(rename = "Initiator")]
    pub initiator: String,
    #[serde(rename = "SecurityCredential")]
    pub security_credential: String,
    #[serde(rename = "CommandID")]
    pub command_id: CommandID,
    #[serde(rename = "PartyA")]
    pub party_a: String, // Shortcode of the business
    #[serde(rename = "IdentifierType")]
    pub identifier_type: IdentifierType,
    #[serde(rename = "Remarks")]
    pub remarks: String,
    #[serde(rename = "QueueTimeOutURL")]
    pub queue_time_out_url: String,
    #[serde(rename = "ResultURL")]
    pub result_url: String,
}

// --- Synchronous Response ---
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BalanceQueryRequestResponse {
    #[serde(rename = "ConversationID")]
    pub conversation_id: String,
    #[serde(rename = "OriginatorConversationID")]
    pub originator_conversation_id: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: String,
    #[serde(rename = "ResponseDescription")]
    pub response_description: String,
}

// --- Asynchronous Callback Payload ---
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BalanceQueryCallbackResponse {
    #[serde(rename = "Result")]
    pub result: CallbackResult,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CallbackResult {
    #[serde(rename = "ResultType")]
    pub result_type: u16,
    #[serde(rename = "ResultCode")]
    pub result_code: String,
    #[serde(rename = "ResultDesc")]
    pub result_desc: String,
    #[serde(rename = "OriginatorConversationID")]
    pub originator_conversation_id: String,
    #[serde(rename = "ConversationID")]
    pub conversation_id: String,
    #[serde(rename = "TransactionID")]
    pub transaction_id: String,
    #[serde(rename = "ResultParameters")]
    pub result_parameters: Option<ResultParameters>,
    #[serde(rename = "ReferenceData")]
    pub reference_data: ReferenceData,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResultParameters {
    #[serde(rename = "ResultParameter")]
    pub result_parameter: Vec<KeyValueEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeyValueEntry {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Value")]
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReferenceData {
    #[serde(rename = "ReferenceItem")]
    pub reference_item: KeyValueEntry,
}

/// Possible result codes for a Balance Query API call.
#[derive(Debug, thiserror::Error)]
pub enum BalanceQueryResultCodes {
    /// The service request was processed successfully.
    #[error("The service request is processed successfully.")]
    Success,
    /// The initiator is not allowed to initiate this request.
    #[error("The initiator is not allowed to initiate this request")]
    InitiatorNotAllowed,
    /// The initiator information provided is invalid.
    #[error("The initiator information is invalid.")]
    InvalidInitiatorInfo,
    /// The operator associated with the phone number or account does not exist.
    #[error("The operator does not exist.")]
    OperatorDoesNotExist,
    /// A request with the same originator conversation ID has been detected before.
    #[error("Duplicate request detected.")]
    DuplicateDetected,
    /// A general internal system error occurred that is not more specifically identified.
    #[error("An internal system error occurred.")]
    InternalFailure,
    /// The initiator's credentials (e.g., password check) failed, possibly due to encryption/decryption issues.
    #[error("Initiator credentials invalid or decryption failed.")]
    InitiatorCredentialCheckFailure,
    /// There was a failure in the sequencing of the request message.
    #[error("Request message sequencing failed.")]
    MessageSequencingFailure,
    /// The initiator username provided in the request cannot be found.
    #[error("Initiator username not found.")]
    UnresolvedInitiator,
    /// The initiator does not have the necessary permissions for the specified primary party.
    #[error("Initiator lacks permission for the primary party.")]
    InitiatorPermissionFailure,
    /// The presented initiator username is not currently active.
    #[error("Initiator is not active.")]
    InitiatorNotActive,
    /// One or more mandatory fields required for the API operation are missing from the request.
    #[error("Required input parameters are missing.")]
    MissingMandatoryFields,
    /// All required parameters were present, but their values failed validation checks.
    #[error("Invalid request parameter values.")]
    InvalidRequestParameters,
    /// A traffic blocking condition is currently in place, indicating the system is busy.
    #[error("System is busy, traffic blocking condition in place.")]
    TrafficBlockingCondition,
    /// The command specified within the request is not recognized or defined.
    #[error("The command specified in the request is invalid.")]
    InvalidCommand,
    /// The request was cached and is awaiting resending.
    #[error("Request cached, waiting for resending.")]
    RequestCached,
    /// The system is currently overloaded and cannot process the request.
    #[error("The system is overloaded.")]
    SystemOverload,
    /// The request was rejected due to throttling limits being exceeded.
    #[error("Request throttled.")]
    ThrottlingError,
    /// A generic internal server error occurred.
    #[error("Internal server error.")]
    InternalServerError,
    /// An input value provided in the request is invalid.
    #[error("Invalid input value.")]
    InvalidInput,
    /// The service's current status is abnormal.
    #[error("Service status is abnormal.")]
    ServiceAbnormal,
    /// The API's current status is abnormal.
    #[error("API status is abnormal.")]
    ApiAbnormal,
    /// The caller does not have sufficient permissions to perform the requested operation.
    #[error("Insufficient permissions.")]
    InsufficientPermissions,
    /// The rate limit for requests has been exceeded.
    #[error("Request rate limit exceeded.")]
    RequestRateExceeded,
    /// The service is currently undergoing maintenance and is unavailable.
    #[error("Service is under maintenance.")]
    ServiceUnderMaintenance,
    /// A catch-all for any internal errors not covered by more specific error codes.
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl BalanceQueryResultCodes {
    fn code(&self) -> &str {
        match self {
            BalanceQueryResultCodes::Success => "0",
            BalanceQueryResultCodes::InitiatorNotAllowed => "21",
            BalanceQueryResultCodes::InvalidInitiatorInfo => "2001",
            BalanceQueryResultCodes::OperatorDoesNotExist => "0xSFC_IC0003",
            BalanceQueryResultCodes::DuplicateDetected => "15",
            BalanceQueryResultCodes::InternalFailure => "17",
            BalanceQueryResultCodes::InitiatorCredentialCheckFailure => "18",
            BalanceQueryResultCodes::MessageSequencingFailure => "19",
            BalanceQueryResultCodes::UnresolvedInitiator => "20",
            BalanceQueryResultCodes::InitiatorPermissionFailure => "21",
            BalanceQueryResultCodes::InitiatorNotActive => "22",
            BalanceQueryResultCodes::MissingMandatoryFields => "24",
            BalanceQueryResultCodes::InvalidRequestParameters => "25",
            BalanceQueryResultCodes::TrafficBlockingCondition => "26",
            BalanceQueryResultCodes::InvalidCommand => "29",
            BalanceQueryResultCodes::RequestCached => "100000000",
            BalanceQueryResultCodes::SystemOverload => "100000001",
            BalanceQueryResultCodes::ThrottlingError => "100000002",
            BalanceQueryResultCodes::InternalServerError => "100000004",
            BalanceQueryResultCodes::InvalidInput => "100000005",
            BalanceQueryResultCodes::ServiceAbnormal => "100000007",
            BalanceQueryResultCodes::ApiAbnormal => "100000009",
            BalanceQueryResultCodes::InsufficientPermissions => "100000010",
            BalanceQueryResultCodes::RequestRateExceeded => "100000011",
            BalanceQueryResultCodes::ServiceUnderMaintenance => "00.002.1001",
            BalanceQueryResultCodes::Internal(_) => "500",
        }
    }
}

impl IntoCallbackPayload<BalanceQuery, BalanceQueryCallbackResponse> for BalanceQueryResultCodes {
    fn get_payload(&self, ctx: &BalanceQuery) -> BalanceQueryCallbackResponse {
        ctx.generate_response(self)
    }
}

impl BalanceQuery {
    pub fn generate_response(&self, res: &BalanceQueryResultCodes) -> BalanceQueryCallbackResponse {
        let transaction_id = Ledger::generate_receipt();
        let message = res.to_string();
        let code = res.code();

        let result_parameters = if matches!(res, BalanceQueryResultCodes::Success) {
            // I really hate whatever TF this is.
            let balance = format!(
                "Working Account|KES|{:.2}|{:.2}|0.00|0.00&Float Account|KES|0.00|0.00|0.00|0.00&Utility Account|KES|{:.2}|{:.2}|0.00|0.00&Charges Paid Account|KES|{:.2}|{:.2}|0.00|0.00&Organization Settlement Account|KES|0.00|0.00|0.00|0.00",
                self.mmf_account.balance as f64 / 100.0,
                self.mmf_account.balance as f64 / 100.0,
                self.utility_account.balance as f64 / 100.0,
                self.utility_account.balance as f64 / 100.0,
                self.business.charges_amount as f64 / 100.0,
                self.business.charges_amount as f64 / 100.0,
            );
            let ts = Utc::now().format("%Y%m%d%H%M%S").to_string();

            Some(ResultParameters {
                result_parameter: vec![
                    KeyValueEntry {
                        key: "AccountBalance".to_string(),
                        value: balance.into(),
                    },
                    KeyValueEntry {
                        key: "BOCompletedTime".to_string(),
                        value: ts.into(),
                    },
                ],
            })
        } else {
            None
        };

        BalanceQueryCallbackResponse {
            result: CallbackResult {
                result_type: 0,
                result_code: code.to_string(),
                result_desc: message,
                originator_conversation_id: self.originator_conversation_id.to_string(),
                conversation_id: self.conversation_id.to_string(),
                result_parameters,
                transaction_id,
                reference_data: ReferenceData {
                    reference_item: KeyValueEntry {
                        key: "QueueTimeoutURL".to_string(),
                        // TODO figure out what is this for
                        value: "http://0.0.0.0:8000/mpesa/balancequeryresults/v1/submit"
                            .to_string()
                            .into(),
                    },
                },
            },
        }
    }
}
