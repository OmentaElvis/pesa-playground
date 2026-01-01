use anyhow::Context;
use chrono::Local;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, pkcs8::DecodePrivateKey};
use sea_orm::TransactionTrait;

use crate::{
    accounts::{mmf_accounts::MmfAccount, user_profiles::User, utility_accounts::UtilityAccount},
    business::Business,
    business_operators::BusinessOperator,
    projects::Project,
    server::{
        ApiError, MpesaError,
        api::{
            auth::INVALID_CREDENTIALS,
            b2c::{
                B2CCallbackResponse, B2CRequest, B2CRequestResponse, B2CResultCodes,
                CallbackResult, CommandID, KeyValueEntry, ReferenceData, ResultParameters,
            },
        },
        async_handler::PpgAsyncRequest,
    },
    transactions::{Ledger, TransactionNote, TransactionType},
};

pub struct B2C {
    pub conversation_id: String,
    pub originator_conversation_id: String,
    pub result_url: String,
    pub business: Business,
    pub utility_account: UtilityAccount,
    pub mmf_account: MmfAccount,
    pub amount: i64,
    pub user: User,
    pub project: Project,
    pub command_id: CommandID,
}

impl PpgAsyncRequest for B2C {
    type RequestData = B2CRequest;
    type SyncResponseData = B2CRequestResponse;
    type CallbackPayload = B2CCallbackResponse;
    type Error = B2CResultCodes;

    fn api_name() -> &'static str {
        "b2c"
    }

    async fn init(
        state: &crate::server::ApiState,
        req: Self::RequestData,
        conversation_id: &str,
        _api_key: crate::api_keys::ApiKey,
    ) -> Result<(Self::SyncResponseData, Self), crate::server::ApiError>
    where
        Self: Sized,
    {
        // FIXME - not entirely sure if b2c amount expects a string or just a normal number. So we have to do some parsing here.
        let amount: f64 = req.amount.parse().map_err(|error| {
            ApiError::new(
                crate::server::MpesaError::InternalError,
                format!(
                    "Failed to parse amount: {}, value passed: {}",
                    error, req.amount
                ),
            )
        })?;
        let business = Business::get_by_short_code(&state.context.db, &req.party_a)
            .await
            .map_err(|error| {
                ApiError::new(
                    crate::server::MpesaError::InternalError,
                    format!("An internal error occured: {}", error),
                )
            })?
            .ok_or(ApiError::new(
                crate::server::MpesaError::InvalidShortcode,
                "Shortcode not found",
            ))?;

        let utility_account = UtilityAccount::find_by_business_id(&state.context.db, business.id)
            .await
            .map_err(|error| {
                ApiError::new(
                    crate::server::MpesaError::InternalError,
                    format!("An internal error occured: {}", error),
                )
            })?
            .ok_or(ApiError::new(
                crate::server::MpesaError::InternalError,
                "Failed to load utility account",
            ))?;

        let mmf_account = MmfAccount::find_by_business_id(&state.context.db, business.id)
            .await
            .map_err(|error| {
                ApiError::new(
                    crate::server::MpesaError::InternalError,
                    format!("An internal error occured: {}", error),
                )
            })?
            .ok_or(ApiError::new(
                crate::server::MpesaError::InternalError,
                "Failed to load mmf account",
            ))?;

        // get the operator by username.
        let operator = BusinessOperator::find_by_business(
            &state.context.db,
            req.initiator_name.clone(),
            business.id,
        )
        .await
        .map_err(|error| {
            ApiError::new(
                MpesaError::InvalidCredentials,
                format!("An internal error occured: {}", error),
            )
        })?
        .ok_or(ApiError::new(
            crate::server::MpesaError::InvalidCredentials,
            "Initiator username not found",
        ))?;

        let settings: crate::settings::models::AppSettings = state.context.settings.get().await;
        let private_key = settings
            .encryption_keys
            .ok_or(ApiError::new(
                MpesaError::InternalError,
                "Settings app public and private keys have not be initialized".to_string(),
            ))?
            .private_key;

        // validate the password
        let private_key = RsaPrivateKey::from_pkcs8_pem(&private_key).map_err(|err| {
            ApiError::new(
                MpesaError::InternalError,
                format!("Failed to load private key {}", err),
            )
        })?;

        let decrypted = private_key
            .decrypt(Pkcs1v15Encrypt, req.security_credential.as_bytes())
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

        let user = match User::get_user_by_phone(&state.context.db, &req.party_b)
            .await
            .map_err(|err| {
                ApiError::new(
                    crate::server::MpesaError::InternalError,
                    format!("An error occured while trying to get user: {}", err),
                )
            })? {
            Some(user) => user,
            None => {
                return Err(ApiError::new(
                    MpesaError::InvalidPhoneNumber,
                    "Invalid phone number",
                ));
            }
        };

        let project = match Project::get_by_id(&state.context.db, state.project_id).await {
            Ok(Some(project)) => project,
            Ok(None) => {
                return Err(ApiError::new(
                    crate::server::MpesaError::InvalidCredentials,
                    INVALID_CREDENTIALS,
                ));
            }
            Err(err) => {
                return Err(ApiError::new(
                    crate::server::MpesaError::InternalError,
                    err.to_string(),
                ));
            }
        };

        Ok((
            B2CRequestResponse {
                conversation_id: conversation_id.to_string(),
                originator_conversation_id: req.originator_conversation_id.clone(),
                response_code: B2CResultCodes::Success.to_string(),
                response_description: String::new(),
            },
            Self {
                conversation_id: conversation_id.to_string(),
                originator_conversation_id: req.originator_conversation_id,
                result_url: req.result_url,
                amount: (amount * 100.0) as i64,
                business,
                utility_account,
                mmf_account,
                user,
                project,
                command_id: req.command_id,
            },
        ))
    }

    async fn execute(
        &mut self,
        state: &crate::server::ApiState,
    ) -> Result<Self::CallbackPayload, Self::Error> {
        let mut receipt = Ledger::generate_receipt();
        let txn = state
            .context
            .db
            .begin()
            .await
            .context("Failed to start transaction")?;

        // check if we have enough funds
        if (self.utility_account.balance - self.amount) < 0 {
            return Ok(self.create_response(B2CResultCodes::InsufficientBalance, &receipt));
        }

        let (transaction, _) = Ledger::transfer(
            &txn,
            Some(self.utility_account.account_id),
            self.user.account_id,
            self.amount,
            &TransactionType::Disbursment,
            Some(&TransactionNote::Disbursment {
                kind: self.command_id.clone(),
            }),
        )
        .await
        .context("Failed to transfer funds")?;

        receipt = transaction.id;

        self.business =
            Business::increment_charges_amount(&txn, self.business.id, -transaction.fee)
                .await
                .context("Failed to increment business charges")?;

        if let Some(utility_account) =
            UtilityAccount::find_by_id(&txn, self.utility_account.account_id)
                .await
                .context("Failed to fetch business utility account")?
        {
            self.utility_account = utility_account;
        }

        txn.commit()
            .await
            .context("Failed to commit transaction.")?;

        Ok(self.create_response(B2CResultCodes::Success, &receipt))
    }

    fn get_callback_url(&self) -> Option<&str> {
        Some(&self.result_url)
    }

    fn get_originator_id(&self) -> &str {
        &self.originator_conversation_id
    }
}

impl B2C {
    fn create_response(&self, code: B2CResultCodes, transaction_id: &str) -> B2CCallbackResponse {
        let params = if matches!(code, B2CResultCodes::Success) {
            let now = Local::now();
            let completed_date_time = now.format("%d.%m.%Y %H:%M:%S").to_string();

            Some(ResultParameters {
                result_parameter: vec![
                    KeyValueEntry {
                        key: "TransactionAmount".to_string(),
                        value: (self.amount as f64 / 100.0).into(),
                    },
                    KeyValueEntry {
                        key: "TransactionAmount".to_string(),
                        value: transaction_id.into(),
                    },
                    KeyValueEntry {
                        key: "ReceiverPartyPublicName".to_string(),
                        value: format!("{} - {}", self.user.phone, self.user.name).into(),
                    },
                    KeyValueEntry {
                        key: "TransactionCompletedDateTime".to_string(),
                        value: completed_date_time.into(),
                    },
                    KeyValueEntry {
                        key: "B2CUtilityAccountAvailableFunds".to_string(),
                        value: (self.utility_account.balance as f64 / 100.0).into(),
                    },
                    KeyValueEntry {
                        key: "B2CWorkingAccountAvailableFunds".to_string(),
                        value: (self.mmf_account.balance as f64 / 100.0).into(),
                    },
                    KeyValueEntry {
                        key: "B2CRecipientIsRegisteredCustomer".to_string(),
                        value: "Y".into(),
                    },
                    KeyValueEntry {
                        key: "B2CRecipientIsRegisteredCustomer".to_string(),
                        value: (self.business.charges_amount as f64 / 100.0).into(),
                    },
                ],
            })
        } else {
            None
        };

        B2CCallbackResponse {
            result: CallbackResult {
                result_type: 0,
                result_code: code.code().to_string(),
                result_desc: code.to_string(),
                originator_conversation_id: self.originator_conversation_id.to_string(),
                conversation_id: self.conversation_id.to_string(),
                transaction_id: transaction_id.to_string(),
                result_parameters: params,
                reference_data: ReferenceData {
                    reference_item: KeyValueEntry {
                        key: "QueueTimeoutURL".to_string(),
                        // TODO - Just a placeholder. I cant figure out where this endpoint is documented
                        value: "http://0.0.0.0:8000/mpesa/b2cresults/v1/submit".into(),
                    },
                },
            },
        }
    }
}
