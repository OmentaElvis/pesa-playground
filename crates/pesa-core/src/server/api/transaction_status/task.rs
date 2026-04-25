use base64::{Engine, engine::general_purpose};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, pkcs8::DecodePrivateKey};

use crate::{
    business::Business,
    business_operators::BusinessOperator,
    server::{
        ApiError, MpesaError,
        api::transaction_status::{
            CallbackItem, ReferenceData, ResultParameters, StatusResult, TxnStatusCallbackPayload,
            TxnStatusRequest, TxnStatusRequestResponse, TxnStatusResultCodes,
        },
        async_handler::PpgAsyncRequest,
    },
    transaction_jobs::TransactionJob,
    transactions::TransactionStatus,
    utils::identifiers::Identifiers,
};

pub struct TransactionStatusQuery {
    pub conversation_id: String,
    pub originator_conversation_id: String,
    pub occasion: String,
    pub result_url: String,
    pub pending_job: Option<TransactionJob>,
    pub business: Business,
}

impl PpgAsyncRequest for TransactionStatusQuery {
    type RequestData = TxnStatusRequest;
    type SyncResponseData = TxnStatusRequestResponse;
    type CallbackPayload = TxnStatusCallbackPayload;
    type Error = TxnStatusResultCodes;

    fn api_name() -> &'static str {
        "transaction_status"
    }

    async fn init(
        state: &crate::server::ApiState,
        req: Self::RequestData,
        ids: &Identifiers,
        _api_key: crate::api_keys::ApiKey,
    ) -> Result<(Self::SyncResponseData, Self), crate::server::ApiError>
    where
        Self: Sized,
    {
        let business = Business::get_by_short_code(&state.context.db, &req.party_a)
            .await
            .map_err(|error| {
                ApiError::new(
                    MpesaError::InternalError,
                    format!("An internal error occurred: {}", error),
                )
            })?
            .ok_or(ApiError::new(
                MpesaError::InvalidShortcode,
                "Shortcode not found",
            ))?;

        let operator = BusinessOperator::find_by_business(
            &state.context.db,
            req.initiator.clone(),
            business.id,
        )
        .await
        .map_err(|error| {
            ApiError::new(
                MpesaError::InvalidCredentials,
                format!("An internal error occurred: {}", error),
            )
        })?
        .ok_or(ApiError::new(
            MpesaError::InvalidCredentials,
            "Initiator username not found",
        ))?;

        let settings: crate::settings::models::AppSettings = state.context.settings.get().await;
        let private_key = settings
            .encryption_keys
            .ok_or(ApiError::new(
                MpesaError::InternalError,
                "Settings app public and private keys have not been initialized".to_string(),
            ))?
            .private_key;

        let private_key = RsaPrivateKey::from_pkcs8_pem(&private_key).map_err(|err| {
            ApiError::new(
                MpesaError::InternalError,
                format!("Failed to load private key {}", err),
            )
        })?;

        let credential_decode = general_purpose::STANDARD
            .decode(&req.security_credential)
            .map_err(|err| {
                ApiError::new(
                    MpesaError::InternalError,
                    format!("Failed to decode security credential base64: {}", err),
                )
            })?;

        let decrypted = private_key
            .decrypt(Pkcs1v15Encrypt, &credential_decode)
            .map_err(|err| {
                ApiError::new(
                    MpesaError::InvalidCredentials,
                    format!("Failed to decrypt SecurityCredential: {}", err),
                )
            })?;

        if !decrypted.eq(operator.password.as_bytes()) {
            return Err(ApiError::new(
                MpesaError::InvalidCredentials,
                "Invalid SecurityCredential",
            ));
        }

        let search_ids = if let Some(ref orig_id) = req.original_conversation_id {
            vec![req.transaction_id.clone(), orig_id.clone()]
        } else {
            vec![req.transaction_id.clone()]
        };

        let mut pending_job = None;

        for id in &search_ids {
            let job = TransactionJob::get_by_id(&state.context.db, id)
                .await
                .map_err(|error| {
                    ApiError::new(
                        MpesaError::InternalError,
                        format!("An internal error occurred: {}", error),
                    )
                })?;

            if job.is_some() {
                pending_job = job;
                break;
            }
        }

        let originator_conversation_id = req
            .original_conversation_id
            .clone()
            .unwrap_or_else(|| ids.originator_conversation_id.clone());

        Ok((
            TxnStatusRequestResponse {
                conversation_id: ids.conversation_id.clone(),
                originator_conversation_id: originator_conversation_id.clone(),
                response_code: "0".to_string(),
                response_description: "Accept the service request successfully.".to_string(),
            },
            Self {
                conversation_id: ids.conversation_id.clone(),
                originator_conversation_id,
                occasion: req.occassion.clone(),
                result_url: req.result_url,
                pending_job,
                business,
            },
        ))
    }

    async fn execute(
        &mut self,
        _state: &crate::server::ApiState,
    ) -> Result<Self::CallbackPayload, Self::Error> {
        if self.pending_job.is_none() {
            return Ok(self.generate_response(&TxnStatusResultCodes::TransactionNotFound));
        }

        let job = self.pending_job.as_ref().unwrap();
        let txn_status = match job.status {
            TransactionStatus::Completed => "Completed",
            TransactionStatus::Pending => "Pending",
            TransactionStatus::Failed => "Failed",
            TransactionStatus::Reversed => "Reversed",
        };

        Ok(self.generate_response_with_status(&TxnStatusResultCodes::Success, txn_status))
    }

    fn get_callback_url(&self) -> Option<&str> {
        Some(&self.result_url)
    }

    fn get_originator_id(&self) -> &str {
        &self.originator_conversation_id
    }
}

impl TransactionStatusQuery {
    pub fn generate_response(&self, res: &TxnStatusResultCodes) -> TxnStatusCallbackPayload {
        self.generate_response_with_status(res, "Unknown")
    }

    pub fn generate_response_with_status(
        &self,
        res: &TxnStatusResultCodes,
        status: &str,
    ) -> TxnStatusCallbackPayload {
        let message = res.to_string();
        let code = res.code();

        let result_parameters = if matches!(res, TxnStatusResultCodes::Success) {
            if let Some(ref job) = self.pending_job {
                let transaction_id = job.identifiers.transaction_id.clone();
                let amount = job.payload.amount as f64 / 100.0;

                let initiated_time = job.created_at.format("%Y%m%d%H%M%S").to_string();
                let finalized_time = job
                    .updated_at
                    .map(|dt| dt.format("%Y%m%d%H%M%S").to_string())
                    .unwrap_or_else(|| initiated_time.clone());

                Some(ResultParameters {
                    result_parameter: vec![
                        CallbackItem {
                            key: "DebitPartyName".to_string(),
                            value: format!("{} - {}", self.business.short_code, self.business.name)
                                .into(),
                        },
                        CallbackItem {
                            key: "OriginatorConversationID".to_string(),
                            value: self.originator_conversation_id.clone().into(),
                        },
                        CallbackItem {
                            key: "InitiatedTime".to_string(),
                            value: initiated_time.into(),
                        },
                        CallbackItem {
                            key: "TransactionStatus".to_string(),
                            value: status.into(),
                        },
                        CallbackItem {
                            key: "FinalisedTime".to_string(),
                            value: finalized_time.into(),
                        },
                        CallbackItem {
                            key: "Amount".to_string(),
                            value: amount.into(),
                        },
                        CallbackItem {
                            key: "ConversationID".to_string(),
                            value: self.conversation_id.clone().into(),
                        },
                        CallbackItem {
                            key: "ReceiptNo".to_string(),
                            value: transaction_id.into(),
                        },
                    ],
                })
            } else {
                None
            }
        } else {
            None
        };

        let transaction_id = if let Some(ref job) = self.pending_job {
            job.identifiers.transaction_id.clone()
        } else {
            crate::transactions::Ledger::generate_receipt()
        };

        TxnStatusCallbackPayload {
            result: StatusResult {
                result_type: 0,
                result_code: code.to_string(),
                result_desc: message,
                originator_conversation_id: self.originator_conversation_id.clone(),
                conversation_id: self.conversation_id.clone(),
                transaction_id,
                result_parameters,
                reference_data: ReferenceData {
                    reference_item: CallbackItem {
                        key: "Occasion".to_string(),
                        value: self.occasion.clone().into(),
                    },
                },
            },
        }
    }
}
