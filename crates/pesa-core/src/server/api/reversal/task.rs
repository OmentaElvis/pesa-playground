use std::time::Duration;

use anyhow::Context;
use base64::{Engine, engine::general_purpose};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, pkcs8::DecodePrivateKey};

use crate::{
    accounts::{mmf_accounts::MmfAccount, utility_accounts::UtilityAccount},
    business::Business,
    business_operators::BusinessOperator,
    events::DomainEventDispatcher,
    projects::Project,
    server::{
        ApiError, MpesaError,
        api::reversal::{
            ReversalCallbackResponse, ReversalRequest, ReversalResponse, ReversalResultCodes,
        },
        async_handler::PpgAsyncRequest,
    },
    transaction_jobs::task::TransferConfig,
    transactions::{
        Transaction, TransactionEngineError, TransactionNote, TransactionStatus, TransactionType,
    },
    utils::identifiers::Identifiers,
};

pub struct Reversal {
    pub ids: Identifiers,
    pub result_url: String,
    pub queue_time_out_url: String,
    pub original_transaction_id: String,
    pub amount: i64,
    pub receiver_party: String,
    pub debit_balance: i64,
    pub utility_balance: i64,
    pub reversal_receipt: Option<String>,
    pub project: Project,
}

impl PpgAsyncRequest for Reversal {
    type RequestData = ReversalRequest;
    type SyncResponseData = ReversalResponse;
    type CallbackPayload = ReversalCallbackResponse;
    type Error = ReversalResultCodes;

    fn api_name() -> &'static str {
        "reversal"
    }

    async fn init(
        state: &crate::server::ApiState,
        req: Self::RequestData,
        ids: &Identifiers,
        _api_key: crate::api_keys::ApiKey,
    ) -> Result<(Self::SyncResponseData, Self), crate::server::ApiError> {
        if req.command_id != "TransactionReversal" {
            return Err(ApiError::new(
                MpesaError::InvalidCommandId,
                "Invalid CommandID. Only 'TransactionReversal' is allowed.",
            ));
        }

        let amount: i64 = req
            .amount
            .parse()
            .map_err(|_| ApiError::new(MpesaError::InvalidAmount, "Invalid Amount"))?;
        if amount <= 0 {
            return Err(ApiError::new(
                MpesaError::InvalidAmount,
                "Amount must be greater than 0.",
            ));
        }

        let business = Business::get_by_short_code(&state.context.db, &req.receiver_party)
            .await
            .map_err(|error| {
                ApiError::new(
                    MpesaError::InternalError,
                    format!("An internal error occurred: {}", error),
                )
            })?
            .ok_or(ApiError::new(
                MpesaError::InvalidShortcode,
                "ReceiverParty shortcode not found",
            ))?;

        let utility_account = UtilityAccount::find_by_business_id(&state.context.db, business.id)
            .await
            .map_err(|error| {
                ApiError::new(
                    MpesaError::InternalError,
                    format!("An internal error occurred: {}", error),
                )
            })?
            .ok_or(ApiError::new(
                MpesaError::InternalError,
                "Failed to load utility account",
            ))?;

        let mmf_account = MmfAccount::find_by_business_id(&state.context.db, business.id)
            .await
            .map_err(|error| {
                ApiError::new(
                    MpesaError::InternalError,
                    format!("An internal error occurred: {}", error),
                )
            })?
            .ok_or(ApiError::new(
                MpesaError::InternalError,
                "Failed to load mmf account",
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
            .decode(req.security_credential)
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

        let project = match Project::get_by_id(&state.context.db, state.project_id).await {
            Ok(Some(project)) => project,
            Ok(None) => {
                return Err(ApiError::new(
                    MpesaError::InvalidCredentials,
                    crate::server::api::auth::INVALID_CREDENTIALS,
                ));
            }
            Err(err) => {
                return Err(ApiError::new(MpesaError::InternalError, err.to_string()));
            }
        };

        let result = ReversalResultCodes::Success;

        Ok((
            ReversalResponse {
                originator_conversation_id: ids.originator_conversation_id.clone(),
                conversation_id: ids.conversation_id.clone(),
                response_code: result.code().to_string(),
                response_description: result.to_string(),
            },
            Self {
                ids: ids.clone(),
                result_url: req.result_url,
                queue_time_out_url: req.queue_time_out_url,
                original_transaction_id: req.transaction_id,
                amount,
                receiver_party: req.receiver_party,
                debit_balance: mmf_account.balance as i64,
                utility_balance: utility_account.balance as i64,
                reversal_receipt: None,
                project,
            },
        ))
    }

    async fn execute(
        &mut self,
        state: &crate::server::ApiState,
    ) -> Result<Self::CallbackPayload, Self::Error> {
        let original = Transaction::get(&state.context.db, &self.original_transaction_id)
            .await
            .map_err(|e| ReversalResultCodes::Internal(anyhow::anyhow!("{}", e)))?
            .ok_or(ReversalResultCodes::InvalidTransaction)?;

        if !matches!(
            original.transaction_type,
            TransactionType::Paybill | TransactionType::BuyGoods
        ) {
            return Err(ReversalResultCodes::InvalidTransaction);
        }

        if original.status == TransactionStatus::Reversed {
            return Err(ReversalResultCodes::AlreadyReversed);
        }

        let destination = original
            .from
            .ok_or(ReversalResultCodes::Internal(anyhow::anyhow!(
                "Original transaction has no source account"
            )))?;

        let (transaction, events) = state
            .context
            .transfer(TransferConfig {
                identifiers: self.ids.clone(),
                delay: Duration::from_secs(self.project.txn_delay as u64),
                source: Some(original.to),
                destination,
                amount: original.amount,
                txn_type: TransactionType::Reversal,
                notes: Some(TransactionNote::Reversal {
                    original_transaction_id: self.original_transaction_id.clone(),
                }),
            })
            .await
            .context("Failed to initiate reversal")?
            .map_err(|e| match e {
                TransactionEngineError::TransactionNotFound => {
                    ReversalResultCodes::InvalidTransaction
                }
                TransactionEngineError::InsufficientFunds => {
                    ReversalResultCodes::InsufficientBalance
                }
                _ => ReversalResultCodes::Internal(anyhow::anyhow!("{}", e)),
            })?;

        self.reversal_receipt = Some(transaction.id.clone());

        DomainEventDispatcher::dispatch_events(&state.context, events)
            .context("Failed to emit events")?;

        Ok(self.generate_response(&ReversalResultCodes::Success))
    }

    fn get_callback_url(&self) -> Option<&str> {
        Some(&self.result_url)
    }

    fn get_originator_id(&self) -> &str {
        &self.ids.originator_conversation_id
    }
}
