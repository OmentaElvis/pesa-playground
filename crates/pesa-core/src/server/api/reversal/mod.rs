use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::server::{api::reversal::task::Reversal, async_handler::IntoCallbackPayload};

pub mod task;

// --- Request Payload ---

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReversalRequest {
    #[serde(rename = "Initiator")]
    pub initiator: String,
    #[serde(rename = "SecurityCredential")]
    pub security_credential: String,
    #[serde(rename = "CommandID")]
    pub command_id: String,
    #[serde(rename = "TransactionID")]
    pub transaction_id: String,
    #[serde(rename = "Amount")]
    pub amount: String,
    #[serde(rename = "ReceiverParty")]
    pub receiver_party: String,
    #[serde(rename = "RecieverIdentifierType")]
    pub reciever_identifier_type: String,
    #[serde(rename = "ResultURL")]
    pub result_url: String,
    #[serde(rename = "QueueTimeOutURL")]
    pub queue_time_out_url: String,
    #[serde(rename = "Remarks")]
    pub remarks: String,
}

// --- Synchronous Response ---

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReversalResponse {
    #[serde(rename = "OriginatorConversationID")]
    pub originator_conversation_id: String,
    #[serde(rename = "ConversationID")]
    pub conversation_id: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: String,
    #[serde(rename = "ResponseDescription")]
    pub response_description: String,
}

// --- Asynchronous Callback Payload ---

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReversalCallbackResponse {
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

// --- Result Codes ---

#[derive(Debug, thiserror::Error)]
pub enum ReversalResultCodes {
    #[error("The service request is processed successfully.")]
    Success,
    #[error("The transaction has already been reversed.")]
    AlreadyReversed,
    #[error("The OriginalTransactionID is invalid.")]
    InvalidTransaction,
    #[error("The initiator is not allowed to initiate this request.")]
    InitiatorNotAllowed,
    #[error("The initiator information is invalid.")]
    InvalidInitiatorInfo,
    #[error("The balance is insufficient.")]
    InsufficientBalance,
    #[error("The security credential is locked.")]
    SecurityCredentialLocked,
    #[error("The DebitParty is in an invalid state.")]
    DebitPartyInvalidState,
    #[error("Declined due to account rule.")]
    AccountRule,
    #[error("Not permitted according to product assignment.")]
    ProductAssignment,
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl ReversalResultCodes {
    fn code(&self) -> &str {
        match self {
            ReversalResultCodes::Success => "0",
            ReversalResultCodes::AlreadyReversed => "R000001",
            ReversalResultCodes::InvalidTransaction => "R000002",
            ReversalResultCodes::InitiatorNotAllowed => "21",
            ReversalResultCodes::InvalidInitiatorInfo => "2001",
            ReversalResultCodes::InsufficientBalance => "1",
            ReversalResultCodes::SecurityCredentialLocked => "8006",
            ReversalResultCodes::DebitPartyInvalidState => "11",
            ReversalResultCodes::AccountRule => "2006",
            ReversalResultCodes::ProductAssignment => "2028",
            ReversalResultCodes::Internal(_) => "500",
        }
    }
}

impl IntoCallbackPayload<Reversal, ReversalCallbackResponse> for ReversalResultCodes {
    fn get_payload(&self, ctx: &Reversal) -> ReversalCallbackResponse {
        ctx.generate_response(self)
    }
}

impl Reversal {
    pub fn generate_response(&self, res: &ReversalResultCodes) -> ReversalCallbackResponse {
        let message = res.to_string();
        let code = res.code();

        let result_parameters = if matches!(res, ReversalResultCodes::Success) {
            let ts = Utc::now().format("%Y%m%d%H%M%S").to_string();
            Some(ResultParameters {
                result_parameter: vec![
                    KeyValueEntry {
                        key: "DebitAccountBalance".to_string(),
                        value: format!(
                            "Working Account|KES|{:.2}|{:.2}|0.00|0.00&Utility Account|KES|{:.2}|{:.2}|0.00|0.00",
                            self.debit_balance as f64 / 100.0,
                            self.debit_balance as f64 / 100.0,
                            self.utility_balance as f64 / 100.0,
                            self.utility_balance as f64 / 100.0,
                        )
                        .into(),
                    },
                    KeyValueEntry {
                        key: "Amount".to_string(),
                        value: (self.amount as f64 / 100.0).into(),
                    },
                    KeyValueEntry {
                        key: "TransCompletedTime".to_string(),
                        value: ts.into(),
                    },
                    KeyValueEntry {
                        key: "OriginalTransactionID".to_string(),
                        value: self.original_transaction_id.clone().into(),
                    },
                    KeyValueEntry {
                        key: "Charge".to_string(),
                        value: 0.0_f64.into(),
                    },
                    KeyValueEntry {
                        key: "CreditPartyPublicName".to_string(),
                        value: "254000000000 - Pesa Playground User".to_string().into(),
                    },
                    KeyValueEntry {
                        key: "DebitPartyPublicName".to_string(),
                        value: format!("{} - Pesa Playground", self.receiver_party).into(),
                    },
                ],
            })
        } else {
            None
        };

        let callback_txn_id = self
            .reversal_receipt
            .clone()
            .unwrap_or("SKE0000000".to_string());

        ReversalCallbackResponse {
            result: CallbackResult {
                result_type: 0,
                result_code: code.to_string(),
                result_desc: message,
                originator_conversation_id: self.ids.originator_conversation_id.clone(),
                conversation_id: self.ids.conversation_id.clone(),
                result_parameters,
                transaction_id: callback_txn_id,
                reference_data: ReferenceData {
                    reference_item: KeyValueEntry {
                        key: "QueueTimeoutURL".to_string(),
                        value: self.queue_time_out_url.clone().into(),
                    },
                },
            },
        }
    }
}
