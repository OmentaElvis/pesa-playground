use serde::{Deserialize, Serialize};

use crate::{
    server::{api::b2c::task::B2C, async_handler::IntoCallbackPayload},
    transactions::Ledger,
};

pub mod task;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum CommandID {
    SalaryPayment,
    BusinessPayment,
    PromotionPayment,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct B2CRequest {
    #[serde(rename = "OriginatorConversationID")]
    pub originator_conversation_id: String,
    #[serde(rename = "InitiatorName")]
    pub initiator_name: String,
    #[serde(rename = "SecurityCredential")]
    pub security_credential: String,
    #[serde(rename = "CommandID")]
    pub command_id: CommandID,
    #[serde(rename = "Amount")]
    pub amount: String,
    #[serde(rename = "PartyA")]
    pub party_a: String,
    #[serde(rename = "PartyB")]
    pub party_b: String,
    #[serde(rename = "Remarks")]
    pub remarks: String,
    #[serde(rename = "QueueTimeOutURL")]
    pub queue_time_out_url: String,
    #[serde(rename = "ResultURL")]
    pub result_url: String,
    #[serde(rename = "Occassion")]
    pub occassion: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct B2CRequestResponse {
    #[serde(rename = "ConversationID")]
    pub conversation_id: String,
    #[serde(rename = "OriginatorConversationID")]
    pub originator_conversation_id: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: String,
    #[serde(rename = "ResponseDescription")]
    pub response_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultParameters {
    #[serde(rename = "ResultParameter")]
    pub result_parameter: Vec<KeyValueEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValueEntry {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Value")]
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceData {
    #[serde(rename = "ReferenceItem")]
    pub reference_item: KeyValueEntry,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallbackResult {
    #[serde(rename = "ResultType")]
    result_type: u16,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct B2CCallbackResponse {
    #[serde(rename = "Result")]
    pub result: CallbackResult,
}

#[derive(Debug, thiserror::Error)]
pub enum B2CResultCodes {
    /// The B2C transaction has been processed successfully on M-PESA.
    #[error("The service request is processed successfully.")]
    Success,
    /// The balance is insufficient for the transaction. The B2C account does not have enough money
    /// in its utility account to complete the transaction requested.
    #[error("The service request is processed successfully.")]
    InsufficientBalance,
    /// Declined due to limit rule.
    /// The amount provided in the request is smaller than the allowed B2C transaction amount.
    #[error("Declined due to limit rule")]
    BelowMinTransactionLimit,
    /// Declined due to limit rule: greater than the maximum transaction amount.
    /// The amount provided in the request is greater than the allowed B2C transaction amount.
    #[error("Declined due to limit rule: greater than the maximum transaction amount.")]
    AboveMaxTransactionLimit,
    /// Declined due to limit rule: would exceed daily transfer limit.
    /// For the business organization, the daily transfer limit is very high; this rule is more likely to apply to the customer, where the customer daily transfer amount limit is currently Ksh 500,000.
    #[error("Declined due to limit rule: would exceed the maximum balance.")]
    DailyTransferLimitExceeded,
    /// Declined due to limit rule: would exceed the maximum balance.
    /// Continuing to process the requested transaction would exceed the maximum customer account balance limit (currently Ksh 500,000).
    #[error("Declined due to limit rule: would exceed the maximum balance.")]
    MaxBalanceExceeded,
    /// The DebitParty is in an invalid state. The B2C account is not active.
    #[error("The DebitParty is in an invalid state.")]
    DebitPartyInvalidState,
    /// The initiator is not allowed to initiate this request.
    /// The API user used in the request has no ORG B2C API initiator role required to successfully initiate B2C request.
    #[error("The initiator is not allowed to initiate this request")]
    InitiatorNotAllowed,
    /// The initiator information is invalid.
    /// The API user credentials provided in the request are invalid.
    /// This can be:
    /// - Wrong API user username.
    /// - Wrong API user password encrypted.
    /// - Wrong algorithm or certificate used for API user password encryption.
    #[error("The initiator information is invalid.")]
    InvalidInitiatorInfo,
    /// Declined due to account rule: The account status does not allow this transaction.
    /// The B2C account is not active.
    #[error("Declined due to account rule: The account status does not allow this transaction.")]
    AccountRuleDeclined,
    /// The request is not permitted according to product assignment.
    /// The PartyA short code specified in the request has no permission to perform B2C payments.
    #[error("The request is not permitted according to product assignment.")]
    ProductAssignmentNotPermitted,
    /// Credit Party customer type (Unregistered or Registered Customer) can't be supported by the service.
    /// The customer is not registered.
    #[error(
        "Credit Party customer type (Unregistered or Registered Customer) can't be supported by the service."
    )]
    UnsupportedCustomerType,
    /// The security credential is locked.
    /// Indicates that the password for the API user is locked.
    /// The Business Administrator can unlock it.
    #[error("The security credential is locked")]
    SecurityCredentialLocked,
    /// The operator does not exist.
    /// The phone number provided in the request is invalid or does not exist on M-PESA.
    #[error("The operator does not exist.")]
    OperatorDoesNotExist,

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl B2CResultCodes {
    fn code(&self) -> &str {
        match self {
            B2CResultCodes::Success => "0",
            B2CResultCodes::InsufficientBalance => "1",
            B2CResultCodes::BelowMinTransactionLimit => "2",
            B2CResultCodes::AboveMaxTransactionLimit => "3",
            B2CResultCodes::DailyTransferLimitExceeded => "4",
            B2CResultCodes::MaxBalanceExceeded => "8",
            B2CResultCodes::DebitPartyInvalidState => "11",
            B2CResultCodes::InitiatorNotAllowed => "21",
            B2CResultCodes::InvalidInitiatorInfo => "2001",
            B2CResultCodes::AccountRuleDeclined => "2006",
            B2CResultCodes::ProductAssignmentNotPermitted => "2028",
            B2CResultCodes::UnsupportedCustomerType => "2040",
            B2CResultCodes::SecurityCredentialLocked => "8006",
            B2CResultCodes::OperatorDoesNotExist => "0xSFC_IC0003",
            B2CResultCodes::Internal(_) => "500",
        }
    }
}

impl IntoCallbackPayload<B2C, B2CCallbackResponse> for B2CResultCodes {
    fn get_payload(&self, ctx: &B2C) -> B2CCallbackResponse {
        let message = self.to_string();
        let code = self.code();

        B2CCallbackResponse {
            result: CallbackResult {
                result_type: 0,
                result_code: code.to_string(),
                result_desc: message,
                conversation_id: ctx.conversation_id.to_string(),
                originator_conversation_id: ctx.originator_conversation_id.to_string(),
                transaction_id: Ledger::generate_receipt(),
                result_parameters: None,
                reference_data: ReferenceData {
                    reference_item: KeyValueEntry {
                        key: "QueueTimeoutURL".to_string(),
                        // TODO - Just a placeholder. I cant figure out where this endpoint is documented
                        value: "http://0.0.0.0:8000/mpesa/b2cresults/v1/submit"
                            .to_string()
                            .into(),
                    },
                },
            },
        }
    }
}
