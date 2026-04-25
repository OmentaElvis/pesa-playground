use serde::{Deserialize, Serialize};

use crate::server::async_handler::IntoCallbackPayload;

pub mod task;

use task::TransactionStatusQuery;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum CommandID {
    TransactionStatusQuery,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum IdentifierType {
    #[serde(rename = "4")]
    OrganisationShortCode,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TxnStatusRequest {
    #[serde(rename = "Initiator")]
    pub initiator: String,
    #[serde(rename = "SecurityCredential")]
    pub security_credential: String,
    #[serde(rename = "CommandID")]
    pub command_id: CommandID,
    #[serde(rename = "TransactionID")]
    pub transaction_id: String,
    #[serde(rename = "OriginalConversationID")]
    pub original_conversation_id: Option<String>,
    #[serde(rename = "PartyA")]
    pub party_a: String,
    #[serde(rename = "IdentifierType")]
    pub identifier_type: IdentifierType,
    #[serde(rename = "ResultURL")]
    pub result_url: String,
    #[serde(rename = "QueueTimeOutURL")]
    pub queue_time_out_url: String,
    #[serde(rename = "Remarks")]
    pub remarks: String,
    #[serde(rename = "Occassion")]
    pub occassion: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxnStatusRequestResponse {
    #[serde(rename = "ConversationID")]
    pub conversation_id: String,
    #[serde(rename = "OriginatorConversationID")]
    pub originator_conversation_id: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: String,
    #[serde(rename = "ResponseDescription")]
    pub response_description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CallbackItem {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Value")]
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResultParameters {
    #[serde(rename = "ResultParameter")]
    pub result_parameter: Vec<CallbackItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReferenceData {
    #[serde(rename = "ReferenceItem")]
    pub reference_item: CallbackItem,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusResult {
    #[serde(rename = "ResultType")]
    pub result_type: i64,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxnStatusCallbackPayload {
    #[serde(rename = "Result")]
    pub result: StatusResult,
}

#[derive(Debug, thiserror::Error)]
pub enum TxnStatusResultCodes {
    #[error("The service request is processed successfully.")]
    Success,
    #[error("Transaction not found.")]
    TransactionNotFound,
    #[error("The initiator is not allowed to initiate this request.")]
    InitiatorNotAllowed,
    #[error("The initiator information is invalid.")]
    InvalidInitiatorInfo,
    #[error("Duplicate request detected.")]
    DuplicateDetected,
    #[error("An internal system error occurred.")]
    InternalFailure,
    #[error("Initiator credentials invalid or decryption failed.")]
    InitiatorCredentialCheckFailure,
    #[error("Request message sequencing failed.")]
    MessageSequencingFailure,
    #[error("Initiator username not found.")]
    UnresolvedInitiator,
    #[error("Initiator lacks permission for the primary party.")]
    InitiatorPermissionFailure,
    #[error("Initiator is not active.")]
    InitiatorNotActive,
    #[error("Required input parameters are missing.")]
    MissingMandatoryFields,
    #[error("Invalid request parameter values.")]
    InvalidRequestParameters,
    #[error("The command specified in the request is invalid.")]
    InvalidCommand,
    #[error("The system is overloaded.")]
    SystemOverload,
    #[error("Request throttled.")]
    ThrottlingError,
    #[error("Internal server error.")]
    InternalServerError,
    #[error("Invalid input value.")]
    InvalidInput,
    #[error("Service status is abnormal.")]
    ServiceAbnormal,
    #[error("API status is abnormal.")]
    ApiAbnormal,
    #[error("Insufficient permissions.")]
    InsufficientPermissions,
    #[error("Request rate limit exceeded.")]
    RequestRateExceeded,
    #[error("Service is under maintenance.")]
    ServiceUnderMaintenance,
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl TxnStatusResultCodes {
    fn code(&self) -> &str {
        match self {
            TxnStatusResultCodes::Success => "0",
            TxnStatusResultCodes::TransactionNotFound => "404.004.01",
            TxnStatusResultCodes::InitiatorNotAllowed => "21",
            TxnStatusResultCodes::InvalidInitiatorInfo => "2001",
            TxnStatusResultCodes::DuplicateDetected => "15",
            TxnStatusResultCodes::InternalFailure => "17",
            TxnStatusResultCodes::InitiatorCredentialCheckFailure => "18",
            TxnStatusResultCodes::MessageSequencingFailure => "19",
            TxnStatusResultCodes::UnresolvedInitiator => "20",
            TxnStatusResultCodes::InitiatorPermissionFailure => "21",
            TxnStatusResultCodes::InitiatorNotActive => "22",
            TxnStatusResultCodes::MissingMandatoryFields => "24",
            TxnStatusResultCodes::InvalidRequestParameters => "25",
            TxnStatusResultCodes::InvalidCommand => "29",
            TxnStatusResultCodes::SystemOverload => "100000001",
            TxnStatusResultCodes::ThrottlingError => "100000002",
            TxnStatusResultCodes::InternalServerError => "100000004",
            TxnStatusResultCodes::InvalidInput => "100000005",
            TxnStatusResultCodes::ServiceAbnormal => "100000007",
            TxnStatusResultCodes::ApiAbnormal => "100000009",
            TxnStatusResultCodes::InsufficientPermissions => "100000010",
            TxnStatusResultCodes::RequestRateExceeded => "100000011",
            TxnStatusResultCodes::ServiceUnderMaintenance => "00.002.1001",
            TxnStatusResultCodes::Internal(_) => "500",
        }
    }
}

impl IntoCallbackPayload<TransactionStatusQuery, TxnStatusCallbackPayload>
    for TxnStatusResultCodes
{
    fn get_payload(&self, ctx: &TransactionStatusQuery) -> TxnStatusCallbackPayload {
        ctx.generate_response(self)
    }
}
